//! The model node's log entries (docs/design/sim.md, W10): one of the reference model's changes,
//! with every content it carries replaced by a reference to the object that holds it in the bytes
//! store, and the identity of the attempt that wrote it. Encoded with each field's length before
//! it, a change's kind as its name, and read back refusing anything short, long or malformed:
//! the records store holds only what a node wrote, but a reader trusts nothing it reads.

use std::sync::Arc;

use reliquary_api::grammar::{Digest, Name, Tag};
use reliquary_api::model::{Change, Stored};
use reliquary_api::request::SessionTag;
use reliquary_core::seams::bytes::Sha256;

/// Where a content's bytes are: an object of the bytes store, and what it must be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectRef {
    pub key: String,
    pub length: u64,
    pub sha256: Sha256,
}

/// A change as the log keeps it: the model's, its contents by reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Logged {
    Open {
        session: SessionTag,
        repository: Name,
        opener: u8,
    },
    Append {
        session: SessionTag,
        bytes: ObjectRef,
    },
    End {
        session: SessionTag,
    },
    Complete {
        session: SessionTag,
        digest: Digest,
        content: ObjectRef,
    },
    Link {
        repository: Name,
        digest: Digest,
        content: ObjectRef,
    },
    Store {
        repository: Name,
        digest: Digest,
        content_type: String,
        bytes: ObjectRef,
        blobs: Vec<Digest>,
        children: Vec<Digest>,
        subject: Option<String>,
        artifact_type: Option<String>,
        annotations: Vec<(String, String)>,
        tag: Option<Tag>,
    },
    Untag {
        repository: Name,
        tag: Tag,
    },
    Remove {
        repository: Name,
        digest: Digest,
    },
    Unlink {
        repository: Name,
        digest: Digest,
    },
}

/// A log entry: who wrote it, and the change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub identity: u128,
    pub change: Logged,
}

/// Why bytes are not an entry.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("not a log entry: {0}")]
pub struct Malformed(pub &'static str);

/// The content a change carries, which the log keeps by reference: an append's bytes, a blob's
/// content, a manifest's bytes.
#[must_use]
pub fn content(change: &Change) -> Option<&Arc<[u8]>> {
    match change {
        Change::Append { bytes, .. } => Some(bytes),
        Change::Complete { content, .. } | Change::Link { content, .. } => Some(content),
        Change::Store { manifest, .. } => Some(&manifest.bytes),
        Change::None
        | Change::Open { .. }
        | Change::End { .. }
        | Change::Untag { .. }
        | Change::Remove { .. }
        | Change::Unlink { .. } => None,
    }
}

/// `change` as the log keeps it, its content at `object`. `None` for a change that changes
/// nothing, which is never logged, or a content-bearing change with no object.
#[must_use]
pub fn logged(change: &Change, object: Option<ObjectRef>) -> Option<Logged> {
    Some(match change.clone() {
        Change::None => return None,
        Change::Open {
            session,
            repository,
            opener,
        } => Logged::Open {
            session,
            repository,
            opener,
        },
        Change::Append { session, .. } => Logged::Append {
            session,
            bytes: object?,
        },
        Change::End { session } => Logged::End { session },
        Change::Complete {
            session, digest, ..
        } => Logged::Complete {
            session,
            digest,
            content: object?,
        },
        Change::Link {
            repository, digest, ..
        } => Logged::Link {
            repository,
            digest,
            content: object?,
        },
        Change::Store {
            repository,
            digest,
            manifest,
            tag,
        } => Logged::Store {
            repository,
            digest,
            content_type: manifest.content_type,
            bytes: object?,
            blobs: manifest.blobs,
            children: manifest.children,
            subject: manifest.subject,
            artifact_type: manifest.artifact_type,
            annotations: manifest.annotations,
            tag,
        },
        Change::Untag { repository, tag } => Logged::Untag { repository, tag },
        Change::Remove { repository, digest } => Logged::Remove { repository, digest },
        Change::Unlink { repository, digest } => Logged::Unlink { repository, digest },
    })
}

impl Logged {
    /// The object it names, if it carries content.
    #[must_use]
    pub fn object(&self) -> Option<&ObjectRef> {
        match self {
            Self::Append { bytes, .. } | Self::Store { bytes, .. } => Some(bytes),
            Self::Complete { content, .. } | Self::Link { content, .. } => Some(content),
            Self::Open { .. }
            | Self::End { .. }
            | Self::Untag { .. }
            | Self::Remove { .. }
            | Self::Unlink { .. } => None,
        }
    }

    /// The model's change, with `content` the bytes of the object it names.
    #[must_use]
    pub fn change(self, content: Arc<[u8]>) -> Change {
        match self {
            Self::Open {
                session,
                repository,
                opener,
            } => Change::Open {
                session,
                repository,
                opener,
            },
            Self::Append { session, .. } => Change::Append {
                session,
                bytes: content,
            },
            Self::End { session } => Change::End { session },
            Self::Complete {
                session, digest, ..
            } => Change::Complete {
                session,
                digest,
                content,
            },
            Self::Link {
                repository, digest, ..
            } => Change::Link {
                repository,
                digest,
                content,
            },
            Self::Store {
                repository,
                digest,
                content_type,
                blobs,
                children,
                subject,
                artifact_type,
                annotations,
                tag,
                ..
            } => Change::Store {
                repository,
                digest,
                manifest: Stored {
                    content_type,
                    bytes: content,
                    blobs,
                    children,
                    subject,
                    artifact_type,
                    annotations,
                },
                tag,
            },
            Self::Untag { repository, tag } => Change::Untag { repository, tag },
            Self::Remove { repository, digest } => Change::Remove { repository, digest },
            Self::Unlink { repository, digest } => Change::Unlink { repository, digest },
        }
    }
}

// ------------------------------------------------------------------------------------- writing

struct Out(Vec<u8>);

impl Out {
    fn count(&mut self, n: usize) {
        self.0
            .extend_from_slice(&u64::try_from(n).unwrap_or(u64::MAX).to_le_bytes());
    }
    fn text(&mut self, s: &str) {
        self.count(s.len());
        self.0.extend_from_slice(s.as_bytes());
    }
    fn maybe(&mut self, s: Option<&str>) {
        match s {
            Some(s) => {
                self.count(1);
                self.text(s);
            }
            None => self.count(0),
        }
    }
    fn object(&mut self, object: &ObjectRef) {
        self.text(&object.key);
        self.0.extend_from_slice(&object.length.to_le_bytes());
        self.0.extend_from_slice(&object.sha256.0);
    }
    fn digests(&mut self, digests: &[Digest]) {
        self.count(digests.len());
        for d in digests {
            self.text(&d.to_string());
        }
    }
}

/// `entry`, encoded.
#[must_use]
pub fn encode(entry: &Entry) -> Vec<u8> {
    let mut out = Out(entry.identity.to_le_bytes().to_vec());
    match &entry.change {
        Logged::Open {
            session,
            repository,
            opener,
        } => {
            out.text("open");
            out.0.extend_from_slice(&session.0.to_le_bytes());
            out.text(repository.as_str());
            out.0.push(*opener);
        }
        Logged::Append { session, bytes } => {
            out.text("append");
            out.0.extend_from_slice(&session.0.to_le_bytes());
            out.object(bytes);
        }
        Logged::End { session } => {
            out.text("end");
            out.0.extend_from_slice(&session.0.to_le_bytes());
        }
        Logged::Complete {
            session,
            digest,
            content,
        } => {
            out.text("complete");
            out.0.extend_from_slice(&session.0.to_le_bytes());
            out.text(&digest.to_string());
            out.object(content);
        }
        Logged::Link {
            repository,
            digest,
            content,
        } => {
            out.text("link");
            out.text(repository.as_str());
            out.text(&digest.to_string());
            out.object(content);
        }
        Logged::Store {
            repository,
            digest,
            content_type,
            bytes,
            blobs,
            children,
            subject,
            artifact_type,
            annotations,
            tag,
        } => {
            out.text("store");
            out.text(repository.as_str());
            out.text(&digest.to_string());
            out.text(content_type);
            out.object(bytes);
            out.digests(blobs);
            out.digests(children);
            out.maybe(subject.as_deref());
            out.maybe(artifact_type.as_deref());
            out.count(annotations.len());
            for (k, v) in annotations {
                out.text(k);
                out.text(v);
            }
            out.maybe(tag.as_ref().map(Tag::as_str));
        }
        Logged::Untag { repository, tag } => {
            out.text("untag");
            out.text(repository.as_str());
            out.text(tag.as_str());
        }
        Logged::Remove { repository, digest } => {
            out.text("remove");
            out.text(repository.as_str());
            out.text(&digest.to_string());
        }
        Logged::Unlink { repository, digest } => {
            out.text("unlink");
            out.text(repository.as_str());
            out.text(&digest.to_string());
        }
    }
    out.0
}

// ------------------------------------------------------------------------------------- reading

struct In<'a>(&'a [u8]);

impl<'a> In<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Malformed> {
        let (head, rest) = self
            .0
            .split_at_checked(n)
            .ok_or(Malformed("it ends early"))?;
        self.0 = rest;
        Ok(head)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Malformed> {
        let mut out = [0u8; N];
        for (to, from) in out.iter_mut().zip(self.take(N)?) {
            *to = *from;
        }
        Ok(out)
    }
    fn count(&mut self) -> Result<usize, Malformed> {
        let n = u64::from_le_bytes(self.array()?);
        // A count is of what follows it, so one past the bytes left is malformed at once.
        let n = usize::try_from(n).map_err(|_| Malformed("a count past memory"))?;
        if n > self.0.len() {
            return Err(Malformed("a count past its bytes"));
        }
        Ok(n)
    }
    fn text(&mut self) -> Result<&'a str, Malformed> {
        let n = self.count()?;
        std::str::from_utf8(self.take(n)?).map_err(|_| Malformed("text that is not UTF-8"))
    }
    fn maybe(&mut self) -> Result<Option<String>, Malformed> {
        match self.count()? {
            0 => Ok(None),
            1 => Ok(Some(self.text()?.to_owned())),
            _ => Err(Malformed("an option that is neither")),
        }
    }
    fn session(&mut self) -> Result<SessionTag, Malformed> {
        Ok(SessionTag(u32::from_le_bytes(self.array()?)))
    }
    fn name(&mut self) -> Result<Name, Malformed> {
        Name::parse(self.text()?, None).ok_or(Malformed("a repository name"))
    }
    fn digest(&mut self) -> Result<Digest, Malformed> {
        Digest::parse(self.text()?).map_err(|_| Malformed("a digest"))
    }
    fn tag(&mut self) -> Result<Tag, Malformed> {
        Tag::parse(self.text()?).ok_or(Malformed("a tag"))
    }
    fn object(&mut self) -> Result<ObjectRef, Malformed> {
        let key = self.text()?.to_owned();
        let length = u64::from_le_bytes(self.array()?);
        let sha256 = Sha256(self.array()?);
        Ok(ObjectRef {
            key,
            length,
            sha256,
        })
    }
    fn digests(&mut self) -> Result<Vec<Digest>, Malformed> {
        let n = self.count()?;
        let mut out = Vec::new();
        for _ in 0..n {
            out.push(self.digest()?);
        }
        Ok(out)
    }
}

/// The entry `bytes` encode.
pub fn decode(bytes: &[u8]) -> Result<Entry, Malformed> {
    let mut r = In(bytes);
    let identity = u128::from_le_bytes(r.array()?);
    let change = match r.text()? {
        "open" => Logged::Open {
            session: r.session()?,
            repository: r.name()?,
            opener: u8::from_le_bytes(r.array()?),
        },
        "append" => Logged::Append {
            session: r.session()?,
            bytes: r.object()?,
        },
        "end" => Logged::End {
            session: r.session()?,
        },
        "complete" => Logged::Complete {
            session: r.session()?,
            digest: r.digest()?,
            content: r.object()?,
        },
        "link" => Logged::Link {
            repository: r.name()?,
            digest: r.digest()?,
            content: r.object()?,
        },
        "store" => {
            let repository = r.name()?;
            let digest = r.digest()?;
            let content_type = r.text()?.to_owned();
            let bytes = r.object()?;
            let blobs = r.digests()?;
            let children = r.digests()?;
            let subject = r.maybe()?;
            let artifact_type = r.maybe()?;
            let n = r.count()?;
            let mut annotations = Vec::new();
            for _ in 0..n {
                annotations.push((r.text()?.to_owned(), r.text()?.to_owned()));
            }
            let tag = r
                .maybe()?
                .map(|t| Tag::parse(&t).ok_or(Malformed("a tag")))
                .transpose()?;
            Logged::Store {
                repository,
                digest,
                content_type,
                bytes,
                blobs,
                children,
                subject,
                artifact_type,
                annotations,
                tag,
            }
        }
        "untag" => Logged::Untag {
            repository: r.name()?,
            tag: r.tag()?,
        },
        "remove" => Logged::Remove {
            repository: r.name()?,
            digest: r.digest()?,
        },
        "unlink" => Logged::Unlink {
            repository: r.name()?,
            digest: r.digest()?,
        },
        _ => return Err(Malformed("an unknown kind of change")),
    };
    if !r.0.is_empty() {
        return Err(Malformed("bytes past its end"));
    }
    Ok(Entry { identity, change })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind of change round-trips, and every cut of an entry, and a byte past it, is refused
    /// rather than read as another (T-RQ0.6.2: the log a restarted node reads is what it wrote).
    #[test]
    fn every_change_round_trips_and_every_cut_is_refused() {
        let name = Name::parse("a/b", None).unwrap();
        let digest = Digest::of(reliquary_api::grammar::Algorithm::Sha256, b"x");
        let tag = Tag::parse("latest").unwrap();
        let object = ObjectRef {
            key: "object-1".to_owned(),
            length: 1,
            sha256: Sha256::of(b"x"),
        };
        let changes = vec![
            Logged::Open {
                session: SessionTag(7),
                repository: name.clone(),
                opener: 3,
            },
            Logged::Append {
                session: SessionTag(7),
                bytes: object.clone(),
            },
            Logged::End {
                session: SessionTag(7),
            },
            Logged::Complete {
                session: SessionTag(7),
                digest: digest.clone(),
                content: object.clone(),
            },
            Logged::Link {
                repository: name.clone(),
                digest: digest.clone(),
                content: object.clone(),
            },
            Logged::Store {
                repository: name.clone(),
                digest: digest.clone(),
                content_type: "application/vnd.oci.image.manifest.v1+json".to_owned(),
                bytes: object.clone(),
                blobs: vec![digest.clone()],
                children: vec![],
                subject: Some(digest.to_string()),
                artifact_type: None,
                annotations: vec![("k".to_owned(), "v".to_owned())],
                tag: Some(tag.clone()),
            },
            Logged::Untag {
                repository: name.clone(),
                tag,
            },
            Logged::Remove {
                repository: name.clone(),
                digest: digest.clone(),
            },
            Logged::Unlink {
                repository: name,
                digest,
            },
        ];
        for (i, change) in changes.into_iter().enumerate() {
            let entry = Entry {
                identity: u128::try_from(i).unwrap() << 64 | 0xfeed,
                change,
            };
            let bytes = encode(&entry);
            assert_eq!(decode(&bytes), Ok(entry));
            for cut in 0..bytes.len() {
                assert!(
                    decode(&bytes[..cut]).is_err(),
                    "a cut at {cut} read as an entry"
                );
            }
            let mut longer = bytes.clone();
            longer.push(0);
            assert!(decode(&longer).is_err());
        }
    }
}
