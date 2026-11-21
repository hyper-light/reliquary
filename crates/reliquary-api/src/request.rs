//! A request to Reliquary: who makes it and what it asks, one variant for each endpoint of the
//! distribution spec v1.1.1 and the upload cancel of its `main` (docs/design/model.md, H1). Text a
//! client writes (names, digests, references, tags) stays text here, so a request can be malformed
//! the way a client's can; the model parses it. A manifest is carried twice: as the bytes a door
//! sends, and as the structure those bytes encode, which is what the model reads (model.md, H3).

use std::fmt;
use std::sync::Arc;

/// Bytes a request carries or an answer returns, shared rather than copied.
pub type Bytes = Arc<[u8]>;

/// Who makes a request: no credentials at all, or the principal a token names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Principal {
    Anonymous,
    Named(u8),
}

/// An upload session as a history names it: the tag of the request that opened it. The
/// implementation's own name for the session (its `Location`) is the door's to map to the tag, so
/// the model's answers do not depend on the order in which sessions were opened (model.md, H8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionTag(pub u32);

/// A `Range` header on a blob `GET`, one range (RFC 9110 §14.1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ByteRange {
    /// `bytes=first-last` or `bytes=first-`.
    From { first: u64, last: Option<u64> },
    /// `bytes=-length`: the last `length` bytes.
    Suffix { length: u64 },
}

/// A `Content-Range` header on a chunk: `^[0-9]+-[0-9]+$`, inclusive at both ends (distribution
/// spec v1.1.1, "Pushing a blob in chunks").
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChunkRange {
    /// No header: the chunk follows the bytes already taken (a streamed upload).
    Absent,
    /// `first-last`.
    Span { first: u64, last: u64 },
    /// A header that does not match the expression, as sent.
    Malformed(String),
}

/// A chunk of an upload: its range header and its bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chunk {
    pub range: ChunkRange,
    pub body: Bytes,
}

/// A descriptor as a manifest writes it (image spec v1.1.1, descriptor.md).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

/// An image manifest's fields the model reads (image spec v1.1.1, manifest.md).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageManifest {
    pub media_type: Option<String>,
    pub artifact_type: Option<String>,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    pub subject: Option<Descriptor>,
    pub annotations: Vec<(String, String)>,
}

/// An index's fields the model reads (image spec v1.1.1, image-index.md).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IndexManifest {
    pub media_type: Option<String>,
    pub artifact_type: Option<String>,
    pub manifests: Vec<Descriptor>,
    pub subject: Option<Descriptor>,
    pub annotations: Vec<(String, String)>,
}

/// What a manifest's bytes are.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ManifestBody {
    Image(ImageManifest),
    Index(IndexManifest),
    /// Bytes of a media type the model does not read.
    Opaque,
    /// Bytes that are not a manifest of any kind, and how.
    Malformed(Malformation),
}

/// How bytes fail to be a manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Malformation {
    /// Not JSON.
    NotJson,
    /// JSON without the fields its media type requires.
    NotAManifest,
}

/// A manifest a client pushes: its `Content-Type`, what its bytes are, and the bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Manifest {
    pub content_type: String,
    pub body: ManifestBody,
    pub bytes: Bytes,
}

/// What a request asks: one variant per endpoint (model.md, H1, and [`Endpoint`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    /// `GET /v2/` (end-1).
    ApiVersion,
    /// `GET /v2/<name>/blobs/<digest>` (end-2).
    GetBlob {
        name: String,
        digest: String,
        range: Option<ByteRange>,
    },
    /// `HEAD /v2/<name>/blobs/<digest>` (end-2).
    HeadBlob { name: String, digest: String },
    /// `GET /v2/<name>/manifests/<reference>` (end-3).
    GetManifest { name: String, reference: String },
    /// `HEAD /v2/<name>/manifests/<reference>` (end-3).
    HeadManifest { name: String, reference: String },
    /// `POST /v2/<name>/blobs/uploads/` (end-4a), opening the session `session`.
    StartUpload { name: String, session: SessionTag },
    /// `POST /v2/<name>/blobs/uploads/?digest=<digest>` with the blob (end-4b).
    PostBlob {
        name: String,
        digest: String,
        body: Bytes,
    },
    /// `PATCH <location>` (end-5).
    PatchUpload {
        session: SessionTag,
        range: ChunkRange,
        body: Bytes,
    },
    /// `PUT <location>?digest=<digest>`, with a last chunk or without (end-6); `None` is a `PUT`
    /// with no `digest` parameter.
    PutUpload {
        session: SessionTag,
        digest: Option<String>,
        chunk: Option<Chunk>,
    },
    /// `GET <location>` (end-13).
    GetUpload { session: SessionTag },
    /// `DELETE <location>` (the spec's `main`, end-14).
    CancelUpload { session: SessionTag },
    /// `POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<other>` (end-11); when it falls back
    /// to an upload, the session it opens is `session`.
    MountBlob {
        name: String,
        digest: String,
        from: Option<String>,
        session: SessionTag,
    },
    /// `PUT /v2/<name>/manifests/<reference>` (end-7).
    PutManifest {
        name: String,
        reference: String,
        manifest: Box<Manifest>,
    },
    /// `DELETE /v2/<name>/manifests/<reference>` (end-9).
    DeleteManifest { name: String, reference: String },
    /// `DELETE /v2/<name>/blobs/<digest>` (end-10).
    DeleteBlob { name: String, digest: String },
    /// `GET /v2/<name>/tags/list`, with `n` and `last` or without (end-8a, end-8b).
    ListTags {
        name: String,
        n: Option<u32>,
        last: Option<String>,
    },
    /// `GET /v2/<name>/referrers/<digest>`, with `artifactType` or without (end-12a, end-12b).
    ListReferrers {
        name: String,
        digest: String,
        artifact_type: Option<String>,
    },
}

/// A request: who asks, and what.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    pub principal: Principal,
    pub op: Op,
}

/// The kinds of request, one per [`Op`] variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpKind {
    ApiVersion,
    GetBlob,
    HeadBlob,
    GetManifest,
    HeadManifest,
    StartUpload,
    PostBlob,
    PatchUpload,
    PutUpload,
    GetUpload,
    CancelUpload,
    MountBlob,
    PutManifest,
    DeleteManifest,
    DeleteBlob,
    ListTags,
    ListReferrers,
}

impl OpKind {
    /// Every kind.
    pub const ALL: &'static [Self] = &[
        Self::ApiVersion,
        Self::GetBlob,
        Self::HeadBlob,
        Self::GetManifest,
        Self::HeadManifest,
        Self::StartUpload,
        Self::PostBlob,
        Self::PatchUpload,
        Self::PutUpload,
        Self::GetUpload,
        Self::CancelUpload,
        Self::MountBlob,
        Self::PutManifest,
        Self::DeleteManifest,
        Self::DeleteBlob,
        Self::ListTags,
        Self::ListReferrers,
    ];
}

/// The endpoints of distribution spec v1.1.1's table ("Endpoints"), and the upload cancel its
/// `main` adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endpoint {
    End1,
    End2,
    End3,
    End4a,
    End4b,
    End5,
    End6,
    End7,
    End8a,
    End8b,
    End9,
    End10,
    End11,
    End12a,
    End12b,
    End13,
    End14,
}

impl Endpoint {
    /// Every endpoint.
    pub const ALL: &'static [Self] = &[
        Self::End1,
        Self::End2,
        Self::End3,
        Self::End4a,
        Self::End4b,
        Self::End5,
        Self::End6,
        Self::End7,
        Self::End8a,
        Self::End8b,
        Self::End9,
        Self::End10,
        Self::End11,
        Self::End12a,
        Self::End12b,
        Self::End13,
        Self::End14,
    ];

    /// The spec's ID for it.
    pub const fn id(self) -> &'static str {
        match self {
            Self::End1 => "end-1",
            Self::End2 => "end-2",
            Self::End3 => "end-3",
            Self::End4a => "end-4a",
            Self::End4b => "end-4b",
            Self::End5 => "end-5",
            Self::End6 => "end-6",
            Self::End7 => "end-7",
            Self::End8a => "end-8a",
            Self::End8b => "end-8b",
            Self::End9 => "end-9",
            Self::End10 => "end-10",
            Self::End11 => "end-11",
            Self::End12a => "end-12a",
            Self::End12b => "end-12b",
            Self::End13 => "end-13",
            Self::End14 => "end-14",
        }
    }
}

impl Op {
    /// The kind of request.
    pub const fn kind(&self) -> OpKind {
        match self {
            Self::ApiVersion => OpKind::ApiVersion,
            Self::GetBlob { .. } => OpKind::GetBlob,
            Self::HeadBlob { .. } => OpKind::HeadBlob,
            Self::GetManifest { .. } => OpKind::GetManifest,
            Self::HeadManifest { .. } => OpKind::HeadManifest,
            Self::StartUpload { .. } => OpKind::StartUpload,
            Self::PostBlob { .. } => OpKind::PostBlob,
            Self::PatchUpload { .. } => OpKind::PatchUpload,
            Self::PutUpload { .. } => OpKind::PutUpload,
            Self::GetUpload { .. } => OpKind::GetUpload,
            Self::CancelUpload { .. } => OpKind::CancelUpload,
            Self::MountBlob { .. } => OpKind::MountBlob,
            Self::PutManifest { .. } => OpKind::PutManifest,
            Self::DeleteManifest { .. } => OpKind::DeleteManifest,
            Self::DeleteBlob { .. } => OpKind::DeleteBlob,
            Self::ListTags { .. } => OpKind::ListTags,
            Self::ListReferrers { .. } => OpKind::ListReferrers,
        }
    }

    /// The endpoint it is sent to.
    pub const fn endpoint(&self) -> Endpoint {
        match self {
            Self::ApiVersion => Endpoint::End1,
            Self::GetBlob { .. } | Self::HeadBlob { .. } => Endpoint::End2,
            Self::GetManifest { .. } | Self::HeadManifest { .. } => Endpoint::End3,
            Self::StartUpload { .. } => Endpoint::End4a,
            Self::PostBlob { .. } => Endpoint::End4b,
            Self::PatchUpload { .. } => Endpoint::End5,
            Self::PutUpload { .. } => Endpoint::End6,
            Self::PutManifest { .. } => Endpoint::End7,
            Self::ListTags {
                n: None,
                last: None,
                ..
            } => Endpoint::End8a,
            Self::ListTags { .. } => Endpoint::End8b,
            Self::DeleteManifest { .. } => Endpoint::End9,
            Self::DeleteBlob { .. } => Endpoint::End10,
            Self::MountBlob { .. } => Endpoint::End11,
            Self::ListReferrers {
                artifact_type: None,
                ..
            } => Endpoint::End12a,
            Self::ListReferrers { .. } => Endpoint::End12b,
            Self::GetUpload { .. } => Endpoint::End13,
            Self::CancelUpload { .. } => Endpoint::End14,
        }
    }
}

impl fmt::Display for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anonymous => f.write_str("anonymous"),
            Self::Named(id) => write!(f, "principal {id}"),
        }
    }
}

impl fmt::Display for ChunkRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => f.write_str("no Content-Range"),
            Self::Span { first, last } => write!(f, "Content-Range {first}-{last}"),
            Self::Malformed(text) => write!(f, "Content-Range {text:?}"),
        }
    }
}

/// A request as one line of a report: the principal, the method and path, and what it carries.
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.principal)?;
        match &self.op {
            Op::ApiVersion => f.write_str("GET /v2/"),
            Op::GetBlob {
                name,
                digest,
                range,
            } => {
                write!(f, "GET /v2/{name}/blobs/{digest}")?;
                match range {
                    None => Ok(()),
                    Some(ByteRange::From { first, last }) => match last {
                        Some(last) => write!(f, " Range bytes={first}-{last}"),
                        None => write!(f, " Range bytes={first}-"),
                    },
                    Some(ByteRange::Suffix { length }) => write!(f, " Range bytes=-{length}"),
                }
            }
            Op::HeadBlob { name, digest } => write!(f, "HEAD /v2/{name}/blobs/{digest}"),
            Op::GetManifest { name, reference } => {
                write!(f, "GET /v2/{name}/manifests/{reference}")
            }
            Op::HeadManifest { name, reference } => {
                write!(f, "HEAD /v2/{name}/manifests/{reference}")
            }
            Op::StartUpload { name, session } => {
                write!(f, "POST /v2/{name}/blobs/uploads/ (session {})", session.0)
            }
            Op::PostBlob { name, digest, body } => write!(
                f,
                "POST /v2/{name}/blobs/uploads/?digest={digest} ({} bytes)",
                body.len()
            ),
            Op::PatchUpload {
                session,
                range,
                body,
            } => write!(
                f,
                "PATCH session {} {range} ({} bytes)",
                session.0,
                body.len()
            ),
            Op::PutUpload {
                session,
                digest,
                chunk,
            } => {
                write!(f, "PUT session {}", session.0)?;
                match digest {
                    Some(digest) => write!(f, "?digest={digest}")?,
                    None => f.write_str(" (no digest)")?,
                }
                match chunk {
                    Some(chunk) => write!(f, " {} ({} bytes)", chunk.range, chunk.body.len()),
                    None => Ok(()),
                }
            }
            Op::GetUpload { session } => write!(f, "GET session {}", session.0),
            Op::CancelUpload { session } => write!(f, "DELETE session {}", session.0),
            Op::MountBlob {
                name,
                digest,
                from,
                session,
            } => {
                write!(f, "POST /v2/{name}/blobs/uploads/?mount={digest}")?;
                if let Some(from) = from {
                    write!(f, "&from={from}")?;
                }
                write!(f, " (session {})", session.0)
            }
            Op::PutManifest {
                name,
                reference,
                manifest,
            } => write!(
                f,
                "PUT /v2/{name}/manifests/{reference} ({}, {} bytes)",
                manifest.content_type,
                manifest.bytes.len()
            ),
            Op::DeleteManifest { name, reference } => {
                write!(f, "DELETE /v2/{name}/manifests/{reference}")
            }
            Op::DeleteBlob { name, digest } => write!(f, "DELETE /v2/{name}/blobs/{digest}"),
            Op::ListTags { name, n, last } => {
                write!(f, "GET /v2/{name}/tags/list")?;
                match (n, last) {
                    (None, None) => Ok(()),
                    (Some(n), None) => write!(f, "?n={n}"),
                    (None, Some(last)) => write!(f, "?last={last}"),
                    (Some(n), Some(last)) => write!(f, "?n={n}&last={last}"),
                }
            }
            Op::ListReferrers {
                name,
                digest,
                artifact_type,
            } => {
                write!(f, "GET /v2/{name}/referrers/{digest}")?;
                match artifact_type {
                    Some(t) => write!(f, "?artifactType={t}"),
                    None => Ok(()),
                }
            }
        }
    }
}
