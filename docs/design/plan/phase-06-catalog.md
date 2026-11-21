# Phase 6: The catalog and search

**Goal RQ6.** People and agents find agents, skills, MCP servers, microVMs and images by what they
are and what they say about themselves, within a second of their being published, and see for each
its versions, platforms, what it asks for, who signed it and how often it is pulled.

**Delivers.** A search API with full text and facets, and a page per artifact, both built from the
records' change feed and filtered by what the caller may read. Kiln's view (kiln's CP-T5.9) is built
on it.

**Serves.** CP-T9.3 (AC-T9.3.1 to AC-T9.3.3, T-T9.3.1). **Needs.** RQ1. Each kind's fields arrive
with its phase (RQ2 artifacts, RQ5 skills and MCP servers, RQ7 signatures); the catalog shows a
kind's facts once that kind's checks record them.

**Order.** RQ6.1 first. RQ6.2, RQ6.4 and RQ6.5 side by side. RQ6.3 after RQ6.2. RQ6.6 closes.

---

### RQ6.1 What the catalog knows about an artifact

Needs: RQ1 · Serves: T-T9.3.1, AC-T9.3.1

Define the catalog's document for a repository and for each of its versions, and the projection that
makes it from the records. Kind comes from the artifact type. Names, descriptions and readable text
come from where each kind keeps them: an image's standard annotations and a readme attached as a
referrer; an agent's, harness's or MCP server's config; a skill's frontmatter and `SKILL.md`; a
`server.json`. Platforms come from an index's members. Asks come from the config, per platform.
Publisher is the namespace's holder. Signature, SBOM and provenance state come from the referrers.
The projection is a pure function of records, so the same records always give the same document.

Done when:
- AC-RQ6.1.1 `docs/design/catalog.md` states, for each kind, each field's source; a doc-truth test holds it to the projection.
- AC-RQ6.1.2 An artifact's asks show as its config states them, for each platform (T-T9.3.1).
- AC-RQ6.1.3 The projection of a repository is the same whether built in one pass from all its records or step by step from the change feed in any order the feed can deliver.
- AC-RQ6.1.4 Text taken from an artifact is bounded per field, by a budget derived from the document's row size; longer text is cut at the bound and marked as cut.
- AC-RQ6.1.5 An unknown artifact type gets a document with its type as its kind; nothing is left out of the catalog for being unknown.

Tests:
- T-RQ6.1.1 [happy] Publish one artifact of each kind → each document has the fields its kind's table lists.
- T-RQ6.1.2 [happy] An agent with different `asks` for `linux/arm64` and `linux/amd64` → the page shows each platform's asks as its config has them.
- T-RQ6.1.3 [edge] An image with no annotations; a readme of only whitespace; a description in right-to-left script; a name at the grammar's longest → documents built, empty fields empty.
- T-RQ6.1.4 [edge] A readme larger than the field's bound → cut at a character boundary, marked.
- T-RQ6.1.5 [model] Replay one repository's change feed in every delivery order the feed allows → the same final document.
- T-RQ6.1.6 [hostile] A description with control characters, HTML, a script tag, and invalid UTF-8 in a JSON string → stored as text, escaped on output, never interpreted.
- T-RQ6.1.7 [race] A tag moves while its document is being projected → the document reflects a state the repository was in, never a mix of two.

---

### RQ6.2 The index

Needs: RQ6.1, RQ0.7 · Serves: AC-T9.3.2, AC-T9.3.3

Build the structure that answers a text query with facets fast. Every node follows the records'
change feed and keeps its own copy, which can always be rebuilt from the records. Decide its design
by measurement on the search benchmark, among at least: an inverted index of immutable segments with
a small in-memory segment for what just changed; a trigram store for substring matching; and the
records store's own queries. Rank by a published relevance function, with its parameters as the
literature gives them or learned from the corpus. A query can ask to see everything published before
a point it names, and waits for the index to reach it.

Done when:
- AC-RQ6.2.1 Search p99 over 100,000 artifacts is at most 50 ms, measured by bench `reliquary_search` (AC-T9.3.2).
- AC-RQ6.2.2 Artifacts missing from search 1 s after publish is exactly 0, measured by a test publishing and searching (AC-T9.3.3).
- AC-RQ6.2.3 The A|B records name the chosen design, with every candidate's latency, index size, build rate and memory.
- AC-RQ6.2.4 An index rebuilt from the records alone answers every query of the test corpus exactly as the incrementally built one does.
- AC-RQ6.2.5 Search latency at the plan's larger scale sizes is recorded, and the growth from 100,000 artifacts is no worse than the design's stated bound.
- AC-RQ6.2.6 The index's memory is within the budget derived for it; past it, older segments are read from disk or Mantle, and answers stay correct.

Tests:
- T-RQ6.2.1 [happy] Index the corpus; query by a word of a name, of a description and of a readme → each artifact holding the word is returned, ranked.
- T-RQ6.2.2 [happy] Publish, then search at once asking for the publish's own position → found.
- T-RQ6.2.3 [edge] Queries of one character, of a phrase, of a word in every script the corpus has, of an emoji, of a term that matches everything → answered within the bound; the last pages.
- T-RQ6.2.4 [edge] An artifact deleted → gone from results by the next query that asks for the delete's position.
- T-RQ6.2.5 [failure] Kill the node while it writes a segment → after restart the index equals a rebuild; no result is duplicated or lost.
- T-RQ6.2.6 [failure] The change feed says the node's position was compacted away → the node rebuilds from a snapshot of the records and answers "catching up" typed until it has.
- T-RQ6.2.7 [race] Many publishes while queries run → every query's results are a state the catalog was in at some instant.
- T-RQ6.2.8 [exhaustion] More documents than the memory budget holds → correct answers, with the latency of the on-disk path recorded.
- T-RQ6.2.9 [hostile] A query built to be expensive (many wildcards, a pathological regex if regexes are offered) → refused or bounded in time by a counted budget; other queries' latency stays in its interval.
- T-RQ6.2.10 [model] The index against a brute-force scan of the documents on generated queries → the same set of results.
- T-RQ6.2.11 [perf] Run bench `reliquary_search` at 100,000 artifacts and at the plan's larger scale sizes → records of record; p99 at most 50 ms at 100,000.

---

### RQ6.3 The search API

Needs: RQ6.2 · Serves: AC-T9.3.1

Offer search to people and agents: a query, facets to narrow by (kind, platform, publisher, signed
or not and by whom), a sort, and a cursor. Results are only what the caller may read, and filtering
by permission never makes a page come back short or leak a count. The native door carries the same
call. `/v2/_catalog` and the tag listings stay as the spec has them. Kiln's console and its agents'
tools use this API.

Done when:
- AC-RQ6.3.1 People and agents find agents, skills, MCP servers and images by search through the API, in a change request that passes the gate (AC-T9.3.1).
- AC-RQ6.3.2 A result set, its facet counts and its total never include or count an artifact the caller may not read.
- AC-RQ6.3.3 Every page but the last is full; walking by cursor while artifacts are published and deleted returns each that existed throughout exactly once.
- AC-RQ6.3.4 The same query gives the same results through the HTTP API and the native door.
- AC-RQ6.3.5 The API's schema is published, and every response validates against it in the gate.

Tests:
- T-RQ6.3.1 [happy] Search "pdf" with `kind=skill` → the PDF skills, each with its latest version and digest.
- T-RQ6.3.2 [happy] Facet by platform, by publisher and by "signed by" at once → only artifacts matching all three; facet counts add up.
- T-RQ6.3.3 [refusal] An unknown facet; a malformed cursor; a page size above the bound → `400` saying which.
- T-RQ6.3.4 [edge] A caller who may read nothing; one who may read one repository among the largest scale size → an empty page; that one, in bounded time.
- T-RQ6.3.5 [hostile] Two callers with different grants run the same query → neither can infer from counts, timing within the interval, or cursors what the other sees.
- T-RQ6.3.6 [race] A repository made private between two pages of a walk → it does not appear on the later page.
- T-RQ6.3.7 [exhaustion] Many concurrent searches past the admitted limit → refused with `Retry-After`; pulls are unaffected.

---

### RQ6.4 A page per artifact

Needs: RQ6.1 · Serves: AC-T9.3.1, T-T9.3.1

Answer "tell me about this artifact" in one call: its versions in order, each version's platforms,
sizes and asks, its signatures and who made them, whether it has an SBOM and provenance, its readme,
what it is built from where its provenance says, what refers to it, and its pull counts. Each part is
paged where it can be long. The page for a version pinned by digest never changes, except for what is
attached to it later.

Done when:
- AC-RQ6.4.1 The page lists exactly the versions the tag list has and the referrers the referrers API has, in a stated order.
- AC-RQ6.4.2 Every fact on the page is traceable to the record it came from, by an ID in the response.
- AC-RQ6.4.3 A page's cost does not grow with the repository's number of versions beyond its page size.
- AC-RQ6.4.4 For an agent, harness, MCP server or Agentfile image, the page lists what it brings and what it asks, per platform.

Tests:
- T-RQ6.4.1 [happy] The page of an agent with three versions, two platforms each, one signed → all of it shown, the signer named.
- T-RQ6.4.2 [edge] A repository with one untagged manifest; with more tags than a page holds; with a version that is an index of indexes → shown, paged, flattened to platforms.
- T-RQ6.4.3 [race] A signature attached while the page is read → on this read or the next, never half.
- T-RQ6.4.4 [refusal] The page of a repository the caller may not read → `404`, as for one that does not exist.

---

### RQ6.5 Pull counts without a hot row

Needs: RQ1 · Serves: AC-T9.3.1

Count pulls per artifact and version without slowing a pull or writing a row for each. A pull emits
an event; nodes sum events in memory and write the sums in batches sized by the commit they ride on.
A count is eventually right and says how fresh it is. Counts are never a metric label.

Done when:
- AC-RQ6.5.1 A pull's latency with counting on is within the interval of its latency with counting off.
- AC-RQ6.5.2 After the system is quiet, each count equals the number of completed pulls, exactly.
- AC-RQ6.5.3 A node crash loses at most the sums it had not written, and the loss is bounded by one batch and counted.
- AC-RQ6.5.4 What counts as a pull is stated (a manifest `GET` by a client that then fetches or already holds its layers), and repeated checks of an image a client already holds are told apart.

Tests:
- T-RQ6.5.1 [happy] Pull one image a known number of times from several nodes → the count equals it once quiet.
- T-RQ6.5.2 [edge] `HEAD` requests, pulls of one layer alone, a pull that fails midway → counted as the stated rule says.
- T-RQ6.5.3 [failure] Kill a node holding unsaved sums → the count is short by at most that batch, and the shortfall counter says so.
- T-RQ6.5.4 [race] Many nodes add to one artifact's count at once → no update lost.
- T-RQ6.5.5 [hostile] A client pulling one manifest in a loop to inflate a count → counted per the rule, and bounded per principal by a rule the page states.

---

### RQ6.6 Kiln's people and agents search Reliquary

Needs: RQ6.3, RQ6.4 · Serves: AC-T9.3.1

Put it to use: kiln's console shows the catalog and each artifact's page, and kiln's agents get a
tool that searches it. Kiln's change request builds the view; Reliquary's part is what the consumer
finds missing or slow.

Done when:
- AC-RQ6.6.1 Kiln's change request that uses the search API and the artifact page passes kiln's gate.
- AC-RQ6.6.2 An agent's search for a skill by what it does returns it within the latency AC-RQ6.2.1 states, end to end from the tool call.
- AC-RQ6.6.3 Nothing the console shows comes from a second store: each field is the API's.

Tests:
- T-RQ6.6.1 [happy] In kiln's console, search, facet, open a page → the API's own answers.
- T-RQ6.6.2 [happy] An agent asks for "a skill that fills PDF forms" → the tool returns the matching skills with their references.
- T-RQ6.6.3 [failure] Reliquary's search is catching up after a restart → the console says so, typed, and pulls still work.
