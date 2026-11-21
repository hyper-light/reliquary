# 08. The seams: what Reliquary's traits to other products rest on

Status: research note, 2026-11-20, for plan RQ0.4 (`docs/design/seams.md`). Sources [1], [2] and [8]
were fetched on 2026-11-20 from the addresses given; [3] to [7] are read at the revisions named, in the
copies Reliquary vendors or in kiln's checkout. Quotes are verbatim; a conclusion that is derived
says so and from what.

## Sources

[1] RFC 9110, "HTTP Semantics" (June 2022), §14.1.1, §14.1.2, §14.4, §15.5.17,
https://www.rfc-editor.org/rfc/rfc9110.txt

[2] RFC 2104, "HMAC: Keyed-Hashing for Message Authentication" (February 1997), §1, §6,
https://www.rfc-editor.org/rfc/rfc2104.txt

[3] mantle at `87ab09e1` (vendored, `vendor/mantle/crates/client/src/lib.rs`): the native client's
crate documentation, `Client::call`, `Client::upload`, `ClientError`.

[4] amphora at `db991a05` (vendored, `vendor/amphora/crates/api/src/lib.rs`): `amphora-api`'s crate
documentation, `Records` and `Refusal`.

[5] quiver at `966ea139` (vendored, `vendor/quiver/crates/quiver-api/src/lib.rs`): `quiver-api`'s
crate documentation and `Topics`.

[6] kiln at `e64eeb7c` (`crates/kiln-seams/src/blobs.rs`, `crates/kiln-seams/src/conformance.rs`):
kiln's own seams, their simulators and suites.

[7] hyper-raft at `e0613f52` (vendored, `vendor/hyper-raft/crates/hyper-measure/src/alloc.rs`):
the counting allocator.

[8] Linux man-pages, `getrandom(2)`, https://man7.org/linux/man-pages/man2/getrandom.2.html (fetched
2026-11-20).

## Findings

**F1. A byte range's validity and what it selects** [1 §14.1.1, §14.1.2].

- "An int-range is invalid if the last-pos value is present and less than the first-pos."
- "If the last-pos value is absent, or if the value is greater than or equal to the current length
  of the representation data, the byte range is interpreted as the remainder of the
  representation".
- "If the selected representation is shorter than the specified suffix-length, the entire
  representation is used."
- A range is satisfiable if it is "an int-range with a first-pos that is less than the current
  length of the selected representation or a suffix-range with a non-zero suffix-length".
- "When a selected representation has zero length, the only satisfiable form of range-spec in a
  GET request is a suffix-range with a non-zero suffix-length."

**F2. An unsatisfiable range is answered with the representation's length** [1 §14.4, §15.5.17].

- "A server generating a 416 (Range Not Satisfiable) response to a byte-range request SHOULD send
  a Content-Range header field with an unsatisfied-range value".
- "The complete-length in a 416 response indicates the current length of the selected
  representation."
- A server "MAY ignore or reject a Range header field that contains an invalid ranges-specifier"
  [1 §14.2]. Derived: the seam refuses an invalid range typed (`Invalid`), and the door decides
  between ignoring it and rejecting it.

**F3. Mantle's client moves an object in bounded memory** [3]. "**Its memory does not grow with an
object.** A put's body comes from a [`Source`] and a get's bytes go to a [`Sink`], a piece at a
time".

**F4. Mantle's upload resumes from the node's durable offset** [3, `Client::upload`]. "Streams
`length` bytes of `source` into `native`'s object as a native upload (gateway.md §2.1), resumable:
runs of the length the node cuts, each its own exchange carrying its checksum in a trailer and
answered with its progress once it committed. An exchange that ends unanswered is resumed from the
node's durable offset, the source read again from there, so no committed run is sent again".

**F5. Mantle's client takes an absolute deadline** [3, `Client::call`]. "Asks `op`, an operation
that sends no object's bytes and gets none back, by `deadline_ns` on the shard's clock". Derived:
the seam's deadline is an `Instant` its caller derives, which a Mantle-backed `Bytes` turns into
the shard clock's `deadline_ns`.

**F6. Mantle names each mutation, and a lost answer may have applied** [3].

- "Every mutation carries its operation's identity ([`identity`]), which each attempt carries
  unchanged (gateway.md §2.1)."
- "[`ClientError::Lost`] says whether the request was sent whole, after which a mutation may have
  been applied, and asking it again with its identity is answered from what it made."

**F7. HMAC authenticates a message to whoever shares its key** [2].

- "Typically, message authentication codes are used between two parties that share a secret key in
  order to validate information transmitted between these parties." [2 §1]
- "The strongest attack known against HMAC is based on the frequency of collisions for the hash
  function H ("birthday attack") [PV,BCK2], and is totally impractical for minimally reasonable
  hash functions." [2 §6]
- Derived: a grant whose fields are changed fails its tag unless the changer holds the store's key,
  which only the store's side holds.

**F8. kiln's presigned URLs sign what they let their holder do** [6, `blobs.rs`]. "The signature
covers the method, the exact key, the expiry and, for a put, the length."

**F9. Amphora's methods take a deadline, and a timeout leaves the outcome open** [4].

- "Methods block until their deadline. Async hosts call them on their blocking pool. A timeout
  after submission leaves the outcome unknown: the transaction may subsequently commit."
- Its `Refusal::UnknownOutcome`: "the commit outcome is unknown; reconcile it before retrying the
  operation".
- Its `Refusal::Timeout`: "the deadline elapsed; a submitted transaction may still commit".

**F10. A counting allocator measures what a run holds at most** [7]. "A global allocator that
counts: allocations, reallocations, frees, the bytes asked for, and the bytes held now and at most
since the count began." It "counts on the calling thread only".

**F11. Quiver's topics are futures, with a simulator and a suite of their own** [5].

- "Topics of messages. Every method's future is `Send`, and none holds a lock across its awaits."
- "[`sim::SimTopics`] is the simulator: in memory, deterministic, every class served as ephemeral.
  [`conformance`] is the suite every implementation passes, the simulator and Quiver's node alike".

**F12. kiln's seams run each clause as a case against a fresh fixture** [6, `conformance.rs`].
"What every seam's conformance suite is made of: cases, each one clause of the seam's contract
(its id is the clause's in docs/seams.md), run against a fresh implementation apiece."

**F13. An operating system's source can be unready, and can give part of what was asked** [8].

- "If the urandom source has not yet been initialized, then getrandom() will block, unless
  GRND_NONBLOCK is specified in flags."
- "if the call is interrupted by a signal handler, it may return a partially filled buffer, or fail
  with the error EINTR."
- Derived: a fill takes its caller's deadline, since a source can be unready for a while, and the
  seam refuses a short fill (`Cut`) rather than give it as whole.

## What this note could not verify

- Mantle's grant for its native door does not exist yet (asked of Mantle's owner, plan RQ3.5).
  The simulator's grant (`docs/design/seams.md` S6) rests on F7 and F8 and on the plan's request,
  and follows Mantle's when its owner defines it.
- Veil has no `-api` crate (research/01 §3). `Secrets` is what Reliquary needs of it, written ahead
  of Veil's own.
