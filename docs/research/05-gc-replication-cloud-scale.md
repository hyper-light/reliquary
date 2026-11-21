# Garbage collection, replication, cloud registries and scale: source notes
Researched 2026-11-20. Question: what do primary sources say about online garbage collection in registries and content-addressed stores, cross-region replication and its consistency (with the clock, CRDT and testing literature behind it), and the limits, behaviour, scale and failure modes of hosted registries, and which windows, limits and numbers should drive a new registry's design? Method: each source was downloaded with `curl` into `~/.cache/reliquary-research/tmp-gc/pages/<name>/` (PDFs through `pdftotext`, HTML through a tag stripper, GitHub and GitLab files raw at a tag where one exists) and every quote below was checked mechanically as a whitespace-normalised exact substring of the downloaded text (`scripts/check_notes.py` in the same scratch directory checks every quote in this file that is followed by its source id); the count is at the end.

Reading conventions:
- A quote is one contiguous passage; line breaks in the source are shown as single spaces. Markup characters (backticks, asterisks, table pipes) are kept where the source file has them. A quote that is a table row shows its cells separated by single spaces (HTML tables) or pipes (Markdown tables).
- "Derived" marks a number computed here from quoted numbers; it is not in the source.
- "Vendor" numbers are the company's own claims, with no independent reproduction found.
- Versions are the latest releases the projects' own APIs reported on 2026-11-20: GitLab Container Registry v4.43.0-gitlab (2026-10-23), CNCF Distribution v3.1.2 (2026-09-24), Harbor v2.15.5 (2026-11-12), zot v2.1.23 (2026-11-02), containerd v2.4.2 (2026-10-13).
- Section and page labels for PDFs are approximate where `pdftotext` scrambled the column order; the quoted text itself is exact.

## Sources
| id | type | title | URL | version/date |
|---|---|---|---|---|
| S1 | docs | GitLab Container Registry, "Online Garbage Collection" (design spec) | https://gitlab.com/gitlab-org/container-registry/-/blob/v4.43.0-gitlab/docs/spec/gitlab/online-garbage-collection.md | tag v4.43.0-gitlab, 2026-10-23; byte-identical to `master` on 2026-11-20 |
| S2 | vendor | GitLab blog, "GitLab's next-generation container registry is now available" | https://about.gitlab.com/blog/gitlabs-next-generation-container-registry-is-now-available/ | 2023-12-04 |
| S3 | docs | GitLab Docs, "Container registry metadata database" | https://docs.gitlab.com/administration/packages/container_registry_metadata_database/ | fetched 2026-11-20 |
| S4 | docs | CNCF Distribution, "Garbage collection" | https://github.com/distribution/distribution/blob/v3.1.2/docs/content/about/garbage-collection.md (rendered: https://distribution.github.io/distribution/about/garbage-collection/) | v3.1.2, 2026-09-24 |
| S5 | docs | Harbor docs, "Garbage Collection" | https://goharbor.io/docs/2.15.0/administration/garbage-collection/ | Harbor 2.15.0 docs |
| S6 | code | Harbor `src/jobservice/job/impl/gc/garbage_collection.go` | https://github.com/goharbor/harbor/blob/v2.15.5/src/jobservice/job/impl/gc/garbage_collection.go#L140-L141 (time window), `#L326-L327` (status) | v2.15.5, 2026-11-12 |
| S7 | docs | Harbor community proposal, "Non-Blocking GC" (Yan Wang, He Weiwei) | https://github.com/goharbor/community/blob/main/proposals/new/Non-Blocking-GC.md | `main`, fetched 2026-11-20 (written for Harbor 2.1, 2020) |
| S8 | docs | Harbor v2.1.0 release notes | https://github.com/goharbor/harbor/releases/tag/v2.1.0 | v2.1.0, 2020-09-18 |
| S9 | docs | zot docs, "Storage Planning with zot" (`docs/articles/storage.md`) | https://github.com/project-zot/project-zot.github.io/blob/main/docs/articles/storage.md | `main`, fetched 2026-11-20 |
| S10 | code | zot `pkg/storage/constants/constants.go` and `pkg/storage/gc/gc.go` | https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/constants/constants.go#L24-L25 ; https://github.com/project-zot/zot/blob/v2.1.23/pkg/storage/gc/gc.go#L40-L41 | v2.1.23, 2026-11-02 |
| S11 | docs | zot docs, "Retention Policies" (`docs/articles/retention.md`) | https://github.com/project-zot/project-zot.github.io/blob/main/docs/articles/retention.md | `main`, fetched 2026-11-20 |
| S12 | docs | containerd, "Garbage Collection" | https://github.com/containerd/containerd/blob/v2.4.2/docs/garbage-collection.md | v2.4.2, 2026-10-13 |
| S13 | docs | Amazon ECR User Guide, "Automate the cleanup of images by using lifecycle policies in Amazon ECR" | https://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html | fetched 2026-11-20 |
| S14 | paper | Douglis et al., "The Logic of Physical Garbage Collection in Deduplicating Storage", USENIX FAST'17 | https://www.usenix.org/conference/fast17/technical-sessions/presentation/douglis (PDF: https://www.usenix.org/system/files/conference/fast17/fast17-douglis.pdf) | Feb 2017 |
| S15 | paper | Calder et al., "Windows Azure Storage: A Highly Available Cloud Storage Service with Strong Consistency", ACM SOSP'11 | https://sigops.org/s/conferences/sosp/2011/current/2011-Cascais/printable/11-calder.pdf | Oct 2011 |
| S16 | docs | Ceph developer docs, "Deduplication" | https://docs.ceph.com/en/latest/dev/deduplication/ | `latest`, fetched 2026-11-20 |
| S17 | paper | Kulkarni, Demirbas, Madeppa, Avva, Leone, "Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases", Univ. at Buffalo tech report 2014-04 | https://cse.buffalo.edu/tech-reports/2014-04.pdf | 2014 |
| S18 | paper | Shapiro, Preguiça, Baquero, Zawirski, "A comprehensive study of Convergent and Commutative Replicated Data Types", INRIA RR-7506 | https://inria.hal.science/inria-00555588 (the HAL host answered with a bot check; text checked from the copy at https://dsf.berkeley.edu/cs286/papers/crdt-tr2011.pdf, whose pages carry "inria-00555588, version 1 - 13 Jan 2011") | Jan 2011 |
| S19 | docs | Microsoft Learn, "Geo-replication in Azure Container Registry" | https://learn.microsoft.com/en-us/azure/container-registry/container-registry-geo-replication | last updated 2026-08-26 |
| S20 | docs | Amazon ECR User Guide, "Private image replication in Amazon ECR" | https://docs.aws.amazon.com/AmazonECR/latest/userguide/replication.html | fetched 2026-11-20 |
| S21 | docs | Harbor docs, "Creating a Replication Rule" | https://github.com/goharbor/website/blob/main/docs/administration/configuring-replication/create-replication-rules.md | `main`, fetched 2026-11-20 |
| S22 | docs | Google Artifact Registry, "Repository locations" | https://cloud.google.com/artifact-registry/docs/repositories/repo-locations | last updated 2026-11-10 |
| S23 | vendor | AWS What's New, "Amazon S3 now delivers strong read-after-write consistency automatically for all applications" | https://aws.amazon.com/about-aws/whats-new/2020/12/amazon-s3-now-delivers-strong-read-after-write-consistency-automatically-for-all-applications/ | 2020-12-01 |
| S24 | vendor | AWS, "Amazon S3 Strong Consistency" | https://aws.amazon.com/s3/consistency/ | fetched 2026-11-20 |
| S25 | paper | Kingsbury and Alvaro, "Elle: Inferring Isolation Anomalies from Experimental Observations", PVLDB 14(3) | https://arxiv.org/abs/2003.10554 (abstract) ; https://www.vldb.org/pvldb/vol14/p268-alvaro.pdf | arXiv 2020-03-23; PVLDB vol. 14 |
| S26 | paper | Zhou et al., "FoundationDB: A Distributed Unbundled Transactional Key Value Store", ACM SIGMOD'21 | https://www.foundationdb.org/files/fdb-paper.pdf | Jun 2021 |
| S27 | paper | Pillai et al., "All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications", USENIX OSDI'14 | https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai | Oct 2014 |
| S28 | paper | Rebello et al., "Can Applications Recover from fsync Failures?", USENIX ATC'20 | https://www.usenix.org/conference/atc20/presentation/rebello | Jul 2020 |
| S29 | docs | Docker Docs, "Docker Hub pull usage and limits" | https://docs.docker.com/docker-hub/usage/pulls/ (Markdown: https://docs.docker.com/docker-hub/usage/pulls.md) | fetched 2026-11-20 |
| S30 | docs | Docker Docs, "Docker Hub usage and limits" | https://docs.docker.com/docker-hub/usage/ (Markdown: https://docs.docker.com/docker-hub/usage.md) | fetched 2026-11-20 |
| S31 | vendor | Docker blog, "Revisiting Docker Hub Policies: Prioritizing Developer Experience" | https://www.docker.com/blog/revisiting-docker-hub-policies-prioritizing-developer-experience/ | posted 2025-02-21, updated 2025-04-08 |
| S32 | docs | Amazon ECR User Guide, "Sync an upstream registry with an Amazon ECR private registry" (pull through cache) | https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache.html | fetched 2026-11-20 |
| S33 | docs | Amazon ECR User Guide, "Amazon ECR service quotas" | https://docs.aws.amazon.com/AmazonECR/latest/userguide/service-quotas.html | fetched 2026-11-20 |
| S34 | vendor | AWS What's New, ECR supports OCI Image and Distribution 1.1 | https://aws.amazon.com/about-aws/whats-new/2024/06/amazon-ecr-oci-image-distribution-version-1-1/ | 2024-06-27 |
| S35 | vendor | AWS What's New, "Amazon ECR Pull Through Cache Now Supports Referrer Discovery and Sync" | https://aws.amazon.com/about-aws/whats-new/2026/04/amazon-ecr-pull-through-cache-referrers/ | 2026-04-17 |
| S36 | docs | GitHub Docs, "Working with the Container registry" | https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry | fetched 2026-11-20 |
| S37 | docs | Google Artifact Registry, "Quotas and limits" | https://cloud.google.com/artifact-registry/quotas | last updated 2026-11-10 |
| S38 | docs | Microsoft Learn, Azure Container Registry SKU features and limits | https://learn.microsoft.com/en-us/azure/container-registry/container-registry-skus | last updated 2026-10-16 |
| S39 | docs | Microsoft Learn, "Artifact streaming in Azure Container Registry (Preview)" | https://learn.microsoft.com/en-us/azure/container-registry/container-registry-artifact-streaming | fetched 2026-11-20 |
| S40 | docs | Google Kubernetes Engine, "Use Image streaming to pull container images" | https://cloud.google.com/kubernetes-engine/docs/how-to/image-streaming | last updated 2026-11-10 |
| S41 | vendor | Red Hat blog, "Announcing the Open Container Initiative Referrers API on Quay.io" | https://www.redhat.com/en/blog/announcing-open-container-initiativereferrers-api-quayio-step-towards-enhanced-security-and-compliance | 2024-12-02 |
| S42 | docs | JFrog docs, "Use Referrers REST API to Discover OCI References" | https://jfrog.com/help/r/jfrog-artifactory-documentation/use-referrers-rest-api-to-discover-oci-references | fetched 2026-11-20 |
| S43 | vendor | JFrog blog, "Proudly Announcing JFrog's Full Conformance to OCI v1.1" | https://jfrog.com/blog/full-conformance-to-oci-v1-1/ | 2024-09-24 |
| S44 | paper | Brooker, Danilov, Greenwood, Piwonka, "On-demand Container Loading in AWS Lambda", USENIX ATC'23 | https://www.usenix.org/conference/atc23/presentation/brooker (PDF: https://www.usenix.org/system/files/atc23-brooker.pdf) | Jul 2023 |
| S45 | paper | Sahraei et al., "XFaaS: Hyperscale and Low Cost Serverless Functions at Meta", ACM SOSP'23 (author copy) | https://www.cs.cmu.edu/~dskarlat/publications/xfaas_sosp23.pdf | Oct 2023 |
| S46 | paper | Tang et al., "Twine: A Unified Cluster Management System for Shared Infrastructure", USENIX OSDI'20 | https://www.usenix.org/conference/osdi20/presentation/tang (PDF: https://www.usenix.org/system/files/osdi20-tang.pdf) | Nov 2020 |
| S47 | paper | Agache et al., "Firecracker: Lightweight Virtualization for Serverless Applications", USENIX NSDI'20 | https://www.usenix.org/conference/nsdi20/presentation/agache (PDF: https://www.usenix.org/system/files/nsdi20-paper-agache.pdf) | Feb 2020 |
| S48 | vendor | Docker solution brief, "Docker Hub: Where development happens" | https://www.docker.com/resources/docker-hub-where-development-happens-solution-brief/ | page metadata 2025-03-19 |
| S49 | vendor | Docker blog, "Docker Index: Dramatic Growth in Docker Usage Affirms the Continued Rising Power of Developers" | https://www.docker.com/blog/docker-index-dramatic-growth-in-docker-usage-affirms-the-continued-rising-power-of-developers/ | 2020-07-30 |
| S50 | vendor | Docker Hub, Verified Publisher Program page | https://hub.docker.com/publisher-program | fetched 2026-11-20 |
| S51 | vendor | E2B, "About" | https://e2b.dev/about | fetched 2026-11-20 |
| S52 | vendor | Modal, "Best code execution sandboxes for coding agents" (resources page) | https://modal.com/resources/best-code-execution-sandboxes-coding-agents | fetched 2026-11-20 |
| S53 | vendor | Modal customer story, "How Modal powered 250,000 Lovable app creations in a weekend" | https://modal.com/blog/lovable-case-study | 2025-07-07 |
| S54 | vendor | Red Hat blog, "About the Quay.io outage post-mortem" | https://www.redhat.com/en/blog/about-the-quay.io-outage-post-mortem | 2020-08-07 |
| S55 | vendor | Docker blog, "Docker Hub Incident Reviews – April 3rd and 15th 2021" | https://www.docker.com/blog/docker-hub-incident-reviews-april-3rd-and-15th-2021/ | 2021-05-11 |
| S56 | vendor | Docker blog, "Docker Hub Incident Report – October 20, 2025" | https://www.docker.com/blog/docker-hub-incident-report-october-20-2025/ | 2025-10-24 |
| S57 | vendor | GitHub blog, "GitHub Availability Report: January 2023" | https://github.blog/news-insights/company-news/github-availability-report-january-2023/ | 2023-02-01 |
| S58 | vendor | GitHub blog, "GitHub Availability Report: November 2025" | https://github.blog/news-insights/company-news/github-availability-report-november-2025/ | 2025-12-11 |
| S59 | vendor | AWS, summary of the Amazon DynamoDB service disruption in N. Virginia (us-east-1) | https://aws.amazon.com/message/101925/ | Oct 2025 |

## Findings

### A. Online garbage collection in registries and content-addressed stores

### F1. GitLab's online GC is a continuous background process that reads candidate objects from review-queue tables; it never marks the whole registry.
- Quote: "The online garbage collector is a continuous and concurrency safe background process, operating asynchronously from the HTTP API server's main registry process." — S1, "Review queues"
- Quote: "The HTTP API process inserts records to signal objects that may be eligible for deletion in these tables. The garbage collector then continuously selects records from them and determines if the corresponding objects are eligible for deletion." — S1, "Review queues"
- Quote: "As described in this document, stages 1 and 2 are operated by database triggers and not by the application (API)." — S1, introduction
- Quote: "Furthermore, the longer it takes to determine blobs eligible for deletion, the higher the chances for race conditions while operating online, leading to potential false positives/negatives." — S1, "Tracking blob and manifest uploads"
- Conditions/notes: two PostgreSQL tables, `gc_blob_review_queue` (keyed by digest) and `gc_manifest_review_queue` (keyed by repository and manifest); the queue rows are written by database triggers in the same transaction as the API's change.
- Relevance: copy the shape. Candidates come from the changes themselves (the record store's change feed, or a queue record written in the same transaction), so the cost of a collection is the size of the change, not of the registry.

### F2. A review is triggered by every blob upload and by every delete that can orphan something: tag delete, tag switch, manifest delete, manifest-list delete, layer delete (and manifest upload).
- Quote: "Considering this, to keep track of blobs uploaded during a failed/canceled image push, a row should be inserted in `gc_blob_review_queue` for every uploaded blob." — S1, "Blob uploads"
- Conditions/notes: the event names `blob_upload`, `manifest_upload`, `tag_delete`, `tag_switch`, `manifest_delete`, `manifest_list_delete` and `layer_delete` are the arguments of `gc_review_after('…')` in the spec's SQL (read from the SQL, not from a prose sentence).
- Relevance: copy. Every uploaded blob is born a GC candidate, so an abandoned push needs no separate upload-reaper.

### F3. GitLab's review delay defaults to 1 day for both queues and every event; the stated reason is that a client gets up to one day to finish a push, and a user a short period to re-tag an untagged manifest.
- Quote: "The default for this column is 1 day for both queues and all kinds of events, but it is possible to set a different default value through application settings." — S1, "Review Delay"
- Quote: "As mentioned previously, by default, `review_after` is set to one day for all uploaded blobs, which means that clients have up to one day to complete an image upload. Otherwise, blobs will be garbage collected." — S1, "Blob uploads"
- Quote: "to postpone their deletion as we do for blobs, allowing users to re-tag/re-reference them for a short period before deleting them." — S1, "Review queues / Manifests"
- Quote: "To avoid scheduling multiple tasks to the exact same time, the `gc_review_after` functions adds a jitter of up to 60 seconds to every returned duration." — S1, "Review Delay"
- Quote: "After import to the new registry, all images are subject to continuous online garbage collection, by default deleting any untagged and unreferenced manifests and layers that remain for longer than 24 hours." — S3, import section
- Conditions/notes: the delay is per event, in a `gc_review_after_defaults` table changeable at run time. No measurement is given for "1 day": it is a product choice, not a tuned constant.
- Relevance: the window is a grace period for the user, not the mechanism that makes deletion safe (F4 is). A new registry should keep the two apart: a lease that makes deletion safe, and a retention period the operator sets.

### F4. GitLab closes races among GC workers with `SELECT FOR UPDATE SKIP LOCKED`, and races between the GC and the API on manifests with explicit row locks on the review-queue record; failed reviews back off exponentially.
- Quote: "A `SELECT FOR UPDATE SKIP LOCKED` statement should be used to synchronize multiple concurrent GC processes consuming the review queues, locking and popping records for review within a transaction, one at a time:" — S1, "Consuming the review queues"
- Quote: "To avoid this, we need to synchronize the API and the garbage collector with explicit locks:" — S1, "Manifests / Race conditions / Deleting the last referencing tag"
- Quote: "The garbage collector should also leverage on `review_count` to implement an exponential backoff algorithm, increasing the delay added to `review_after` on every failed review:" — S1, "Handling failures"
- Conditions/notes: the spec lists five manifest races, each closed the same way: deleting the last referencing tag, deleting the last referencing manifest list, tagging an untagged manifest, creating a manifest list that references an unreferenced manifest, and creating a referrer of an unreferenced subject. The API takes `SELECT … FOR UPDATE` on the `(repository, manifest)` review row before the tag or reference change; whichever side locks first decides.
- Relevance: copy, in the record store's terms. The review record is the key both sides read and write in one conditional transaction: the collector's delete commits only if the review record and the reference set are unchanged; the API's tag or reference write touches the same record, so one of the two transactions aborts.

### F5. GitLab designed a lock for the blob race (HEAD says "exists", the GC deletes, the manifest PUT then fails) and chose not to implement it: the worst case is a push retry.
- Quote: "Checking for records to be reviewed one hour ahead of the existence check ensures that we do not postpone a record's review unless it will be reviewed (and possibly deleted) before the manifest upload that is expected to follow." — S1, "Blobs / Race conditions"
- Quote: "the API pushes the `review_after` of the blob review record forward by one day and commits the transaction:" — S1, "Blobs / Race conditions"
- Quote: "For all these reasons, we decided not to implement this explicit locking from the start." — S1, "Blobs / Race conditions"
- Quote: "Unlike the possible race conditions around manifests (described in the next section), these do not pose any integrity risks." — S1, "Blobs / Race conditions"
- Quote: "In the worst-case scenario, they will lead to an image push retry." — S1, "Blobs / Race conditions"
- Quote: "This timeout will be configurable and default to 2 seconds." — S1, "Blobs / Race conditions" (the storage-delete timeout that bounds how long the review row stays locked)
- Conditions/notes: the reasons given are that the lock needs a read-write connection for every blob existence check (so read replicas can't serve HEAD), that a heavily shared blob would hold many API transactions open, and that the chance of the collision is "negligible". Whether a later release implemented it was not checked in code.
- Relevance: beat it. A blob existence check that renews a lease on the blob (a conditional write to one record) removes the race without a lock held across a storage delete; accepting a push retry as the worst case means a blob the client was just told exists can vanish, which is a silent drop (rule 3).

### F6. GitLab does not prune the review queues when an object becomes referenced: the collector always re-validates.
- Quote: "For these reasons, we decided to keep it simple at the start, with no pruning." — S1, "Review queue pruning"
- Conditions/notes: pruning was rejected because it is "prone to complex race conditions" (an untag between the tag and the pruning delete would lose the review). Inserts use `ON CONFLICT … DO UPDATE` so a review is never lost while the collector holds the row.
- Relevance: copy. A candidate record is advice; the decision is the collector's own check inside its transaction.

### F7. Vendor-reported results: offline GC of GitLab.com was estimated at 64 days even after a 90% speed-up; online GC deletes terabytes a day.
- Quote: "Even with the performance improvements, we estimated a staggering 64 days to run garbage collection for GitLab.com." — S2, "Solving the online garbage collection problem" lead-in
- Quote: "Garbage collection deletes terabytes of data from GitLab.com each day." — S2, results list
- Quote: "Faced with petabytes of scale and the requirement to maintain our error budgets, we designed and implemented an online migration of GitLab.com with zero degradation in service." — S2
- Conditions/notes: vendor blog, 2023-12-04; no latency, throughput or queue-depth figures for the online collector were found in the spec or the blog.
- Relevance: the 64-day figure is the cost of a design that must stop writes to collect; it is the number that rules out stop-the-world collection at petabyte scale.

### F8. CNCF Distribution's collector is a stop-the-world mark-and-sweep that needs the registry read-only or stopped; an upload during a run can corrupt an image.
- Quote: "You should ensure that the registry is in read-only mode or not running at all. If you were to upload an image while garbage collection is running, there is the risk that the image’s layers are mistakenly deleted leading to a corrupted image." — S4, "More details about garbage collection" (rendered page; the Markdown source has the same words inside a `>` block)
- Quote: "This type of garbage collection is known as stop-the-world garbage collection." — S4, same section
- Quote: "Secondly, in the 'sweep' phase, the process scans all the blobs and if a blob's content address digest is not in the mark set, the process deletes it." — S4, same section
- Conditions/notes: v3.1.2; the command is `registry garbage-collect [--dry-run] [--delete-untagged] [--quiet]`. The page dates the command to v2.4.0; whether the text has changed since was not checked.
- Relevance: avoid. This is the race every online design below exists to close: a blob uploaded after the mark phase is not in the mark set.

### F9. Harbor's non-blocking GC (since v2.1.0) marks zero-reference blobs from its database, deletes them through a status machine that HEAD and PUT requests can interrupt, and adds a 2-hour time window because the status machine alone does not close every race.
- Quote: "Run garbage collection uninterrupted so users can continue pushing, pulling, deleting artifacts." — S8, "Non-blocking Garbage Collection"
- Quote: "To avoid damaging the uploading artifact, the garbage collection introduces a time windows(2 hours) to reserve the recent uploaded layers. Garbage collection does not sweep the manifest & blob files that have a timestamp in the time window." — S5, "Run Garbage Collection"
- Quote: "Harbor runs garbage collection without interrupting your ability to continue use Harbor, for example you are able to push, pull, or delete artifacts while garbage collection is running." — S5, "Run Garbage Collection"
- Quote: "// time window: default is 2 hours, and for testing/debugging, it can be set to 0." — S6, `parseParams`, L140
- Quote: "// set the status firstly, if the blob is updated by any HEAD/PUT request, it should be fail and skip." — S6, sweep, L326
- Quote: "if the blob is in deleting status, fail the request with 404." — S7, "Sweep / Question 1"
- Quote: "if the blob is in delete status, remove it from the candidate." — S7, "Sweep / Question 1"
- Quote: "Cutoff time -- We need to introduce the cutoff time, any to be deleted blob & manifest, the update time must not be later than the cutoff time." — S7, "DOUBLE GUARANTEE"
- Quote: "Time window -- We need to introduce the time window, the blobs/manifest in the time window will be reserved." — S7, "DOUBLE GUARANTEE"
- Counter: the design's authors say the optimistic `update_time` check does not close every race, which is why the window exists — "Even we introduce a update time to resolve the read/write, it still cannot resolve all of problems, like:" — S7
- Conditions/notes: blob states are none → `delete` (candidate) → `deleting` → removed, with `delete_failed` as a retry state; in the code the sweep's move to `deleting` is a conditional update (`UpdateBlobStatus`) whose affected-row count decides: zero rows means a HEAD or PUT got there first and the blob is skipped. Before 2.1 Harbor's collection needed read-only mode ("In the current release, Harbor uses online garbage collection, which needs Harbor to run with setting to readonly mode." — S7, "Motivation"). No reason is given for "2 hours" in the docs, the code or the proposal.
- Relevance: copy the status machine as a conditional transaction (candidate → deleting only if the record's version is unchanged); avoid relying on a wall-clock window whose size nobody justified, and avoid a window compared against clocks of different machines.

### F10. zot collects inline while serving; `gcDelay` defaults to 1 hour, to be set by "client network speeds and the size of blobs"; the periodic interval defaults to 1 hour in code and in one table, while another table on the same page says there is no periodic collection without a value.
- Quote: "since Distribution Specification APIs are not transactional between blob and manifest lifecycle, care must be taken so as not to put the storage in an inconsistent state." — S9, "Garbage collection"
- Quote: "The configuration model also allows the configuration of a tunable delay (`gcDelay`), which can be set depending on client network speeds and the size of blobs." — S9, "Configuring garbage collection"
- Quote: "This attribute has a default value of one hour (`1h`)." — S9, "Configuring garbage collection"
- Quote: "| omitted | n/a | GC enabled with 1 hour interval (default) |" — S9, "Configuring garbage collection" table
- Quote: "DefaultGCDelay = 1 * time.Hour DefaultGCInterval = 1 * time.Hour" — S10, `constants.go` L24-L25
- Quote: "// will garbage collect blobs older than Delay" — S10, `gc.go` L40
- Counter: the configuration-attribute table on the same page says the opposite about the interval's default — "If no value is specified, there is no periodic collection." — S9
- Quote: "A storage listing or existence-check failure aborts the affected collection pass; it is not treated as an empty repository." — S9, "Configuring garbage collection"
- Conditions/notes: zot v2.1.23. The delay is an age threshold applied to blobs and to upload sessions (`isBlobOlderThan` and `deleteBlobUploads` in `gc.go`); the examples in the docs set `gcDelay` to `1h` and `gcInterval` to `24h`. `gcTimeWindow` (a daily UTC window in which periodic sweeps may start) is a third setting beside the two the question named.
- Relevance: zot states the reason the window exists (the push protocol is not transactional between blobs and manifests) and what it should be scaled by (how long the slowest legitimate push takes). Copy the fail-closed rule: an incomplete listing is an error, never "nothing is referenced".

### F11. zot's default retention deletes every untagged manifest not referenced by an index or artifact once `gcDelay` has passed; the retention `delay` defaults to `gcDelay`.
- Quote: "By default, if `retention` is not configured, garbage collection deletes all untagged manifests which are not referenced by indexes or artifacts after the `gcDelay` passes." — S9, "Configuring garbage collection"
- Quote: "| delay | time | Remove untagged and referrers only if they are older than the specified <time\> hours. Defaults to `gcDelay`. |" — S11, configuration table
- Conditions/notes: retention policies add `keepTags` rules, `keepUntagged` criteria (`mostRecentlyPushedCount`, `mostRecentlyPulledCount`, `pushedWithin`, `pulledWithin`), `deleteReferrers` (default false) and `deleteUntagged` (default true), with a `dryRun` flag (default false).
- Relevance: an untagged manifest pulled by digest is live data for whoever pinned it; a default that deletes it after one hour is a default to avoid without an explicit policy.

### F12. containerd's collector is lease-based: a resource is eligible for removal unless it is referenced or held by a lease; the Go client's lease expires after 24 hours by default.
- Quote: "The client is responsible for ensuring that all resources which are created are either used or held by a lease at all times, else they will be considered eligible for removal." — S12, introduction
- Quote: "Leases are needed to inform the `containerd` daemon that a resource may be used in the future after the client completes an operation, even though it currently is not seen as utilized." — S12, "What is a lease?"
- Quote: "This will create a lease which will defer its own deletion and have a default expiry of 24 hours (in case the process dies before defer)." — S12, "How to use leases"
- Quote: "By default the garbage collector will attempt to keep the database unlocked 98% of the time based on prior calculations of lock time from garbage collection." — S12, "Garbage Collection configuration"
- Conditions/notes: v2.4.2. References are label-driven (`containerd.io/gc.ref.content`, `containerd.io/gc.root`, `containerd.io/gc.expire`); scheduler defaults are `pause_threshold = 0.02`, `deletion_threshold = 0`, `mutation_threshold = 100`, `schedule_delay = "0ms"`, `startup_delay = "100ms"`. It is a single-node store that locks its database while it marks.
- Relevance: copy the lease as the thing that protects an in-flight push (an upload session is a lease on every blob it has named, with an expiry), in place of an age window. The 24-hour expiry exists only for the client that dies.

### F13. Amazon ECR documents lifecycle expiry as eventual: an image expires within 24 hours of meeting a rule, and its referrers within 24 hours of the subject's deletion.
- Quote: "Once a lifecycle policy is applied to a repository, you should expect that images become expired within 24 hours after they meet the expiration criteria." — S13, "How lifecycle policies work"
- Quote: "When reference artifacts are present in a repository, Amazon ECR lifecycle policies automatically expire or archive those artifacts within 24 hours of the deletion or archival of the subject image." — S13, "Lifecycle policy evaluation rules"
- Conditions/notes: as fetched on 2026-11-20 a rule's action is expire or archive. AWS does not document when the bytes of an expired image are reclaimed, only when the image expires.
- Relevance: a hosted registry promises a bound on policy evaluation, not immediacy; a new registry should state its own bound and meet it.

### F14. A production deduplicating store (Data Domain) chose mark-and-sweep over reference counts for complexity and scale, and says reference counts are hard to keep correct under error conditions; the cited concurrent alternative is epoch-based deletion with counters.
- Quote: "DDFS uses a mark-and-sweep [27] algorithm that determines the set of live chunks reachable from the live files and then frees up unreferenced space." — S14, §1
- Quote: "There are alternatives such as reference counting [8], but as we discuss in §6, complexity and scalability issues have led to our current approach." — S14, §1
- Quote: "Finally, reference counts are also difficult to maintain correctly in the presence of complex error conditions." — S14, §6
- Quote: "Strzelczak et al. [22] implemented an epoch-based deletion system with counters to support concurrent writes and deletes." — S14, §2.1.1
- Conditions/notes: DDFS chunks are about 8 KB and the paper's subject is making the mark phase scale with physical rather than logical size (perfect hashes over fingerprints). A registry's objects are layers and manifests, orders of magnitude fewer and larger, so the paper's scalability argument against counters is weaker there; its correctness argument is not.
- Relevance: if references are counted, the count and the reference must change in one transaction and be checkable against a scan; the epoch idea (a delete takes effect only after every writer that could have seen the old state has finished) is the lease of F12 in another form.

### F15. Windows Azure Storage collects an extent once no stream references it, and names garbage collection as the standing cost of an append-only store.
- Quote: "When an extent is no longer referenced by any stream, the SM garbage collects the extent and notifies the ENs to reclaim the space." — S15, §4.1 "Stream Manager and Extent Nodes"
- Quote: "An efficient and scalable garbage collection (GC) system is crucial to keep the space overhead low, and GC comes at a cost of extra I/O." — S15, §8 (design choices and lessons learned, append-only system)
- Relevance: reclaiming bytes in the object store is a second collection under the registry's own; the registry's delete of a blob is only a dropped reference there.

### F16. Ceph's deduplication reference counts can be wrong and are repaired by a periodic scrub.
- Quote: "Reference mismatches can on rare occasions occur to false positives when handling reference counts for deduplicated RADOS objects. These mismatches will be fixed by periodically scrubbing the pool:" — S16, "Run scrub to fix reference count"
- Relevance: a counter needs an auditor. Whatever the new registry counts, a background scan must be able to recompute it and report the difference.

### B. Replication, regions and consistency

### F17. A hybrid logical clock sets its logical part `l` to the maximum of its previous value, the incoming message's value and the local physical clock, and counts ties in `c`.
- Quote: "When a new send event f is created, l.j is set to max(l.e, pt.j), where e is the previous event on j." — S17, HLC algorithm (text beside Figure 5)
- Quote: "When a new receive event is created, l.j is set to max(l.e, l.m, pt.j)." — S17, HLC algorithm
- Quote: "If l.e differs from l.f then c.j is reset, and this allows us to guarantee that c values remain bounded." — S17, HLC algorithm
- Conditions/notes: Figure 5 gives the rules in full: on send or local event, `l' := l; l := max(l', pt); if l = l' then c := c + 1 else c := 0`; on receive of `m`, `l := max(l', l.m, pt)`, then `c := max(c, c.m) + 1` if all three are equal, `c + 1` if `l = l'`, `c.m + 1` if `l = l.m`, else `0`. (Figure text paraphrased here because `pdftotext` renders the primes as digits.)
- Relevance: the timestamp for last-writer-wins across regions; it orders causally related writes correctly even when a region's clock is behind.

### F18. HLC's distance from physical time is bounded: `l` never falls behind the local clock and exceeds it by at most the clock synchronization uncertainty.
- Quote: "l.e is close to pt.e, i.e., |l.e − pt.e| is bounded." — S17, problem statement, requirement 4
- Quote: "Using Theorem 3, we can show that |l − pt| is bounded." — S17, before Corollary 1
- Quote: "Given the relation between l and pt from Theorem 2, the drift between l and pt is limited to the clock drift." — S17, stress-test section
- Quote: "Dually, HLC can be used in lieu of physical/NTP clocks since it maintains its logical clock to be always close to the NTP clock." — S17, abstract
- Conditions/notes: Theorem 2 is `l.f ≥ pt.f`; Corollary 1 is `|l.f − pt.f| ≤ ε`, with ε the clock synchronization uncertainty. The corollary's formula is not among the checked quotes: `pdftotext` drops the ε glyph. The bound holds only while clocks stay within ε. Inference from the receive rule (it takes the maximum of the incoming `l`): a node whose clock jumps ahead carries every node it talks to ahead with it. The paper's resilience section sets a bound ∆ on `l − pt` and describes ignoring messages that violate it; that passage is interleaved with another column in the extracted text, so only its first clause is quoted below.
- Quote: "we set a very conservative value, ∆, on the l − pt bound." — S17, resilience section (before §4.1 "Self-stabilization")
- Relevance: a registry using HLC must refuse a message whose `l` is further ahead of local time than a stated bound (a typed refusal), or one bad clock sets every region's timestamps into the future.

### F19. A last-writer-wins register totally orders assignments by timestamp and merges by keeping the value with the greatest timestamp; timestamps must be unique, totally ordered and consistent with causal order.
- Quote: "A Last-Writer-Wins Register (LWW-Register) creates a total order of assignments by associating a timestamp with each update." — S18, §3.2.1
- Quote: "Timestamps are assumed unique, totally ordered, and consistent with causal order" — S18, §3.2.1
- Quote: "The monotonic semilattice orders two values by their associated timestamp; merge procedure selects the value with the maximal timestamp." — S18, §3.2.1
- Quote: "concurrent assignments commute since, whatever the order of execution, only the one with the highest timestamp takes effect." — S18, §3.2.1
- Counter: which of two concurrent writes wins is decided by how timestamps happen to be allocated, not by anything the users meant — "in LWW-Set (Figure 8) the outcome of concurrent updates depends on opaque details of how timestamps are allocated." — S18, §3.3 (on LWW-Set)
- Conditions/notes: uniqueness needs a tiebreak (the paper suggests a per-replica counter with a replica identifier); an HLC `(l, c)` pair with a region identifier satisfies the three requirements.
- Relevance: a tag is an LWW register (name → digest). The losing push must not vanish silently: keep it as a recorded superseded write (rule 3), and make the manifest it pointed at subject to the GC grace period, not immediately unreferenced.

### F20. State-based CRDT replicas converge if every update eventually reaches every replica; because merge is idempotent and commutative, loss, reordering and duplication are tolerated.
- Quote: "Eventual convergence requires that all replicas receive all updates." — S18, §2.2 (state-based replication)
- Quote: "Any two object replicas of a CvRDT eventually converge, assuming the system transmits payload infinitely often between pairs of replicas over eventually-reliable point-to-point channels." — S18, Proposition 2.1
- Relevance: the replication stream between regions may redeliver and reorder; what it may not do is drop an update for good, so it needs an anti-entropy pass that proves two regions hold the same state.

### F21. Azure Container Registry documents geo-replication as eventually consistent and lists the failures that follow: cross-region read-after-write misses, a tag resolving to different digests in different regions, deletes still served, and a push whose layers and manifest land in different regions.
- Quote: "After you push or delete an image in any geo-replica, ACR eventually replicates the change to all geo-replicas in the background." — S19, "Consistency model"
- Quote: "Pushing an image to one geo-replica and immediately pulling it from a different geo-replica can fail with manifest unknown until replication catches up." — S19, "Consistency model"
- Quote: "Tag overwrite races — Pushing myapp:v1, then re-pushing myapp:v1 shortly after with a different digest (same tag, different content), can leave different geo-replicas resolving the same tag to different digests during the replication window." — S19, "Consistency model"
- Quote: "Pulls from geo-replicas where the delete hasn't propagated yet can still return the deleted content." — S19, "Consistency model"
- Quote: "Because replication is eventually consistent, the manifest can land on a replica that doesn't yet have the layers it references, and the manifest validation fails." — S19, troubleshooting, "Push fails with manifest errors"
- Conditions/notes: page last updated 2026-08-26. No conflict rule is documented: the page says replicas can disagree "during the replication window" and does not say which of two concurrent pushes to one tag in two regions wins, nor give a number for the lag ("Replication time depends on image size"). Mitigations offered are webhooks on replication completion, retry on pull, and regional endpoints that pin a client to one replica.
- Relevance: these four are the test cases for the new registry's multi-region suite. Beat each: a documented, deterministic conflict rule (F19); a manifest never accepted in a region that lacks its blobs; and a way for a client to wait for a write to be visible in a named region.

### F22. Amazon ECR replication is asynchronous, happens once per push with no chaining, and documents a typical delay: most images in under 30 minutes.
- Quote: "The majority of images replicate in less than 30 minutes, but in rare cases the replication might take longer." — S20, "Considerations for private image replication"
- Quote: "A replication action only occurs once per image push or image restore." — S20, "Considerations for private image replication"
- Quote: "If tag immutability is enabled on a repository and an image is replicated that uses the same tag as an existing image, the image is replicated but won't contain the duplicated tag." — S20, "Considerations for private image replication"
- Quote: "Registry replication doesn't perform any archive actions, and by default it doesn't perform any delete actions." — S20, "Considerations for private image replication"
- Quote: "The replication configuration for a private registry may contain up to 25 unique destinations across all rules, with a maximum of 25 rules total." — S20, "Considerations for private image replication"
- Conditions/notes: replication is one-way per rule (source to destinations). The only conflict behaviour documented is the immutable-tag case above, where the replicated image arrives untagged; what happens to a mutable tag pushed in both regions is not stated. Replication of deletes (opt-in per rule) and replication backfill are on the page as fetched on 2026-11-20.
- Relevance: "less than 30 minutes" is the lag a new registry's cross-region replication is compared against; a tag silently dropped on arrival is behaviour to avoid.

### F23. Harbor replicates by rule, with three trigger modes: manual, scheduled by cron, or event-based on push, retag and (optionally) delete.
- Quote: "* **Event Based**: When a new resource is pushed to the project, or an artifact is retagged, it is replicated to the remote registry immediately." — S21, "Creating a Replication Rule", trigger mode
- Quote: "* **Scheduled**: Replicate the resources periodically by defining a cron job. **Note**: Deletion operations are not replicated." — S21, same list
- Quote: "Optionally select the Override checkbox to force replicated resources to replace resources at the destination with the same name." — S21, same procedure
- Conditions/notes: push-based or pull-based per rule, between separate registries; the conflict rule is the per-rule Override checkbox (replace the destination's resource of the same name, or not). No consistency statement or lag figure on the page. Copy-by-chunk defaults to 10MB chunks (`REPLICATION_CHUNK_SIZE`).
- Relevance: Harbor replication is a copy job between two registries, not one registry in several regions; it is not the model for rule 8's "same code path".

### F24. Google Artifact Registry's multi-region repositories are documented for availability and latency only; no consistency or replication-lag statement was found.
- Quote: "Multi-region repositories may continue to work if one of their regions are down. However, operations in multi-region repositories may be slower than single-region repositories." — S22, location considerations
- Conditions/notes: the same paragraph adds that an operational failure of a multi-region repository affects all its users regardless of region.
- Relevance: nothing to copy; the absence of a stated model is itself the finding.

### F25. Amazon S3 has given strong read-after-write and list consistency within a Region since 1 December 2020.
- Quote: "Amazon S3 now delivers strong read-after-write consistency automatically for all applications." — S23, announcement, 2020-12-01
- Quote: "After a successful write of a new object, or an overwrite or delete of an existing object, any subsequent read request immediately receives the latest version of the object." — S24, "Amazon S3 Strong Consistency"
- Conditions/notes: the announcement words the guarantee as holding "without sacrificing regional isolation for applications"; S3's cross-Region replication was not examined here.
- Relevance: the bar for a single region's blob store is read-after-write, including for deletes and listings; a registry must not design around an eventually consistent object store that no longer exists.

### F26. Elle infers an Adya-style dependency graph from client-observed histories and can detect and explain every Adya anomaly except those on predicates.
- Quote: "Elle can detect every anomaly in Adya et al's formalism [Adya et al. 2000] (except for predicates), discriminate between them, and provide concise explanations of each." — S25, abstract
- Relevance: the checker for the record store's transactions as the registry uses them (list-append workloads on tag and reference records), under a nemesis.

### F27. General linearizability checkers (Knossos, Porcupine) are limited by the NP-completeness of the problem.
- Quote: "its use is limited by the NP-complete nature of linearizability checking, and the combinatorial explosion of states" — S25, §1 (PVLDB PDF)
- Conditions/notes: this is the Elle paper's statement about Jepsen's own linearizability checker; no separate Jepsen document was fetched.
- Relevance: keep linearizability histories short and per key (one tag, one upload session); use Elle-style inference for long multi-key histories.

### F28. FoundationDB runs the real database code under randomized workloads and injected faults in a deterministic discrete-event simulation, so every bug found is reproducible; the authors also state what simulation cannot find.
- Quote: "real database software is run, together with randomized synthetic workloads and fault injection, in a deterministic discrete-event simulation." — S26, §4 "Simulation Testing"
- Quote: "determinism guarantees that every bug found this way can be reproduced, diagnosed, and fixed." — S26, §4
- Counter: simulation does not find performance problems, and cannot test dependencies outside the simulated code — "Simulation is not able to reliably detect performance issues, such as an imperfect load balancing algorithm." — S26, §4 "Limitations"
- Relevance: the GC and replication cores should be sans-IO state machines driven by a simulator with a nemesis (rule 9, rule 6); the limitation is why the object store and the record store each need their own conformance suite.

### F29. Application crash-consistency depends on file-system behaviours that differ between file systems: 60 vulnerabilities were found in eleven widely used systems.
- Quote: "Using ALICE, we analyze eleven widely-used systems (including databases, key-value stores, version control systems, distributed systems, and virtualization software) and find a total of 60 vulnerabilities, many of which lead to severe consequences." — S27, abstract
- Relevance: any local-disk path (the laptop mode's store, upload staging) needs crash injection at every write, not reasoning about what one file system does.

### F30. No application studied handles an fsync failure safely: failures cause data loss and corruption, and file systems mark the failed pages clean.
- Quote: "Our findings show that although applications use many failure-handling strategies, none are sufficient: fsync failures can cause catastrophic outcomes such as data loss and corruption." — S28, abstract
- Quote: "We find commonalities across file systems (pages are always marked clean, certain block writes always lead to unavailability), as well as differences (page content and failure reporting is varied)." — S28, abstract
- Conditions/notes: three Linux file systems (ext4, XFS, Btrfs) and five applications (PostgreSQL, LMDB, LevelDB, SQLite, Redis).
- Relevance: an fsync error is a typed error that fails the write and discards the affected state; it is never retried on the same file descriptor as if the pages were still dirty.

### C. Cloud and hosted registries: limits, behaviour, scale

### F31. Docker Hub's documented pull limits on 2026-11-20: 100 pulls per 6 hours per IPv4 address or IPv6 /64 subnet unauthenticated, 200 per 6 hours for a Personal account, and no pull rate limit for Pro, Team and Business.
- Quote: "| Personal (authenticated) | 200 | | Unauthenticated Users | 100 per IPv4 address or IPv6 /64 subnet |" — S29, table "Pull rate limit per 6 hours"
- Quote: "In the previous example, the pull limit is 100 pulls per 21600 seconds (6 hours), and there are 20 pulls remaining." — S29, "View pull rate and limit"
- Conditions/notes: the table's first rows read `Business (authenticated) | Unlimited`, `Team (authenticated) | Unlimited`, `Pro (authenticated) | Unlimited`, "subject to fair use".
- Relevance: a pull-through cache must budget against the upstream's window (100 or 200 manifest GETs per 21,600 s) and authenticate to it.

### F32. Docker Hub counts a manifest GET as a pull and does not count a version check (a manifest HEAD); a multi-arch pull counts once per architecture.
- Quote: "Version checks do not count towards pull rate limits." — S29, "Pull definition"
- Quote: "These headers are returned on both GET and HEAD requests. Using GET emulates a real pull and counts towards the limit. Using HEAD won't." — S29, "View pull rate and limit"
- Quote: "A Docker pull includes both a version check and any download that occurs as a result of the pull." — S29, "Pull definition"
- Quote: "A pull for a normal image makes one pull for a [single manifest](https://github.com/opencontainers/image-spec/blob/main/manifest.md)." — S29, "Pull definition"
- Quote: "A pull for a multi-arch image will count as one pull for each different architecture." — S29, "Pull definition"
- Conditions/notes: blob GETs are not mentioned as counted; the limit is enforced "when the manifest is requested" (F33).
- Relevance: a cache revalidates with HEAD (free) and GETs a manifest only when the digest changed; it must not GET every platform manifest of an index it was asked for one platform of.

### F33. Docker Hub reports the limit in `ratelimit-limit` and `ratelimit-remaining` headers with a `;w=<seconds>` window, and answers 429 with a fixed body when the manifest is requested over the limit.
- Quote: "ratelimit-limit: 100;w=21600 ratelimit-remaining: 20;w=21600 docker-ratelimit-source: 192.0.2.1" — S29, "View pull rate and limit", step 3
- Quote: "When you issue a pull and you are over the limit, Docker Hub returns a `429` response code with the following body when the manifest is requested:" — S29, "View pull rate and limit"
- Quote: "You have reached your pull rate limit. You may increase the limit by authenticating and upgrading: https://www.docker.com/increase-rate-limits" — S29, "View pull rate and limit"
- Quote: "If you don't see any `ratelimit` header, it could be because the image or your IP is unlimited in partnership with a publisher, provider, or an open source organization." — S29, "View pull rate and limit"
- Conditions/notes: the probe repository is `ratelimitpreview/test`; the token comes from `auth.docker.io/token?service=registry.docker.io&scope=repository:ratelimitpreview/test:pull`. The page shows the 429 body as the plain sentence above; whether it is wrapped in an OCI error document on the wire was not tested.
- Relevance: a cache parses `w=` rather than assuming 21600, treats absent headers as "unlimited", and surfaces the upstream's 429 as its own typed refusal.

### F34. Docker Hub has a second, separate abuse limit on all requests, per IP, in the order of thousands of requests per minute, distinguishable by its bare 429.
- Quote: "while the limit changes over time depending on load and other factors, it's in the order of thousands of requests per minute." — S30, "Abuse rate limit"
- Quote: "The abuse limit returns a simple `429 Too Many Requests` response. The pull limit returns a longer error message that includes a link to documentation." — S30, "Abuse rate limit"
- Relevance: HEAD revalidation is free of the pull limit but not of this one; a cache bounds its own request rate per upstream.

### F35. The hourly limits Docker announced for 1 April 2025 were never enforced; the 6-hour limits stand, and paid plans became unlimited.
- Quote: "We did not enforce the Docker Hub rate limit changes previously scheduled for April 1, 2025." — S31, update of 2025-04-08
- Conditions/notes: the same update restates "100 pulls per 6 hours for unauthenticated users and 200 pulls per 6 hours for Docker Personal users"; the docs fetched on 2026-11-20 (S29, S30) show the same numbers, so no 2026 change was found. Hourly figures for unauthenticated pulls in 2025 press coverage describe the plan that was not enforced.
- Relevance: design against the documented numbers and read the headers, since the vendor has changed its mind once.

### F36. ECR pull-through cache supports nine upstream kinds and re-validates a cached tag against its upstream when it is pulled and the last validation is more than 24 hours old; referrers use a 6-hour window; if the upstream can't be reached the cached image is served.
- Quote: "Amazon ECR Public, Kubernetes container image registry, and Quay (doesn't require authentication)" — S32, introduction
- Quote: "Docker Hub, Microsoft Azure Container Registry, GitHub Container Registry, GitLab Container Registry and Chainguard Registry (requires authentication with AWS Secrets Manager secret)" — S32, introduction
- Quote: "Amazon ECR checks whether it has validated the image against the upstream registry within the last 24 hours." — S32, considerations
- Quote: "If the 24-hour window has expired, Amazon ECR sends a request upstream to check for a newer version and updates the cache if one exists." — S32, considerations
- Quote: "Amazon ECR checks whether the referrer artifacts have been updated within the last 6 hours." — S32, considerations
- Quote: "If Amazon ECR is unable to update the image from the upstream registry for any reason and the image is pulled, the last cached image will still be pulled." — S32, considerations
- Conditions/notes: the ninth upstream is another Amazon ECR registry (authenticated by IAM role). The quota is 50 pull-through cache rules per registry (S33). The page's introduction words the same rule as an update "at least once every 24 hours". A tag moved upstream can therefore be stale for up to 24 hours.
- Relevance: beat the 24-hour staleness with a HEAD revalidation (free at Docker Hub, F32) on a short, stated interval; copy serving the cached copy when the upstream is down, and say so in the response.

### F37. ECR has supported OCI Image and Distribution 1.1 with reference types since June 2024, and since April 2026 its pull-through cache discovers and syncs upstream referrers.
- Quote: "Today, Amazon Elastic Container Registry (ECR) announced that it supports Open Container Initiative (OCI) Image and Distribution specification version 1.1, which includes support for Reference Types" — S34, 2024-06-27
- Quote: "Amazon Elastic Container Registry (Amazon ECR) now automatically discovers and syncs OCI referrers, such as image signatures, SBOMs, and attestations, from upstream registries into your Amazon ECR private repositories with its pull through cache feature." — S35, 2026-04-17
- Conditions/notes: the 2024 announcement also says replication copies referrers and lifecycle policies delete them with their subject.
- Relevance: referrers travel with their subject through every path (replication, cache, lifecycle); a new registry's three paths need the same property and a test for each.

### F38. ECR's size quotas: layer at most 52,000 MiB, layer parts between 5 and 10 MiB and at most 4,200 of them, 100,000 images per repository (adjustable), 1,000 tags per image.
- Quote: "Maximum layer size Each supported Region: 52,000 No The maximum size (MiB) of a layer." — S33, quota table
- Quote: "Maximum layer part size Each supported Region: 10 No The maximum size (MiB) of a layer part." — S33, quota table
- Quote: "Minimum layer part size Each supported Region: 5 No" — S33, quota table
- Quote: "Layer parts Each supported Region: 4,200 No The maximum number of layer parts." — S33, quota table
- Quote: "Images per repository Each supported Region: 100,000 Yes" — S33, quota table
- Quote: "Tags per image Each supported Region: 1,000 No" — S33, quota table
- Conditions/notes: the part limits apply to ECR's own upload API (`UploadLayerPart`), not to OCI chunked upload through the registry endpoint. Derived: 4,200 parts × 10 MiB = 42,000 MiB, which is less than the 52,000 MiB layer limit; the page does not explain the gap. A maximum image manifest size was not found on the quotas page.
- Relevance: state a bound for each of these (rule 2), larger where the workload needs it (microVM images), and refuse over it with a typed error.

### F39. ECR's default API rates per Region: 10 PutImage, 100 InitiateLayerUpload, 100 CompleteLayerUpload, 500 UploadLayerPart, 1,000 BatchCheckLayerAvailability, 2,000 BatchGetImage, 3,000 GetDownloadUrlForLayer and 500 GetAuthorizationToken requests per second.
- Quote: "Rate of PutImage requests Each supported Region: 10 per second Yes" — S33, quota table
- Quote: "Rate of InitiateLayerUpload requests Each supported Region: 100 per second Yes" — S33, quota table
- Quote: "Rate of CompleteLayerUpload requests Each supported Region: 100 per second Yes" — S33, quota table
- Quote: "Rate of UploadLayerPart requests Each supported Region: 500 per second Yes" — S33, quota table
- Quote: "Rate of BatchCheckLayerAvailability requests Each supported Region: 1,000 per second Yes" — S33, quota table
- Quote: "Rate of BatchGetImage requests Each supported Region: 2,000 per second Yes" — S33, quota table
- Quote: "Rate of GetDownloadUrlForLayer requests Each supported Region: 3,000 per second Yes" — S33, quota table
- Quote: "Rate of GetAuthorizationToken requests Each supported Region: 500 per second Yes" — S33, quota table
- Conditions/notes: all adjustable ("Yes"), per account per Region. The page gives rates only; token-bucket parameters (burst capacity, refill) for ECR were not found. Derived: 10 manifest writes per second and 2,000 manifest reads per second is a 1:200 default write-to-read ratio.
- Relevance: these are the per-tenant defaults of the largest hosted registry, the class-for-class comparison for a per-tenant admission limit; 15,000 containers a second (F45) can't be fed from a 2,000-per-second manifest read quota without caching below the registry.

### F40. Azure Container Registry documents per-registry and per-identity request limits by tier, enforced by a token bucket, with a 195 GiB layer limit and a 4 MiB manifest limit.
- Quote: "Maximum image layer size (GiB) 195 195 195 Maximum manifest size (MiB) 4 4 4" — S38, "SKU features and limits" table (Basic, Standard, Premium)
- Quote: "DataplaneRead Per registry 10,000 r/m 20,000 r/m" — S38, rate limit table (Basic and Standard; Premium)
- Quote: "DataplaneWrite Per registry 2,000 r/m 4,000 r/m" — S38, rate limit table
- Quote: "ListReferrers1 Per registry 500 r/m 2,000 r/m" — S38, rate limit table (the `1` is a footnote mark)
- Quote: "Rate limits are enforced by using a token bucket algorithm. Each operation category has a bucket of request capacity that refills continuously at the rate shown in the preceding table." — S38, "How rate limits are enforced"
- Quote: "If a burst empties the bucket, subsequent requests are rejected with 429 Too Many Requests until capacity refills, which can result in requests being throttled for up to a full minute after a large burst." — S38, "How rate limits are enforced"
- Counter: the published numbers are not guarantees — "The rate limits listed in the preceding table are best-effort approximate maximums and are not backed by an SLA." — S38
- Conditions/notes: page last updated 2026-10-16. Other rows: per identity per registry limits are half the per-registry ones (DataplaneRead 5,000 and 10,000 r/m); MetadataRead 4,000 and 8,000 r/m; DataplaneDelete 1,000 and 4,000 r/m; OAuth 10,000 and 20,000 r/m. A request can count against several categories; 429 carries a `Retry-After` that counts down. Bucket capacity is not stated as a number.
- Relevance: copy the structure (per tenant and per identity, by operation class, `Retry-After` on refusal) and state the bucket's capacity as well as its rate; a 4 MiB manifest limit is the peer value for the registry's own manifest bound.

### F41. GitHub's Container registry documents a 10 GB limit per layer and a 10-minute limit per upload.
- Quote: "The Container registry has a 10 GB size limit for each layer." — S36, "Troubleshooting"
- Quote: "The Container registry has a 10 minute timeout limit for uploads." — S36, "Troubleshooting"
- Conditions/notes: the page lists supported formats as Docker Image Manifest V2 Schema 2 and OCI; it says nothing about the referrers API, a manifest size limit or request rates.
- Relevance: a wall-clock upload timeout is a limit to avoid; bound an upload session by its lease and its bytes instead.

### F42. Google Artifact Registry's quotas are per project per location: 60,000 requests per minute, of which at most 18,000 writes, with every HTTP request of a pull counted.
- Quote: "Requests per minute in each region or multi-region 60,000" — S37, "Per-project request quota"
- Quote: "Write requests per minute in each region or multi-region 18,000" — S37, "Per-project request quota"
- Quote: "A Docker pull or push usually makes multiple HTTP requests, so quota is charged for each request." — S37, "Per-project request quota"
- Quote: "Max data retrieval from upstream of remote repository per request 9.9 GB" — S37, "Usage Limits"
- Conditions/notes: page last updated 2026-11-10. Also: delete requests 18,000 per minute; cleanup policy deletions 300,000 per repository per day; remote repositories read public upstreams at 600 requests per organization per region per minute by default. A maximum layer or manifest size and referrers API support were not found on the pages fetched.
- Relevance: Derived: 60,000 per minute is 1,000 requests per second per project, half of ECR's default manifest-read rate alone; hosted limits differ by an order of magnitude and a new registry should publish its own.

### F43. The OCI referrers API is supported by ECR (2024-06), Quay.io (2024-12), JFrog Artifactory (from 7.90.1) and ACR (it rate-limits `ListReferrers`); first-party statements for GHCR and Google Artifact Registry were not found.
- Quote: "To help enhance IT security and compliance, we are announcing the availability of the Open Container Initiative (OCI) Referrers API on Quay.io." — S41, 2024-12-02
- Quote: "Starting from Artifactory version 7.90.1, Artifactory supports Referrers API, part of the OCI specification version 1.1 which provides a new way to link between images and their related information." — S42
- Quote: "JFrog is now fully certified to the OCI v1.1 standard. We are proud to be one of only two vendors who have made this commitment thus far." — S43, 2024-09-24
- Conditions/notes: ECR is F37; ACR is F40. Secondary sources (a 2026 third-party advisory and community threads, not fetched) say GHCR does not serve `/v2/<name>/referrers/<digest>`; treat as unverified.
- Relevance: a client of a new registry will still meet registries without the endpoint, so the pull-through cache needs the referrers tag-schema fallback of the distribution spec; the registry itself serves the API.

### F44. Lazy loading is offered around registries, not by the distribution API: ACR's artifact streaming is still a preview limited to Linux AMD64, and GKE Image streaming mounts a remote filesystem for images held in Artifact Registry or public Docker Hub.
- Quote: "Only images with Linux AMD64 architecture are supported in the preview release." — S39, "Current limitations and requirements"
- Quote: "With Image streaming, GKE uses a remote filesystem as the root filesystem for any containers that use eligible container images." — S40, "Overview"
- Quote: "Your container images must be stored in standard or remote repositories in Artifact Registry or in public registries on Docker Hub." — S40, "Requirements"
- Conditions/notes: ACR artifact streaming is Premium tier only and converts images on push when a repository's streaming is active. GKE's page was last updated 2026-11-10.
- Relevance: both tie lazy loading to one vendor's node runtime; the companion notes (`lazy-loading-p2p-chunking.md`) cover the open formats.

### F45. AWS Lambda adds up to 15,000 containers per second for one customer from images of up to 10 GiB, by loading 512 KiB content-addressed chunks on demand.
- Quote: "adding up to 15,000 new containers per second for a single customer, and much more in aggregate" — S44, abstract
- Quote: "it has reliably processed hundreds of trillions of Lambda invocations for over a million AWS customers" — S44, abstract
- Quote: "Each fixed-size chunk is 512KiB. Smaller chunks lead to better deduplication by minimizing false-sharing, and can accelerate loading for workloads with highly random access patterns." — S44, PDF §2 (chunking)
- Conditions/notes: 2023; the same abstract gives "millions of requests per second", "millions of unique workloads" and start-up "as low as 50ms".
- Relevance: the start-rate anchor for a registry feeding sandboxes: 15,000 starts per second per tenant is met by chunk caches below the registry, not by registry API calls per start.

### F46. Firecracker: under 5 MB of memory overhead per microVM, under 125 ms to application code, up to 150 microVMs per second per host.
- Quote: "it offers memory overhead of less than 5MB per container, boots to application code in less than 125ms, and allows creation of up to 150 MicroVMs per second per host." — S47, PDF §1
- Quote: "where it supports millions of production workloads, and trillions of requests per month." — S47, abstract
- Conditions/notes: 2020, with the paper's minimal guest kernel configuration; the evaluation section measures about 3 MB of VMM overhead.
- Relevance: Derived: 15,000 starts per second (F45) at 150 per host is at least 100 hosts creating at full rate; each needs its image bytes local or lazily served.

### F47. Meta's XFaaS runs trillions of function calls a day on more than 100,000 servers; Twine manages one million machines per region with one control plane.
- Quote: "XFaaS currently processes trillions of function calls per day on more than 100,000 servers." — S45, abstract
- Quote: "Twine scales a single control plane to manage one million machines across all data centers in a geographic region and transparently move jobs across clusters." — S46, abstract
- Conditions/notes: 2023 and 2020. Neither paper gives a registry pull rate.
- Relevance: the fleet-size anchors for "global multi-region": 10^5 to 10^6 machines per region pulling from one logical registry.

### F48. Docker's own figures for Docker Hub: 20 billion pulls a month on one current page, over 26 billion image downloads a month in a 2025 brief; the last first-party repository count found is 7 million (2020).
- Quote: "30M+ developers on Docker Hub 20B pulls per month 200x more usage than community repos, on average" — S50, statistics tiles, fetched 2026-11-20
- Quote: "With over 26 billion image downloads and 29 million unique IPs pulling monthly, Hub is where modern development happens." — S48, page metadata 2025-03-19
- Quote: "But July has already left that number in the dust, with 11 billion pulls in the past month, according to the Docker Index." — S49, 2020-07-30
- Quote: "the number of repositories on Docker Hub has grown from 6 million to 7 million" — S49, 2020-07-30
- Conditions/notes: vendor numbers; the two current figures (20 billion and 26 billion) disagree and neither page defines "pull" or "download". Derived: 20 billion a month is about 7,700 a second on average over a 30-day month (20e9 / 2,592,000 s).
- Relevance: the public-registry anchor: 10^10 pulls a month, average in the high thousands per second, peak unknown.

### F49. Sandbox platforms state volumes in the billions of sandboxes: E2B three billion starts in total, Modal three billion as of October 2026 and more than 10,000 creations per second in a demonstration.
- Quote: "Three billion sandbox starts and counting, across coding agents, data analysis, computer use, and RL training runs." — S51, statistics, fetched 2026-11-20
- Quote: "Monthly sandbox volume crosses 15 million." — S51, timeline entry "2025"
- Quote: "Modal has demonstrated creating 1M+ sandboxes in under a minute, exceeding 10,000 sandbox creations per second, and reports running 3 billion sandboxes as of October 2026." — S52
- Quote: "Over the course of the event, Modal ran over 1 million sandboxes, powering up to 20,000 concurrent sandboxes at peak." — S53, 2025-07-07 (a 48-hour customer event in June 2025)
- Conditions/notes: vendor numbers, cumulative unless stated. E2B's timeline on the same page has a billion sandboxes for June 2026 and two billion for October 2026, so the headline (three billion) is ahead of its own newest timeline entry. No first-party volume was found for Daytona.
- Relevance: agent sandboxes start at 10^4 per second in bursts on one platform; each start resolves an image, which is the read load a registry for agents is sized for.

### F50. Outage: Quay.io, May 2020. A storm of tens of thousands of database connections locked its MySQL instance; the application ran with default (unbounded) connection settings per worker.
- Quote: "The primary symptom of the outage was a storm of tens of thousands of database connections, effectively locking our MySQL instance." — S54, "Root cause analysis"
- Quote: "After the first outage, we discovered that Quay was using the default database connection settings within our gevent workers." — S54
- Quote: "disabled as many non-essential features as possible (e.g. namespace garbage collection) to reduce database load" — S54
- Relevance: every connection pool has a bound (rule 2), and garbage collection must be able to shed load without being switched off by hand.

### F51. Outage: Docker Hub, April 2021. A registry deploy and its service-discovery configuration went out in the wrong order, so the load balancers had no valid backends and returned 503.
- Quote: "our load balancers were unaware of any valid backends to serve Docker Hub registry traffic, and registry requests were met with a 503 error response." — S55
- Relevance: a rollout step that can leave zero routable backends is a failure mode of the deploy, to be tested as one.

### F52. Outage: Docker Hub, 20 October 2025. A single-region dependency: the AWS us-east-1 disruption took Docker Hub fully down; Docker's stated fix is regional redundancy, reads first.
- Quote: "Docker experienced significant disruptions due to a widespread outage in AWS’s US-East-1 region on October 20, 2025." — S56
- Quote: "Enabling regional redundancy for Docker Hub’s read operations, beginning with image pulls." — S56, "Multi-region resilience"
- Quote: "The incident was triggered by a latent defect within the service’s automated DNS management system that caused endpoint resolution failures for DynamoDB." — S59
- Conditions/notes: AWS's summary does not name Amazon ECR among the services it describes; no AWS post-mortem that names ECR was found.
- Relevance: Docker's remediation implies that in 2025 pulls from the largest public registry depended on one region. A pull must be servable by any region holding the bytes, with the metadata it needs replicated there (rule 8).

### F53. Outages: GitHub Container Registry. In January 2023 a backend change made most registry requests fail until it was reverted; in November 2025 an expired internal TLS certificate did.
- Quote: "Most requests to the container registry were failing and some GitHub Pages builds were also impacted. We determined this was caused by a backend change and mitigated by reverting that change." — S57 (incident of 30 January 2023)
- Quote: "The root cause was an expired TLS certificate used for internal service-to-service communication." — S58 (incident of 17 November 2025)
- Relevance: certificate expiry is a wait with a known deadline, so it is alarmed and rotated by the system; a configuration change is rolled out with a check that requests still succeed.

## Tables

### 1. GC designs
| system (version) | online? | mechanism | protection window and its default | known race, as documented |
|---|---|---|---|---|
| GitLab Container Registry (v4.43.0-gitlab) | yes, continuous | per-object review queues written by DB triggers; workers pop with `FOR UPDATE SKIP LOCKED`; API and collector lock the same review row for manifests | `review_after` 1 day for every event, plus up to 60 s jitter; reason: time to finish a push or re-tag (S1) | blob deleted between HEAD and manifest PUT: left open by decision, "image push retry" (S1) |
| CNCF Distribution (v3.1.2) | no, stop-the-world | mark all manifests, sweep unmarked blobs | none: registry must be read-only or stopped | upload during a run: layers deleted, "corrupted image" (S4) |
| Harbor (2.1.0 onward; docs 2.15.0) | yes, scheduled or manual job | DB reference count selects candidates (status `delete`), sweep moves each to `deleting` by a conditional update, HEAD and PUT take candidates back | time window 2 hours (code default; no reason given), plus a cutoff on `update_time` (S5, S6, S7) | residual read/write race the `update_time` check can't close, hence the window (S7) |
| zot (v2.1.23) | yes, inline periodic | per-repository mark and sweep with an age check; fails closed on an incomplete listing | `gcDelay` 1 hour, to be set by client network speed and blob size; interval 1 hour (code), docs inconsistent (S9, S10) | push protocol not transactional between blob and manifest (S9) |
| containerd (v2.4.2) | yes, scheduled by mutation count and pause budget | trace from roots through label references; leases are roots | lease expiry 24 hours by default in the Go client (S12) | a resource neither referenced nor leased is eligible at once (S12) |
| Amazon ECR lifecycle policies | yes, hosted | rule evaluation by priority, expire or archive | none for pushes; expiry "within 24 hours" of meeting a rule (S13) | not documented |
| Data Domain DDFS (FAST'17) | background | mark-and-sweep at the physical level; reference counts rejected | not applicable | reference counts "difficult to maintain correctly" under errors (S14) |
| Windows Azure Storage (SOSP'11) | background | stream manager collects extents no stream references | not applicable | not documented |
| Ceph deduplication | background | reference counts on chunk objects, repaired by `chunk-scrub` | not applicable | false-positive reference mismatches (S16) |

### 2. Cross-region replication
| system | sync or async | conflict rule documented | lag documented |
|---|---|---|---|
| Azure Container Registry geo-replication | async, "eventual consistency" | none: the same tag can resolve to different digests "during the replication window"; the winner is not stated (S19) | no number: "Replication time depends on image size" (S19) |
| Amazon ECR replication | async, one-way per rule, once per push, no chaining | only for immutable tags: the replicated image arrives without the duplicated tag; mutable-tag conflicts not stated (S20) | "majority of images replicate in less than 30 minutes" (S20) |
| Harbor replication | async copy job: manual, cron, or event-based "immediately" | per-rule Override checkbox: replace the destination's resource of the same name, or leave it (S21) | none |
| Google Artifact Registry multi-region | not stated | not stated | not stated (S22) |
| Amazon S3 (within a Region) | strong read-after-write and list consistency since 2020-12-01 | not applicable | none within a Region (S23, S24) |

### 3. Hosted registry limits
| registry | rate limits | max sizes | referrers API |
|---|---|---|---|
| Docker Hub | 100 pulls / 6 h per IPv4 or IPv6 /64 unauthenticated; 200 / 6 h Personal; unlimited Pro, Team, Business; HEAD not counted; abuse limit in the order of thousands of requests per minute per IP (S29, S30) | not found | not found |
| Amazon ECR | per Region per second: PutImage 10, InitiateLayerUpload 100, CompleteLayerUpload 100, UploadLayerPart 500, BatchCheckLayerAvailability 1,000, BatchGetImage 2,000, GetDownloadUrlForLayer 3,000, GetAuthorizationToken 500; all adjustable (S33) | layer 52,000 MiB; layer part 5 to 10 MiB, 4,200 parts; 100,000 images per repository; 1,000 tags per image; manifest size not found (S33) | yes, since 2024-06-27; pull-through cache syncs referrers since 2026-04-17 (S34, S35) |
| GitHub Container Registry | not found | layer 10 GB; upload timeout 10 minutes (S36) | not found in first-party docs |
| Google Artifact Registry | per project per location per minute: 60,000 requests, 18,000 writes, 18,000 deletes (S37) | 9.9 GB per request from a remote repository's upstream; layer and manifest size not found (S37) | not found |
| Azure Container Registry | per registry per minute, Basic and Standard / Premium: DataplaneRead 10,000 / 20,000; DataplaneWrite 2,000 / 4,000; ListReferrers 500 / 2,000; DataplaneDelete 1,000 / 4,000; per identity half of each; token bucket; best effort (S38) | layer 195 GiB; manifest 4 MiB; storage 40 TiB (Basic, Standard), 100 TiB (Premium) (S38) | yes (`ListReferrers` is a rate-limited category) (S38) |
| Quay.io | not found | not found | yes, announced 2024-12-02 (S41) |
| JFrog Artifactory | not found | not found | yes, from 7.90.1 (S42, S43) |

### 4. Scale anchors
| system | number | unit | date | source |
|---|---|---|---|---|
| AWS Lambda | up to 15,000 | new containers per second for a single customer | 2023 | S44 |
| AWS Lambda | hundreds of trillions | invocations processed by the image-loading system; over a million customers | 2023 | S44 |
| AWS Lambda | 512 | KiB per chunk; images up to 10 GiB | 2023 | S44 |
| Firecracker | up to 150 | microVMs created per second per host | 2020 | S47 |
| Firecracker | less than 125 | ms to application code | 2020 | S47 |
| Firecracker | less than 5 | MB memory overhead per microVM | 2020 | S47 |
| Meta XFaaS | trillions; more than 100,000 | function calls per day; servers | 2023 | S45 |
| Meta Twine | one million | machines per region under one control plane | 2020 | S46 |
| Docker Hub | 20 billion (20B) | pulls per month | fetched 2026-11-20 | S50 |
| Docker Hub | over 26 billion; 29 million | image downloads per month; unique IPs pulling monthly | 2025-03 | S48 |
| Docker Hub | 11 billion; 7 million | pulls in July 2020; repositories | 2020-07 | S49 |
| GitLab.com registry | terabytes; petabytes | deleted by online GC per day; total scale | 2023-12 | S2 |
| GitLab.com registry | 64 | days estimated for one offline GC run | 2023-12 | S2 |
| E2B | three billion | sandbox starts, cumulative | fetched 2026-11-20 | S51 |
| E2B | 15 million | sandboxes per month | 2025 | S51 |
| Modal | 3 billion; more than 10,000 | sandboxes run, cumulative; sandbox creations per second (demonstration) | 2026-10 | S52 |
| Modal | over 1 million; 20,000 | sandboxes in a 48-hour event; concurrent at peak | 2025-06 | S53 |
| Amazon ECR, GHCR, Daytona | not found | | | |

## Catalogue of constants
| system | constant | value | where (URL) | justification given? |
|---|---|---|---|---|
| GitLab Container Registry | review delay (`review_after`), blobs and manifests, all events | 1 day | S1, "Review Delay" | yes, as a grace period: clients "have up to one day to complete an image upload"; not measured |
| GitLab Container Registry | jitter added to each review time | up to 60 seconds | S1, "Review Delay" | yes: avoid scheduling tasks at the same instant |
| GitLab Container Registry | look-ahead for the (unimplemented) blob existence lock | 1 hour | S1, "Blobs / Race conditions" | yes: only postpone a review that would run before the manifest upload |
| GitLab Container Registry | postponement by that lock | 1 day | S1, "Blobs / Race conditions" | no |
| GitLab Container Registry | storage delete timeout inside a blob review | 2 seconds (configurable) | S1, "Blobs / Race conditions" | yes: don't hold the transaction and the row lock open |
| Harbor | GC time window protecting recent uploads | 2 hours (`time_window`, 0 allowed for testing) | S5 ; S6 L140-L141 | no |
| Harbor | GC workers | 1 | S6 L158-L159 | no |
| Harbor | minimum spacing of manual runs | once per minute | S5 | yes: avoid triggering too often |
| Harbor | replication copy-by-chunk size | 10MB (`REPLICATION_CHUNK_SIZE`) | S21 | no |
| zot | `gcDelay` | 1 hour | S9 ; S10 `constants.go` L24 | partly: "depending on client network speeds and the size of blobs" |
| zot | `gcInterval` | 1 hour (code and one docs table; another docs table says none) | S9 ; S10 `constants.go` L25 | no |
| zot | maximum random delay before a scheduled GC task | 30 seconds | S10 `gc.go` L97-L99 | no |
| containerd | default lease expiry (`client.WithLease`) | 24 hours | S12 | yes: "in case the process dies before defer" |
| containerd | `pause_threshold` | 0.02 (database unlocked 98% of the time) | S12 | stated as a goal, not measured |
| containerd | `mutation_threshold` | 100 database mutations | S12 | no |
| containerd | `startup_delay` | 100 ms | S12 | yes: run after other start-up work |
| Amazon ECR | lifecycle expiry latency | within 24 hours | S13 | no |
| Amazon ECR | pull-through cache re-validation window | 24 hours (images), 6 hours (referrers) | S32 | no |
| Amazon ECR | replication lag, typical | less than 30 minutes for the majority of images | S20 | observation, no method |
| Amazon ECR | layer size; part size; parts | 52,000 MiB; 5 to 10 MiB; 4,200 | S33 | no |
| Amazon ECR | PutImage; BatchGetImage; GetDownloadUrlForLayer | 10; 2,000; 3,000 per second per Region | S33 | no |
| Amazon ECR | replication rules; destinations; filters per rule | 25; 25; 100 | S20, S33 | no |
| Docker Hub | pull limit window | 21,600 seconds (6 hours) | S29 | no |
| Docker Hub | pulls per window | 100 unauthenticated (per IPv4 or IPv6 /64); 200 Personal | S29 | no |
| Docker Hub | abuse limit | in the order of thousands of requests per minute per IP | S30 | yes: "to protect the application and infrastructure" |
| Azure Container Registry | layer size; manifest size | 195 GiB; 4 MiB | S38 | no |
| Azure Container Registry | DataplaneRead; DataplaneWrite per registry | 10,000 and 2,000 r/m (Basic, Standard); 20,000 and 4,000 r/m (Premium) | S38 | no; "best-effort approximate maximums" |
| GitHub Container Registry | layer size; upload timeout | 10 GB; 10 minutes | S36 | no |
| Google Artifact Registry | requests; writes per project per location | 60,000; 18,000 per minute | S37 | no |
| AWS Lambda | chunk size | 512 KiB | S44 | yes: trade-off between deduplication and metadata size, stated as revisable |
| Firecracker | creation rate; boot; overhead | 150 per second per host; under 125 ms; under 5 MB | S47 | measured in the paper |

## Not verified / open questions
- HLC Corollary 1 (`|l.f − pt.f| ≤ ε`) is stated in F18 from the PDF but is not a checked quote: `pdftotext` drops the ε glyph. The checked quotes say only that the difference "is bounded" and "limited to the clock drift".
- RR-7506 was read from a university copy because the HAL host returned a bot check to `curl`; the copy's page headers carry the HAL identifier and version.
- GitLab: no latency, throughput or backlog figures for online GC were found (only the terabytes-a-day statement of F7). Whether the blob existence lock of F5 was implemented after the spec was written was not checked in code. The spec is pinned at a tag; GitLab's rendered docs page (S3) is not versioned.
- Harbor: no source gives a reason for the 2-hour window. The proposal (S7) and the website docs (S21) are at `main`, not at a commit (the GitHub API rate limit stopped the lookup).
- zot: the docs disagree with themselves on the default `gcInterval` (F10); the code at v2.1.23 defines 1 hour, but how the configuration loader applies it when the key is absent was not traced.
- ECR: token-bucket parameters (burst capacity, refill) for its API limits were not found, nor a maximum image manifest size, nor any first-party statement of ECR's scale. AWS does not document when an expired image's bytes are reclaimed.
- ECR us-east-1: no AWS post-mortem naming ECR was found; S59 is the October 2025 DynamoDB event, cited for Docker Hub's outage (F52).
- GHCR: no first-party statement on the referrers API, on manifest size or on request rates was found; the claim that GHCR does not serve the referrers endpoint rests on secondary sources that were not fetched. No first-party statement of GHCR's scale was found.
- Google Artifact Registry: no statement of its multi-region consistency model, of referrers API support, or of a maximum layer size was found on the pages fetched.
- Quay.io: no documented size or rate limits were found.
- Docker Hub: the current repository count was not found (7 million is from 2020); the two current pull figures (20 billion and 26 billion a month) disagree and are undefined. Peak pull rate is not published.
- GitHub's January 2023 report (S57) says only "a backend change"; a Redis configuration cause appears in search-result summaries of a later report that was not fetched.
- Jepsen-style linearizability checking is quoted only through the Elle paper's description of Knossos and Porcupine.
- Daytona: no first-party volume figure was found. E2B's and Modal's figures are their own, cumulative, and unaudited.
- Section and page labels for the PDFs (S14, S15, S17, S18, S26, S44, S47) were assigned from the extracted text and may be off by a subsection.
- Open design questions these sources do not answer: what protection window a lease-based design needs when a push spans regions (no source measures push durations); how a tag conflict between regions should be reported to the losing writer; and the peak, not average, read rate of a public registry.

Mechanical check, 2026-11-20: 178 of 178 cited quotes (every `"…" — S<n>` pair outside the Sources table) are whitespace-normalised exact substrings of the saved text of the source they cite; each of the 59 sources has at least one checked quote. Quoted section labels and source titles are not quotes of source text and are not counted.
