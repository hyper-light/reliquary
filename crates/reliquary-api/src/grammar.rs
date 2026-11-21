//! The grammars every request passes through, as the specifications state them: repository names
//! and tags (distribution spec v1.1.1, "Pulling manifests"), digests (image spec v1.1.1,
//! descriptor.md, "Digests"), references, and the manifest media types the model tells apart
//! (docs/design/model.md, H4). Each check is one loop over its input, linear in its length.

use std::cmp::Ordering;
use std::fmt;

use aws_lc_rs::digest as sha;

/// The most bytes a tag may have: "`<reference>` as a tag MUST be at most 128 characters in length"
/// (distribution spec v1.1.1, "Pulling manifests"). Format: every character the tag grammar allows
/// is ASCII, so characters and bytes count alike.
pub const TAG_MAX_BYTES: usize = 128;

/// Hex digits per byte of a digest's encoded part. Format: base 16 writes each byte as two digits
/// (RFC 4648 §8), and the image spec's registered algorithms are hex (`/[a-f0-9]{64}/` for 32 bytes).
const HEX_DIGITS_PER_BYTE: usize = 2;

/// A repository name that matches the spec's expression
/// `[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// `text` as a name, when it matches the expression and, if `bound` is given, has at most that
    /// many bytes (clients limit a registry host, `/` and name together to 255 characters: the
    /// spec's implementer's note, which leaves the bound to the deployment).
    pub fn parse(text: &str, bound: Option<usize>) -> Option<Self> {
        if bound.is_some_and(|most| text.len() > most) || !name_matches(text) {
            return None;
        }
        Some(Self(text.to_owned()))
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a name's matcher stands after a byte: the expression read as a state machine.
#[derive(Clone, Copy, PartialEq, Eq)]
enum At {
    /// At a component's start, or after a separator: an `[a-z0-9]` must come.
    NeedAlnum,
    /// Inside a run of `[a-z0-9]`: the only state a name may end in.
    Alnum,
    /// After one `_`: a second `_` or an `[a-z0-9]`.
    Underscore,
    /// After `-`: more `-` or an `[a-z0-9]`.
    Hyphens,
}

fn name_matches(text: &str) -> bool {
    let mut at = At::NeedAlnum;
    for byte in text.bytes() {
        let alnum = byte.is_ascii_lowercase() || byte.is_ascii_digit();
        at = match (at, byte) {
            (_, _) if alnum => At::Alnum,
            (At::Alnum, b'/' | b'.') => At::NeedAlnum,
            (At::Alnum, b'_') => At::Underscore,
            (At::Underscore, b'_') => At::NeedAlnum,
            (At::Alnum | At::Hyphens, b'-') => At::Hyphens,
            _ => return false,
        };
    }
    at == At::Alnum
}

/// A tag: `[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}`. Tags order as the spec lists them (model.md, H6):
/// case-insensitively, and by their bytes where that ties, so the order is total.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tag(String);

impl Tag {
    /// `text` as a tag, when it matches the expression.
    pub fn parse(text: &str) -> Option<Self> {
        let mut bytes = text.bytes();
        let first_ok = bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        let rest_ok = bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
        (first_ok && rest_ok && text.len() <= TAG_MAX_BYTES).then(|| Self(text.to_owned()))
    }

    /// The tag as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The order of tag listings: ASCII case folded first, then the bytes themselves (model.md, H6).
/// It applies to any text, so a listing's `last` need not be a tag that exists.
pub fn tag_order(a: &str, b: &str) -> Ordering {
    let folded = a
        .bytes()
        .map(|c| c.to_ascii_lowercase())
        .cmp(b.bytes().map(|c| c.to_ascii_lowercase()));
    folded.then_with(|| a.as_bytes().cmp(b.as_bytes()))
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        tag_order(&self.0, &other.0)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The digest algorithms the image spec registers (descriptor.md, "Registered algorithms").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    /// The identifier a digest writes before its `:`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    fn hash(self) -> &'static sha::Algorithm {
        match self {
            Self::Sha256 => &sha::SHA256,
            Self::Sha512 => &sha::SHA512,
        }
    }

    /// How many hex digits the encoded part has: the hash's output, two digits a byte.
    fn hex_len(self) -> Option<usize> {
        self.hash().output_len().checked_mul(HEX_DIGITS_PER_BYTE)
    }

    fn named(text: &str) -> Option<Self> {
        [Self::Sha256, Self::Sha512]
            .into_iter()
            .find(|a| a.name() == text)
    }
}

/// A digest of a registered algorithm whose encoded part is that algorithm's lowercase hex.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest {
    algorithm: Algorithm,
    encoded: String,
}

/// Why text is not a digest the model can use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestError {
    /// It breaks the grammar, or its algorithm is registered and its encoded part is not that
    /// algorithm's (`DIGEST_INVALID`).
    Malformed,
    /// It keeps the grammar, but its algorithm is not registered: the image spec lets it "pass
    /// validation" (descriptor.md, "Digests"), and a registry cannot compute it (`UNSUPPORTED`,
    /// plan T-RQ1.1.4).
    Unregistered,
}

impl Digest {
    /// The digest of `bytes` under `algorithm`.
    pub fn of(algorithm: Algorithm, bytes: &[u8]) -> Self {
        let hash = sha::digest(algorithm.hash(), bytes);
        Self {
            algorithm,
            encoded: hash.as_ref().iter().map(|b| format!("{b:02x}")).collect(),
        }
    }

    /// `text` as a digest: `algorithm ":" encoded`, where `algorithm ::= algorithm-component
    /// (algorithm-separator algorithm-component)*`, `algorithm-component ::= [a-z0-9]+`,
    /// `algorithm-separator ::= [+._-]` and `encoded ::= [a-zA-Z0-9=_-]+` (descriptor.md).
    pub fn parse(text: &str) -> Result<Self, DigestError> {
        let (algorithm, encoded) = text.split_once(':').ok_or(DigestError::Malformed)?;
        if !algorithm_matches(algorithm) || !encoded_matches(encoded) {
            return Err(DigestError::Malformed);
        }
        let algorithm = Algorithm::named(algorithm).ok_or(DigestError::Unregistered)?;
        let lowercase_hex = encoded
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !lowercase_hex || Some(encoded.len()) != algorithm.hex_len() {
            return Err(DigestError::Malformed);
        }
        Ok(Self {
            algorithm,
            encoded: encoded.to_owned(),
        })
    }

    /// The algorithm it was computed with.
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.name(), self.encoded)
    }
}

fn algorithm_matches(text: &str) -> bool {
    // A separator may stand only between two components.
    let mut need_component = true;
    for byte in text.bytes() {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            need_component = false;
        } else if matches!(byte, b'+' | b'.' | b'_' | b'-') && !need_component {
            need_component = true;
        } else {
            return false;
        }
    }
    !need_component
}

fn encoded_matches(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'=' | b'_' | b'-'))
}

/// What a manifest URL's `<reference>` names: "either (a) the digest of the manifest or (b) a tag".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Tag(Tag),
    Digest(Digest),
}

/// Why a `<reference>` is neither.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceError {
    /// It has a `:`, so it can only be a digest, and it is not one the model can use.
    Digest(DigestError),
    /// It has no `:`, so it can only be a tag, and it does not match the tag expression.
    Tag,
}

impl Reference {
    /// `text` as a reference: a digest when it has a `:` (no tag can), else a tag.
    pub fn parse(text: &str) -> Result<Self, ReferenceError> {
        if text.contains(':') {
            Digest::parse(text)
                .map(Self::Digest)
                .map_err(ReferenceError::Digest)
        } else {
            Tag::parse(text).map(Self::Tag).ok_or(ReferenceError::Tag)
        }
    }
}

/// The OCI image manifest (image spec v1.1.1, manifest.md).
pub const OCI_IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// The OCI image index (image spec v1.1.1, image-index.md).
pub const OCI_IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// Docker's image manifest, schema 2, which the OCI manifest's fields follow.
pub const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
/// Docker's manifest list, schema 2, which the OCI index's fields follow.
pub const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
/// Docker's schema 1 manifest, refused by name (plan T-RQ1.1.8).
pub const DOCKER_SCHEMA1: &str = "application/vnd.docker.distribution.manifest.v1+json";
/// Docker's signed schema 1 manifest, refused by name (plan T-RQ1.1.8).
pub const DOCKER_SCHEMA1_SIGNED: &str = "application/vnd.docker.distribution.manifest.v1+prettyjws";
/// The empty descriptor's media type (image spec v1.1.1, manifest.md, "Guidance for an Empty
/// Descriptor").
pub const OCI_EMPTY: &str = "application/vnd.oci.empty.v1+json";

/// How the model reads a manifest of a media type (model.md, H4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestKind {
    /// An image manifest: a config and layers, each a blob of the repository.
    Image,
    /// An index: manifests of the repository.
    Index,
    /// Docker's schema 1, refused.
    Schema1,
    /// A media type the model does not know: stored and served, never checked (plan AC-RQ1.5.8).
    Unknown,
}

/// A media type without its parameters: "The registry SHOULD ignore parameters on the
/// `Content-Type` header" (distribution spec v1.1.1, "Pushing Manifests").
pub fn essence(media_type: &str) -> &str {
    media_type
        .split_once(';')
        .map_or(media_type, |(essence, _)| essence)
        .trim()
}

/// The kind of a manifest pushed with this `Content-Type`, compared without parameters and
/// without case, as media types are (RFC 6838 §4.2).
pub fn manifest_kind(content_type: &str) -> ManifestKind {
    let essence = essence(content_type);
    let is = |known: &str| essence.eq_ignore_ascii_case(known);
    if is(OCI_IMAGE_MANIFEST) || is(DOCKER_MANIFEST) {
        ManifestKind::Image
    } else if is(OCI_IMAGE_INDEX) || is(DOCKER_MANIFEST_LIST) {
        ManifestKind::Index
    } else if is(DOCKER_SCHEMA1) || is(DOCKER_SCHEMA1_SIGNED) {
        ManifestKind::Schema1
    } else {
        ManifestKind::Unknown
    }
}
