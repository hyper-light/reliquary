# Phase 10: Fast distribution

**Goal RQ10.** A CI leg or an agent's microVM starts before its image has finished downloading. A
node fetches only the blocks it reads, verifies each one alone, shares identical blocks between
images, gets them from its neighbors when they have them, and has ready the blocks it read last time.

**Delivers.** Hash trees that let any range of an artifact be verified by itself. A chunk store for
what is read by range. A cache on every node that serves blocks to the VMM on demand, on Linux, macOS
and Windows alike. Recorded working sets, prefetched. Peer-to-peer transfer between nodes with every
piece verified. Images published with their boot-ready forms beside them.

**Serves.** CP-T9.6 (AC-T9.6.1 to AC-T9.6.3, T-T9.6.1), Hex's CP-T7.8. **Needs.** RQ3 (the native
door, the client, Hex's image store) and RQ9.3 (collection, for chunks).

**Order.** RQ10.1 first: its measurements decide the rest. RQ10.2 and RQ10.3 side by side. Then
RQ10.4, RQ10.6, RQ10.7 and RQ10.8 side by side; RQ10.5 after RQ10.4. RQ10.9 closes.

---

### RQ10.1 Measure the images before designing for them

Needs: RQ0.7 · Serves: AC-T9.6.2

No published study says what chunk size, what kind of chunking, or how much sharing to expect on
today's container and microVM images, and the systems that ship each picked a number. Measure
ours. Take the corpora Reliquary will serve: kiln's own images and runner binaries across releases,
Shards microVM disks, and the most pulled public images. For each, measure: how much is shared at
each chunk size, fixed and content-defined, before and after compression; what the index costs at
each size; which blocks a start reads, in what order, and how that varies between starts; and the
size and layer distributions. Publish the tables. Every later item of this phase cites them, and a
figure not in them is not used.

Done when:
- AC-RQ10.1.1 `bench/corpus/` holds, for each corpus, sharing and index cost against chunk size for fixed and content-defined chunking, on compressed and uncompressed bytes, with the corpus's digests so it can be re-run.
- AC-RQ10.1.2 It holds, for each bootable image in the corpus, the blocks read from start to "workload running", over repeated starts, and how much of that set is common to all starts.
- AC-RQ10.1.3 The cost model that later items use (index bytes, bytes fetched, requests made, each priced on a measured path) reproduces the measured transfer of a sample of images within its interval.
- AC-RQ10.1.4 The report states which content is worth storing by chunk and which is not, and what would change the answer.

Tests:
- T-RQ10.1.1 [happy] Re-run the measurement on the recorded corpus → the tables reproduce exactly (they are deterministic).
- T-RQ10.1.2 [edge] A corpus of one image; of two identical images; of two images with nothing shared → sharing of none, all, none.
- T-RQ10.1.3 [model] The cost model's prediction for a held-out image against its measured lazy start → within the interval.
- T-RQ10.1.4 [non-vacuity] Check each corpus for an image where content-defined chunking beats fixed and one where it does not → both exist, or the report says the corpus has none and why.

---

### RQ10.2 Any range, verified by itself

Needs: RQ10.1 · Serves: T-T9.6.1, AC-T9.6.1

A digest of a whole blob cannot check a block of it. When a block image or any blob that will be read
by range is pushed, compute a hash tree over its blocks as it streams in, keep the tree as a referrer
of the blob, and bind its root to the blob's digest in the records, in the tag's signed statement
and in the tag history. A reader given the root verifies any block, or any run of blocks, with a
short proof, at whatever fetch size its path makes cheapest. Use the tree shape Linux's own verity
uses where the measurements allow, so a kernel can check the same image. A node that started lazily
still checks the whole blob's digest once it has all of it.

Done when:
- AC-RQ10.2.1 For every blob in the test corpus, every block and every aligned run verifies against the root with its proof, and any single flipped bit in a block or a proof fails.
- AC-RQ10.2.2 The tree is computed in the same pass that hashes the upload: the bytes are read once.
- AC-RQ10.2.3 The root is bound to the blob's digest in a record written in the blob's own commit; a root for another blob never verifies this one's blocks.
- AC-RQ10.2.4 The fetch size is not in the format: one tree serves a reader that fetches a block at a time and one that fetches megabytes at a time.
- AC-RQ10.2.5 Where the tree is Linux verity's, a Linux kernel given the root and the tree accepts the image and refuses a corrupted one.
- AC-RQ10.2.6 A lazily started artifact's whole digest is verified when its last byte arrives; a mismatch is an alarm, and the artifact is marked damaged.

Tests:
- T-RQ10.2.1 [happy] Read every block of an image at random, verifying each → all verify; the union equals the blob.
- T-RQ10.2.2 [hostile] Flip one bit in a block; in a proof; swap two blocks; serve a block of another image → each refused at that block.
- T-RQ10.2.3 [edge] A blob of zero bytes; of one byte; of exactly one block; not a multiple of the block; larger than memory → trees built and verified.
- T-RQ10.2.4 [edge] Runs of one block and of the whole blob through the same tree → both verify.
- T-RQ10.2.5 [race] A blob read by range while its tree referrer is being deleted by retention → the read fails typed or completes verified; never unverified bytes.
- T-RQ10.2.6 [failure] The tree's own bytes are damaged → detected by its own digest; rebuilt from the blob.
- T-RQ10.2.7 [model] The tree against a reference implementation on generated inputs → identical roots.

---

### RQ10.3 A chunk store for what is read by range

Needs: RQ10.1, RQ9.3 · Serves: AC-T9.6.1, AC-T9.6.2

Content that is read whole stays whole: an OCI layer is one blob, served as fast as RQ4 made it.
Content that is read by range (a microVM disk, a flattened image, a large binary across releases) is
kept as a recipe: an ordered list of chunks, each stored once however many recipes name it. The chunk
size and whether chunks are cut at fixed offsets or by content come from RQ10.1's tables for each
kind of content, and are recorded in the recipe, so they can change without a migration. A chunk is
a blob to the records: recipes link chunks, and RQ9.3 collects a chunk when no recipe links it. A
client that wants the whole artifact gets it back byte for byte.

Done when:
- AC-RQ10.3.1 An artifact stored as a recipe reassembles to its original bytes and digest, for every artifact in the corpus.
- AC-RQ10.3.2 Stored bytes for the corpus are within the interval of what RQ10.1's table predicts for the chosen chunking.
- AC-RQ10.3.3 No chunk size or boundary rule is fixed in code: each is a parameter in the recipe, chosen by the stated rule from measured tables.
- AC-RQ10.3.4 A whole read of a recipe-stored artifact meets the same bar against competitors as a whole blob, or the artifact is also kept whole while it is read whole often, by a rule on measured reads.
- AC-RQ10.3.5 Chunks shared between namespaces are charged to each (RQ9.5), and whether a chunk already exists is never observable to a pusher.
- AC-RQ10.3.6 The property of RQ9.3 holds for chunks: referenced chunks collected is exactly 0.

Tests:
- T-RQ10.3.1 [happy] Push two microVM disks built from one base → stored bytes are about one base plus two differences; each pulls back identical.
- T-RQ10.3.2 [edge] An artifact of one chunk; of all-identical chunks; of chunks none of which repeats → stored and read back.
- T-RQ10.3.3 [race] Two pushes store the same new chunk at once → one copy; both recipes valid.
- T-RQ10.3.4 [race] A recipe deleted while another recipe that shares its chunks is being pushed → the second's chunks stay.
- T-RQ10.3.5 [hostile] A pusher times its uploads to learn whether a chunk exists → upload time and answers do not differ within the interval.
- T-RQ10.3.6 [exhaustion] A recipe with more chunks than one records transaction holds → committed in stages; invisible until whole.
- T-RQ10.3.7 [crash] Power cut at every write of a recipe push → after restart the artifact is whole or absent; no chunk is orphaned past the collector's reach.

---

### RQ10.4 Blocks on demand, on every platform

Needs: RQ10.2, RQ10.3, RQ3.8 · Serves: AC-T9.6.1, AC-T9.6.2

Extend the node's image store (RQ3.8) so a microVM can boot from a disk that is not all there. The
store presents the disk to the VMM as a block backend it calls in its own process: a read of blocks
the node holds is served from local disk; a read of blocks it lacks fetches them, verifies them
(RQ10.2), stores them and answers. Because the backend lives in the VMM's device, it needs no host
kernel feature and is the same code under KVM, Hypervisor.framework and the Windows Hypervisor
Platform; an NBD endpoint serves a VMM that cannot link it. Behind the first reads, the rest of the
disk fills in the background at a pace that yields to the guest's own reads. How the guest maps the
disk is Shards' and Hex's decision, made with their owners from A|B records. A guest's read never
returns unverified or wrong bytes, and never hangs: a block that cannot be fetched is an I/O error
the guest sees.

Done when:
- AC-RQ10.4.1 A Shards microVM and a Firecracker microVM each boot and run a workload from a disk of which the node held nothing, on Linux; a Shards microVM does on macOS and on Windows.
- AC-RQ10.4.2 Time to a 2 GB image's workload running, lazy against a full pull, falls by at least 70% compared with a full pull, measured by bench `image_start` on fedora (AC-T9.6.2).
- AC-RQ10.4.3 Every block handed to a guest was verified against the image's root.
- AC-RQ10.4.4 Once the disk is whole, a guest's read latency is within the interval of a disk that was pulled in full.
- AC-RQ10.4.5 A block shared by two images on one node is fetched and stored once.
- AC-RQ10.4.6 The background fill's pace and the fetch size are learned on the path (RQ0.8) and yield to foreground reads; neither is a fixed figure.
- AC-RQ10.4.7 The A|B records compare the candidate device paths on each host platform, and the record names what Shards' and Hex's owners chose.

Tests:
- T-RQ10.4.1 [happy] Boot from an unheld image, run the workload → it completes; the blocks fetched before "running" are a fraction of the disk.
- T-RQ10.4.2 [failure] Reliquary and every peer unreachable mid-boot → the guest's read of a missing block returns an I/O error within the wait RQ0.8 learns; the VM is reported failed, naming the image; blocks already held still read.
- T-RQ10.4.3 [failure] The network drops and returns during the background fill → the fill resumes; the guest never notices.
- T-RQ10.4.4 [hostile] A source that serves a corrupted block → refused at that block; fetched from another source; the guest gets the right bytes or an error.
- T-RQ10.4.5 [race] Two VMs booting from one image fault on the same block at once → one fetch; both served.
- T-RQ10.4.6 [race] The guest reads a block the background fill is writing → the right bytes; no torn block.
- T-RQ10.4.7 [crash] Kill the VMM or the node agent mid-fill → after restart, blocks marked held are verified ones; the fill resumes.
- T-RQ10.4.8 [exhaustion] The node's disk fills during the fill → the fill stops; reads of unheld blocks still fetch into memory within budget or fail typed; nothing a running VM holds is dropped.
- T-RQ10.4.9 [deadlock] The guest's read waits on a fetch whose connection is flow-controlled by a fill that the guest's own I/O is delaying → foreground reads have their own credit; the read completes.
- T-RQ10.4.10 [perf] Run bench `image_start` on fedora, lazy against a full pull, on a 2 GB image and across the corpus's sizes, then read latency after the fill → records of record; the fall is at least 70% on the 2 GB image.

---

### RQ10.5 The blocks it read last time, ready this time

Needs: RQ10.4 · Serves: AC-T9.6.2

What a start reads is nearly the same each time. Record it: the node notes the blocks read between
"start" and "workload running", and Reliquary merges those notes across starts and nodes into a
working set for that image and command, kept as a referrer. On the next start anywhere, the store
fetches the working set first, in the order it will be read, before the guest asks. The set follows
the image: blocks that stop being read age out of it, new ones join, and a working set that stops
helping is dropped by the same measurement that built it.

Done when:
- AC-RQ10.5.1 With a working set, the guest's reads that had to wait on a fetch, between start and running, fall to those blocks the set lacked, by counter.
- AC-RQ10.5.2 Start time with a working set is no worse than without one, within the interval, on every image in the corpus; where the set does not help, it is not used.
- AC-RQ10.5.3 The working set's size and membership come from recorded starts; no fraction or count is picked.
- AC-RQ10.5.4 A working set reveals nothing across namespaces: it is derived from, and served to, only principals who may read the image.
- AC-RQ10.5.5 Recording costs the guest's reads nothing measurable, within the interval.

Tests:
- T-RQ10.5.1 [happy] Start an image cold, then again on another node → the second start's waits on fetch are near none; its time is shorter.
- T-RQ10.5.2 [edge] An image whose start reads differ every time (a random file) → the set holds the common part only.
- T-RQ10.5.3 [edge] The image's command changes → a separate working set; the old one is not applied.
- T-RQ10.5.4 [failure] A working set naming blocks past the image's end, or a set for another image (planted) → refused; the start proceeds without it.
- T-RQ10.5.5 [race] Many nodes report starts of one image at once → one merged set; no report lost.
- T-RQ10.5.6 [exhaustion] An image started under so many commands that sets would exceed their budget → least used sets are dropped; starts still work.

---

### RQ10.6 Containers that start before they are pulled

Needs: RQ10.2, RQ3.8 · Serves: AC-T9.6.1

Hex's `oci` class runs containers on Linux hosts from layers. Let them start lazily where the host's
kernel can: the node keeps each image as an EROFS image with an index over its layers and mounts it
with missing blocks fetched on demand through the mechanism that host's kernel offers. Measure the
candidates the kernel gives on the lab's hosts and keep the one that wins. A host without any is told
so by name, and the image is pulled in full as today. Images stay ordinary OCI images in the
registry: the index is a referrer, and a stock client that ignores it pulls as it always has.

Done when:
- AC-RQ10.6.1 On a host whose kernel has a supported mechanism, an `oci` box starts from an unheld image and its workload runs before the image is whole.
- AC-RQ10.6.2 On a host without one, the box starts by a full pull, and the reason is in its record.
- AC-RQ10.6.3 Every file a container reads is verified against the image's tree before it is returned.
- AC-RQ10.6.4 A stock Docker or containerd pull of the same image is unchanged, byte for byte.
- AC-RQ10.6.5 The A|B record names the mechanism kept for each kernel in the lab and what the others measured.

Tests:
- T-RQ10.6.1 [happy] An `oci` box from an unheld multi-gigabyte image → running before the image is whole; output correct.
- T-RQ10.6.2 [edge] An image of many tiny layers; of one huge layer; with hard links and whiteouts across layers → the merged view equals a full pull's, file for file.
- T-RQ10.6.3 [failure] The source vanishes mid-run → a read of an unheld file fails with an I/O error the container sees; held files still read.
- T-RQ10.6.4 [refusal] A host kernel older than any supported mechanism → the typed fallback, naming the kernel and the mechanism it lacks.
- T-RQ10.6.5 [perf] Steady-state file read latency after the fill against a full pull's → within the interval.

---

### RQ10.7 Nodes fetch from each other

Needs: RQ10.2, RQ3.8 · Serves: AC-T9.6.3, T-T9.6.1

When a hundred runners want one image, the origin should send it about once. Each node already
tells Reliquary what it holds. A node that needs blocks asks Reliquary who has them and fetches from
those nodes over hyper-quic, mutually authenticated, preferring by measured rate. Every piece is
verified against the image's tree before it is used or passed on, so a peer cannot poison another.
A peer that serves a wrong piece is dropped for that transfer and reported. A node always can, and
when no peer serves in time does, fetch from the origin. Whether peers form a tree or a mesh for a
given kind of transfer is decided in the lab, not assumed.

Done when:
- AC-RQ10.7.1 Origin bytes sent for one image pulled by 100 runners, in image sizes, is at most 3, measured by a P2P lab with simulated runners (AC-T9.6.3).
- AC-RQ10.7.2 A peer serving corrupted chunks is caught by digest and dropped, and the pull completes (T-T9.6.1).
- AC-RQ10.7.3 No node uses or forwards a piece it has not verified.
- AC-RQ10.7.4 A node serves peers only artifacts those peers may read, checked by a grant Reliquary issues; a node cannot learn what another holds beyond what it may read.
- AC-RQ10.7.5 Serving peers never slows the serving node's own workloads beyond a share the operator states or the node derives from idle capacity.
- AC-RQ10.7.6 With every peer gone, a pull completes from the origin with no error.
- AC-RQ10.7.7 The lab's record compares the candidate topologies at the criterion's 100 runners, at a tenth of that, and at the largest simulated count, and names the one kept for each kind of transfer.

Tests:
- T-RQ10.7.1 [happy] A hundred simulated runners pull one image at once → all verify; origin bytes within the bound.
- T-RQ10.7.2 [hostile] One peer serves flipped bytes; one serves pieces of another image; one stalls after accepting → each dropped for the transfer; the pull completes from others or the origin.
- T-RQ10.7.3 [hostile] A peer advertises holdings it lacks → asked once, refused or silent, dropped; its report is corrected.
- T-RQ10.7.4 [refusal] A node asks a peer for an artifact it may not read → refused; the peer's holdings of it are not revealed.
- T-RQ10.7.5 [failure] Many of the peers die mid-transfer → the rest and the origin complete it.
- T-RQ10.7.6 [race] Every runner starts at the same instant with no peer holding anything → the origin serves the first copies; later pieces come from peers; origin bytes still within the bound.
- T-RQ10.7.7 [livelock] Two nodes each waiting for the other to fetch a piece first → neither waits: a node that needs a piece and finds no holder fetches it from the origin.
- T-RQ10.7.8 [exhaustion] One popular node asked by every peer → it serves within its share and refuses the rest typed; they go elsewhere.
- T-RQ10.7.9 [edge] Small images, where a peer round trip costs more than the origin's → fetched from the origin, by the measured rule.

---

### RQ10.8 Published once, ready to boot everywhere

Needs: RQ10.2, RQ2.4, RQ2.5 · Serves: AC-T9.6.1

Turning an OCI image into a disk a microVM boots takes time that should be spent once. When an image
is published for microVMs, its boot-ready forms are attached to it as referrers: the EROFS disk for
each platform, its hash tree, its recipe and, when recorded, its working set; for Firecracker, the
pair. The builder that has the image makes them: kiln's CI or Shards at build time, or a conversion
job Hex runs on demand. Reliquary's part is to say what forms an image has, to hand a node the form
it asks for, and to hold the forms to the image: a form made from another image's digest is refused.

Done when:
- AC-RQ10.8.1 "The disk for image digest D on platform P" answers with the attached disk, or says there is none; a node never converts an image that has its form attached.
- AC-RQ10.8.2 A form names the digest it was made from and how; one whose statement does not verify against its subject is refused.
- AC-RQ10.8.3 A form made twice from one image by one version of the converter has one digest, so two builders do not double the storage.
- AC-RQ10.8.4 Deleting an image removes or keeps its forms by the retention rule; a form never outlives a rule that removes its image without the rule saying so.

Tests:
- T-RQ10.8.1 [happy] Publish an image and its disks for two platforms → each node gets its platform's disk and boots it without converting.
- T-RQ10.8.2 [refusal] A disk attached to an image it was not made from; a disk for a platform the image lacks → refused.
- T-RQ10.8.3 [edge] An image with no form attached → the node converts as today and may attach the result, where its principal may push.
- T-RQ10.8.4 [race] Two builders attach the same form at once → one copy.
- T-RQ10.8.5 [failure] The conversion job dies midway → nothing partial is attached.

---

### RQ10.9 CI legs and agent VMs start before their image is fully downloaded

Needs: RQ10.4, RQ10.5, RQ10.7 · Serves: AC-T9.6.1 to AC-T9.6.3

Put it to use in kiln, and record the three criteria. Kiln's CI legs and agent VMs take their images
through the node's store with lazy loading, working sets and peers on. The benchmark and the lab run
on the machines kiln's plan names, on its bench calendar.

Done when:
- AC-RQ10.9.1 CI legs and agent VMs start before their image is fully downloaded, in a change request that passes the gate (AC-T9.6.1).
- AC-RQ10.9.2 Bench `image_start` on fedora and the P2P lab are of record, with AC-T9.6.2 and AC-T9.6.3 met.
- AC-RQ10.9.3 A leg's result is the same lazy as with a full pull, over the conformance of kiln's CI suite.
- AC-RQ10.9.4 With lazy loading turned off by its capability, kiln behaves as before.

Tests:
- T-RQ10.9.1 [happy] A CI leg on a runner that never held its image → it starts before the pull completes and passes.
- T-RQ10.9.2 [happy] An agent VM from a default Agentfile on a fresh runner → usable before its image is whole.
- T-RQ10.9.3 [failure] Reliquary unreachable after the leg started → the leg's reads of unheld blocks fail and the leg is failed as infrastructure, naming the cause; it is never a wrong result.
- T-RQ10.9.4 [perf] Run bench `image_start` and the P2P lab on kiln's lab → records of record meeting AC-T9.6.2 and AC-T9.6.3.
