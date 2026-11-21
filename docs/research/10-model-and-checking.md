# 10. The reference model and how it is checked

Status: research note, 2026-11-20. Sources [1] to [11] (all but [5], §9) were fetched on 2026-11-20
from the addresses given and saved under `~/.cache/reliquary-research/tmp-model/`; quotes are
verbatim from the saved copies, checked by script (§8). Pages are the journal's for [1] and [2], the
copy's for [3], [4] and [9]. The design record of plan RQ0.5 (the reference model and the
differential harness) cites this note.

## Sources

[1] M. P. Herlihy and J. M. Wing, "Linearizability: A Correctness Condition for Concurrent Objects",
ACM TOPLAS 12(3):463-492, 1990, https://cs.brown.edu/people/mph/HerlihyW90/p463-herlihy.pdf (ACM scan)

[2] J. M. Wing and C. Gong, "Testing and Verifying Concurrent Objects", JPDC 17(1-2):164-182, 1993,
https://www.cs.cmu.edu/~wing/publications/WingGong93.pdf (a scan with no text layer, read by OCR)

[3] G. Lowe, "Testing for linearizability", Concurrency Computat.: Pract. Exper. 29(4):e3928, 2017,
the author's copy, https://www.cs.ox.ac.uk/people/gavin.lowe/LinearizabiltyTesting/paper.pdf

[4] A. Horn and D. Kroening, "Faster linearizability checking via P-compositionality", FORTE 2015,
LNCS 9039:50-65, as arXiv:1504.00204v1, https://arxiv.org/pdf/1504.00204

[5] P. B. Gibbons and E. Korach, "Testing Shared Memories", SIAM J. Comput. 26(4):1208-1244, 1997,
doi:10.1137/S0097539794279614. Not read (§9).

[6] Porcupine at `97cd067def` (2026-09-21): `README.md`, `model.go`, `checker.go`, `porcupine_test.go`,
https://raw.githubusercontent.com/anishathalye/porcupine/97cd067def/README.md (and siblings)

[7] Knossos at `14ead19fb3` (2026-07-17): `src/knossos/{history,wgl}.clj`, `src/knossos/wgl/dll_history.clj`,
https://raw.githubusercontent.com/jepsen-io/knossos/14ead19fb3/src/knossos/wgl.clj (and siblings)

[8] `jepsen.history`, Jepsen's history library, at `1a00decd14` (2026-11-12): `README.md`,
https://raw.githubusercontent.com/jepsen-io/history/1a00decd14/README.md

[9] A. Zeller and R. Hildebrandt, "Simplifying and Isolating Failure-Inducing Input", IEEE TSE
28(2):183-200, 2002, https://www.st.cs.uni-saarland.de/papers/tse2002/tse2002.pdf (pages 1 to 17)

[10] RFC 9110, "HTTP Semantics", June 2022, https://www.rfc-editor.org/rfc/rfc9110.txt

[11] RFC 6585, "Additional HTTP Status Codes", April 2012, https://www.rfc-editor.org/rfc/rfc6585.txt

## 1. Linearizability: what a concurrent history is judged by

- F1. "Linearizability provides the illusion that each operation applied by concurrent processes
  takes effect instantaneously at some point between its invocation and its response" [1 p. 463].
  "A history H is linearizable if it can be extended (by appending zero or more response events) to
  some history H’ such that:" "complete(H’) is equivalent to some legal sequential history S", and
  "this apparent sequential interleaving respects the real-time precedence ordering of operations."
  [1 p. 469] Legal: each object's part is in its sequential specification, "a prefix-closed set of
  single-object sequential histories" [1 p. 468].
- F2. Locality. "A property P of a concurrent system is said to be local if the system as a whole
  satisfies P whenever each individual object satisfies P. Linearizability is a local property:"
  Theorem 1: "H is linearizable if and only if, for each object x," H|x is linearizable [1 p. 470].

**Derived.** The model is the sequential specification, one pure function from state and operation
to next state and answer (AC-RQ0.5.1). A history passes when some order of its operations keeping
every real-time precedence replays on the model to the answers the clients saw.

## 2. The checker: Wing and Gong's search, Lowe's cache, a counted budget

- F3. The search. "we try every possible sequential order of H's concurrent operations while
  preserving its real-time order relation" [2 p. 170]. Lowe: "We say that an operation op is minimal
  in a given history if no return event of another operation is before the call of op; this means
  that op could be linearized first."; "The recursion of Figure 1 is replaced by an iteration,
  storing each operation op in a stack to support backtracking." [3 p. 4]
- F4. Lowe's cache makes it WGL. "The Wing & Gong Graph Search Algorithm overcomes this
  inefficiency, by using the standard technique of remembering which configurations have been seen
  earlier, using a hash set: here, a configuration comprises the state of the sequential object and
  the set of operations linearized so far."; it needs "the sequential object to be immutable"
  [3 p. 5]. "We call Lowe’s extension of Wing and Gong’s algorithm the WGL algorithm." [4 p. 2]
  Porcupine "implements the algorithm described in" Horn and Kroening's paper, "an optimization of
  the algorithm described in" Lowe's [6 README L18-21]; its step "Returns whether or not the system
  could take this step with the given inputs and outputs and also returns the new state."
  [6 model.go L109-111] Knossos's `wgl` is "An implementation of the Wing and Gong linearizability
  checker, plus Lowe's extensions" [7 wgl.clj L2-3].
- F5. Lowe's other algorithm is just-in-time, "since they linearize operations as late as possible"
  [3 p. 5]; against WGL, "the two algorithms have similar behaviour on a map, with the JIT Algorithm
  performing better on longer runs." [3 p. 15]
- F6. NP-complete. "Gibbons and Korach [11] show that testing a single history for linearizability
  is NP-complete in general." [3 p. 3] Horn and Kroening, citing the same paper: "checking
  linearizability is NP-complete [3]." [4 p. 2] Porcupine: "Checking for linearizability is
  decidable, but it is an NP-hard problem" [6 model.go L361]. Wing and Gong: "analyzing many short
  (100 operations) histories is tractable", and bugs are found "by testing many short histories
  rather than testing one long one." [2 p. 173]
- F7. Budgets and a third verdict. Lowe's graph search, N events by p threads, explores O(p^N)
  configurations at worst and at most (N+1)·2^p·(p+1) for a register (read from the page image):
  "thus the algorithm is linear in the length of the history; it is exponential in the number of
  threads" [3 p. 7]. His runs were capped by count: "we limited executions to 1.3 million
  iterations for the Wing & Gong Graph Search Algorithm" [3 p. 13]. Porcupine: "if a timeout is
  supplied, then some functions may return Unknown. Depending on the use case, you can interpret an
  Unknown result as Ok" [6 model.go L361-366].

**Derived.** Porcupine and Knossos's `wgl` build WGL, Wing and Gong's search with Lowe's cache of
(linearized set, model state) (Porcupine's `checkSingle`, `checker.go` L209-L333, read as code);
just-in-time linearization is not WGL, so the design names the one it builds. Model states are
values (comparable, hashable), the search loops over an explicit stack (CLAUDE.md §11), and a value
the server chooses (an upload's location) is an answer the step records. The checker counts
configurations, not time (CLAUDE.md §2, §4), and says linearizable, not (with the longest
linearizable prefix), or out of budget, never read as a pass. Histories are many and short (F6).

## 3. Splitting a history: P-compositionality

- F8. "A specification φ is called P-compositional whenever any history H is linearizable with
  respect to φ if and only if, for every history H′ ∈ P(H), H′ is linearizable with respect to
  φ." [4 p. 5] "For example, the operations on a concurrent unordered set and map are linearizable
  if and only if the restriction to each key is linearizable." [4 p. 2] It holds "because all
  operations in one partition are, informally speaking, unaffected by all operations in every
  other partition"; "a ‘size’ operation that returns the number of elements in a concurrent
  collection data type cannot be generally partitioned this way." [4 p. 6] Porcupine has "Partition
  functions, such that a history is linearizable if and only if each partition is linearizable."
  [6 model.go L102-103]

**Derived.** The registry splits by repository (inside one, tags, manifests, blobs and subjects
answer for each other). A cross-repository mount and the catalog are like `size`: an operation joins
the classes of the repositories it touches, so a history with a catalog read is checked whole.

## 4. Operations whose outcome is unknown

- F9. "Informally, extending H to H’ captures the notion that some pending invocations may have
  taken effect even though their responses have not yet been returned to the caller"; "Restricting
  attention to complete(H’) captures the notion that the remaining pending invocations have not yet
  had an effect." [1 p. 469] (Lowe's tester "considers only complete histories" [3 p. 3].)
- F10. "`:info` denotes an indeterminate result: the operation either may or may not have
  completed."; "If a process crashes (e.g. by beginning an `:invoke` followed by an `:info`) it can
  never invoke an operation again." [8 README L50-61] "An `:info` may do either." [8 README L96]
- F11. Knossos, before checking: "5. We strip out failed ops; they can't influence state." "6. We
  append synthetic :info operations for any incomplete invocations." [7 history.clj L363-365] Its
  WGL holds infos back, "Save infos for later", then "OK we're out of history elements! Append info
  nodes" [7 dll_history.clj L100-104]; the search ends at them: "The sequence of :invoke nodes in
  our dll-history is exactly those operations we chose not to linearize, and each corresponds to an
  info here at the end of the history." [7 wgl.clj L579-585]

**Derived.** A request whose answer never came (lost reply, timeout, crash) is a call completed after
every other event: the checker may linearize it anywhere after its call or not at all; its client
goes on as a new process. A typed refusal is not `:fail`: the client saw it, the model must give the
same refusal, and it stays in the history as an operation that changed nothing.

## 5. Shrinking a failing history: delta debugging

- F12. "Our minimizing delta debugging algorithm ddmin is fed with a failing test case, which it
  simplifies by successive testing." [9 p. 1] A failing case "would be 1-minimal if removing any
  single change would cause the failure to disappear."; removing more at once "may result in an
  even smaller, still failing test case" [9 p. 4]. "Proposition 11 (ddmin minimizes) For any c"
  ⊆ c✘, ddmin(c) "is 1-minimal in the sense of definition 10."; by Proposition 12 it takes at most
  |c✘|² + 3|c✘| tests (read from the page image) [9 p. 6].
- F13. Only one outcome is the failure: "The test has produced the failure it was intended to
  capture"; otherwise "The test produced indeterminate results" (unresolved) [9 p. 3].

**Derived.** ddmin runs over a failing history's operations, each removed with its completion; a
candidate replays from the seed and fails only on the same failure (same check, same operation
kind). The result is 1-minimal (AC-RQ0.5.4) within |c|² + 3|c| replays; ddmin's calls are tail
calls, so it runs as a loop.

## 6. Byte ranges on blob reads (RFC 9110)

- F14. "An int-range is invalid if the last-pos value is present and less than the first-pos."
  [10 §14.1.1] "A server that supports range requests MAY ignore or reject a Range header field that
  contains an invalid ranges-specifier (Section 14.1.1)"; 416 is for a field that "contains a valid
  ranges-specifier" where "the ranges-specifier is unsatisfiable with respect to the selected
  representation, the server SHOULD send a 416 (Range Not Satisfiable) response." [10 §14.2]
- F15. "For a GET request, a valid bytes range-spec is satisfiable if it is either:" "an int-range
  with a first-pos that is less than the current length of the selected representation or" "a
  suffix-range with a non-zero suffix-length." [10 §14.1.2] The grammar admits a zero suffix:
  "suffix-length = 1*DIGIT" [10 §14.1.1]. "If the selected representation is shorter than the
  specified suffix-length, the entire representation is used."; a last-pos past the end: "the byte
  range is interpreted as the remainder of the representation" [10 §14.1.2].
- F16. "If a single part is being transferred, the server generating the 206 response MUST generate
  a Content-Range header field, describing what range of the selected representation is enclosed"
  [10 §15.3.7.1]. "A server that generates a 416 response to a byte-range request SHOULD generate a
  Content-Range header field specifying the current length of the selected representation"
  [10 §15.5.17]; its example is `Content-Range: bytes */47022`.
- F17. "When a selected representation has zero length, the only satisfiable form of range-spec in
  a GET request is a suffix-range with a non-zero suffix-length." [10 §14.1.2] "A server that
  supports range requests MAY ignore a Range header field when the selected representation has no
  content" [10 §14.2]. Yet "A Content-Range field value is invalid if it contains a range-resp that
  has a last-pos value less than its first-pos value, or a complete-length value less than or equal
  to its last-pos value." [10 §14.4]
- F18. "For this specification, GET is the only method for which range handling is defined."; Range
  is evaluated "only if the result in absence of the Range header field would be a 200 (OK)
  response." [10 §14.2] "recipients MUST anticipate potentially large decimal numerals and prevent
  parsing errors due to integer conversion overflows." [10 §14.1.2]

**Derived.** For `GET` of a blob of length L with one bytes range:

| Range | RFC 9110 | The model answers |
|---|---|---|
| (a) `bytes=9-3` | invalid: ignore or reject; no status named, 416 is for valid specifiers | one, fixed by the design: 200 whole, or a typed refusal |
| (b) first-pos ≥ L > 0, as `bytes=L-` | valid, unsatisfiable: SHOULD 416 | 416, `Content-Range: bytes */L` |
| (c) `bytes=-0` | valid, unsatisfiable | 416, `Content-Range: bytes */L` |
| (d) `bytes=-N`, N > L > 0 | satisfiable: the entire representation | 206, `Content-Range: bytes 0-(L-1)/L` |
| (e) any, L = 0 | int-ranges unsatisfiable; a suffix satisfiable but not describable; MAY ignore | 200, empty: valid for every form |

A last-pos at or past L reads to the end; a numeral too wide for the parser is past L as a first-pos
(416), the end as a last-pos, the whole blob as a suffix. `HEAD` ignores Range; an absent blob is 404.

## 7. 429 Too Many Requests (RFC 6585 §4)

- F19. "The 429 status code indicates that the user has sent too many requests in a given amount of
  time"; "The response representations SHOULD include details explaining the condition, and MAY
  include a Retry-After header indicating how long to wait before making a new request."; "this
  specification does not define how the origin server identifies the user, nor how it counts
  requests." [11 §4] The distribution spec's unreleased next version makes Retry-After a SHOULD
  (research/02 §2).

**Derived.** A 429 follows from load, not state, so the model cannot predict it: the harness records
it as an operation that changed nothing, accepted in any state, and counts it so that a run refused
mostly by rate limits fails (T-RQ0.5.5).

## 8. How the quotes were checked

RFCs and GitHub files were saved as fetched (and fetched again at the commits named: byte-identical);
PDF text came from `pdftotext`. [2], a scan, was rendered at 300 dpi and read by macOS Vision's text
recognizer; its quoted passages and the formulas given outside quotes ([3 p. 7], [9 p. 6]) were
compared by eye with the page images. `~/.cache/reliquary-research/scripts-model/check_quotes.py`
checked each quoted span in §1 to §7 against the source cited after it: whitespace removed from
both, a substring of the saved text, at the cited page, section or lines (a comment marker opening
a line of Go or Clojure counts as part of the line break). Result: all 71 quotes found at their
cited places; one, at [2 p. 170], only after joining a hyphen that ends a line of the scan.

## 9. What this note could not verify

- Gibbons and Korach [5]: the publisher's page answered with a bot challenge (HTTP 403), no author
  copy was found, and its Crossref abstract does not state the result. NP-completeness rests on [3]
  and [4], which cite it, on Wing and Gong [2 p. 173], and on Porcupine's NP-hard [6].
- [2] was read through OCR, so a quote could carry an error the page comparison missed. [1]'s text
  layer is ACM's OCR; its order symbols and Theorem 1's H|x are garbled there and are paraphrased.
  [3], [4] and [9] are author or preprint copies; the typeset versions were not compared.
- Porcupine's README and comments state no treatment of unknown outcomes; its Jepsen-log test closes
  each unanswered call with a return at the end of the history, output `etcdOutput{unknown: true}`
  (`porcupine_test.go` L379-381). That it builds WGL, not JIT linearization, rests on its README and
  code. Lowe states his bounds (F7) for his JIT graph search; no source read says they hold for WGL,
  so a budget derived from them assumes the design builds that search. Nothing was run or measured.
