//! What Reliquary answers: a status, the spec's error code when it refuses, and what a client reads
//! from the response that the harness compares (docs/design/model.md, H1). [`ANSWERS`] states every
//! answer the model gives at each endpoint; the model never gives another.

use std::fmt;

use crate::grammar::{Digest, Name, Tag};
use crate::request::{Bytes, Endpoint, SessionTag};

/// The HTTP statuses the model answers with (distribution spec v1.1.1, "Endpoints"; RFC 9110 §15
/// for 206 and 416 on ranges; RFC 6585 §4 for 429).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    Ok,
    Created,
    Accepted,
    NoContent,
    PartialContent,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    ContentTooLarge,
    RangeNotSatisfiable,
    TooManyRequests,
}

/// The fourteen error codes of distribution spec v1.1.1 ("Error Codes"): "The `code` field MUST be
/// one of the following".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCode {
    BlobUnknown,
    BlobUploadInvalid,
    BlobUploadUnknown,
    DigestInvalid,
    ManifestBlobUnknown,
    ManifestInvalid,
    ManifestUnknown,
    NameInvalid,
    NameUnknown,
    SizeInvalid,
    Unauthorized,
    Denied,
    Unsupported,
    TooManyRequests,
}

impl ErrorCode {
    /// Every code, in the spec's order (code-1 to code-14).
    pub const ALL: &'static [Self] = &[
        Self::BlobUnknown,
        Self::BlobUploadInvalid,
        Self::BlobUploadUnknown,
        Self::DigestInvalid,
        Self::ManifestBlobUnknown,
        Self::ManifestInvalid,
        Self::ManifestUnknown,
        Self::NameInvalid,
        Self::NameUnknown,
        Self::SizeInvalid,
        Self::Unauthorized,
        Self::Denied,
        Self::Unsupported,
        Self::TooManyRequests,
    ];

    /// The code as the spec writes it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BlobUnknown => "BLOB_UNKNOWN",
            Self::BlobUploadInvalid => "BLOB_UPLOAD_INVALID",
            Self::BlobUploadUnknown => "BLOB_UPLOAD_UNKNOWN",
            Self::DigestInvalid => "DIGEST_INVALID",
            Self::ManifestBlobUnknown => "MANIFEST_BLOB_UNKNOWN",
            Self::ManifestInvalid => "MANIFEST_INVALID",
            Self::ManifestUnknown => "MANIFEST_UNKNOWN",
            Self::NameInvalid => "NAME_INVALID",
            Self::NameUnknown => "NAME_UNKNOWN",
            Self::SizeInvalid => "SIZE_INVALID",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Denied => "DENIED",
            Self::Unsupported => "UNSUPPORTED",
            Self::TooManyRequests => "TOOMANYREQUESTS",
        }
    }
}

/// A referrer as the referrers API lists it (distribution spec v1.1.1, "Listing Referrers").
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Referrer {
    pub media_type: String,
    pub digest: Digest,
    pub size: u64,
    /// The manifest's `artifactType`, else an image manifest's config media type; an index without
    /// one has none.
    pub artifact_type: Option<String>,
    pub annotations: Vec<(String, String)>,
}

/// What a client reads besides the status and the code: the headers and body the spec makes
/// meaningful. Locations are the door's; sessions appear by their [`SessionTag`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    Empty,
    /// A blob: its digest and length, and its bytes for a `GET`.
    Blob {
        digest: Digest,
        size: u64,
        content: Option<Bytes>,
    },
    /// `206`: the bytes from `first` to `last` of a blob of `size` bytes.
    Part {
        digest: Digest,
        size: u64,
        first: u64,
        last: u64,
        content: Bytes,
    },
    /// `416` on a blob read: `Content-Range: bytes */size`.
    Unsatisfiable {
        size: u64,
    },
    /// A manifest: its digest, the media type it was pushed with, its length, and its bytes for a
    /// `GET`.
    Manifest {
        digest: Digest,
        content_type: String,
        size: u64,
        content: Option<Bytes>,
    },
    /// `201` for a blob: `Docker-Content-Digest`.
    BlobCreated {
        digest: Digest,
    },
    /// `201` for a manifest: `Docker-Content-Digest`, and `OCI-Subject` when it has a subject.
    ManifestCreated {
        digest: Digest,
        subject: Option<String>,
    },
    /// An upload session and the bytes it holds (`Range: 0-<offset - 1>`).
    Upload {
        session: SessionTag,
        offset: u64,
    },
    /// A tag listing; `more` is the `Link` header's presence.
    Tags {
        name: Name,
        tags: Vec<Tag>,
        more: bool,
    },
    /// A referrers index; `filtered` is `OCI-Filters-Applied: artifactType`.
    Referrers {
        referrers: Vec<Referrer>,
        filtered: bool,
    },
    /// The descriptor digest a manifest refusal names (plan AC-RQ1.5.2).
    Reference {
        digest: String,
    },
}

/// An answer: the status, the code when the request is refused, and the rest.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Answer {
    pub status: Status,
    pub error: Option<ErrorCode>,
    pub body: Body,
}

impl Answer {
    /// A refusal with nothing more to say.
    pub const fn refusal(status: Status, code: ErrorCode) -> Self {
        Self {
            status,
            error: Some(code),
            body: Body::Empty,
        }
    }

    /// A success.
    pub const fn success(status: Status, body: Body) -> Self {
        Self {
            status,
            error: None,
            body,
        }
    }
}

/// An answer as a report's line: the status, the code, and the body by what identifies it, its bytes
/// by their length.
impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.status)?;
        if let Some(code) = self.error {
            write!(f, " {}", code.as_str())?;
        }
        let with = |content: bool| if content { ", with its bytes" } else { "" };
        match &self.body {
            Body::Empty => Ok(()),
            Body::Blob {
                digest,
                size,
                content,
            } => write!(
                f,
                " blob {digest} ({size} bytes{})",
                with(content.is_some())
            ),
            Body::Part {
                digest,
                size,
                first,
                last,
                ..
            } => write!(f, " blob {digest} bytes {first}-{last}/{size}"),
            Body::Unsatisfiable { size } => write!(f, " bytes */{size}"),
            Body::Manifest {
                digest,
                content_type,
                size,
                content,
            } => write!(
                f,
                " manifest {digest} ({content_type}, {size} bytes{})",
                with(content.is_some())
            ),
            Body::BlobCreated { digest } => write!(f, " blob {digest}"),
            Body::ManifestCreated { digest, subject } => {
                write!(f, " manifest {digest}")?;
                match subject {
                    Some(subject) => write!(f, ", subject {subject}"),
                    None => Ok(()),
                }
            }
            Body::Upload { session, offset } => write!(f, " session {} at {offset}", session.0),
            Body::Tags { tags, more, .. } => {
                let tags: Vec<&str> = tags.iter().map(Tag::as_str).collect();
                write!(f, " tags [{}]", tags.join(", "))?;
                if *more {
                    f.write_str(" and more")
                } else {
                    Ok(())
                }
            }
            Body::Referrers {
                referrers,
                filtered,
            } => {
                write!(f, " {} referrers", referrers.len())?;
                if *filtered {
                    f.write_str(", filtered")
                } else {
                    Ok(())
                }
            }
            Body::Reference { digest } => write!(f, " naming {digest}"),
        }
    }
}

use ErrorCode as E;
use Status as S;

/// Every answer the model gives, by endpoint: its status and, for a refusal, its code
/// (docs/design/model.md, "Answers", which a test holds to this table). The spec's table lists some
/// failures Reliquary never gives, and model.md says why for each.
pub const ANSWERS: &[(Endpoint, Status, Option<ErrorCode>)] = &[
    (Endpoint::End1, S::Ok, None),
    (Endpoint::End1, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End2, S::Ok, None),
    (Endpoint::End2, S::PartialContent, None),
    (Endpoint::End2, S::RangeNotSatisfiable, None),
    (Endpoint::End2, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End2, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End2, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End2, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End2, S::Forbidden, Some(E::Denied)),
    (Endpoint::End2, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End2, S::NotFound, Some(E::BlobUnknown)),
    (Endpoint::End3, S::Ok, None),
    (Endpoint::End3, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End3, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End3, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End3, S::BadRequest, Some(E::ManifestInvalid)),
    (Endpoint::End3, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End3, S::Forbidden, Some(E::Denied)),
    (Endpoint::End3, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End3, S::NotFound, Some(E::ManifestUnknown)),
    (Endpoint::End4a, S::Accepted, None),
    (Endpoint::End4a, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End4a, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End4a, S::Forbidden, Some(E::Denied)),
    (Endpoint::End4b, S::Created, None),
    (Endpoint::End4b, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End4b, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End4b, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End4b, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End4b, S::Forbidden, Some(E::Denied)),
    (
        Endpoint::End4b,
        S::TooManyRequests,
        Some(E::TooManyRequests),
    ),
    (Endpoint::End5, S::Accepted, None),
    (Endpoint::End5, S::BadRequest, Some(E::BlobUploadInvalid)),
    (Endpoint::End5, S::BadRequest, Some(E::SizeInvalid)),
    (Endpoint::End5, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End5, S::Forbidden, Some(E::Denied)),
    (Endpoint::End5, S::NotFound, Some(E::BlobUploadUnknown)),
    (
        Endpoint::End5,
        S::RangeNotSatisfiable,
        Some(E::BlobUploadInvalid),
    ),
    (Endpoint::End5, S::TooManyRequests, Some(E::TooManyRequests)),
    (Endpoint::End6, S::Created, None),
    (Endpoint::End6, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End6, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End6, S::BadRequest, Some(E::BlobUploadInvalid)),
    (Endpoint::End6, S::BadRequest, Some(E::SizeInvalid)),
    (Endpoint::End6, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End6, S::Forbidden, Some(E::Denied)),
    (Endpoint::End6, S::NotFound, Some(E::BlobUploadUnknown)),
    (
        Endpoint::End6,
        S::RangeNotSatisfiable,
        Some(E::BlobUploadInvalid),
    ),
    (Endpoint::End6, S::TooManyRequests, Some(E::TooManyRequests)),
    (Endpoint::End7, S::Created, None),
    (Endpoint::End7, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End7, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End7, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End7, S::BadRequest, Some(E::ManifestInvalid)),
    (Endpoint::End7, S::BadRequest, Some(E::ManifestBlobUnknown)),
    (Endpoint::End7, S::BadRequest, Some(E::SizeInvalid)),
    (Endpoint::End7, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End7, S::Forbidden, Some(E::Denied)),
    (Endpoint::End7, S::ContentTooLarge, Some(E::ManifestInvalid)),
    (Endpoint::End8a, S::Ok, None),
    (Endpoint::End8a, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End8a, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End8a, S::Forbidden, Some(E::Denied)),
    (Endpoint::End8a, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End8b, S::Ok, None),
    (Endpoint::End8b, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End8b, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End8b, S::Forbidden, Some(E::Denied)),
    (Endpoint::End8b, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End9, S::Accepted, None),
    (Endpoint::End9, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End9, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End9, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End9, S::BadRequest, Some(E::ManifestInvalid)),
    (Endpoint::End9, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End9, S::Forbidden, Some(E::Denied)),
    (Endpoint::End9, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End9, S::NotFound, Some(E::ManifestUnknown)),
    (Endpoint::End9, S::MethodNotAllowed, Some(E::Unsupported)),
    (Endpoint::End10, S::Accepted, None),
    (Endpoint::End10, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End10, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End10, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End10, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End10, S::Forbidden, Some(E::Denied)),
    (Endpoint::End10, S::NotFound, Some(E::NameUnknown)),
    (Endpoint::End10, S::NotFound, Some(E::BlobUnknown)),
    (Endpoint::End10, S::MethodNotAllowed, Some(E::Unsupported)),
    (Endpoint::End11, S::Created, None),
    (Endpoint::End11, S::Accepted, None),
    (Endpoint::End11, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End11, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End11, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End11, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End11, S::Forbidden, Some(E::Denied)),
    (Endpoint::End12a, S::Ok, None),
    (Endpoint::End12a, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End12a, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End12a, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End12a, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End12a, S::Forbidden, Some(E::Denied)),
    (Endpoint::End12b, S::Ok, None),
    (Endpoint::End12b, S::BadRequest, Some(E::NameInvalid)),
    (Endpoint::End12b, S::BadRequest, Some(E::DigestInvalid)),
    (Endpoint::End12b, S::BadRequest, Some(E::Unsupported)),
    (Endpoint::End12b, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End12b, S::Forbidden, Some(E::Denied)),
    (Endpoint::End13, S::NoContent, None),
    (Endpoint::End13, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End13, S::Forbidden, Some(E::Denied)),
    (Endpoint::End13, S::NotFound, Some(E::BlobUploadUnknown)),
    (Endpoint::End14, S::NoContent, None),
    (Endpoint::End14, S::Unauthorized, Some(E::Unauthorized)),
    (Endpoint::End14, S::Forbidden, Some(E::Denied)),
    (Endpoint::End14, S::NotFound, Some(E::BlobUploadUnknown)),
];
