# Seams

Status: built, 2026-11-20 (plan RQ0.4, `crates/reliquary-core/src/seams`); not merged, not live.

Reliquary reaches every other product through a seam (CLAUDE.md §9). Its records go through
Amphora's `Records` (`amphora-api`), and its events through Quiver's `Topics` (`quiver-api`).
Where no product's `-api` crate gives a seam yet, the traits are Reliquary's own:

- `Bytes`: objects' bytes, on Mantle.
- `Secrets`: a named credential, leased, from Veil.
- `Upstream`: another registry, for pull-through.
- `Clock`: time.
- `Entropy`: unpredictable bytes.

Each seam has three parts. Its trait. A simulator, in memory, that can inject every fault the trait
can meet. A conformance suite, which the simulator passes now and the real implementation must pass
when it lands (Mantle's in RQ1.8, Veil's in RQ8.6, Amphora's store in RQ1.8).

Each section below gives what the seam is, what implements it, its methods, the faults its
simulator injects, and its contract, clause by clause. A clause's id is its conformance case's,
and its words are the case's. `crates/reliquary-core/tests/doc_truth.rs` fails when this page and
the code differ:

- a method, a fault kind or a clause added, removed or reworded on one side only;
- a refusal the trait names that its simulator cannot inject;
- a method that takes a timeout as a number.

## What every seam holds to

**Sans-IO.** No trait names a socket, a file, a runtime or a clock of its own. Time comes into a
call as a deadline its caller derives from a `Clock`. Randomness comes from an `Entropy`. Only a
shell's implementation does I/O. The simulators do none: they run in memory, on a simulated
clock, and a run is a function of its seed.

**Deadlines, never timeouts (AC-RQ0.4.5).**

- Every method that waits takes an `Instant` its caller derives: its own now plus what it learned
  that path's answers take (architecture D8, plan RQ0.8). No method takes a `Duration` or a number
  of seconds.
- Two methods take no deadline, because neither waits: `Clock::now` and `Clock::wall` answer at
  once. `Entropy::fill` waits for a source not yet ready (Linux's `getrandom` blocks until its pool
  is seeded), so it takes a deadline like the rest.
- Amphora's methods take `until: Instant`.
- Quiver's methods are futures and take no deadline. A caller holds each to the deadline it
  derived with `clock::within`, which drops the call at the deadline; what the call did by then,
  the caller learns by reading (TOPICS-5).

**Bounded pieces (AC-RQ0.4.4).**

- A body moves as pieces: into an upload at most the store's plan's piece at a time, and out of a
  reading or a fetch into its caller's own buffer.
- Records and messages move a page or a batch at a time. A credential, a fill and a sleep are each
  one bounded thing, and nothing of one stays once it is done.
- No method carries a whole object, so what moves through a seam costs memory that does not grow
  with it.
- `tests/memory.rs` measures this for every seam, with hyper-measure's counting allocator as the
  process's allocator. Each test moves 64 and 1024 times the least through its seam, and the heap's
  peak must stay at the least's. What the seam's far side keeps (a store's objects, an upstream's
  blobs, a table's rows, a topic's messages) is made before the count, since it is that side's,
  not the seam's.

  | Seam | What moved: the least, then 64 and 1024 times it | Peak heap at each |
  |---|---|---|
  | `Bytes` | an object of one piece of 16 KiB put and got back (T-RQ0.4.1) | 21,584 bytes: one piece and 5,200 of counted overhead |
  | `Upstream` | a blob of one piece of 16 KiB fetched whole, held to its digest | 17,278 bytes |
  | `Records` | a table of one page of 16 rows scanned, and its change feed read, a page at a time | 1,228 bytes |
  | `Topics` | a topic of one batch of 16 messages read a batch at a time | 1,496 bytes |
  | `Secrets` | a credential leased and given back | 1,292 bytes |
  | `Entropy` | a caller's buffer of one piece of 16 KiB filled | 400 bytes |
  | `Clock` | a sleep | 456 bytes |

  The 1024-piece object is 16 MiB, 777 times the peak of its put and get. These are counts of the
  code's allocations, not times: measured on macOS arm64 on 2026-11-20, they are the same on any
  machine of the same pointer width.
- Writing these tests found two things a simulator held that grew with what it served, both
  fixed. The records simulator's scan gathered every key under its prefix before taking a page.
  Its upload and lease names grew a digit as their count grew, and are now of one width.

**Faults (AC-RQ0.4.3).** A test, or a nemesis (plan RQ0.6), arms a fault for one operation of a
simulator, and that operation's next call takes it. One fault is armed per operation at a time:
arming a second is refused (`Unarmed::Armed`). A fault an operation cannot meet is refused too
(`Unarmed::Unfit`), so no fault is armed and then silently never applied. The kinds are:

| Kind | What the simulator does | What the caller sees |
|---|---|---|
| a refusal (each of its trait's) | refuses the call at once; nothing applies | that refusal, after one attempt |
| `answer delayed` | applies the call, holds its answer for the time armed | the answer, or, past its deadline, a late refusal at the deadline (`Late`, Amphora's `Timeout`) |
| `request vanished` | applies nothing, answers nothing | a late refusal at its deadline |
| `answer lost` | applies the call, loses its answer | `Lost` (Amphora's `UnknownOutcome`; Quiver's `Unavailable`), which it settles by reading |
| `stream cut` | cuts a stream once it reaches the byte, row or message armed | what came before the cut, then a `Cut` refusal or a page or batch that stops there |

Each simulator counts what it injected, by kind. Each suite's run gathers the counts, and
T-RQ0.4.9 holds every kind of every seam above zero.

**Suites (AC-RQ0.4.2).** Kiln-seams' shape (kiln's `crates/kiln-seams/src/conformance.rs`): a
case per clause, each run against a fresh fixture by `conformance::run`. A fixture runs a case's
work as its implementation runs. A simulator's fixture moves its simulated clock to the next sleep
whenever nothing else can go on (`SimClock::run`), and fails work that waits on nothing its clock
can end, so a deadlock is found rather than waited on. No case waits on the wall clock. Every size
a case uses is its fixture's, which the test that makes it states. The suites live in `src/`, so a
real implementation's tests in another crate run the same cases; they hold no picked number
(`docs/design/numbers.md`, plan RQ0.2).

## Decisions

### S1. Each trait is asynchronous, and each method that waits takes a deadline

**Chosen.**

- A seam's methods return boxed, sendable futures (`seams::BoxFuture`), and every one that waits
  takes an `Instant` deadline.
- Amphora's `Records` blocks until its deadline, and Quiver's `Topics` takes none; both are taken
  as their owners wrote them.

**Rests on.**

- Kiln's seams are futures ("Each module holds the trait, its simulator (`Sim*`: deterministic, in
  memory, bounded) and its conformance suite", kiln's `crates/kiln-seams/src/lib.rs`), as Quiver's
  are (research/08 F11).
- Mantle's client takes an absolute deadline on the shard's clock (`deadline_ns`, research/08 F5).
- Amphora's methods take `until: Instant` (research/08 F9).
- The person's rule on waits: "The cold start, and every wait after a run-out, follows the party's
  liveness and progress. After that the bound is the slowest answer seen" (research/01 §5). A
  deadline lets a caller apply it; a timeout fixed in the call would not.

**Falsified by.** A real implementation that cannot keep a caller's deadline without a number of
its own; the shells of RQ1.8 are where that would show.

### S2. A put streams into an upload that resumes from its durable offset

**Chosen.** `begin`, then `send` of pieces at the upload's next byte, `status`, `finish` held to a
length and SHA-256, and `abort`. The store's plan names the largest piece and the run its bytes
become durable in.

- A cut drops what was not yet durable. The caller learns the durable offset (from the `Cut`
  refusal, or from `status`) and sends on from there.
- No byte before it is sent twice, and at most one run's bytes are sent again (BYTES-4, T-RQ0.4.3).
- A send at another byte is refused `Offset`, naming the next, as the distribution spec answers
  an out-of-order chunk with 416 (research/02 §1).

**Rests on.** Mantle's native upload: "runs of the length the node cuts, each its own exchange
carrying its checksum in a trailer and answered with its progress once it committed. An exchange
that ends unanswered is resumed from the node's durable offset, the source read again from there,
so no committed run is sent again" (research/08 F3, F4).

**Falsified by.** Mantle's native door unable to tell a client its durable offset after a cut, or
cutting runs the client cannot learn (RQ1.8).

### S3. Create-once is a condition at the begin and at the finish

**Chosen.**

- `Put::create_once` is `If-None-Match: *`.
- A key that holds an object is refused `Exists` at the begin, which saves sending the bytes, and
  at the finish, which is the one that counts.
- Of two create-once puts finishing at once, exactly one wins (BYTES-7, T-RQ0.4.5).

**Rests on.** Kiln's plan for the track: "content-addressed blobs are created once, with
`If-None-Match: *`" (research/01 §1); architecture D3.

**Falsified by.** Two create-once finishes of one key both answered as made.

### S4. A begin carries its operation's identity

**Chosen.**

- `Put::identity` is 128 bits from the caller's `Entropy`. A begin asked again with it, after its
  answer was lost, is answered with the upload it made (BYTES-16).
- So a lost answer leaves no upload its caller cannot name. A finish asked again after it applied
  is refused `NoUpload`, and reading the key settles it.

**Rests on.** Mantle's client: "Every mutation carries its operation's identity ... which each
attempt carries unchanged", and "asking it again with its identity is answered from what it made"
(research/08 F6).

**Falsified by.** An upload made twice for one identity.

### S5. Ranges are RFC 9110's, decided by one function

**Chosen.**

- `range::resolve` decides what a range selects of a body, for every seam that reads by range and
  every implementation.
- A start at or past the end, or a zero suffix, is unsatisfiable and carries the body's length
  for the 416's `Content-Range: bytes */length`.
- A last byte before the first is invalid.
- An end past the body stops at its last byte.
- A suffix of an empty body selects all of it, which is nothing.

**Rests on.** RFC 9110 §14.1.1, §14.1.2, §14.4, §15.5.17 (research/08 F1, F2).

**Falsified by.** A range the conformance suite of the distribution spec (RQ1) answers otherwise.

### S6. A grant names one key, one method, one length, one holder and one expiry, under the store's tag

**Chosen.**

- A grant carries those five, and an HMAC-SHA256 tag the store takes over them in an encoding no
  two grants share: the method, the length, the expiry in nanoseconds since the Unix epoch, and the
  key and the holder, each after its length.
- Mantle's side refuses any of them changed (`Forbidden`), a presenter other than the holder
  (`Forbidden`), and a grant past its expiry by its own wall clock (`Expired`); BYTES-13, T-RQ0.4.8.
- Its wall clock as the store judges a grant by it is the latest it has read: a clock stepped back
  never brings back a grant it refused as expired (plan RQ0.6, T-RQ0.6.6; research/12 F13).
- The expiry is a wall time the caller derives (architecture D4: the transfer's bytes over the
  path's learned rate), never a lifetime written down.

**Rests on.**

- The plan's request to Mantle's owner: "A grant its native door honors: one principal, one
  object, one method, one length, one life" (`plan/README.md`).
- HMAC is a MAC: without the key no one makes a valid tag for other fields (research/08 F7).
- Kiln's presigned URLs sign the method, the key, the expiry and a put's length (research/08 F8).

**Falsified by.** Mantle's grant, when its owner defines it, naming something this one does not.
The simulator's encoding then follows Mantle's.

### S7. An answer lost or late is a typed refusal, settled by reading

**Chosen.**

- An answer lost after its call applied is `Lost`; one not come by the caller's deadline is
  `Late`. Each says the call may have applied.
- The caller settles it by reading: `head`, `status`, `get`, `bounds`.
- Every mutation is conditioned so that asking it again never applies it twice: a create-once, a
  `NoUpload` finish, an `Absent` or `At` condition (BYTES-15, BYTES-16, RECORDS-10, T-RQ0.4.6).
- No seam asks a call again itself (BYTES-17, RECORDS-13, SECRETS-9, UPSTREAM-10, T-RQ0.4.7).
  Whether and when to ask again is the caller's, by the path's learned waits (plan RQ0.8).

**Rests on.** Amphora's own `UnknownOutcome`: "the commit outcome is unknown; reconcile it before
retrying the operation" (research/08 F9). Mantle's `Lost`: "a mutation may have been applied"
(research/08 F6).

**Falsified by.** A history in which a caller that followed these clauses applied an operation
twice: the simulated world's runs (plan RQ0.6).

### S8. A credential is leased and never stored

**Chosen.**

- A lease gives a `Credential`, which has no `Clone` and no serialization, says only its length
  in `Debug`, and is overwritten when dropped.
- A lease ends at a wall time the store sets.
- A lease whose answer was lost, or whose value was cut, is the store's to end, so nothing leaks
  (SECRETS-6, SECRETS-8).

**Rests on.** Architecture D16 ("Signing is a lease through the `Secrets` seam; Reliquary holds no
key") and D25.

**Falsified by.** A credential's bytes found in a record, a log or a core dump of a test run.

### S9. An upstream is not trusted

**Chosen.**

- A body asked by digest is hashed as it arrives, and one that does not hash to its digest ends
  `Mismatch`, never as a whole body (UPSTREAM-4).
- A manifest's head costs none of the upstream's budget and its get costs one, as the largest
  public registry counts them (UPSTREAM-2, UPSTREAM-9).

**Rests on.** Docker Hub counts a manifest `GET` as a pull and a `HEAD` as none (research/05 F31
to F36); architecture D17.

**Falsified by.** An upstream whose budget a head does spend: the simulator then charges heads by
that upstream's own statement.

### S10. A simulation finds its deadlocks

**Chosen.** `SimClock::run` polls its work and, whenever the work waits, moves the clock to the
next sleep. Work that waits with no sleep left is given up (`Stalled::Waiting`), and work that
outlives its budget of steps is too.

**Rests on.** CLAUDE.md §4: "A test waits on the fact it needs, never on a wall-clock guess";
kiln's simulated clock (kiln's `crates/kiln-seams/src/clock.rs`).

**Falsified by.** Work that `run` gives up which would have ended.

### S11. Amphora's and Quiver's traits are used as their owners wrote them

**Chosen.**

- Reliquary implements no wrapper trait over `Records` or `Topics`.
- Its records simulator implements Amphora's trait. Amphora's methods block until their deadline,
  and nothing else runs in a simulation while a call blocks, so the simulator moves its simulated
  clock by the time a call takes: a delayed answer by its delay, an unanswered call or a wait for
  a change that never comes to the caller's deadline.
- Its topics simulator wraps Quiver's own (`FaultyTopics<T: Topics>`), generic over what it wraps,
  so the recursion gate sees no call through itself.

**Rests on.** CLAUDE.md §9: "Other products are reached only through their `-api` crates".

**Falsified by.** Reliquary needing of either product what its trait does not give. That is a
request to its owner (`plan/README.md`, "What the plan asks of other owners").

## Clock

Time: a monotonic now (what deadlines are derived from), the wall clock (what records are stamped
with and grants and leases end by), and sleeps.

- **Implementations:**
  - The node's clock on hyper-rt's timers (with RQ1.8).
  - `seams::clock::SimClock`, whose time moves only when it is moved (`advance`, `advance_to`, and
    `run`, which moves it to each next sleep). Its sleeps complete in deadline order, at most the
    number its maker states waiting at once; one past it is refused `Full`, never completed early.
    It keeps no waker: whoever moves the clock polls the work again, as `run` does after every move.
    A waker kept in the clock would be dropped with it, and a waker's drop runs its executor's code
    through a function pointer, which could drop the clock again: a cycle CLAUDE.md §11 refuses.
- **Used by:** every other seam's deadlines, and their simulators' held answers.

Methods: `now`, `wall`, `sleep_until`.

Faults: `Full`, `Ended`, `Lost`, `answer delayed`, `answer lost`, `stream cut`.

For a clock, each kind means:

- **A delayed answer:** a sleep woken late, past its deadline, as a descheduled thread wakes.
- **A lost answer:** a sleep that was armed, holding its place in the clock's bound until its
  deadline, while its caller is told `Lost` and arms another. A lost answer must leak no sleep.
- **A cut stream:** the clock's stream of wakes cut at an instant, as a runtime closing mid-run.
  Every sleep due by then completes, and every later one is refused `Ended`. `tests/seams.rs` cuts
  a fresh clock at each tenth of a quantum from before three sleeps' deadlines to after them, and
  the rule holds at each instant.

- **CLOCK-1** now() never goes back, and moves by the time that passes
- **CLOCK-2** wall() moves forward by the time that passes
- **CLOCK-3** a sleep until now, or until a deadline already past, completes at once
- **CLOCK-4** a sleep never completes before its deadline, and completes once it has passed
- **CLOCK-5** sleeps complete in deadline order, whatever order they started in
- **CLOCK-6** a sleep woken late completes after its deadline, and now() says how late
- **CLOCK-7** past the most sleeps a clock holds, a sleep is refused Full at once, never completed early
- **CLOCK-8** once the clock ends, every sleep waiting and every later one is refused Ended
- **CLOCK-9** a sleep whose answer is lost once it was armed is refused Lost, and holds its place until its deadline and no longer: a lost answer leaks no sleep
- **CLOCK-10** a clock whose stream of wakes is cut at an instant completes every sleep due by then, and refuses every later one Ended

## Entropy

Bytes no one can predict: an operation's identity, an upload's key, a grant's key. A fill waits
for a source not yet ready until its caller's deadline: a machine's source can be unready for a
while after it starts, as Linux's `getrandom` blocks until its pool is seeded, and an interrupted
call can give part of what was asked (research/08 F13).

- **Implementations:**
  - The operating system's source (with RQ1.8).
  - `seams::entropy::SimEntropy`, whose bytes are a `Stream`: SHA-256 of its seed and a block's
    number, block after block. So a simulated run is a function of its seed. A test draws content
    or a key from a `Stream` directly, with no clock.
- **Used by:** `Put::identity`, and the bytes simulator's grant key, drawn by its maker.

Methods: `fill`.

Faults: `Unavailable`, `Late`, `Lost`, `Cut`, `answer delayed`, `request vanished`, `answer lost`,
`stream cut`.

For a source, each kind means:

- **A delayed answer:** a source not ready by the caller's deadline.
- **A lost answer:** a fill whose bytes were drawn and whose answer did not reach its caller. A
  fill asked again draws other bytes, so an identity is never drawn twice.
- **A cut stream:** a source that gave the first bytes of a fill and stopped. Such a fill is refused
  `Cut`, never taken as whole.

- **ENTROPY-1** a fill fills every byte asked, and no two operation identities drawn repeat
- **ENTROPY-2** a fill whose source is not ready by its caller's deadline, or whose answer never comes, is refused Late at the deadline
- **ENTROPY-3** a fill cut at any byte is refused Cut, naming the bytes given, never given as whole
- **ENTROPY-4** a fill whose answer is lost after its bytes were drawn is refused Lost, and asked again it draws other bytes
- **ENTROPY-5** every refusal the source gives reaches its caller as it is, after one attempt: the seam never asks again

## Bytes

Objects' bytes, under keys of their own (architecture D3, D4).

- A put streams into an upload that resumes after a cut, and the object appears only whole, held
  to its length and SHA-256.
- A get reads a range in pieces into its caller's buffer.
- Head, delete and create-once.
- A grant another party redeems at Mantle.

The implementations:

- Mantle, over `mantle-client` on hyper-rt (RQ1.8), with the grant its native door is asked for
  (RQ3.5).
- `seams::bytes::sim::SimBytes`, which keeps bytes in a `Store` its maker gives it and their facts
  in memory, holding the objects, uploads and bytes its maker states. Its Mantle side is `redeem`.

Used by: the upload and blob cores (RQ1.3, RQ1.8), the OCI door's redirects and the native door's
grants (RQ3.5, RQ4.5), collection (RQ9.3).

Methods: `begin`, `send`, `status`, `finish`, `abort`, `get`, `head`, `delete`, `grant`; a
body's `read`.

Faults: `NotFound`, `NoUpload`, `Exists`, `Offset`, `Mismatch`, `Unsatisfiable`, `Invalid`,
`TooLarge`, `Full`, `Busy`, `Forbidden`, `Expired`, `Cut`, `Late`, `Lost`, `Unavailable`,
`answer delayed`, `request vanished`, `answer lost`, `stream cut`.

- **BYTES-1** a put sent in pieces and finished whole is read back the same, and head says its length and SHA-256
- **BYTES-2** a piece larger than the upload's plan is refused TooLarge, and the upload is as it was
- **BYTES-3** a send is taken only at the upload's next byte; one at any other is refused Offset, naming the next
- **BYTES-4** a put cut at any byte keeps what was durable and nothing after it, within one run of the cut; sent on from there it finishes whole, and no byte before the durable offset is sent twice
- **BYTES-5** a finish whose length or SHA-256 is not what the upload received is refused Mismatch: nothing appears, and the upload stays open
- **BYTES-6** create-once on a key that holds an object is refused Exists, at begin and at finish, and the object there is unchanged
- **BYTES-7** of two create-once puts of one key finishing at once, exactly one wins; the other is refused Exists
- **BYTES-8** an object appears only whole, at its finish: until then its key shows what was there before
- **BYTES-9** a range is answered as RFC 9110 has it: the first byte, the last, and a suffix; an end past the object stops at its last byte; a start at or past the end, or a zero suffix, is refused Unsatisfiable with the object's length; a last byte before the first is refused Invalid; of an empty object only a suffix selects, and it selects nothing
- **BYTES-10** a reading fills its caller's buffer and no more, whatever its size: an object is never in memory whole
- **BYTES-11** a get or head of a key with no object is refused NotFound; a delete removes an object, and of a key with none is answered Absent
- **BYTES-12** a reading begun before a delete or an overwrite reads the object it began with, whole
- **BYTES-13** a grant lets its holder alone get, or put exactly its length, at its key until it expires: with its key, method, length, expiry or holder changed it is refused Forbidden, and past its expiry Expired
- **BYTES-14** a granted put of another length than its grant's is refused, and nothing appears
- **BYTES-15** an answer delayed past its caller's deadline is refused Late at the deadline, and what the call did is learned by reading
- **BYTES-16** an answer lost after its call applied is refused Lost; reading shows what it did, and asking again never applies it twice
- **BYTES-17** every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again
- **BYTES-18** a reading cut at any byte gives every byte before the cut, then is refused Cut there; a reading from there gives the rest
- **BYTES-19** an upload given up is gone: a send or a finish to it is refused NoUpload, and nothing appears
- **BYTES-20** the uploads a store holds open are bounded: past the most, a begin is refused Full, and none open is dropped

## Records

Amphora's `Records` (`amphora-api`): rows by family and key, each at a revision; conditional
transactions; one change feed, kept back to a horizon (architecture D2).

- **Implementations:**
  - Amphora's store, embedded or a cluster (RQ1.8).
  - `seams::records::SimRecords`, rows in memory holding the rows and changes its maker states.
    Past its changes, its oldest revisions go whole and its horizon rises past them. It serves a
    read at a revision from the changes it keeps since, and moves its simulated clock by the time
    a call takes (S11).
- **Used by:** every core's records (RQ1.2 onward).

Methods: `get`, `scan`, `commit`, `changes`, `wait`, `head`, `compact`, `compact_step`.

Faults: `Conflict`, `TooLarge`, `Invalid`, `Full`, `Compacted`, `Ahead`, `Timeout`,
`Unavailable`, `UnknownOutcome`, `NotLeader`, `Fenced`, `answer delayed`, `request vanished`,
`answer lost`, `stream cut`.

- **RECORDS-1** a commit whose conditions hold applies every write at one new revision: a get gives each row at it, and head is it
- **RECORDS-2** a commit with an Absent condition on a row that exists, or an At condition at another revision, is refused Conflict naming the row's revision, and applies none of its writes
- **RECORDS-3** of two commits conditioned on one row at one revision, exactly one applies; the other is refused Conflict
- **RECORDS-4** revisions only grow: each commit's is above every earlier one's
- **RECORDS-5** a scan gives a family's rows under a prefix in key order, a page of at most its limit at a time; going on after each page's last key gives every row once
- **RECORDS-6** the change feed gives every change after a revision, oldest first, a delete as no value; a revision ahead of head is refused Ahead
- **RECORDS-7** a read at a revision gives the rows as they were then; once compacted past it, it is refused Compacted, naming it
- **RECORDS-8** a wait completes once a change of its family after its revision exists, and is refused Timeout at its deadline when none comes
- **RECORDS-9** a transaction past amphora-api's bounds, or with an empty key or a family outside its alphabet, is refused before anything applies
- **RECORDS-10** a commit whose answer is lost after it applied is refused UnknownOutcome; a read shows it, and asked again under its condition it is refused Conflict: nothing applies twice
- **RECORDS-11** an answer delayed past its caller's deadline is refused Timeout at the deadline, and what the call did is learned by reading
- **RECORDS-12** a scan or a change read cut at any row gives the rows before the cut; going on from the last gives the rest, none twice
- **RECORDS-13** every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again

## Topics

Quiver's `Topics` (`quiver-api`): topics of messages, each given the topic's next offset, read from
any offset kept or followed by a subscription (architecture D26: Reliquary publishes its events to
them from its records' change feed).

- **Implementations:**
  - Quiver's node (RQ7.13).
  - `quiver_api::sim::SimTopics`, Quiver's simulator, inside `seams::topics::FaultyTopics`, which
    injects what Quiver's simulator does not. With nothing armed it is Quiver's simulator
    unchanged: it passes Quiver's own suite, every clause (`tests/seams.rs`).
  - A refused call, or one that never arrives, is never made of the store, since a store may apply
    a call as it is made.
- **Used by:** the event publisher (RQ7.13).
- **Deadlines:** Quiver's methods take none. Each call is held to its caller's deadline with
  `clock::within`, which drops it there; a request to Quiver's owner is a deadline on each method,
  as Amphora's have.

Methods: `create`, `delete`, `publish`, `read`, `wait`, `bounds`, `subscribe`; a subscription's
`poll_ready`, `take`, `position`.

Faults: `NoTopic`, `Exists`, `Invalid`, `TooLarge`, `Full`, `Behind`, `Unsupported`, `Unavailable`,
`NoQuorum`, `answer delayed`, `request vanished`, `answer lost`, `stream cut`.

- **TOPICS-1** a publish is given the topic's next offset, and a read from an offset gives the messages from it in publish order, with their keys and payloads
- **TOPICS-2** a topic created again with the same settings is unchanged; with other settings it is refused Exists
- **TOPICS-3** a read from an offset retention let go is refused Behind, naming the first kept
- **TOPICS-4** a publish whose answer is lost after it applied is refused Unavailable, and the topic's bounds and a read show it was kept
- **TOPICS-5** a call held to its caller's deadline is dropped there when its answer is delayed past it or never comes, and what it did is learned by reading
- **TOPICS-6** a read cut at any message gives the messages before the cut and where to go on from; reading on gives the rest, none twice
- **TOPICS-7** a subscription cut at any message gives every message before the cut, then ends; one from its position gives the rest, none skipped and none twice
- **TOPICS-8** every refusal the store gives reaches its caller as it is, after one attempt, and a refused publish applies nothing: the seam never asks again

## Secrets

A named credential, leased and never stored (S8): an upstream registry's credential for a cache
(RQ8.6), a signing lease (RQ7.10).

- **Implementations:**
  - Veil, once it has an `-api` crate (asked of Veil's owner, `plan/README.md`).
  - `seams::secrets::SimSecrets`, Veil's side in memory: credentials kept by name, and the leases
    held on them, at most the number its maker states, each for the life its maker states.
- **Used by:** the pull-through cache (RQ8.6), signing (RQ7.10).

Methods: `lease`, `release`.

Faults: `NotFound`, `Denied`, `Full`, `Busy`, `Cut`, `Late`, `Lost`, `Unavailable`,
`answer delayed`, `request vanished`, `answer lost`, `stream cut`.

- **SECRETS-1** a lease of a named credential gives its value, and an end after now
- **SECRETS-2** a credential the store does not keep is refused NotFound; one its holder may not lease, Denied
- **SECRETS-3** a credential's value never shows: its Debug says only its length
- **SECRETS-4** a lease given back is no longer held: given back again, it is answered NotHeld
- **SECRETS-5** the leases a store holds are bounded: past the most, a lease is refused Full, and none held is dropped
- **SECRETS-6** a lease whose answer is lost is held until its end, and no longer: a lost answer leaks nothing
- **SECRETS-7** an answer delayed past its caller's deadline is refused Late at the deadline, and a call that never arrived can be asked again
- **SECRETS-8** a lease whose value is cut at any byte is refused Cut, never given as a shorter value
- **SECRETS-9** every refusal the store gives reaches its caller as it is, after one attempt: the seam never asks again

## Upstream

Another registry, which a pull-through cache or a mirror reads from (RQ8; architecture D17): a
manifest by tag or digest, its head alone, and a blob by digest, whole or by range, each body in
pieces into its caller's buffer, held to its digest (S9).

- **Implementations:**
  - The distribution spec's client on hyper-rt (RQ8.1).
  - `seams::upstream::SimUpstream`, a registry in memory with a budget of pulls, which can be made
    to serve bytes that are not a digest's (`corrupt`).
- **Used by:** the pull-through cache and mirrors (RQ8).

Methods: `manifest`, `head`, `blob`; a body's `read`.

Faults: `NotFound`, `Unauthorized`, `Denied`, `TooMany`, `Unsatisfiable`, `Invalid`, `Mismatch`,
`Cut`, `Late`, `Lost`, `Unavailable`, `answer delayed`, `request vanished`, `answer lost`,
`stream cut`.

- **UPSTREAM-1** a manifest by tag or by digest gives its digest, its media type and its bytes, which hash to its digest
- **UPSTREAM-2** a manifest's head gives its facts without its body and costs none of the upstream's budget; its get costs one
- **UPSTREAM-3** a blob by digest gives its bytes in pieces into its caller's buffer, and a range is answered as RFC 9110 has it
- **UPSTREAM-4** a body that does not hash to the digest it was asked by ends refused Mismatch, never as a whole body
- **UPSTREAM-5** what the upstream does not hold is refused NotFound, a blob in a repository that does not hold it included
- **UPSTREAM-6** the upstream's limit reached is refused TooMany with its Retry-After, after one attempt: the seam never asks again
- **UPSTREAM-7** a body cut at any byte gives every byte before the cut, then is refused Cut there; a range from there gives the rest
- **UPSTREAM-8** an answer delayed past its caller's deadline, or a request that never arrives, is refused Late at the deadline
- **UPSTREAM-9** a manifest whose answer is lost still cost the upstream's budget: the budget it states is one lower
- **UPSTREAM-10** every refusal the upstream gives reaches its caller as it is, after one attempt: the seam never asks again
