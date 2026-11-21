//! The reference model: the registry's meaning as a state, and one function from a state and a
//! request to an answer and a change (docs/design/model.md, H1 and H2). [`Model::decide`] reads
//! `&self` only; [`Model::apply`] is the only writer. There is no I/O, clock, thread, lock or cache
//! here: every answer is a function of the state and the request (plan AC-RQ0.5.1).
//!
//! The state holds what its history put in it and nothing else: each request adds at most one
//! repository, one session, one blob, one manifest and one tag, so the caller's bound on a history's
//! length bounds it (CLAUDE.md §2).

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::iter;
use std::sync::Arc;

use crate::answer::{Answer, Body, ErrorCode, Referrer, Status};
use crate::grammar::{
    self, Algorithm, Digest, DigestError, ManifestKind, Name, Reference, ReferenceError, Tag,
};
use crate::request::{
    ByteRange, Bytes, Chunk, ChunkRange, Descriptor, Manifest, ManifestBody, Op, Principal,
    Request, SessionTag,
};

/// What a grant lets a principal do to a repository, as the token specification's scopes name
/// them (`repository:<name>:pull,push,delete`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Pull,
    Push,
    Delete,
}

/// Who may do what to which repository: the operator's stated rules, fixed for a run (architecture
/// D25 makes the real ones; the model takes them as given).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grants {
    granted: BTreeSet<(u8, String, Action)>,
}

impl Grants {
    /// Lets `principal` do `action` to the repository named `repository`.
    pub fn grant(&mut self, principal: u8, repository: &str, action: Action) {
        self.granted
            .insert((principal, repository.to_owned(), action));
    }

    /// Whether `principal` may do `action` to `repository`.
    pub fn allows(&self, principal: u8, repository: &Name, action: Action) -> bool {
        self.granted
            .contains(&(principal, repository.as_str().to_owned(), action))
    }
}

/// What a run states about the registry it models. Each bound is the deployment's, stated to the
/// model by whoever runs it, so the model holds no number of its own (docs/design/numbers.md).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub grants: Grants,
    /// The most bytes a manifest may have; more is refused `413` (plan AC-RQ1.5.3 derives it).
    pub manifest_bound: Option<u64>,
    /// The most bytes one principal's open uploads may hold; a request that would stage more is
    /// refused `429` (plan RQ1.3: sessions "bounded by the bytes they stage, charged to who opened
    /// them"; model.md, H7).
    pub staging_budget: Option<u64>,
    /// The most bytes a repository name may have.
    pub name_bound: Option<usize>,
}

/// A manifest as the repository keeps it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stored {
    /// The media type it was pushed with, without parameters.
    pub content_type: String,
    pub bytes: Bytes,
    /// An image manifest's config and layers, as listed.
    pub blobs: Vec<Digest>,
    /// An index's manifests, as listed.
    pub children: Vec<Digest>,
    /// The subject's digest as written.
    pub subject: Option<String>,
    /// What the referrers API lists as its `artifactType`.
    pub artifact_type: Option<String>,
    pub annotations: Vec<(String, String)>,
}

/// One repository's blobs, manifests and tags.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Repository {
    blobs: BTreeMap<Digest, Bytes>,
    manifests: BTreeMap<Digest, Stored>,
    tags: BTreeMap<Tag, Digest>,
}

impl Repository {
    /// The blobs it links, by digest.
    pub fn blobs(&self) -> impl Iterator<Item = (&Digest, &Bytes)> {
        self.blobs.iter()
    }

    /// Its manifests, by digest.
    pub fn manifests(&self) -> impl Iterator<Item = (&Digest, &Stored)> {
        self.manifests.iter()
    }

    /// Its tags, in listing order.
    pub fn tags(&self) -> impl Iterator<Item = (&Tag, &Digest)> {
        self.tags.iter()
    }

    /// The blob it links under `digest`.
    pub fn blob(&self, digest: &Digest) -> Option<&Bytes> {
        self.blobs.get(digest)
    }

    /// What `tag` names.
    pub fn tag(&self, tag: &Tag) -> Option<&Digest> {
        self.tags.get(tag)
    }
}

/// An open upload session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Session {
    pub repository: Name,
    /// Who opened it, and is charged for what it stages.
    pub opener: u8,
    /// The bytes taken so far.
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct State {
    repositories: BTreeMap<Name, Repository>,
    sessions: BTreeMap<SessionTag, Session>,
}

/// What a request changes. [`Model::apply`] is total: a change naming a session or repository the
/// state lacks changes nothing of it, which a change decided on the same state never does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    None,
    /// Open `session` in `repository`, making the repository if it is new.
    Open {
        session: SessionTag,
        repository: Name,
        opener: u8,
    },
    /// Take `bytes` at the end of `session`.
    Append {
        session: SessionTag,
        bytes: Bytes,
    },
    /// End `session`, freeing what it staged.
    End {
        session: SessionTag,
    },
    /// End `session`, linking its content in its repository as `digest`.
    Complete {
        session: SessionTag,
        digest: Digest,
        content: Bytes,
    },
    /// Link `content` in `repository` as `digest`.
    Link {
        repository: Name,
        digest: Digest,
        content: Bytes,
    },
    /// Keep `manifest` in `repository` as `digest`, and point `tag` at it.
    Store {
        repository: Name,
        digest: Digest,
        manifest: Stored,
        tag: Option<Tag>,
    },
    /// Remove `tag`.
    Untag {
        repository: Name,
        tag: Tag,
    },
    /// Remove the manifest `digest` and every tag on it.
    Remove {
        repository: Name,
        digest: Digest,
    },
    /// Remove the repository's link to the blob `digest`.
    Unlink {
        repository: Name,
        digest: Digest,
    },
}

/// A request's answer and what it changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub answer: Answer,
    pub change: Change,
}

/// An invariant the state broke (plan T-RQ0.5.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Broken {
    /// A tag names a manifest its repository lacks.
    DanglingTag { repository: Name, tag: Tag },
    /// A manifest lists a blob its repository does not link.
    MissingBlob {
        repository: Name,
        manifest: Digest,
        blob: Digest,
    },
    /// An index lists a manifest its repository lacks.
    MissingChild {
        repository: Name,
        manifest: Digest,
        child: Digest,
    },
    /// A linked blob's bytes are not its digest's.
    BlobContent { repository: Name, blob: Digest },
    /// A manifest's bytes are not its digest's.
    ManifestContent { repository: Name, manifest: Digest },
    /// A session's repository does not exist.
    OrphanSession { session: SessionTag },
    /// A principal's open uploads stage more than the budget.
    OverBudget { principal: u8, staged: u64 },
}

/// The model: a stated configuration and the state its history made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    config: Arc<Config>,
    state: State,
}

/// Two models of one configuration are told apart by their state alone; the linearizability
/// checker hashes states (crate::check).
impl Hash for Model {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.state.hash(hasher);
    }
}

fn len64(bytes: &[u8]) -> u64 {
    // usize is at most 64 bits on every target Reliquary builds for (CLAUDE.md §12).
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

const fn answer_only(answer: Answer) -> Decision {
    Decision {
        answer,
        change: Change::None,
    }
}

fn digest_of(text: &str) -> Result<Digest, Answer> {
    Digest::parse(text).map_err(|why| match why {
        DigestError::Malformed => Answer::refusal(Status::BadRequest, ErrorCode::DigestInvalid),
        DigestError::Unregistered => Answer::refusal(Status::BadRequest, ErrorCode::Unsupported),
    })
}

fn reference_of(text: &str) -> Result<Reference, Answer> {
    Reference::parse(text).map_err(|why| match why {
        ReferenceError::Digest(DigestError::Malformed) => {
            Answer::refusal(Status::BadRequest, ErrorCode::DigestInvalid)
        }
        ReferenceError::Digest(DigestError::Unregistered) => {
            Answer::refusal(Status::BadRequest, ErrorCode::Unsupported)
        }
        ReferenceError::Tag => Answer::refusal(Status::BadRequest, ErrorCode::ManifestInvalid),
    })
}

fn span_of(range: &ChunkRange) -> Result<Option<(u64, u64)>, Answer> {
    match range {
        ChunkRange::Absent => Ok(None),
        ChunkRange::Span { first, last } => Ok(Some((*first, *last))),
        ChunkRange::Malformed(_) => Err(Answer::refusal(
            Status::BadRequest,
            ErrorCode::BlobUploadInvalid,
        )),
    }
}

/// A chunk at `offset` fits when its range, if it has one, starts at the offset (else `416`,
/// distribution spec v1.1.1) and spans exactly its bytes (else `SIZE_INVALID`).
fn fits(span: Option<(u64, u64)>, offset: u64, body: &[u8]) -> Result<(), Answer> {
    let Some((first, last)) = span else {
        return Ok(());
    };
    if first > last || first != offset {
        return Err(Answer::refusal(
            Status::RangeNotSatisfiable,
            ErrorCode::BlobUploadInvalid,
        ));
    }
    let spanned = last.checked_sub(first).and_then(|n| n.checked_add(1));
    if spanned != Some(len64(body)) {
        return Err(Answer::refusal(Status::BadRequest, ErrorCode::SizeInvalid));
    }
    Ok(())
}

/// A blob read with a `Range` header, by RFC 9110 §14 (model.md, H9; research/10 F14 to F17): an
/// invalid range (bounds reversed) is ignored and the whole blob sent, as §14.2 lets a server do
/// and names no status for refusing it; a valid range that no byte of the blob satisfies is `416`
/// with the blob's length; a suffix of an empty blob, satisfiable yet stated by no `Content-Range`,
/// is ignored too.
fn ranged(digest: Digest, content: &Bytes, range: ByteRange) -> Answer {
    let size = len64(content);
    let whole = |digest| {
        Answer::success(
            Status::Ok,
            Body::Blob {
                digest,
                size,
                content: Some(Arc::clone(content)),
            },
        )
    };
    let span = match range {
        ByteRange::From {
            first,
            last: Some(last),
        } if last < first => return whole(digest),
        ByteRange::From { first, last } => size
            .checked_sub(1)
            .filter(|end| first <= *end)
            .map(|end| (first, last.map_or(end, |last| last.min(end)))),
        ByteRange::Suffix { length: 0 } => None,
        ByteRange::Suffix { .. } if size == 0 => return whole(digest),
        ByteRange::Suffix { length } => size.checked_sub(length.min(size)).zip(size.checked_sub(1)),
    };
    let part = span.and_then(|(first, last)| {
        let from = usize::try_from(first).ok()?;
        let to = usize::try_from(last).ok()?;
        content.get(from..=to).map(|bytes| (first, last, bytes))
    });
    match part {
        Some((first, last, bytes)) => Answer::success(
            Status::PartialContent,
            Body::Part {
                digest,
                size,
                first,
                last,
                content: bytes.into(),
            },
        ),
        None => Answer::success(Status::RangeNotSatisfiable, Body::Unsatisfiable { size }),
    }
}

/// A manifest's `mediaType` field, when it has one, agrees with its `Content-Type` (plan RQ1.5).
fn agrees(media_type: Option<&String>, content_type: &str) -> Result<(), Answer> {
    match media_type {
        Some(field) if !grammar::essence(field).eq_ignore_ascii_case(content_type) => Err(
            Answer::refusal(Status::BadRequest, ErrorCode::ManifestInvalid),
        ),
        _ => Ok(()),
    }
}

/// Each descriptor's digest parsed: `None` for a registered-looking digest of an algorithm the
/// image spec does not register, which validates (descriptor.md) and names nothing a registry can
/// hold; a malformed one makes the manifest invalid.
fn parse_all(descriptors: &[&Descriptor]) -> Result<Vec<Option<Digest>>, Answer> {
    descriptors
        .iter()
        .map(|d| match Digest::parse(&d.digest) {
            Ok(digest) => Ok(Some(digest)),
            Err(DigestError::Unregistered) => Ok(None),
            Err(DigestError::Malformed) => Err(Answer::refusal(
                Status::BadRequest,
                ErrorCode::ManifestInvalid,
            )),
        })
        .collect()
}

fn subject_well_formed(subject: Option<&Descriptor>) -> Result<(), Answer> {
    match subject.map(|s| Digest::parse(&s.digest)) {
        Some(Err(DigestError::Malformed)) => Err(Answer::refusal(
            Status::BadRequest,
            ErrorCode::ManifestInvalid,
        )),
        _ => Ok(()),
    }
}

/// The referenced content, in document order: each must be present with the size its descriptor
/// states (model.md, H5), else the first that is not is named.
fn present(
    descriptors: &[&Descriptor],
    parsed: Vec<Option<Digest>>,
    size_of: impl Fn(&Digest) -> Option<u64>,
) -> Result<Vec<Digest>, Answer> {
    let mut found = Vec::new();
    for (descriptor, digest) in descriptors.iter().zip(parsed) {
        let named = || Body::Reference {
            digest: descriptor.digest.clone(),
        };
        let Some(digest) = digest else {
            return Err(Answer {
                status: Status::BadRequest,
                error: Some(ErrorCode::ManifestBlobUnknown),
                body: named(),
            });
        };
        match size_of(&digest) {
            None => {
                return Err(Answer {
                    status: Status::BadRequest,
                    error: Some(ErrorCode::ManifestBlobUnknown),
                    body: named(),
                });
            }
            Some(size) if size != descriptor.size => {
                return Err(Answer {
                    status: Status::BadRequest,
                    error: Some(ErrorCode::SizeInvalid),
                    body: named(),
                });
            }
            Some(_) => found.push(digest),
        }
    }
    Ok(found)
}

fn nonempty(text: Option<&String>) -> Option<String> {
    text.filter(|t| !t.is_empty()).cloned()
}

/// A manifest checked against its repository (model.md, H4 and H5): its kind from its
/// `Content-Type`, its fields against its kind, then everything it references.
fn checked(repository: Option<&Repository>, manifest: &Manifest) -> Result<Stored, Answer> {
    let content_type = grammar::essence(&manifest.content_type).to_owned();
    let invalid = || Answer::refusal(Status::BadRequest, ErrorCode::ManifestInvalid);
    let unchecked = |subject, artifact_type, annotations| Stored {
        content_type: content_type.clone(),
        bytes: Arc::clone(&manifest.bytes),
        blobs: Vec::new(),
        children: Vec::new(),
        subject,
        artifact_type,
        annotations,
    };
    match (grammar::manifest_kind(&content_type), &manifest.body) {
        (ManifestKind::Unknown, _) => Ok(unchecked(None, None, Vec::new())),
        (ManifestKind::Image, ManifestBody::Image(image)) => {
            agrees(image.media_type.as_ref(), &content_type)?;
            let listed: Vec<&Descriptor> = iter::once(&image.config)
                .chain(image.layers.iter())
                .collect();
            let parsed = parse_all(&listed)?;
            subject_well_formed(image.subject.as_ref())?;
            let blobs = present(&listed, parsed, |digest| {
                repository
                    .and_then(|r| r.blobs.get(digest))
                    .map(|b| len64(b))
            })?;
            let artifact_type = nonempty(image.artifact_type.as_ref())
                .or_else(|| Some(image.config.media_type.clone()));
            Ok(Stored {
                blobs,
                ..unchecked(
                    image.subject.as_ref().map(|s| s.digest.clone()),
                    artifact_type,
                    image.annotations.clone(),
                )
            })
        }
        (ManifestKind::Index, ManifestBody::Index(index)) => {
            agrees(index.media_type.as_ref(), &content_type)?;
            let listed: Vec<&Descriptor> = index.manifests.iter().collect();
            let parsed = parse_all(&listed)?;
            subject_well_formed(index.subject.as_ref())?;
            let children = present(&listed, parsed, |digest| {
                repository
                    .and_then(|r| r.manifests.get(digest))
                    .map(|m| len64(&m.bytes))
            })?;
            Ok(Stored {
                children,
                ..unchecked(
                    index.subject.as_ref().map(|s| s.digest.clone()),
                    nonempty(index.artifact_type.as_ref()),
                    index.annotations.clone(),
                )
            })
        }
        // Schema 1, or bytes that are not the manifest their media type says.
        _ => Err(invalid()),
    }
}

impl Model {
    /// An empty registry under `config`.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            state: State::default(),
        }
    }

    /// What the run stated.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The repository `name`, if anything was ever written to it.
    pub fn repository(&self, name: &Name) -> Option<&Repository> {
        self.state.repositories.get(name)
    }

    /// Every repository, by name.
    pub fn repositories(&self) -> impl Iterator<Item = (&Name, &Repository)> {
        self.state.repositories.iter()
    }

    /// Every open upload session, by tag.
    pub fn sessions(&self) -> impl Iterator<Item = (&SessionTag, &Session)> {
        self.state.sessions.iter()
    }

    /// The open session `session`.
    pub fn session(&self, session: SessionTag) -> Option<&Session> {
        self.state.sessions.get(&session)
    }

    /// What `principal`'s open uploads hold, in bytes.
    pub fn staged(&self, principal: u8) -> u64 {
        // A sum of lengths in memory cannot pass u64; saturation is the stated meaning.
        self.state
            .sessions
            .values()
            .filter(|s| s.opener == principal)
            .fold(0, |sum: u64, s| sum.saturating_add(len64(&s.content)))
    }

    /// The answer to `request` in this state, and what it changes.
    pub fn decide(&self, request: &Request) -> Decision {
        self.decide_or_refuse(request).unwrap_or_else(answer_only)
    }

    /// Answers `request` and makes its change.
    pub fn step(&mut self, request: &Request) -> Answer {
        let Decision { answer, change } = self.decide(request);
        self.apply(change);
        answer
    }

    /// Makes `change`.
    pub fn apply(&mut self, change: Change) {
        let state = &mut self.state;
        match change {
            Change::None => {}
            Change::Open {
                session,
                repository,
                opener,
            } => {
                state.repositories.entry(repository.clone()).or_default();
                state.sessions.insert(
                    session,
                    Session {
                        repository,
                        opener,
                        content: Vec::new(),
                    },
                );
            }
            Change::Append { session, bytes } => {
                if let Some(open) = state.sessions.get_mut(&session) {
                    open.content.extend_from_slice(&bytes);
                }
            }
            Change::End { session } => {
                state.sessions.remove(&session);
            }
            Change::Complete {
                session,
                digest,
                content,
            } => {
                if let Some(open) = state.sessions.remove(&session) {
                    state
                        .repositories
                        .entry(open.repository)
                        .or_default()
                        .blobs
                        .insert(digest, content);
                }
            }
            Change::Link {
                repository,
                digest,
                content,
            } => {
                state
                    .repositories
                    .entry(repository)
                    .or_default()
                    .blobs
                    .insert(digest, content);
            }
            Change::Store {
                repository,
                digest,
                manifest,
                tag,
            } => {
                let repository = state.repositories.entry(repository).or_default();
                repository.manifests.insert(digest.clone(), manifest);
                if let Some(tag) = tag {
                    repository.tags.insert(tag, digest);
                }
            }
            Change::Untag { repository, tag } => {
                if let Some(repository) = state.repositories.get_mut(&repository) {
                    repository.tags.remove(&tag);
                }
            }
            Change::Remove { repository, digest } => {
                if let Some(repository) = state.repositories.get_mut(&repository) {
                    repository.manifests.remove(&digest);
                    repository.tags.retain(|_, named| *named != digest);
                }
            }
            Change::Unlink { repository, digest } => {
                if let Some(repository) = state.repositories.get_mut(&repository) {
                    repository.blobs.remove(&digest);
                }
            }
        }
    }

    /// The model's invariants (plan T-RQ0.5.1, AC-T9.7.4): a tag names a manifest that exists; a
    /// manifest's blobs and an index's manifests exist; what is stored is what its digest names;
    /// every session's repository exists; no principal stages past the budget.
    pub fn check(&self) -> Result<(), Broken> {
        for (name, repository) in &self.state.repositories {
            for (tag, digest) in &repository.tags {
                if !repository.manifests.contains_key(digest) {
                    return Err(Broken::DanglingTag {
                        repository: name.clone(),
                        tag: tag.clone(),
                    });
                }
            }
            for (digest, manifest) in &repository.manifests {
                if Digest::of(digest.algorithm(), &manifest.bytes) != *digest {
                    return Err(Broken::ManifestContent {
                        repository: name.clone(),
                        manifest: digest.clone(),
                    });
                }
                if let Some(blob) = manifest
                    .blobs
                    .iter()
                    .find(|b| !repository.blobs.contains_key(*b))
                {
                    return Err(Broken::MissingBlob {
                        repository: name.clone(),
                        manifest: digest.clone(),
                        blob: blob.clone(),
                    });
                }
                if let Some(child) = manifest
                    .children
                    .iter()
                    .find(|c| !repository.manifests.contains_key(*c))
                {
                    return Err(Broken::MissingChild {
                        repository: name.clone(),
                        manifest: digest.clone(),
                        child: child.clone(),
                    });
                }
            }
            for (digest, content) in &repository.blobs {
                if Digest::of(digest.algorithm(), content) != *digest {
                    return Err(Broken::BlobContent {
                        repository: name.clone(),
                        blob: digest.clone(),
                    });
                }
            }
        }
        for (session, open) in &self.state.sessions {
            if !self.state.repositories.contains_key(&open.repository) {
                return Err(Broken::OrphanSession { session: *session });
            }
            let staged = self.staged(open.opener);
            if self
                .config
                .staging_budget
                .is_some_and(|budget| staged > budget)
            {
                return Err(Broken::OverBudget {
                    principal: open.opener,
                    staged,
                });
            }
        }
        Ok(())
    }

    fn name(&self, text: &str) -> Result<Name, Answer> {
        Name::parse(text, self.config.name_bound)
            .ok_or(Answer::refusal(Status::BadRequest, ErrorCode::NameInvalid))
    }

    fn authenticate(who: Principal) -> Result<u8, Answer> {
        match who {
            Principal::Anonymous => Err(Answer::refusal(
                Status::Unauthorized,
                ErrorCode::Unauthorized,
            )),
            Principal::Named(id) => Ok(id),
        }
    }

    fn may(&self, who: Principal, name: &Name, action: Action) -> bool {
        matches!(who, Principal::Named(id) if self.config.grants.allows(id, name, action))
    }

    /// The principal, when it may do `action` to `name`: `401` without credentials, `403`
    /// without the grant, whether or not the repository exists (architecture D18).
    fn allow(&self, who: Principal, name: &Name, action: Action) -> Result<u8, Answer> {
        let id = Self::authenticate(who)?;
        if self.config.grants.allows(id, name, action) {
            Ok(id)
        } else {
            Err(Answer::refusal(Status::Forbidden, ErrorCode::Denied))
        }
    }

    fn existing(&self, name: &Name) -> Result<&Repository, Answer> {
        self.state
            .repositories
            .get(name)
            .ok_or(Answer::refusal(Status::NotFound, ErrorCode::NameUnknown))
    }

    fn open(&self, session: SessionTag) -> Result<&Session, Answer> {
        self.state.sessions.get(&session).ok_or(Answer::refusal(
            Status::NotFound,
            ErrorCode::BlobUploadUnknown,
        ))
    }

    /// Whether `principal` may stage `more` bytes (model.md, H7).
    fn admit(&self, principal: u8, more: &[u8]) -> Result<(), Answer> {
        let Some(budget) = self.config.staging_budget else {
            return Ok(());
        };
        match self.staged(principal).checked_add(len64(more)) {
            Some(total) if total <= budget => Ok(()),
            _ => Err(Answer::refusal(
                Status::TooManyRequests,
                ErrorCode::TooManyRequests,
            )),
        }
    }

    /// The decision, or the refusal that ends it. Each request is checked in one order (model.md,
    /// H2): the text it carries, then who asks, then what exists, then what it would do.
    fn decide_or_refuse(&self, request: &Request) -> Result<Decision, Answer> {
        let who = request.principal;
        match &request.op {
            Op::ApiVersion => {
                Self::authenticate(who)?;
                Ok(answer_only(Answer::success(Status::Ok, Body::Empty)))
            }
            Op::GetBlob {
                name,
                digest,
                range,
            } => self.read_blob(who, name, digest, Some(*range)),
            Op::HeadBlob { name, digest } => self.read_blob(who, name, digest, None),
            Op::GetManifest { name, reference } => self.read_manifest(who, name, reference, true),
            Op::HeadManifest { name, reference } => self.read_manifest(who, name, reference, false),
            Op::StartUpload { name, session } => {
                let name = self.name(name)?;
                let opener = self.allow(who, &name, Action::Push)?;
                Ok(Decision {
                    answer: Answer::success(
                        Status::Accepted,
                        Body::Upload {
                            session: *session,
                            offset: 0,
                        },
                    ),
                    change: Change::Open {
                        session: *session,
                        repository: name,
                        opener,
                    },
                })
            }
            Op::PostBlob { name, digest, body } => {
                let name = self.name(name)?;
                let digest = digest_of(digest)?;
                let opener = self.allow(who, &name, Action::Push)?;
                self.admit(opener, body)?;
                if Digest::of(digest.algorithm(), body) != digest {
                    return Err(Answer::refusal(
                        Status::BadRequest,
                        ErrorCode::DigestInvalid,
                    ));
                }
                Ok(Decision {
                    answer: Answer::success(
                        Status::Created,
                        Body::BlobCreated {
                            digest: digest.clone(),
                        },
                    ),
                    change: Change::Link {
                        repository: name,
                        digest,
                        content: Arc::clone(body),
                    },
                })
            }
            Op::PatchUpload {
                session,
                range,
                body,
            } => {
                let span = span_of(range)?;
                Self::authenticate(who)?;
                let open = self.open(*session)?;
                self.allow(who, &open.repository, Action::Push)?;
                let offset = len64(&open.content);
                fits(span, offset, body)?;
                self.admit(open.opener, body)?;
                Ok(Decision {
                    answer: Answer::success(
                        Status::Accepted,
                        Body::Upload {
                            session: *session,
                            offset: offset.saturating_add(len64(body)),
                        },
                    ),
                    change: Change::Append {
                        session: *session,
                        bytes: Arc::clone(body),
                    },
                })
            }
            Op::PutUpload {
                session,
                digest,
                chunk,
            } => self.close_upload(who, *session, digest.as_deref(), chunk.as_ref()),
            Op::GetUpload { session } => {
                Self::authenticate(who)?;
                let open = self.open(*session)?;
                self.allow(who, &open.repository, Action::Push)?;
                Ok(answer_only(Answer::success(
                    Status::NoContent,
                    Body::Upload {
                        session: *session,
                        offset: len64(&open.content),
                    },
                )))
            }
            Op::CancelUpload { session } => {
                Self::authenticate(who)?;
                let open = self.open(*session)?;
                self.allow(who, &open.repository, Action::Push)?;
                Ok(Decision {
                    answer: Answer::success(Status::NoContent, Body::Empty),
                    change: Change::End { session: *session },
                })
            }
            Op::MountBlob {
                name,
                digest,
                from,
                session,
            } => self.mount(who, name, digest, from.as_deref(), *session),
            Op::PutManifest {
                name,
                reference,
                manifest,
            } => self.put_manifest(who, name, reference, manifest),
            Op::DeleteManifest { name, reference } => self.delete_manifest(who, name, reference),
            Op::DeleteBlob { name, digest } => {
                let name = self.name(name)?;
                let digest = digest_of(digest)?;
                self.allow(who, &name, Action::Delete)?;
                let repository = self.existing(&name)?;
                if !repository.blobs.contains_key(&digest) {
                    return Err(Answer::refusal(Status::NotFound, ErrorCode::BlobUnknown));
                }
                if repository
                    .manifests
                    .values()
                    .any(|m| m.blobs.contains(&digest))
                {
                    return Err(Answer::refusal(
                        Status::MethodNotAllowed,
                        ErrorCode::Unsupported,
                    ));
                }
                Ok(Decision {
                    answer: Answer::success(Status::Accepted, Body::Empty),
                    change: Change::Unlink {
                        repository: name,
                        digest,
                    },
                })
            }
            Op::ListTags { name, n, last } => self.list_tags(who, name, *n, last.as_deref()),
            Op::ListReferrers {
                name,
                digest,
                artifact_type,
            } => self.list_referrers(who, name, digest, artifact_type.as_ref()),
        }
    }

    /// `GET` (with `get` holding its range, if any) or `HEAD` of a blob.
    fn read_blob(
        &self,
        who: Principal,
        name: &str,
        digest: &str,
        get: Option<Option<ByteRange>>,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let digest = digest_of(digest)?;
        self.allow(who, &name, Action::Pull)?;
        let content = self
            .existing(&name)?
            .blobs
            .get(&digest)
            .ok_or(Answer::refusal(Status::NotFound, ErrorCode::BlobUnknown))?;
        let size = len64(content);
        let answer = match get {
            None => Answer::success(
                Status::Ok,
                Body::Blob {
                    digest,
                    size,
                    content: None,
                },
            ),
            Some(None) => Answer::success(
                Status::Ok,
                Body::Blob {
                    digest,
                    size,
                    content: Some(Arc::clone(content)),
                },
            ),
            Some(Some(range)) => ranged(digest, content, range),
        };
        Ok(answer_only(answer))
    }

    fn read_manifest(
        &self,
        who: Principal,
        name: &str,
        reference: &str,
        get: bool,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let reference = reference_of(reference)?;
        self.allow(who, &name, Action::Pull)?;
        let repository = self.existing(&name)?;
        let unknown = || Answer::refusal(Status::NotFound, ErrorCode::ManifestUnknown);
        let digest = match reference {
            Reference::Tag(tag) => repository.tags.get(&tag).cloned().ok_or_else(unknown)?,
            Reference::Digest(digest) => digest,
        };
        let stored = repository.manifests.get(&digest).ok_or_else(unknown)?;
        Ok(answer_only(Answer::success(
            Status::Ok,
            Body::Manifest {
                digest,
                content_type: stored.content_type.clone(),
                size: len64(&stored.bytes),
                content: get.then(|| Arc::clone(&stored.bytes)),
            },
        )))
    }

    /// The closing `PUT`: the digest parameter, the last chunk if any, then the whole content
    /// against the digest. Content that is not the digest's ends the session and frees what it
    /// staged (plan AC-RQ1.3.3).
    fn close_upload(
        &self,
        who: Principal,
        session: SessionTag,
        digest: Option<&str>,
        chunk: Option<&Chunk>,
    ) -> Result<Decision, Answer> {
        let digest = digest_of(digest.ok_or(Answer::refusal(
            Status::BadRequest,
            ErrorCode::DigestInvalid,
        ))?)?;
        let span = match chunk {
            Some(chunk) => span_of(&chunk.range)?,
            None => None,
        };
        Self::authenticate(who)?;
        let open = self.open(session)?;
        self.allow(who, &open.repository, Action::Push)?;
        let mut content = open.content.clone();
        if let Some(chunk) = chunk {
            fits(span, len64(&open.content), &chunk.body)?;
            self.admit(open.opener, &chunk.body)?;
            content.extend_from_slice(&chunk.body);
        }
        if Digest::of(digest.algorithm(), &content) != digest {
            return Ok(Decision {
                answer: Answer::refusal(Status::BadRequest, ErrorCode::DigestInvalid),
                change: Change::End { session },
            });
        }
        Ok(Decision {
            answer: Answer::success(
                Status::Created,
                Body::BlobCreated {
                    digest: digest.clone(),
                },
            ),
            change: Change::Complete {
                session,
                digest,
                content: content.into(),
            },
        })
    }

    /// A mount links the blob only from a repository the caller may read that links it; anything
    /// else opens an upload, saying nothing of where the blob is (architecture D18, plan
    /// AC-RQ1.3.7; model.md, H8).
    fn mount(
        &self,
        who: Principal,
        name: &str,
        digest: &str,
        from: Option<&str>,
        session: SessionTag,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let digest = digest_of(digest)?;
        let from = from.map(|text| self.name(text)).transpose()?;
        let opener = self.allow(who, &name, Action::Push)?;
        let source = from
            .filter(|from| self.may(who, from, Action::Pull))
            .and_then(|from| self.state.repositories.get(&from))
            .and_then(|from| from.blobs.get(&digest));
        Ok(match source {
            Some(content) => Decision {
                answer: Answer::success(
                    Status::Created,
                    Body::BlobCreated {
                        digest: digest.clone(),
                    },
                ),
                change: Change::Link {
                    repository: name,
                    digest,
                    content: Arc::clone(content),
                },
            },
            None => Decision {
                answer: Answer::success(Status::Accepted, Body::Upload { session, offset: 0 }),
                change: Change::Open {
                    session,
                    repository: name,
                    opener,
                },
            },
        })
    }

    fn put_manifest(
        &self,
        who: Principal,
        name: &str,
        reference: &str,
        manifest: &Manifest,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let reference = reference_of(reference)?;
        self.allow(who, &name, Action::Push)?;
        if self
            .config
            .manifest_bound
            .is_some_and(|bound| len64(&manifest.bytes) > bound)
        {
            return Err(Answer::refusal(
                Status::ContentTooLarge,
                ErrorCode::ManifestInvalid,
            ));
        }
        let (digest, tag) = match reference {
            Reference::Digest(digest) => {
                if Digest::of(digest.algorithm(), &manifest.bytes) != digest {
                    return Err(Answer::refusal(
                        Status::BadRequest,
                        ErrorCode::DigestInvalid,
                    ));
                }
                (digest, None)
            }
            Reference::Tag(tag) => (Digest::of(Algorithm::Sha256, &manifest.bytes), Some(tag)),
        };
        let stored = checked(self.state.repositories.get(&name), manifest)?;
        Ok(Decision {
            answer: Answer::success(
                Status::Created,
                Body::ManifestCreated {
                    digest: digest.clone(),
                    subject: stored.subject.clone(),
                },
            ),
            change: Change::Store {
                repository: name,
                digest,
                manifest: stored,
                tag,
            },
        })
    }

    /// By tag, the tag alone; by digest, the manifest and every tag on it, unless an index of the
    /// repository lists it (model.md, H5).
    fn delete_manifest(
        &self,
        who: Principal,
        name: &str,
        reference: &str,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let reference = reference_of(reference)?;
        self.allow(who, &name, Action::Delete)?;
        let repository = self.existing(&name)?;
        let unknown = Answer::refusal(Status::NotFound, ErrorCode::ManifestUnknown);
        let change = match reference {
            Reference::Tag(tag) => {
                if !repository.tags.contains_key(&tag) {
                    return Err(unknown);
                }
                Change::Untag {
                    repository: name,
                    tag,
                }
            }
            Reference::Digest(digest) => {
                if !repository.manifests.contains_key(&digest) {
                    return Err(unknown);
                }
                if repository
                    .manifests
                    .values()
                    .any(|m| m.children.contains(&digest))
                {
                    return Err(Answer::refusal(
                        Status::MethodNotAllowed,
                        ErrorCode::Unsupported,
                    ));
                }
                Change::Remove {
                    repository: name,
                    digest,
                }
            }
        };
        Ok(Decision {
            answer: Answer::success(Status::Accepted, Body::Empty),
            change,
        })
    }

    /// Tags after `last`, in listing order, at most `n`; `more` when `n` cut the list short. With
    /// `n` zero the list is empty and has no `Link` (distribution spec v1.1.1, "Listing Tags").
    fn list_tags(
        &self,
        who: Principal,
        name: &str,
        n: Option<u32>,
        last: Option<&str>,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        self.allow(who, &name, Action::Pull)?;
        let repository = self.existing(&name)?;
        let mut after = repository
            .tags
            .keys()
            .filter(|tag| last.is_none_or(|last| grammar::tag_order(tag.as_str(), last).is_gt()));
        let (tags, more) = match n.map(usize::try_from) {
            None => (after.cloned().collect(), false),
            Some(Ok(0)) => (Vec::new(), false),
            Some(Ok(n)) => {
                let page: Vec<Tag> = after.by_ref().take(n).cloned().collect();
                (page, after.next().is_some())
            }
            // An `n` past usize asks for more than memory holds: every tag.
            Some(Err(_)) => (after.cloned().collect(), false),
        };
        Ok(answer_only(Answer::success(
            Status::Ok,
            Body::Tags { name, tags, more },
        )))
    }

    /// Every manifest of the repository whose subject is `digest`, by digest, filtered by
    /// `artifactType` when asked. Never `404`: a repository that does not exist has none
    /// (distribution spec v1.1.1, "Listing Referrers"; model.md, H10).
    fn list_referrers(
        &self,
        who: Principal,
        name: &str,
        digest: &str,
        artifact_type: Option<&String>,
    ) -> Result<Decision, Answer> {
        let name = self.name(name)?;
        let digest = digest_of(digest)?;
        self.allow(who, &name, Action::Pull)?;
        let subject = digest.to_string();
        let referrers = self
            .state
            .repositories
            .get(&name)
            .map(|repository| {
                repository
                    .manifests
                    .iter()
                    .filter(|(_, m)| m.subject.as_deref() == Some(subject.as_str()))
                    .filter(|(_, m)| {
                        artifact_type.is_none_or(|t| m.artifact_type.as_ref() == Some(t))
                    })
                    .map(|(digest, m)| Referrer {
                        media_type: m.content_type.clone(),
                        digest: digest.clone(),
                        size: len64(&m.bytes),
                        artifact_type: m.artifact_type.clone(),
                        annotations: m.annotations.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(answer_only(Answer::success(
            Status::Ok,
            Body::Referrers {
                referrers,
                filtered: artifact_type.is_some(),
            },
        )))
    }
}
