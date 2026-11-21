//! Byte ranges, as RFC 9110 §14.1 has them, for every seam that reads a body by range ([`Bytes`],
//! [`Upstream`]): one function decides what a range selects of a body of a given length, so the OCI
//! door, the native door and every implementation answer a range alike (T-RQ0.4.4).
//!
//! RFC 9110 §14.1.1: "An int-range is invalid if the last-pos value is present and less than the
//! first-pos." §14.1.2: "If the last-pos value is absent, or if the value is greater than or equal
//! to the current length of the representation data, the byte range is interpreted as the
//! remainder of the representation"; "If the selected representation is shorter than the
//! specified suffix-length, the entire representation is used"; and a range is satisfiable if it
//! is "an int-range with a first-pos that is less than the current length of the selected
//! representation or a suffix-range with a non-zero suffix-length", so "When a selected
//! representation has zero length, the only satisfiable form of range-spec in a GET request is a
//! suffix-range with a non-zero suffix-length". An unsatisfiable range is answered 416 with the
//! representation's current length (§14.4, §15.5.17), which [`Unresolved::Unsatisfiable`] carries.
//!
//! [`Bytes`]: crate::seams::bytes::Bytes
//! [`Upstream`]: crate::seams::upstream::Upstream

/// One byte range asked of a body (RFC 9110 §14.1.2): `bytes=first-last`, `bytes=first-`, or
/// `bytes=-suffix`. A set of several ranges is a door's to split into these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Range {
    /// An int-range: from byte `first`, through byte `last` (inclusive), or to the end.
    From { first: u64, last: Option<u64> },
    /// A suffix-range: the body's last this many bytes.
    Suffix(u64),
}

/// The bytes a satisfiable range selects: `length` of them, from byte `first`. A suffix of an
/// empty body selects all of it, which is no bytes, so a span can be empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub first: u64,
    pub length: u64,
}

impl Span {
    /// The whole of a body of `length` bytes.
    #[must_use]
    pub const fn whole(length: u64) -> Self {
        Self { first: 0, length }
    }

    /// The byte after the span's last: where it ends.
    #[must_use]
    pub const fn end(&self) -> u64 {
        // A span is resolved within a body whose length is a u64, so first + length never
        // exceeds that length.
        self.first.saturating_add(self.length)
    }
}

/// Why a range selects nothing of a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unresolved {
    /// The range is invalid: its last byte is before its first (RFC 9110 §14.1.1).
    #[error("the range's last byte {last} is before its first {first}")]
    Invalid { first: u64, last: u64 },
    /// The range is unsatisfiable for a body of `length` bytes (§14.1.2): answered 416 with
    /// `Content-Range: bytes */length`.
    #[error("the range selects nothing of the body's {length} bytes")]
    Unsatisfiable { length: u64 },
}

/// What `range` selects of a body of `length` bytes, as RFC 9110 §14.1.2 has it.
pub fn resolve(range: Range, length: u64) -> Result<Span, Unresolved> {
    match range {
        Range::From {
            first,
            last: Some(last),
        } if last < first => Err(Unresolved::Invalid { first, last }),
        Range::From { first, last } => {
            // Satisfiable only when the first byte is in the body; then the body has a last byte.
            let Some(end) = length.checked_sub(1).filter(|end| first <= *end) else {
                return Err(Unresolved::Unsatisfiable { length });
            };
            let last = last.map_or(end, |last| last.min(end));
            // first <= last <= end < length, so neither sum nor difference can leave u64.
            let span = last
                .checked_sub(first)
                .and_then(|n| n.checked_add(1))
                .ok_or(Unresolved::Unsatisfiable { length })?;
            Ok(Span {
                first,
                length: span,
            })
        }
        Range::Suffix(0) => Err(Unresolved::Unsatisfiable { length }),
        Range::Suffix(suffix) => {
            let length_taken = suffix.min(length);
            Ok(Span {
                first: length.saturating_sub(length_taken),
                length: length_taken,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-RQ0.4.4: the first byte, the last byte, an end past the body, a start past it, a zero
    /// suffix, an empty body, and a last before a first, each as RFC 9110 §14.1 answers it.
    #[test]
    fn ranges_resolve_as_rfc_9110_has_them() {
        let n = 10;
        let from = |first, last| Range::From { first, last };
        assert_eq!(
            resolve(from(0, Some(0)), n),
            Ok(Span {
                first: 0,
                length: 1
            })
        );
        assert_eq!(
            resolve(from(9, Some(9)), n),
            Ok(Span {
                first: 9,
                length: 1
            })
        );
        assert_eq!(
            resolve(Range::Suffix(1), n),
            Ok(Span {
                first: 9,
                length: 1
            })
        );
        assert_eq!(
            resolve(from(4, Some(400)), n),
            Ok(Span {
                first: 4,
                length: 6
            })
        );
        assert_eq!(
            resolve(from(4, None), n),
            Ok(Span {
                first: 4,
                length: 6
            })
        );
        assert_eq!(resolve(Range::Suffix(400), n), Ok(Span::whole(n)));
        assert_eq!(
            resolve(from(10, None), n),
            Err(Unresolved::Unsatisfiable { length: n })
        );
        assert_eq!(
            resolve(Range::Suffix(0), n),
            Err(Unresolved::Unsatisfiable { length: n })
        );
        assert_eq!(
            resolve(from(5, Some(4)), n),
            Err(Unresolved::Invalid { first: 5, last: 4 })
        );
        assert_eq!(
            resolve(from(0, None), 0),
            Err(Unresolved::Unsatisfiable { length: 0 })
        );
        assert_eq!(resolve(Range::Suffix(3), 0), Ok(Span::whole(0)));
        assert_eq!(
            resolve(from(u64::MAX - 1, None), u64::MAX),
            Ok(Span {
                first: u64::MAX - 1,
                length: 1
            })
        );
    }
}
