//! The generator of histories (plan AC-RQ0.5.2; docs/design/model.md, H11). From a seed and a mix it
//! makes requests one at a time: most are valid, built from what a shadow copy of the model holds so
//! they meet existing repositories, blobs, manifests, tags and sessions; one in the mix's stated
//! number is broken in one of the ways the spec refuses. Every request steps the shadow, so the next
//! one is built on what the last one did. Sequential histories are one client's requests; concurrent
//! plans give each client its own requests, made in turn against the one shadow, and interleave them.
//!
//! Every figure a generation uses is its caller's, in [`Mix`], [`Shape`] and [`Race`]: the library
//! states none (docs/design/numbers.md). Choices among equals are uniform.

use std::collections::BTreeMap;

use hyper_sim::rng::{Seeded, stream_seed};

use crate::answer::Body;
use crate::grammar::{self, Algorithm, Digest, Name, Tag};
use crate::history::{Plan, PlanError};
use crate::json;
use crate::model::{Action, Config, Grants, Model, Repository, Stored};
use crate::request::{
    ByteRange, Bytes, Chunk, ChunkRange, Descriptor, ImageManifest, IndexManifest, Malformation,
    Manifest, ManifestBody, Op, Principal, Request, SessionTag,
};

/// The principal that owns every repository a generation names.
const OWNER: u8 = 0;

/// Shapes of valid repository names, one component or several, with each separator the grammar has.
const NAME_PREFIXES: &[&str] = &["", "org/", "a.b-c/", "x__y/"];
/// Names the grammar refuses, one way each (plan T-RQ1.1.2).
const INVALID_NAMES: &[&str] = &[
    "Upper", "-lead", "trail-", "a..b", "a//b", "a___b", "", "a/", "/a", "a_-b", "a.-b", "a b",
    "a:b",
];
/// Tags, with case variants so the listing order is exercised.
const TAG_NAMES: &[&str] = &["latest", "Latest", "v1", "v1.0", "V1.1", "_x", "a-b", "Z"];
/// Tags the grammar refuses; a tag one byte too long is added where they are used.
const INVALID_TAGS: &[&str] = &[".dot", "-dash", "bad tag", "\u{e9}", "a/b"];
const ARTIFACT_TYPES: &[&str] = &[
    "application/vnd.example.sbom.v1",
    "application/vnd.example.signature.v1",
];
const CONFIG_TYPES: &[&str] = &[
    "application/vnd.oci.image.config.v1+json",
    grammar::OCI_EMPTY,
];
const LAYER_TYPE: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
/// A manifest media type no specification defines: stored and served unchecked.
const UNKNOWN_MANIFEST: &str = "application/vnd.example.unknown.manifest.v1+json";
const ANNOTATION_KEYS: &[&str] = &["org.opencontainers.image.created", "com.example.note"];
const ANNOTATION_VALUES: &[&str] = &["2026-11-20T00:00:00Z", "a \"quoted\" \\ value"];
/// `Content-Range` values that do not match `^[0-9]+-[0-9]+$`.
const MALFORMED_RANGES: &[&str] = &["bytes 0-5/10", "5-", "-5", "a-b", "0-5,6-7", ""];
/// Bytes that are not JSON.
const NOT_JSON: &str = "{\"schemaVersion\":2,\"mediaType\":";
/// JSON that is no manifest: it lacks what every manifest media type requires.
const NOT_A_MANIFEST: &str = "{\"schemaVersion\":2}";
/// A digest of an algorithm the image spec does not register, from its own example (descriptor.md).
const MULTIHASH: &str = "multihash+base58:QmRZxt2b1FVZPNqd8hsiykDL3TdBDeTSPX9Kv46HmX4Gx8";

/// What a generation's histories are made of. Every figure is the caller's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mix {
    /// Repository names written to; as many more are named only to be read and stay unwritten.
    pub repositories: u8,
    /// Principals besides the owner, each granted each action on each repository by a fair coin.
    pub principals: u8,
    /// Tag names in play.
    pub tags: u8,
    /// Distinct blob contents; the first is empty.
    pub contents: u8,
    /// The most bytes a content has.
    pub largest_blob: u16,
    /// One request in this many is broken one way; zero breaks none.
    pub invalid_one_in: u32,
    /// The model's manifest bound for the run.
    pub manifest_bound: Option<u64>,
    /// The model's staging budget for the run.
    pub staging_budget: Option<u64>,
}

/// A concurrent plan's shape: a setup client's sequential requests, then clients with requests of
/// their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub clients: u16,
    pub setup: usize,
    pub per_client: usize,
    /// One request or answer in this many is lost; zero loses none.
    pub lose_one_in: u32,
}

/// A race on one tag (plan T-RQ0.5.4): writers moving it among manifests a setup pushed, readers
/// reading it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Race {
    pub writers: u16,
    pub readers: u16,
    /// Manifests the tag moves among.
    pub manifests: u8,
    /// Moves per writer.
    pub moves: usize,
    /// Reads per reader.
    pub reads: usize,
    pub lose_one_in: u32,
}

/// Requests that should succeed, given what the shadow holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Valid {
    ApiVersion,
    PostBlob,
    StartUpload,
    Patch,
    PatchStreamed,
    Close,
    Status,
    Cancel,
    MountFrom,
    MountBare,
    PutImage,
    PutIndex,
    PutOpaque,
    Retag,
    GetManifest,
    HeadManifest,
    GetBlob,
    HeadBlob,
    ListTags,
    ListReferrers,
    DeleteTag,
    DeleteManifest,
    DeleteBlob,
}

impl Valid {
    const ALL: &'static [Self] = &[
        Self::ApiVersion,
        Self::PostBlob,
        Self::StartUpload,
        Self::Patch,
        Self::PatchStreamed,
        Self::Close,
        Self::Status,
        Self::Cancel,
        Self::MountFrom,
        Self::MountBare,
        Self::PutImage,
        Self::PutIndex,
        Self::PutOpaque,
        Self::Retag,
        Self::GetManifest,
        Self::HeadManifest,
        Self::GetBlob,
        Self::HeadBlob,
        Self::ListTags,
        Self::ListReferrers,
        Self::DeleteTag,
        Self::DeleteManifest,
        Self::DeleteBlob,
    ];

    /// The ones whose requests name a repository.
    const NAMED: &'static [Self] = &[
        Self::PostBlob,
        Self::StartUpload,
        Self::MountFrom,
        Self::MountBare,
        Self::PutImage,
        Self::PutIndex,
        Self::PutOpaque,
        Self::Retag,
        Self::GetManifest,
        Self::HeadManifest,
        Self::GetBlob,
        Self::HeadBlob,
        Self::ListTags,
        Self::ListReferrers,
        Self::DeleteTag,
        Self::DeleteManifest,
        Self::DeleteBlob,
    ];

    /// The ones whose requests carry a digest, as a parameter or a reference.
    const DIGESTED: &'static [Self] = &[
        Self::PostBlob,
        Self::Close,
        Self::MountFrom,
        Self::MountBare,
        Self::PutImage,
        Self::GetManifest,
        Self::HeadManifest,
        Self::GetBlob,
        Self::HeadBlob,
        Self::ListReferrers,
        Self::DeleteManifest,
        Self::DeleteBlob,
    ];
}

/// Requests broken one way each, so that between them every refusal the model gives is produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Invalid {
    BadName,
    Anonymous,
    Ungranted,
    BadDigest,
    UnregisteredDigest,
    WrongContent,
    OutOfOrder,
    MalformedRange,
    RangeLength,
    UnknownSession,
    MissingBlob,
    WrongSize,
    NotJson,
    NotAManifest,
    MediaTypeMismatch,
    Schema1,
    TooLarge,
    DigestMismatch,
    BadTag,
    AbsentManifest,
    AbsentBlob,
    Unwritten,
    ReferencedDelete,
    OverBudget,
    BadRange,
    NoDigest,
    MissingChild,
    UnregisteredDescriptor,
    BadDescriptor,
}

impl Invalid {
    const ALL: &'static [Self] = &[
        Self::BadName,
        Self::Anonymous,
        Self::Ungranted,
        Self::BadDigest,
        Self::UnregisteredDigest,
        Self::WrongContent,
        Self::OutOfOrder,
        Self::MalformedRange,
        Self::RangeLength,
        Self::UnknownSession,
        Self::MissingBlob,
        Self::WrongSize,
        Self::NotJson,
        Self::NotAManifest,
        Self::MediaTypeMismatch,
        Self::Schema1,
        Self::TooLarge,
        Self::DigestMismatch,
        Self::BadTag,
        Self::AbsentManifest,
        Self::AbsentBlob,
        Self::Unwritten,
        Self::ReferencedDelete,
        Self::OverBudget,
        Self::BadRange,
        Self::NoDigest,
        Self::MissingChild,
        Self::UnregisteredDescriptor,
        Self::BadDescriptor,
    ];
}

/// How an image manifest the generator writes is made wrong, if it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flaw {
    None,
    MissingBlob,
    WrongSize,
    MediaTypeMismatch,
    UnregisteredDescriptor,
    BadDescriptor,
    TooLarge,
}

/// Uniform in `[0, n)`; zero when `n` is zero.
fn below(rng: &mut Seeded, n: usize) -> usize {
    usize::try_from(rng.below(u64::try_from(n).unwrap_or(u64::MAX))).unwrap_or(0)
}

/// A fair coin: the lowest bit of a draw.
fn coin(rng: &mut Seeded) -> bool {
    rng.next_u64() & 1 == 0
}

fn pick<'a, T>(rng: &mut Seeded, items: &'a [T]) -> Option<&'a T> {
    items.get(below(rng, items.len()))
}

fn len64(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

fn random_bytes(rng: &mut Seeded, length: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    while bytes.len() < length {
        bytes.extend(rng.next_u64().to_le_bytes());
    }
    bytes.truncate(length);
    bytes
}

/// `Content-Range` for `body` at `offset`; none for an empty body, which no range can state.
fn span(offset: u64, body: &[u8]) -> ChunkRange {
    match len64(body)
        .checked_sub(1)
        .and_then(|n| offset.checked_add(n))
    {
        Some(last) => ChunkRange::Span {
            first: offset,
            last,
        },
        None => ChunkRange::Absent,
    }
}

const fn owned(op: Op) -> Request {
    Request {
        principal: Principal::Named(OWNER),
        op,
    }
}

/// Puts `bad` where `op` names a repository (or, for a mount when `in_from`, its source).
fn rename(op: &mut Op, bad: String, in_from: bool) {
    match op {
        Op::MountBlob { from, .. } if in_from => *from = Some(bad),
        Op::GetBlob { name, .. }
        | Op::HeadBlob { name, .. }
        | Op::GetManifest { name, .. }
        | Op::HeadManifest { name, .. }
        | Op::StartUpload { name, .. }
        | Op::PostBlob { name, .. }
        | Op::MountBlob { name, .. }
        | Op::PutManifest { name, .. }
        | Op::DeleteManifest { name, .. }
        | Op::DeleteBlob { name, .. }
        | Op::ListTags { name, .. }
        | Op::ListReferrers { name, .. } => *name = bad,
        Op::ApiVersion
        | Op::PatchUpload { .. }
        | Op::PutUpload { .. }
        | Op::GetUpload { .. }
        | Op::CancelUpload { .. } => {}
    }
}

/// Puts `text` where `op` carries a digest.
fn redigest(op: &mut Op, text: String) {
    match op {
        Op::GetBlob { digest, .. }
        | Op::HeadBlob { digest, .. }
        | Op::PostBlob { digest, .. }
        | Op::DeleteBlob { digest, .. }
        | Op::MountBlob { digest, .. }
        | Op::ListReferrers { digest, .. } => *digest = text,
        Op::PutUpload { digest, .. } => *digest = Some(text),
        Op::GetManifest { reference, .. }
        | Op::HeadManifest { reference, .. }
        | Op::DeleteManifest { reference, .. }
        | Op::PutManifest { reference, .. } => *reference = text,
        Op::ApiVersion
        | Op::StartUpload { .. }
        | Op::PatchUpload { .. }
        | Op::GetUpload { .. }
        | Op::CancelUpload { .. }
        | Op::ListTags { .. } => {}
    }
}

/// The repository-wide names, tags and contents of a generation.
#[derive(Clone, Debug)]
struct World {
    names: Vec<String>,
    unwritten: Vec<String>,
    tags: Vec<String>,
    contents: Vec<Bytes>,
    principals: u8,
}

/// The generator: a seeded source of requests and the shadow model it builds them on.
#[derive(Clone, Debug)]
pub struct Generator {
    rng: Seeded,
    world: World,
    shadow: Model,
    sessions: u32,
    /// What each session the generator opened is meant to hold.
    intended: BTreeMap<SessionTag, Bytes>,
    /// Each manifest the generator pushed and the registry took, by digest.
    pushed: BTreeMap<Digest, Box<Manifest>>,
    invalid_one_in: u32,
}

impl Generator {
    /// The generator for `seed` under `mix`: its names, contents and grants are drawn first, from a
    /// stream of their own.
    pub fn new(seed: u64, mix: &Mix) -> Self {
        let mut rng = Seeded::new(stream_seed(seed, "reliquary.generate", &[]));
        let names: Vec<String> = (0..mix.repositories)
            .map(|i| {
                let prefix = usize::from(i)
                    .checked_rem(NAME_PREFIXES.len())
                    .and_then(|k| NAME_PREFIXES.get(k))
                    .copied()
                    .unwrap_or_default();
                format!("{prefix}r{i}")
            })
            .collect();
        let unwritten: Vec<String> = (0..mix.repositories).map(|i| format!("ghost{i}")).collect();
        let tags = (0..mix.tags)
            .map(|i| {
                TAG_NAMES
                    .get(usize::from(i))
                    .map_or_else(|| format!("t{i}"), |t| (*t).to_owned())
            })
            .collect();
        let mut contents = Vec::new();
        for k in 0..mix.contents {
            let length = if k == 0 {
                0
            } else {
                below(&mut rng, usize::from(mix.largest_blob).saturating_add(1))
            };
            contents.push(Bytes::from(random_bytes(&mut rng, length)));
        }
        let mut grants = Grants::default();
        for name in names.iter().chain(&unwritten) {
            for action in [Action::Pull, Action::Push, Action::Delete] {
                grants.grant(OWNER, name, action);
                for principal in 1..=mix.principals {
                    if coin(&mut rng) {
                        grants.grant(principal, name, action);
                    }
                }
            }
        }
        let config = Config {
            grants,
            manifest_bound: mix.manifest_bound,
            staging_budget: mix.staging_budget,
            name_bound: None,
        };
        Self {
            rng,
            world: World {
                names,
                unwritten,
                tags,
                contents,
                principals: mix.principals,
            },
            shadow: Model::new(config),
            sessions: 0,
            intended: BTreeMap::new(),
            pushed: BTreeMap::new(),
            invalid_one_in: mix.invalid_one_in,
        }
    }

    /// The configuration the generation's requests are meant for.
    pub fn config(&self) -> Config {
        self.shadow.config().clone()
    }

    /// The next request, after which the shadow holds what it did.
    pub fn next_request(&mut self) -> Request {
        let broken = self.invalid_one_in > 0 && self.rng.below(u64::from(self.invalid_one_in)) == 0;
        let request = if broken {
            let flaw = pick(&mut self.rng, Invalid::ALL)
                .copied()
                .unwrap_or(Invalid::Anonymous);
            self.invalid(flaw)
        } else {
            let intent = pick(&mut self.rng, Valid::ALL)
                .copied()
                .unwrap_or(Valid::ApiVersion);
            owned(self.valid(intent))
        };
        let answer = self.shadow.step(&request);
        if let (Op::PutManifest { manifest, .. }, Body::ManifestCreated { digest, .. }) =
            (&request.op, &answer.body)
        {
            self.pushed.insert(digest.clone(), manifest.clone());
        }
        request
    }

    fn name(&mut self) -> String {
        pick(&mut self.rng, &self.world.names)
            .cloned()
            .unwrap_or_default()
    }

    fn tag(&mut self) -> String {
        pick(&mut self.rng, &self.world.tags)
            .cloned()
            .unwrap_or_default()
    }

    fn content(&mut self) -> Bytes {
        pick(&mut self.rng, &self.world.contents)
            .cloned()
            .unwrap_or_else(|| Bytes::from(Vec::new()))
    }

    /// SHA-256 three times in four, SHA-512 the fourth (two fair coins).
    fn digest(&mut self, bytes: &[u8]) -> Digest {
        let algorithm = if coin(&mut self.rng) && coin(&mut self.rng) {
            Algorithm::Sha512
        } else {
            Algorithm::Sha256
        };
        Digest::of(algorithm, bytes)
    }

    /// A digest of bytes no request sends: a fresh draw, named as a stray.
    fn stray(&mut self) -> Digest {
        let mut bytes = b"stray ".to_vec();
        bytes.extend(self.rng.next_u64().to_le_bytes());
        Digest::of(Algorithm::Sha256, &bytes)
    }

    fn stray_descriptor(&mut self) -> Descriptor {
        Descriptor {
            media_type: LAYER_TYPE.to_owned(),
            digest: self.stray().to_string(),
            size: 0,
        }
    }

    /// The next session tag. A generation of more than `u32::MAX` sessions would hand out the last
    /// one again, which [`Plan::check`] refuses as a reused session.
    fn fresh_session(&mut self) -> SessionTag {
        let session = SessionTag(self.sessions);
        self.sessions = self.sessions.saturating_add(1);
        session
    }

    /// A tag, or else a digest under the generation's algorithms, of `bytes`.
    fn reference(&mut self, bytes: &[u8]) -> String {
        if coin(&mut self.rng) {
            self.tag()
        } else {
            self.digest(bytes).to_string()
        }
    }

    fn repository(&self, name: &str) -> Option<&Repository> {
        Name::parse(name, None).and_then(|name| self.shadow.repository(&name))
    }

    fn linked(&self, name: &str) -> Vec<(Digest, Bytes)> {
        self.repository(name)
            .map(|r| {
                r.blobs()
                    .map(|(d, b)| (d.clone(), Bytes::clone(b)))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn manifests(&self, name: &str) -> Vec<(Digest, Stored)> {
        self.repository(name)
            .map(|r| r.manifests().map(|(d, m)| (d.clone(), m.clone())).collect())
            .unwrap_or_default()
    }

    fn tagged(&self, name: &str) -> Vec<Tag> {
        self.repository(name)
            .map(|r| r.tags().map(|(t, _)| t.clone()).collect())
            .unwrap_or_default()
    }

    /// Open sessions the generator means to fill: the session, its offset, and what is still to
    /// send.
    fn unfinished(&self) -> Vec<(SessionTag, u64, Bytes)> {
        self.shadow
            .sessions()
            .filter_map(|(session, open)| {
                let meant = self.intended.get(session)?;
                let rest = meant.strip_prefix(open.content.as_slice())?;
                Some((*session, len64(&open.content), Bytes::from(rest)))
            })
            .collect()
    }

    fn intended_content(&self, session: SessionTag) -> Bytes {
        self.intended
            .get(&session)
            .cloned()
            .unwrap_or_else(|| Bytes::from(Vec::new()))
    }

    fn annotations(&mut self) -> Vec<(String, String)> {
        if coin(&mut self.rng) {
            let key = pick(&mut self.rng, ANNOTATION_KEYS)
                .copied()
                .unwrap_or_default();
            let value = pick(&mut self.rng, ANNOTATION_VALUES)
                .copied()
                .unwrap_or_default();
            vec![(key.to_owned(), value.to_owned())]
        } else {
            Vec::new()
        }
    }

    fn artifact_type(&mut self) -> Option<String> {
        if coin(&mut self.rng) {
            pick(&mut self.rng, ARTIFACT_TYPES).map(|t| (*t).to_owned())
        } else {
            None
        }
    }

    /// A subject: a manifest of the repository, or a digest nothing has, which the spec says a
    /// registry must take ("Pushing Manifests").
    fn subject(&mut self, name: &str) -> Descriptor {
        let stored = self.manifests(name);
        match pick(&mut self.rng, &stored) {
            Some((digest, manifest)) if coin(&mut self.rng) => Descriptor {
                media_type: manifest.content_type.clone(),
                digest: digest.to_string(),
                size: len64(&manifest.bytes),
            },
            _ => Descriptor {
                media_type: grammar::OCI_IMAGE_MANIFEST.to_owned(),
                digest: self.stray().to_string(),
                size: 0,
            },
        }
    }

    fn valid(&mut self, intent: Valid) -> Op {
        match intent {
            Valid::ApiVersion => Op::ApiVersion,
            Valid::PostBlob => {
                let name = self.name();
                self.post_blob(name)
            }
            Valid::StartUpload => self.start_upload(),
            Valid::Patch => self.patch(true),
            Valid::PatchStreamed => self.patch(false),
            Valid::Close => self.close(),
            Valid::Status => {
                let open: Vec<SessionTag> = self.shadow.sessions().map(|(s, _)| *s).collect();
                match pick(&mut self.rng, &open).copied() {
                    Some(session) => Op::GetUpload { session },
                    None => self.start_upload(),
                }
            }
            Valid::Cancel => {
                let open: Vec<SessionTag> = self.shadow.sessions().map(|(s, _)| *s).collect();
                match pick(&mut self.rng, &open).copied() {
                    Some(session) => Op::CancelUpload { session },
                    None => self.start_upload(),
                }
            }
            Valid::MountFrom => self.mount(true),
            Valid::MountBare => self.mount(false),
            Valid::PutImage => self.put_image(Flaw::None),
            Valid::PutIndex => self.put_index(false),
            Valid::PutOpaque => self.put_opaque(),
            Valid::Retag => self.retag(),
            Valid::GetManifest => self.read_manifest(true),
            Valid::HeadManifest => self.read_manifest(false),
            Valid::GetBlob => self.read_blob(true, false),
            Valid::HeadBlob => self.read_blob(false, false),
            Valid::ListTags => self.list_tags(),
            Valid::ListReferrers => self.list_referrers(),
            Valid::DeleteTag => self.delete_tag(),
            Valid::DeleteManifest => self.delete_manifest(),
            Valid::DeleteBlob => self.delete_blob(),
        }
    }

    fn post_blob(&mut self, name: String) -> Op {
        let body = self.content();
        let digest = self.digest(&body).to_string();
        Op::PostBlob { name, digest, body }
    }

    fn start_upload(&mut self) -> Op {
        let name = self.name();
        let session = self.fresh_session();
        let meant = self.content();
        self.intended.insert(session, meant);
        Op::StartUpload { name, session }
    }

    /// The next piece of an unfinished session: with a `Content-Range` (at least a byte) or
    /// streamed without one (possibly empty).
    fn patch(&mut self, ranged: bool) -> Op {
        let unfinished: Vec<(SessionTag, u64, Bytes)> = self
            .unfinished()
            .into_iter()
            .filter(|(_, _, rest)| !ranged || !rest.is_empty())
            .collect();
        let Some((session, offset, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let length = if ranged {
            below(&mut self.rng, rest.len()).saturating_add(1)
        } else {
            below(&mut self.rng, rest.len().saturating_add(1))
        };
        let body = Bytes::from(rest.get(..length).unwrap_or_default());
        let range = if ranged {
            span(offset, &body)
        } else {
            ChunkRange::Absent
        };
        Op::PatchUpload {
            session,
            range,
            body,
        }
    }

    /// The closing `PUT` of an unfinished session, with what is left as its last chunk.
    fn close(&mut self) -> Op {
        let unfinished = self.unfinished();
        let Some((session, offset, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let whole = self.intended_content(session);
        let digest = self.digest(&whole).to_string();
        let chunk = if rest.is_empty() {
            None
        } else {
            let range = if coin(&mut self.rng) {
                span(offset, &rest)
            } else {
                ChunkRange::Absent
            };
            Some(Chunk { range, body: rest })
        };
        Op::PutUpload {
            session,
            digest: Some(digest),
            chunk,
        }
    }

    /// A mount from a repository that links the blob (when `from_source` finds one), or with no
    /// source; the session it may open is meant to hold the blob.
    fn mount(&mut self, from_source: bool) -> Op {
        let name = self.name();
        let session = self.fresh_session();
        let sources: Vec<(String, Digest, Bytes)> = if from_source {
            self.world
                .names
                .iter()
                .flat_map(|n| {
                    self.linked(n)
                        .into_iter()
                        .map(move |(d, b)| (n.clone(), d, b))
                })
                .collect()
        } else {
            Vec::new()
        };
        let (digest, content, from) = match pick(&mut self.rng, &sources).cloned() {
            Some((from, digest, content)) => (digest, content, Some(from)),
            None => {
                let content = self.content();
                let digest = self.digest(&content);
                let from = if from_source { Some(self.name()) } else { None };
                (digest, content, from)
            }
        };
        self.intended.insert(session, content);
        Op::MountBlob {
            name,
            digest: digest.to_string(),
            from,
            session,
        }
    }

    /// An image manifest of blobs its repository links, made wrong by `flaw`.
    fn put_image(&mut self, flaw: Flaw) -> Op {
        let name = self.name();
        let described: Vec<Descriptor> = self
            .linked(&name)
            .iter()
            .map(|(digest, bytes)| Descriptor {
                media_type: LAYER_TYPE.to_owned(),
                digest: digest.to_string(),
                size: len64(bytes),
            })
            .collect();
        let config = match pick(&mut self.rng, &described).cloned() {
            Some(mut config) => {
                config.media_type = pick(&mut self.rng, CONFIG_TYPES)
                    .copied()
                    .unwrap_or(LAYER_TYPE)
                    .to_owned();
                config
            }
            None if flaw == Flaw::None => return self.post_blob(name),
            None => self.stray_descriptor(),
        };
        let count = below(&mut self.rng, described.len().saturating_add(1));
        let mut layers = Vec::new();
        for _ in 0..count {
            if let Some(layer) = pick(&mut self.rng, &described) {
                layers.push(layer.clone());
            }
        }
        let subject = if coin(&mut self.rng) {
            Some(self.subject(&name))
        } else {
            None
        };
        let artifact_type = self.artifact_type();
        let annotations = self.annotations();
        let content_type = if coin(&mut self.rng) && coin(&mut self.rng) {
            grammar::DOCKER_MANIFEST
        } else {
            grammar::OCI_IMAGE_MANIFEST
        };
        let media_type = coin(&mut self.rng).then(|| content_type.to_owned());
        let mut image = ImageManifest {
            media_type,
            artifact_type,
            config,
            layers,
            subject,
            annotations,
        };
        match flaw {
            Flaw::None | Flaw::TooLarge => {}
            Flaw::MissingBlob => image.layers.push(self.stray_descriptor()),
            Flaw::WrongSize => image.config.size = image.config.size.saturating_add(1),
            Flaw::MediaTypeMismatch => {
                image.media_type = Some(grammar::OCI_IMAGE_INDEX.to_owned());
            }
            Flaw::UnregisteredDescriptor => {
                let mut layer = self.stray_descriptor();
                layer.digest = self.unregistered_digest();
                image.layers.push(layer);
            }
            Flaw::BadDescriptor => {
                let mut layer = self.stray_descriptor();
                layer.digest = self.malformed_digest();
                image.layers.push(layer);
            }
        }
        let mut bytes = json::image(&image);
        if flaw == Flaw::TooLarge
            && let Some(bound) = self.shadow.config().manifest_bound
        {
            // One annotation long enough to take the bytes past the bound.
            let pad = usize::try_from(bound)
                .ok()
                .and_then(|bound| bound.checked_sub(bytes.len()))
                .and_then(|room| room.checked_add(1))
                .unwrap_or(0);
            image
                .annotations
                .push(("com.example.padding".to_owned(), "x".repeat(pad)));
            bytes = json::image(&image);
        }
        let reference = self.reference(&bytes);
        Op::PutManifest {
            name,
            reference,
            manifest: Box::new(Manifest {
                content_type: content_type.to_owned(),
                body: ManifestBody::Image(image),
                bytes: bytes.into(),
            }),
        }
    }

    /// An index of manifests its repository holds, and, when `missing_child`, one it does not.
    fn put_index(&mut self, missing_child: bool) -> Op {
        let name = self.name();
        let described: Vec<Descriptor> = self
            .manifests(&name)
            .iter()
            .map(|(digest, manifest)| Descriptor {
                media_type: manifest.content_type.clone(),
                digest: digest.to_string(),
                size: len64(&manifest.bytes),
            })
            .collect();
        if described.is_empty() && !missing_child {
            return self.put_image(Flaw::None);
        }
        let count = below(&mut self.rng, described.len()).saturating_add(1);
        let mut manifests = Vec::new();
        for _ in 0..count {
            if let Some(child) = pick(&mut self.rng, &described) {
                manifests.push(child.clone());
            }
        }
        if missing_child {
            let mut stray = self.stray_descriptor();
            stray.media_type = grammar::OCI_IMAGE_MANIFEST.to_owned();
            manifests.push(stray);
        }
        let subject = if coin(&mut self.rng) {
            Some(self.subject(&name))
        } else {
            None
        };
        let artifact_type = self.artifact_type();
        let annotations = self.annotations();
        let content_type = if coin(&mut self.rng) && coin(&mut self.rng) {
            grammar::DOCKER_MANIFEST_LIST
        } else {
            grammar::OCI_IMAGE_INDEX
        };
        let index = IndexManifest {
            media_type: coin(&mut self.rng).then(|| content_type.to_owned()),
            artifact_type,
            manifests,
            subject,
            annotations,
        };
        let bytes = json::index(&index);
        let reference = self.reference(&bytes);
        Op::PutManifest {
            name,
            reference,
            manifest: Box::new(Manifest {
                content_type: content_type.to_owned(),
                body: ManifestBody::Index(index),
                bytes: bytes.into(),
            }),
        }
    }

    fn put_opaque(&mut self) -> Op {
        let name = self.name();
        let bytes = format!("{{\"opaque\":{}}}", self.rng.next_u64()).into_bytes();
        let reference = self.reference(&bytes);
        Op::PutManifest {
            name,
            reference,
            manifest: Box::new(Manifest {
                content_type: UNKNOWN_MANIFEST.to_owned(),
                body: ManifestBody::Opaque,
                bytes: bytes.into(),
            }),
        }
    }

    /// A manifest the repository holds pushed again under a tag: the way a tag moves.
    fn retag(&mut self) -> Op {
        let name = self.name();
        let held: Vec<Box<Manifest>> = self
            .manifests(&name)
            .into_iter()
            .filter_map(|(digest, _)| self.pushed.get(&digest).cloned())
            .collect();
        let Some(manifest) = pick(&mut self.rng, &held).cloned() else {
            return self.put_image(Flaw::None);
        };
        let reference = self.tag();
        Op::PutManifest {
            name,
            reference,
            manifest,
        }
    }

    fn read_manifest(&mut self, get: bool) -> Op {
        let name = self.name();
        let mut references: Vec<String> = self
            .tagged(&name)
            .iter()
            .map(|t| t.as_str().to_owned())
            .collect();
        references.extend(self.manifests(&name).iter().map(|(d, _)| d.to_string()));
        let reference = match pick(&mut self.rng, &references).cloned() {
            Some(reference) => reference,
            None => self.tag(),
        };
        if get {
            Op::GetManifest { name, reference }
        } else {
            Op::HeadManifest { name, reference }
        }
    }

    /// A blob the repository links (else one it does not), read whole, by a good range, or, when
    /// `bad_range`, by one no blob of its size satisfies.
    fn read_blob(&mut self, get: bool, bad_range: bool) -> Op {
        let name = self.name();
        let linked = self.linked(&name);
        let (digest, size) = match pick(&mut self.rng, &linked) {
            Some((digest, bytes)) => (digest.clone(), len64(bytes)),
            None => {
                let content = self.content();
                (self.digest(&content), len64(&content))
            }
        };
        let digest = digest.to_string();
        if !get {
            return Op::HeadBlob { name, digest };
        }
        let range = if bad_range {
            let unsatisfiable = [
                ByteRange::From {
                    first: size,
                    last: None,
                },
                ByteRange::From {
                    first: size.saturating_add(1),
                    last: Some(size),
                },
                ByteRange::Suffix { length: 0 },
            ];
            pick(&mut self.rng, &unsatisfiable).copied()
        } else if coin(&mut self.rng) {
            None
        } else if coin(&mut self.rng) {
            let first = self.rng.below(size);
            let last = if coin(&mut self.rng) {
                let room = size.saturating_sub(first);
                Some(first.saturating_add(self.rng.below(room)))
            } else {
                None
            };
            Some(ByteRange::From { first, last })
        } else {
            Some(ByteRange::Suffix {
                length: self.rng.below(size).saturating_add(1),
            })
        };
        Op::GetBlob {
            name,
            digest,
            range,
        }
    }

    fn list_tags(&mut self) -> Op {
        let name = self.name();
        let n = if coin(&mut self.rng) {
            u32::try_from(below(
                &mut self.rng,
                self.world.tags.len().saturating_add(1),
            ))
            .ok()
        } else {
            None
        };
        let last = if coin(&mut self.rng) {
            Some(self.tag())
        } else {
            None
        };
        Op::ListTags { name, n, last }
    }

    fn list_referrers(&mut self) -> Op {
        let name = self.name();
        let stored = self.manifests(&name);
        let mut subjects: Vec<String> = stored
            .iter()
            .filter_map(|(_, m)| m.subject.clone())
            .collect();
        subjects.extend(stored.iter().map(|(d, _)| d.to_string()));
        let digest = match pick(&mut self.rng, &subjects).cloned() {
            Some(digest) => digest,
            None => self.stray().to_string(),
        };
        let artifact_type = if coin(&mut self.rng) {
            let types: Vec<&str> = ARTIFACT_TYPES.iter().chain(CONFIG_TYPES).copied().collect();
            pick(&mut self.rng, &types).map(|t| (*t).to_owned())
        } else {
            None
        };
        Op::ListReferrers {
            name,
            digest,
            artifact_type,
        }
    }

    fn delete_tag(&mut self) -> Op {
        let name = self.name();
        let tags = self.tagged(&name);
        let reference = match pick(&mut self.rng, &tags) {
            Some(tag) => tag.as_str().to_owned(),
            None => self.tag(),
        };
        Op::DeleteManifest { name, reference }
    }

    /// A manifest no index of its repository lists, else a stray digest.
    fn delete_manifest(&mut self) -> Op {
        let name = self.name();
        let stored = self.manifests(&name);
        let free: Vec<Digest> = stored
            .iter()
            .filter(|(d, _)| !stored.iter().any(|(_, m)| m.children.contains(d)))
            .map(|(d, _)| d.clone())
            .collect();
        let reference = match pick(&mut self.rng, &free) {
            Some(digest) => digest.to_string(),
            None => self.stray().to_string(),
        };
        Op::DeleteManifest { name, reference }
    }

    /// A blob no manifest of its repository lists, else a stray digest.
    fn delete_blob(&mut self) -> Op {
        let name = self.name();
        let stored = self.manifests(&name);
        let free: Vec<Digest> = self
            .linked(&name)
            .into_iter()
            .filter(|(d, _)| !stored.iter().any(|(_, m)| m.blobs.contains(d)))
            .map(|(d, _)| d)
            .collect();
        let digest = match pick(&mut self.rng, &free) {
            Some(digest) => digest.to_string(),
            None => self.stray().to_string(),
        };
        Op::DeleteBlob { name, digest }
    }

    /// A digest the grammar refuses, one way each, every one with a `:` so that as a reference it
    /// is read as a digest.
    fn malformed_digest(&mut self) -> String {
        let real = Digest::of(Algorithm::Sha256, &self.rng.next_u64().to_le_bytes()).to_string();
        let hex = real.split_once(':').map_or("", |(_, hex)| hex).to_owned();
        let mut short = hex.clone();
        short.pop();
        let choices = [
            "sha256:totallywrong".to_owned(),
            format!("sha256:{short}"),
            format!("sha256:{hex}a"),
            format!("sha256:{}", hex.to_ascii_uppercase()),
            format!("SHA256:{hex}"),
            format!(":{hex}"),
            format!("sha256:{short} "),
            format!("sha256::{hex}"),
            format!("sha256-:{hex}"),
        ];
        pick(&mut self.rng, &choices).cloned().unwrap_or_default()
    }

    /// A digest the grammar takes whose algorithm the image spec does not register.
    fn unregistered_digest(&mut self) -> String {
        let real = Digest::of(Algorithm::Sha256, &self.rng.next_u64().to_le_bytes()).to_string();
        let hex = real.split_once(':').map_or("", |(_, hex)| hex).to_owned();
        let choices = [
            format!("blake3:{hex}"),
            format!("sha384:{hex}"),
            MULTIHASH.to_owned(),
        ];
        pick(&mut self.rng, &choices).cloned().unwrap_or_default()
    }

    /// What a principal must be granted to make `op`, and on which repository.
    fn needs(&self, op: &Op) -> Option<(Name, Action)> {
        let named = |name: &str, action| Name::parse(name, None).map(|n| (n, action));
        match op {
            Op::GetBlob { name, .. }
            | Op::HeadBlob { name, .. }
            | Op::GetManifest { name, .. }
            | Op::HeadManifest { name, .. }
            | Op::ListTags { name, .. }
            | Op::ListReferrers { name, .. } => named(name, Action::Pull),
            Op::StartUpload { name, .. }
            | Op::PostBlob { name, .. }
            | Op::MountBlob { name, .. }
            | Op::PutManifest { name, .. } => named(name, Action::Push),
            Op::DeleteManifest { name, .. } | Op::DeleteBlob { name, .. } => {
                named(name, Action::Delete)
            }
            Op::PatchUpload { session, .. }
            | Op::PutUpload { session, .. }
            | Op::GetUpload { session }
            | Op::CancelUpload { session } => self
                .shadow
                .session(*session)
                .map(|open| (open.repository.clone(), Action::Push)),
            Op::ApiVersion => None,
        }
    }

    fn invalid(&mut self, flaw: Invalid) -> Request {
        match flaw {
            Invalid::BadName => {
                let intent = pick(&mut self.rng, Valid::NAMED)
                    .copied()
                    .unwrap_or(Valid::PostBlob);
                let mut op = self.valid(intent);
                let bad = pick(&mut self.rng, INVALID_NAMES)
                    .copied()
                    .unwrap_or_default()
                    .to_owned();
                let in_from = coin(&mut self.rng);
                rename(&mut op, bad, in_from);
                owned(op)
            }
            Invalid::Anonymous => {
                let intent = pick(&mut self.rng, Valid::ALL)
                    .copied()
                    .unwrap_or(Valid::ApiVersion);
                Request {
                    principal: Principal::Anonymous,
                    op: self.valid(intent),
                }
            }
            Invalid::Ungranted => {
                let intent = pick(&mut self.rng, Valid::ALL)
                    .copied()
                    .unwrap_or(Valid::ApiVersion);
                let op = self.valid(intent);
                let lacking: Vec<u8> = match self.needs(&op) {
                    Some((name, action)) => (1..=self.world.principals)
                        .filter(|p| !self.shadow.config().grants.allows(*p, &name, action))
                        .collect(),
                    None => (1..=self.world.principals).collect(),
                };
                let principal = pick(&mut self.rng, &lacking)
                    .copied()
                    .map_or(Principal::Named(OWNER), Principal::Named);
                Request { principal, op }
            }
            Invalid::BadDigest | Invalid::UnregisteredDigest => {
                let intent = pick(&mut self.rng, Valid::DIGESTED)
                    .copied()
                    .unwrap_or(Valid::PostBlob);
                let mut op = self.valid(intent);
                let text = if flaw == Invalid::BadDigest {
                    self.malformed_digest()
                } else {
                    self.unregistered_digest()
                };
                redigest(&mut op, text);
                owned(op)
            }
            Invalid::WrongContent => owned(self.wrong_content()),
            Invalid::OutOfOrder => owned(self.out_of_order()),
            Invalid::MalformedRange => owned(self.malformed_range()),
            Invalid::RangeLength => owned(self.range_length()),
            Invalid::UnknownSession => owned(self.unknown_session()),
            Invalid::MissingBlob => owned(self.put_image(Flaw::MissingBlob)),
            Invalid::WrongSize => owned(self.put_image(Flaw::WrongSize)),
            Invalid::MediaTypeMismatch => owned(self.put_image(Flaw::MediaTypeMismatch)),
            Invalid::UnregisteredDescriptor => owned(self.put_image(Flaw::UnregisteredDescriptor)),
            Invalid::BadDescriptor => owned(self.put_image(Flaw::BadDescriptor)),
            Invalid::TooLarge => owned(self.put_image(Flaw::TooLarge)),
            Invalid::MissingChild => owned(self.put_index(true)),
            Invalid::NotJson => owned(self.not_a_manifest(Malformation::NotJson)),
            Invalid::NotAManifest => owned(self.not_a_manifest(Malformation::NotAManifest)),
            Invalid::Schema1 => owned(self.schema1()),
            Invalid::DigestMismatch => owned(self.digest_mismatch()),
            Invalid::BadTag => owned(self.bad_tag()),
            Invalid::AbsentManifest => owned(self.absent_manifest()),
            Invalid::AbsentBlob => owned(self.absent_blob()),
            Invalid::Unwritten => owned(self.unwritten()),
            Invalid::ReferencedDelete => owned(self.referenced_delete()),
            Invalid::OverBudget => owned(self.over_budget()),
            Invalid::BadRange => owned(self.read_blob(true, true)),
            Invalid::NoDigest => owned(self.no_digest()),
        }
    }

    /// A blob whose bytes are not its digest's: by one `POST`, or by a session's closing `PUT`.
    fn wrong_content(&mut self) -> Op {
        let extra = self.rng.next_u64().to_le_bytes();
        if coin(&mut self.rng) {
            let name = self.name();
            let content = self.content();
            let digest = self.digest(&content).to_string();
            let mut body = content.to_vec();
            body.extend(extra);
            return Op::PostBlob {
                name,
                digest,
                body: body.into(),
            };
        }
        let unfinished = self.unfinished();
        let Some((session, offset, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let mut other = self.intended_content(session).to_vec();
        other.extend(extra);
        let digest = self.digest(&other).to_string();
        let chunk = (!rest.is_empty()).then(|| Chunk {
            range: span(offset, &rest),
            body: rest,
        });
        Op::PutUpload {
            session,
            digest: Some(digest),
            chunk,
        }
    }

    /// A chunk that does not start where the session's bytes end: one past, one before, or with
    /// its bounds reversed; by `PATCH` or in the closing `PUT`.
    fn out_of_order(&mut self) -> Op {
        let unfinished = self.unfinished();
        let Some((session, offset, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let body = if rest.is_empty() {
            Bytes::from(self.rng.next_u64().to_le_bytes().to_vec())
        } else {
            rest
        };
        let reach = len64(&body).saturating_sub(1);
        let mut ranges: Vec<(u64, u64)> = [offset.checked_add(1), offset.checked_sub(1)]
            .into_iter()
            .flatten()
            .filter_map(|first| first.checked_add(reach).map(|last| (first, last)))
            .collect();
        if let Some(first) = offset.checked_add(1) {
            ranges.push((first, offset));
        }
        let Some((first, last)) = pick(&mut self.rng, &ranges).copied() else {
            return self.start_upload();
        };
        self.send_chunk(session, ChunkRange::Span { first, last }, body)
    }

    fn malformed_range(&mut self) -> Op {
        let unfinished = self.unfinished();
        let Some((session, _, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let text = pick(&mut self.rng, MALFORMED_RANGES)
            .copied()
            .unwrap_or_default()
            .to_owned();
        let body = if rest.is_empty() {
            Bytes::from(self.rng.next_u64().to_le_bytes().to_vec())
        } else {
            rest
        };
        self.send_chunk(session, ChunkRange::Malformed(text), body)
    }

    /// A chunk at the right offset whose range says one byte more than it carries; by `PATCH` or
    /// in the closing `PUT`.
    fn range_length(&mut self) -> Op {
        let unfinished = self.unfinished();
        let Some((session, offset, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let body = if rest.is_empty() {
            Bytes::from(self.rng.next_u64().to_le_bytes().to_vec())
        } else {
            rest
        };
        let last = offset.saturating_add(len64(&body));
        self.send_chunk(
            session,
            ChunkRange::Span {
                first: offset,
                last,
            },
            body,
        )
    }

    /// A chunk sent by a `PATCH`, or as the last chunk of the session's closing `PUT` with the
    /// digest the session is meant to end with: a fair coin says which.
    fn send_chunk(&mut self, session: SessionTag, range: ChunkRange, body: Bytes) -> Op {
        if coin(&mut self.rng) {
            return Op::PatchUpload {
                session,
                range,
                body,
            };
        }
        let whole = self.intended_content(session);
        Op::PutUpload {
            session,
            digest: Some(self.digest(&whole).to_string()),
            chunk: Some(Chunk { range, body }),
        }
    }

    /// A request to a session never opened, or one finished or cancelled (plan T-RQ1.3.5).
    fn unknown_session(&mut self) -> Op {
        let ended: Vec<SessionTag> = self
            .intended
            .keys()
            .filter(|s| self.shadow.session(**s).is_none())
            .copied()
            .collect();
        let session = match pick(&mut self.rng, &ended).copied() {
            Some(session) if coin(&mut self.rng) => session,
            _ => self.fresh_session(),
        };
        let body = self.content();
        let digest = self.digest(&body).to_string();
        let choices = [
            Op::PatchUpload {
                session,
                range: ChunkRange::Absent,
                body,
            },
            Op::PutUpload {
                session,
                digest: Some(digest),
                chunk: None,
            },
            Op::GetUpload { session },
            Op::CancelUpload { session },
        ];
        pick(&mut self.rng, &choices)
            .cloned()
            .unwrap_or(Op::GetUpload { session })
    }

    fn not_a_manifest(&mut self, how: Malformation) -> Op {
        let name = self.name();
        let bytes = match how {
            Malformation::NotJson => NOT_JSON,
            Malformation::NotAManifest => NOT_A_MANIFEST,
        }
        .as_bytes()
        .to_vec();
        let content_type = if coin(&mut self.rng) {
            grammar::OCI_IMAGE_MANIFEST
        } else {
            grammar::OCI_IMAGE_INDEX
        };
        let reference = self.reference(&bytes);
        Op::PutManifest {
            name,
            reference,
            manifest: Box::new(Manifest {
                content_type: content_type.to_owned(),
                body: ManifestBody::Malformed(how),
                bytes: bytes.into(),
            }),
        }
    }

    fn schema1(&mut self) -> Op {
        let name = self.name();
        let bytes = b"{\"schemaVersion\":1,\"name\":\"r\",\"tag\":\"t\"}".to_vec();
        let content_type = if coin(&mut self.rng) {
            grammar::DOCKER_SCHEMA1
        } else {
            grammar::DOCKER_SCHEMA1_SIGNED
        };
        let reference = self.reference(&bytes);
        Op::PutManifest {
            name,
            reference,
            manifest: Box::new(Manifest {
                content_type: content_type.to_owned(),
                body: ManifestBody::Opaque,
                bytes: bytes.into(),
            }),
        }
    }

    /// A manifest pushed by a digest of other bytes.
    fn digest_mismatch(&mut self) -> Op {
        match self.put_image(Flaw::None) {
            Op::PutManifest { name, manifest, .. } => {
                let mut other = manifest.bytes.to_vec();
                other.push(b' ');
                let reference = self.digest(&other).to_string();
                Op::PutManifest {
                    name,
                    reference,
                    manifest,
                }
            }
            other => other,
        }
    }

    /// A manifest request whose reference has no `:` and is no tag.
    fn bad_tag(&mut self) -> Op {
        let mut tags: Vec<String> = INVALID_TAGS.iter().map(|t| (*t).to_owned()).collect();
        tags.push("x".repeat(grammar::TAG_MAX_BYTES.saturating_add(1)));
        let reference = pick(&mut self.rng, &tags).cloned().unwrap_or_default();
        let mut push = self.put_image(Flaw::None);
        if let Op::PutManifest { reference: r, .. } = &mut push {
            r.clone_from(&reference);
        }
        let name = self.name();
        let choices = [
            Op::GetManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::HeadManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::DeleteManifest { name, reference },
            push,
        ];
        pick(&mut self.rng, &choices)
            .cloned()
            .unwrap_or(Op::ApiVersion)
    }

    /// A manifest request for a tag or digest the repository does not have.
    fn absent_manifest(&mut self) -> Op {
        let name = self.name();
        let held = self.tagged(&name);
        let unheld: Vec<String> = self
            .world
            .tags
            .iter()
            .filter(|t| !held.iter().any(|h| h.as_str() == t.as_str()))
            .cloned()
            .collect();
        let reference = match pick(&mut self.rng, &unheld).cloned() {
            Some(tag) if coin(&mut self.rng) => tag,
            _ => self.stray().to_string(),
        };
        let choices = [
            Op::GetManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::HeadManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::DeleteManifest { name, reference },
        ];
        pick(&mut self.rng, &choices)
            .cloned()
            .unwrap_or(Op::ApiVersion)
    }

    fn absent_blob(&mut self) -> Op {
        let name = self.name();
        let digest = self.stray().to_string();
        let choices = [
            Op::GetBlob {
                name: name.clone(),
                digest: digest.clone(),
                range: None,
            },
            Op::HeadBlob {
                name: name.clone(),
                digest: digest.clone(),
            },
            Op::DeleteBlob { name, digest },
        ];
        pick(&mut self.rng, &choices)
            .cloned()
            .unwrap_or(Op::ApiVersion)
    }

    /// A read or delete in a repository nothing was ever written to.
    fn unwritten(&mut self) -> Op {
        let name = pick(&mut self.rng, &self.world.unwritten)
            .cloned()
            .unwrap_or_default();
        let digest = self.stray().to_string();
        let reference = self.tag();
        let choices = [
            Op::GetBlob {
                name: name.clone(),
                digest: digest.clone(),
                range: None,
            },
            Op::HeadBlob {
                name: name.clone(),
                digest: digest.clone(),
            },
            Op::GetManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::HeadManifest {
                name: name.clone(),
                reference: reference.clone(),
            },
            Op::ListTags {
                name: name.clone(),
                n: None,
                last: None,
            },
            Op::ListTags {
                name: name.clone(),
                n: u32::try_from(self.world.tags.len()).ok(),
                last: None,
            },
            Op::DeleteManifest {
                name: name.clone(),
                reference,
            },
            Op::DeleteBlob { name, digest },
        ];
        pick(&mut self.rng, &choices)
            .cloned()
            .unwrap_or(Op::ApiVersion)
    }

    /// A delete of a blob a manifest lists, or of a manifest an index lists.
    fn referenced_delete(&mut self) -> Op {
        let mut choices = Vec::new();
        for name in &self.world.names {
            for (_, manifest) in self.manifests(name) {
                for blob in &manifest.blobs {
                    choices.push(Op::DeleteBlob {
                        name: name.clone(),
                        digest: blob.to_string(),
                    });
                }
                for child in &manifest.children {
                    choices.push(Op::DeleteManifest {
                        name: name.clone(),
                        reference: child.to_string(),
                    });
                }
            }
        }
        match pick(&mut self.rng, &choices).cloned() {
            Some(op) => op,
            None => self.absent_blob(),
        }
    }

    /// One byte past what the staging budget leaves: a whole blob by one `POST`, or a chunk of an
    /// open session.
    fn over_budget(&mut self) -> Op {
        let Some(budget) = self.shadow.config().staging_budget else {
            return self.patch(true);
        };
        if coin(&mut self.rng) {
            let Some(length) = usize::try_from(budget).ok().and_then(|b| b.checked_add(1)) else {
                return self.patch(true);
            };
            let body = random_bytes(&mut self.rng, length);
            let name = self.name();
            let digest = self.digest(&body).to_string();
            return Op::PostBlob {
                name,
                digest,
                body: body.into(),
            };
        }
        let open: Vec<(SessionTag, u64, u8)> = self
            .shadow
            .sessions()
            .map(|(s, open)| (*s, len64(&open.content), open.opener))
            .collect();
        let Some((session, offset, opener)) = pick(&mut self.rng, &open).copied() else {
            return self.start_upload();
        };
        let room = budget.saturating_sub(self.shadow.staged(opener));
        let Some(length) = usize::try_from(room).ok().and_then(|r| r.checked_add(1)) else {
            return self.start_upload();
        };
        let body = random_bytes(&mut self.rng, length);
        Op::PatchUpload {
            session,
            range: span(offset, &body),
            body: body.into(),
        }
    }

    fn no_digest(&mut self) -> Op {
        let unfinished = self.unfinished();
        let Some((session, _, rest)) = pick(&mut self.rng, &unfinished).cloned() else {
            return self.start_upload();
        };
        let chunk = if rest.is_empty() {
            None
        } else {
            Some(Chunk {
                range: ChunkRange::Absent,
                body: rest,
            })
        };
        Op::PutUpload {
            session,
            digest: None,
            chunk,
        }
    }
}

/// One client's history of `length` requests, from `seed` under `mix`, and the configuration it
/// is meant for.
pub fn sequential(seed: u64, mix: &Mix, length: usize) -> (Config, Vec<Request>) {
    let mut generator = Generator::new(seed, mix);
    let requests = (0..length).map(|_| generator.next_request()).collect();
    (generator.config(), requests)
}

/// A concurrent plan from `seed` under `mix`: the setup's requests, then each client's, made in
/// turn against one shadow and interleaved by a stream of the seed's own.
pub fn concurrent(seed: u64, mix: &Mix, shape: Shape) -> Result<(Config, Plan), PlanError> {
    let mut generator = Generator::new(seed, mix);
    let setup = (0..shape.setup).map(|_| generator.next_request()).collect();
    let mut scripts: Vec<Vec<Request>> = vec![Vec::new(); usize::from(shape.clients)];
    for _ in 0..shape.per_client {
        for script in &mut scripts {
            script.push(generator.next_request());
        }
    }
    let mut rng = Seeded::new(stream_seed(seed, "reliquary.interleave", &[]));
    let plan = Plan::interleave(setup, scripts, &mut rng, shape.lose_one_in)?;
    Ok((generator.config(), plan))
}

/// Writers moving one tag among manifests the setup pushed while readers read it (plan T-RQ0.5.4).
pub fn tag_race(seed: u64, race: &Race) -> Result<(Config, Plan), PlanError> {
    let mut rng = Seeded::new(stream_seed(seed, "reliquary.race", &[]));
    let name = "race".to_owned();
    let tag = "latest".to_owned();
    let mut grants = Grants::default();
    for action in [Action::Pull, Action::Push, Action::Delete] {
        grants.grant(OWNER, &name, action);
    }
    let config = Config {
        grants,
        ..Config::default()
    };
    let mut setup = Vec::new();
    let mut manifests = Vec::new();
    for k in 0..race.manifests {
        let body = Bytes::from(format!("config {k}").into_bytes());
        let digest = Digest::of(Algorithm::Sha256, &body);
        setup.push(owned(Op::PostBlob {
            name: name.clone(),
            digest: digest.to_string(),
            body: Bytes::clone(&body),
        }));
        let image = ImageManifest {
            media_type: Some(grammar::OCI_IMAGE_MANIFEST.to_owned()),
            artifact_type: None,
            config: Descriptor {
                media_type: CONFIG_TYPES
                    .first()
                    .copied()
                    .unwrap_or(LAYER_TYPE)
                    .to_owned(),
                digest: digest.to_string(),
                size: len64(&body),
            },
            layers: Vec::new(),
            subject: None,
            annotations: Vec::new(),
        };
        let bytes = json::image(&image);
        let manifest = Box::new(Manifest {
            content_type: grammar::OCI_IMAGE_MANIFEST.to_owned(),
            body: ManifestBody::Image(image),
            bytes: bytes.into(),
        });
        setup.push(owned(Op::PutManifest {
            name: name.clone(),
            reference: Digest::of(Algorithm::Sha256, &manifest.bytes).to_string(),
            manifest: manifest.clone(),
        }));
        manifests.push(manifest);
    }
    if let Some(first) = manifests.first() {
        setup.push(owned(Op::PutManifest {
            name: name.clone(),
            reference: tag.clone(),
            manifest: first.clone(),
        }));
    }
    let mut scripts = Vec::new();
    for _ in 0..race.writers {
        let mut moves = Vec::new();
        for _ in 0..race.moves {
            if let Some(manifest) = pick(&mut rng, &manifests) {
                moves.push(owned(Op::PutManifest {
                    name: name.clone(),
                    reference: tag.clone(),
                    manifest: manifest.clone(),
                }));
            }
        }
        scripts.push(moves);
    }
    for _ in 0..race.readers {
        let mut reads = Vec::new();
        for _ in 0..race.reads {
            let reference = tag.clone();
            reads.push(owned(if coin(&mut rng) {
                Op::GetManifest {
                    name: name.clone(),
                    reference,
                }
            } else {
                Op::HeadManifest {
                    name: name.clone(),
                    reference,
                }
            }));
        }
        scripts.push(reads);
    }
    let plan = Plan::interleave(setup, scripts, &mut rng, race.lose_one_in)?;
    Ok((config, plan))
}
