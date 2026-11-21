# Measurement: change points, instruction counts, load and comparison design: source notes
Researched 2026-11-20. Question: what do the primary sources say about (a) the exact algorithm and parameters of the E-Divisive change-point detector and how MongoDB and Hunter run it on CI performance series, (b) counting instructions in a Linux CI container without privilege, (c) what the Linux load average measures and whether a container sees the host's, and (d) how layout and environment bias, interleaved randomized trials, judging by interval overlap and closed-loop load generation bear on comparing two builds? Method: each source was downloaded with curl (a generic browser User-Agent) into its own new, empty directory `~/.cache/reliquary-research/tmp-measure/pages/<id>/`; PDFs were converted with `pdftotext -layout` and, because `-layout` sets the two columns of a two-column paper side by side on each line, also with pdftotext's reading-order mode; HTML was reduced to text by a standard-library parser run with `python3 -I`. Every `Quote:` line below was checked by script (`~/.cache/reliquary-research/tmp-measure/scripts/check_quotes.py`) to be an exact, whitespace-normalised substring of the saved text of the cited source, and every "PDF p.N" locator to name the page the quote is on; 245 of 245 passed on 2026-11-20 (86 of them matched only the reading-order text of a two-column PDF; 17 added on the same day for findings F38 to F40, from Mantle's design at f41d8344 and pages already saved) with no wrong page, a copy with one altered number and one wrong source id failed the same check on exactly those two lines, and a copy with one wrong page number failed on that line.

Reading rules for this file: a quote is copied character for character from the saved text (Markdown markers, the `*` and `#` of code comments, typos, curly quotes and the extractor's renderings included: superscripts dropped or set on their own line, small capitals split as in "S TABILIZER"); line breaks inside a quote are folded to one space. "PDF p.N" is the page of the PDF file, counted from 1. Equations whose extracted text is garbled are transcribed under Conditions/notes from the typeset page and marked as transcriptions, not quotes. Arithmetic marked "derived" is this note's, from the quoted constants. Where a site failed or refuses automated fetches (valgrind.org answered 503 on 2026-11-20; Taylor & Francis answers 403; one author host no longer resolves), the saved copy is the Internet Archive's unmodified capture (`id_`) and the row says so.

## Sources
| id | type | title | URL | version/date |
|---|---|---|---|---|
| S1 | paper | Matteson & James, "A Nonparametric Approach for Multiple Change Point Analysis of Multivariate Data" (JASA 109(505), 2014, doi:10.1080/01621459.2013.849605), arXiv PDF | https://arxiv.org/pdf/1306.4933 | arXiv v2, 2013-10-15 |
| S2 | docs | R package `ecp` reference manual (CRAN) | https://cran.r-project.org/web/packages/ecp/ecp.pdf | ecp 3.1.6 (2024-08-25), manual built 2026-05-08 |
| S3 | paper | Daly, Brown, Ingo, O'Leary, Bradford, "The Use of Change Point Detection to Identify Software Performance Regressions in a Continuous Integration System" (ICPE '20, doi:10.1145/3358960.3375791), arXiv PDF | https://arxiv.org/pdf/2003.00584 | arXiv v1, 2020-03-01 |
| S4 | paper | Fleming, Kołaczkowski, Kumar, Das, McCarthy, Pattabhiraman, Ingo, "Hunter: Using Change Point Detection to Hunt for Performance Regressions" (ICPE '23, doi:10.1145/3578244.3583719), arXiv PDF | https://arxiv.org/pdf/2301.03034 | arXiv v1, 2023-01-08 |
| S5 | code | mongodb/signal-processing-algorithms, `energy_statistics.py` (MongoDB's E-Divisive library) | https://raw.githubusercontent.com/mongodb/signal-processing-algorithms/aec099a9696d17ccd1e85a357313614a01f859a2/src/signal_processing_algorithms/energy_statistics/energy_statistics.py | aec099a, 2025-03-20 (HEAD) |
| S6 | code | same repository, `determinism.py` | https://raw.githubusercontent.com/mongodb/signal-processing-algorithms/aec099a9696d17ccd1e85a357313614a01f859a2/src/signal_processing_algorithms/determinism.py | aec099a, 2025-03-20 |
| S7 | code | Hunter `hunter/analysis.py` as of the paper (repository now apache/otava) | https://raw.githubusercontent.com/apache/otava/2f5734cdc3c0/hunter/analysis.py | 2f5734c, 2022-10-21 (last change before arXiv v1) |
| S8 | code | Apache Otava (formerly Hunter) `otava/analysis.py` | https://raw.githubusercontent.com/apache/otava/d4f62a1c38dea6053ce31d1f1fba7e53105d2e66/otava/analysis.py | d4f62a1, 2026-11-11 |
| S9 | docs | Apache Otava `docs/MATH.md` ("Change Point Detection") | https://raw.githubusercontent.com/apache/otava/d4f62a1c38dea6053ce31d1f1fba7e53105d2e66/docs/MATH.md | d4f62a1, 2026-11-11 |
| S10 | docs | Valgrind User Manual, ch. 5 "Cachegrind: a high-precision tracing profiler" | https://valgrind.org/docs/manual/cg-manual.html (503 on 2026-11-20; saved copy: https://web.archive.org/web/20261110071516id_/https://valgrind.org/docs/manual/cg-manual.html) | capture 2026-11-10; current release valgrind-3.28.0 |
| S11 | docs | Valgrind User Manual, ch. 2 "Using and understanding the Valgrind core" | https://valgrind.org/docs/manual/manual-core.html (503; saved copy: https://web.archive.org/web/20261106185419id_/https://valgrind.org/docs/manual/manual-core.html) | capture 2026-11-06 |
| S12 | docs | Linux kernel docs, "Perf events and tool security" | https://docs.kernel.org/admin-guide/perf-security.html | docs for 7.4.0-rc3 |
| S13 | docs | Linux kernel docs, "Documentation for /proc/sys/kernel/" (`perf_event_paranoid`) | https://docs.kernel.org/admin-guide/sysctl/kernel.html | docs for 7.4.0-rc3 |
| S14 | docs | perf_event_open(2) man page | https://man7.org/linux/man-pages/man2/perf_event_open.2.html | man-pages 6.19 (2026-05-04) |
| S15 | docs | Docker Docs, "Seccomp security profiles for Docker" (served to curl as Markdown) | https://docs.docker.com/engine/security/seccomp/ | fetched 2026-11-20 |
| S16 | code | moby/profiles `seccomp/default.json` (Docker's default seccomp profile) | https://raw.githubusercontent.com/moby/profiles/9ee3204f0d0cfa19ca2c6a7b0fcf5a2b353954ae/seccomp/default.json | 9ee3204, 2026-11-10 |
| S17 | docs | Kubernetes docs, "Restrict a Container's Syscalls with seccomp" | https://kubernetes.io/docs/tutorials/security/seccomp/ | Kubernetes v1.37 |
| S18 | docs | rust-lang/rustc-perf `collector/README.md` | https://raw.githubusercontent.com/rust-lang/rustc-perf/2d6241ab742ac49e162391cbb16304b7565b08e6/collector/README.md | 2d6241a, 2026-11-17 |
| S19 | docs | rust-lang/rustc-perf `docs/comparison-analysis.md` | https://raw.githubusercontent.com/rust-lang/rustc-perf/2d6241ab742ac49e162391cbb16304b7565b08e6/docs/comparison-analysis.md | 2d6241a, 2026-11-17 |
| S20 | docs | bheisler/iai README | https://raw.githubusercontent.com/bheisler/iai/58e28e940c62898a45d17cb531f9012fb173d236/README.md | 58e28e9, 2021-02-25 (HEAD) |
| S21 | docs | gungraun/gungraun README (formerly iai-callgrind) | https://raw.githubusercontent.com/gungraun/gungraun/2f805d269fd95b9fa8245c48d9e19aea976aed83/README.md | 2f805d2, 2026-11-15; release v0.20.0 (2026-09-26) |
| S22 | code | Linux `kernel/sched/loadavg.c` | https://raw.githubusercontent.com/torvalds/linux/d0f871bfb831705d9f55479f5b2b40f636ea7b16/kernel/sched/loadavg.c | d0f871b, 2026-11-20 |
| S23 | code | Linux `include/linux/sched/loadavg.h` | https://raw.githubusercontent.com/torvalds/linux/d0f871bfb831705d9f55479f5b2b40f636ea7b16/include/linux/sched/loadavg.h | d0f871b, 2026-11-20 |
| S24 | code | Linux `fs/proc/loadavg.c` | https://raw.githubusercontent.com/torvalds/linux/d0f871bfb831705d9f55479f5b2b40f636ea7b16/fs/proc/loadavg.c | d0f871b, 2026-11-20 |
| S25 | docs | getloadavg(3) man page | https://man7.org/linux/man-pages/man3/getloadavg.3.html | man-pages 6.19 |
| S26 | docs | proc_loadavg(5) man page (/proc/loadavg) | https://man7.org/linux/man-pages/man5/proc_loadavg.5.html | man-pages 6.19 |
| S27 | docs | lxc/lxcfs README | https://raw.githubusercontent.com/lxc/lxcfs/e1c5a7db157a6630f3b9b53d6bc799a78e076466/README.md | e1c5a7d, 2026-05-15 |
| S28 | paper | Mytkowicz, Diwan, Hauswirth, Sweeney, "Producing Wrong Data Without Doing Anything Obviously Wrong!" (ASPLOS '09), author's copy on Diwan's university page | http://www-plan.cs.colorado.edu/diwan/asplos09.pdf (host no longer resolves; saved copy: https://web.archive.org/web/20160422052734id_/http://www-plan.cs.colorado.edu/diwan/asplos09.pdf) | March 2009; capture 2016-04-22 |
| S29 | paper | Curtsinger & Berger, "STABILIZER: Statistically Sound Performance Evaluation" (ASPLOS '13), author's copy | https://people.cs.umass.edu/~emery/pubs/stabilizer-asplos13.pdf | March 2013 |
| S30 | paper | Abedi & Brecht, "Conducting Repeatable Experiments in Highly Variable Cloud Computing Environments" (ICPE '17, doi:10.1145/3030207.3030229), SPEC Research Group proceedings copy | https://research.spec.org/icpe_proceedings/2017/proceedings/p287.pdf | April 2017 |
| S31 | paper | Schenker & Gentleman, "On Judging the Significance of Differences by Examining the Overlap Between Confidence Intervals", The American Statistician 55(3):182–186, publisher's abstract page | https://www.tandfonline.com/doi/abs/10.1198/000313001317097960 (403 to curl; saved copy: https://web.archive.org/web/20230510012853id_/https://www.tandfonline.com/doi/abs/10.1198/000313001317097960) | 2001; capture 2023-05-10 |
| S32 | paper | Schroeder, Wierman, Harchol-Balter, "Open Versus Closed: A Cautionary Tale" (NSDI '06), USENIX PDF | https://www.usenix.org/legacy/event/nsdi06/tech/full_papers/schroeder/schroeder.pdf | May 2006 |
| S33 | paper | Payton, Greenstone, Schenker, "Overlapping confidence intervals or standard error intervals: What do they mean in terms of statistical significance?", J Insect Sci 3:34 (PMC524673) | https://pmc.ncbi.nlm.nih.gov/articles/PMC524673/ | 2003-10-30 |
| S34 | code | lxc/lxcfs `src/lxcfs.c` | https://raw.githubusercontent.com/lxc/lxcfs/e1c5a7db157a6630f3b9b53d6bc799a78e076466/src/lxcfs.c | e1c5a7d, 2026-05-15 |
| S35 | docs | Linux kernel docs, "PSI - Pressure Stall Information" | https://docs.kernel.org/accounting/psi.html | docs for 7.4.0-rc3 |
| S36 | docs | Valgrind, "Supported Platforms" | https://valgrind.org/info/platforms.html (503; saved copy: https://web.archive.org/web/20261117135739id_/https://valgrind.org/info/platforms.html) | capture 2026-11-17 |
| S37 | docs | Docker Docs, "Running containers" (capabilities; served as Markdown) | https://docs.docker.com/engine/containers/run/ | fetched 2026-11-20 |
| S38 | docs | Docker Docs, `docker container run` reference (sysctls; served as Markdown) | https://docs.docker.com/reference/cli/docker/container/run/ | fetched 2026-11-20 |
| S39 | code | Linux `include/uapi/linux/personality.h` | https://raw.githubusercontent.com/torvalds/linux/d0f871bfb831705d9f55479f5b2b40f636ea7b16/include/uapi/linux/personality.h | d0f871b, 2026-11-20 |
| S40 | docs | Mantle, `docs/design/measurement.md` ("Measuring a point"), the design `reliquary bench` follows (plan RQ0.7) | `git show f41d8344:docs/design/measurement.md` in github.com/hyper-light/mantle | f41d83449c92, 2026-11-20 (branch perf-10k) |

Supplementary lookups used for versions, dates and repository moves only (no quote taken): GitHub REST API (repositories and commits) for mongodb/signal-processing-algorithms, apache/otava (the `datastax-labs/hunter` repository URL redirects to it; commit "rename Hunter to Otava", 2025-03-14; commit d0fdf97 "Use sample standard deviations for significance and reporting (#194)", 2026-11-10), rust-lang/rustc-perf, bheisler/iai, gungraun/gungraun (`iai-callgrind/iai-callgrind` redirects to it), moby/profiles, lxc/lxcfs and torvalds/linux; the Internet Archive availability API for capture timestamps; arXiv version stamps read from the PDFs' margins; all on 2026-11-20.

## Findings

### Assumptions in the brief that are outdated on 2026-11-20
- The MongoDB paper at arXiv:2003.00584 is titled "The Use of Change Point Detection to Identify Software Performance Regressions in a Continuous Integration System"; the brief's title, "Change Point Detection in Software Performance Testing", is not on the paper, and research note 06 lists the paper under the brief's title. The paper states no permutation count, threshold or seed; those are in MongoDB's library (F10).
- Hunter's repository moved to Apache Otava (`datastax-labs/hunter` redirects to `apache/otava`); its defaults changed after the paper (F14, F15).
- `iai-callgrind` is now Gungraun, which drives Linux perf as well as Valgrind (F20).
- valgrind.org answered 503 throughout 2026-11-20; the Valgrind pages are Internet Archive captures from 2026-11-06 to 2026-11-17.
- Schenker and Gentleman's full text could not be fetched (publisher 403, no author or archive copy found); the abstract (S31) and Payton, Greenstone and Schenker 2003 (S33), which restates their result with numbers, are used instead (F35, F36).
- "The default seccomp profile blocks `perf_event_open`" holds for Docker, unless the container holds CAP_PERFMON or CAP_SYS_ADMIN (F24); a Kubernetes pod runs with no seccomp profile by default (F25).

### A. The E-Divisive detector (Matteson and James)

### F1. E-Divisive scores a split by the energy divergence Ê between the two sides, for an exponent α in (0, 2), scaled by mn/(m+n) into Q̂; the population divergence is zero only when the two distributions are equal.
- Quote: "We do not make any assumptions regarding the nature of the change in distribution or any distribution assumptions beyond the existence of the αth absolute moment, for some α ∈ (0, 2)." — S1, abstract, PDF p.1
- Quote: "E(X, Y ; α) = 0 if and only if X and Y are identically distributed." — S1, §2.1 Lemma 1, PDF p.5
- Quote: "denote the scaled sample measure of divergence discussed above." — S1, §2.2, PDF p.6
- Quote: "$\hat{𝒬}(X_\tau,Y_\tau(\kappa);\alpha)=\dfrac{\tau(\kappa - \tau)}{\kappa}\hat{ℰ}(X_\tau,Y_\tau(\kappa);\alpha),$" — S9, "Original Work"
- Quote: "return np.linalg.norm(series[:, np.newaxis] - series, axis=2)" — S5, `_get_distance_matrix()`
- Quote: "x_avg = x / (n**2) if n > 0 else 0" — S5, `_calculate_stats()`, line 103
- Conditions/notes: transcription of S1 Eq. (4)–(6), PDF p.5–6, whose extracted text is garbled: E(X, Y; α) = 2E|X − Y|^α − E|X − X′|^α − E|Y − Y′|^α; for samples X_n, Y_m, Ê(X_n, Y_m; α) = (2/(mn)) Σ_i Σ_j |X_i − Y_j|^α − C(n,2)^−1 Σ_{i<k} |X_i − X_k|^α − C(m,2)^−1 Σ_{j<k} |Y_j − Y_k|^α, and Q̂ = (mn/(m+n)) Ê. S9 is Otava's restatement with m = τ, n = κ − τ. On a scalar series |·| is the absolute difference. MongoDB's library applies no exponent to the distance (α = 1) and averages the within-sample sums over n² (its `x` is the sum over i < j, passed doubled), a V-statistic, where the paper divides by C(n,2) pairs; the two differ for short segments.
- Relevance: AC-RQ0.7.6, AC-RQ0.7.8: Q̂ is a deterministic function of the series, so the only run-to-run randomness in E-Divisive is its significance test (F4); the decision record must also choose the U- or V-statistic form, since implementations differ.

### F2. The split τ is searched jointly with an end point κ, left free unless at most one change is assumed, so that a mixture of later segments cannot hide a real change.
- Quote: "If it is known that at most one change point exists, we fix κ = T . Otherwise, the variable κ is introduced to alleviate a weakness of bisection" — S1, §2.2, PDF p.7
- Quote: "then it is possible that the resulting mixture distribution in Y τ (T ) is indistinguishable from the distribution of the observations in X τ , even when τ corresponds to a valid change point." — S1, §2.2, PDF p.7
- Quote: "# Y = {Yj; tau <= j < len(signal) }" — S5, `_calculate_t_stats()`
- Conditions/notes: transcription of S1 Eq. (7): (τ̂, κ̂) = argmax over 1 ≤ τ < κ ≤ T of Q̂(X_τ, Y_τ(κ); α), with X_τ = {Z_1, …, Z_τ} and Y_τ(κ) = {Z_τ+1, …, Z_κ}; the paper evaluates it in O(T²) by updating Q̂ from τ − 1 with stored distances. MongoDB's library scans τ only, with the end of Y fixed at the end of the current segment (κ = segment end).
- Relevance: AC-RQ0.7.6: searching κ is what lets the paper's E-Divisive see a regression that is fixed later in the series; an implementation that fixes κ, as MongoDB's does, needs another remedy, such as Hunter's windows (F13).

### F3. Several change points are found by bisection run as a loop: at each step the segment holding the largest Q̂ is split, at a total cost of O(kT²).
- Quote: "The proposed method utilizes a bisection approach for its computational efficiency." — S1, §1, PDF p.3
- Quote: "To estimate multiple change points we iteratively apply the above technique as follows." — S1, §2.3, PDF p.7
- Quote: "Given these clusters, we then apply the procedure for finding a single change point to the observations within each of the k clusters." — S1, §2.3, PDF p.7
- Quote: "This iterative procedure has running time O(kT 2 ), in which k is the unknown number of change points." — S1, §2.3, PDF p.7
- Conditions/notes: transcription of Eq. (8): among the k current segments, the candidate whose Q̂ is largest (i* = argmax) becomes the k-th change point, and its Q̂ is the test statistic q̂_k. "O(kT 2 )" is O(kT²) with the superscript lost.
- Relevance: AC-RQ0.7.6: the algorithm is a work list of segments, not recursion (CLAUDE.md §11), and its cost is bounded by k·T² for a stated history length T.

### F4. Each new change point must pass a permutation test that shuffles observations within each current segment and reruns the search; the approximate p-value is the share of R shuffles whose statistic reaches the observed one, and the paper uses R = 499 and p0 = 0.05.
- Quote: "Under the null hypothesis of no additional change points, we conduct a permutation test as follows." — S1, §2.4, PDF p.8
- Quote: "First, the observations within each cluster are permuted to construct a new sequence of length T . Then, we reapply the estimation procedure as described in Sections 2.2 and 2.3 to the permuted observations." — S1, §2.4, PDF p.8
- Quote: "approximate p-value is defined as #{r : q̂k ≥ q̂k }/(R + 1). In our analysis we fix p0 = 0.05 and use R = 499 permutations for all of our testing." — S1, §2.4, PDF p.8
- Quote: "The kth change point is deemed significant, given {τ̂1 , . . . , τ̂k−1 }, if the approximate p-value is less than p0 , and the procedure then estimates an additional location." — S1, §2.4, PDF p.8
- Quote: "unable to reject the null hypothesis of no additional change points and the algorithm terminates." — S1, §2.4, PDF p.9
- Quote: "Determining a suitably large R to obtain an adequate approximation depends on the distribution of the observations, as well as the number and size of clusters." — S1, §2.4, PDF p.8
- Quote: "As an alternative, a sequential implementation of the random permutations may be implemented with a uniformly bounded resampling risk, see Gandy (2009)." — S1, §2.4, PDF p.8
- Conditions/notes: the extracted formula has lost a superscript; the typeset p-value is #{r : q̂_k^(r) ≥ q̂_k}/(R + 1), where q̂_k^(r) is the statistic after the r-th permutation; the numerator does not count the observed arrangement. The test is conditional on the change points already found and is the algorithm's stopping rule.
- Relevance: AC-RQ0.7.8: the verdict depends on random permutations, so two runs over one history can disagree unless the generator is seeded (F10) or the test is replaced (F12); R and p0 are parameters every record must carry.

### F5. The paper sets a minimum segment length a priori, 30 in most examples and 10 in one, and uses α = 1 throughout, reporting α = 0.5 and 1.5 as similar; it assumes independent observations.
- Quote: "In application, we impose a minimum size for each cluster when estimating the location of a change point. This minimum cluster size should be specified a priori ; in our examples we primarily use 30 as the minimum size, but larger sizes may be needed when E(X, Y ; α) is relatively small." — S1, §3.1, PDF p.11
- Quote: "Throughout this section the E-Divisive procedure was implemented with α = 1; results for α = 0.5, 1.5 were similar, and within the margin of error." — S1, §4, PDF p.16
- Quote: "We used R = 499 iterations when performing the permutation test, which was conducted at the marginal p0 = 0.05 significance level. Furthermore, we set the minimum cluster size for the E-Divisive procedure to 30." — S1, §4, PDF p.16
- Quote: "using α = 1, a minimum cluster size of 10 observations, R = 499 permutations and p0 = 0.05 in our significance testing." — S1, §5.1, PDF p.20
- Quote: "the resampling procedure must also consider the serial dependence structure of the observations." — S1, §2.4, PDF p.9
- Conditions/notes: the last quote ends the paper's remark that the procedure is not suitable for blockwise stationary (serially dependent) observations; the method's model is an independent sequence (§2, PDF p.3).
- Relevance: AC-RQ0.7.6: the minimum segment length is the number of nightly results that must follow a shift before it can be reported, so it fixes part of the alert delay; serially correlated nightly noise (F11) breaks the test's assumption.

### F6. The `ecp` package's `e.divisive` defaults are sig.lvl = 0.05, R = 199, min.size = 30 and alpha = 1, and its R is the maximum for Gandy's sequential p-value.
- Quote: "e.divisive(X, sig.lvl=.05, R=199, k=NULL, min.size=30, alpha=1)" — S2, `e.divisive` usage, PDF p.8
- Quote: "The maximum number of random permutations to use in each iteration of the permutation test. The permutation test p-value is calculated using the method outlined in Gandy (2009)." — S2, `e.divisive` arguments, PDF p.9
- Quote: "Minimum number of observations between change points." — S2, `e.divisive` arguments, PDF p.9
- Quote: "Number of change point locations to estimate, suppressing permutation based testing. If k=NULL then only the statistically significant estimated change points are returned." — S2, `e.divisive` arguments, PDF p.9
- Quote: "Segments are found through the use of a binary bisection method and a permutation test. The computational complexity of this method is O(kT^2), where k is the number of estimated change points, and T is the number of observations." — S2, `e.divisive` details, PDF p.9
- Quote: "y2 = e.divisive(X=x2,sig.lvl=0.05,R=499,k=NULL,min.size=30,alpha=1)" — S2, `e.divisive` examples, PDF p.10
- Conditions/notes: the default R (199) is lower than the paper's 499; the manual's own second example uses 499. The manual does not say whether κ is searched.
- Relevance: AC-RQ0.7.6, AC-RQ0.7.8: the reference implementation's defaults are a published starting point, not a derivation; the record states level, R, minimum size and α, with their sources.

### B. MongoDB's deployment (Daly et al.)

### F7. MongoDB runs E-Divisive (α = 1) on each result series after every set of test runs, replaces the stored change points, and accepts that a change point may move or vanish as data arrives; the newest result is never a change point.
- Quote: "This work describes our recent implementation of a change point detection system built upon the E-Divisive means [15] algorithm." — S3, abstract, PDF p.1
- Quote: "Our continuous integration system runs thousands of benchmarks periodically (most commonly every 2 hours or every 24 hours), which each produce one or more scalar values as a result." — S3, §1, PDF p.1
- Quote: "The α parameter can be any value between 0 and 2. We use 1 for" — S3, §2.1.1, PDF p.3
- Quote: "The algorithm is run at the completion of every set of performance tests." — S3, §3, PDF p.4
- Quote: "We replace any existing change points with the newly generated change points." — S3, §3, PDF p.4
- Quote: "As such, it is expected that on occasion a previously computed change point will “move” to a new commit or even completely go away as more data becomes available." — S3, §3, PDF p.4
- Quote: "Additionally, the change point algorithm will never find that newest result to be a change point. If a regression was introduced, the algorithm will only detect a change point after several more test results are collected. A three to five day delay is common for our tests." — S3, §3, PDF p.4
- Conditions/notes: the α sentence ends with "simplicity" after a displayed equation in the extracted text. A series is one combination of system variant, task, test and test configuration (PDF p.4).
- Relevance: AC-RQ0.7.6, AC-RQ0.7.8: recomputing over the whole history means a verdict about an old commit can change when new points arrive, even under a deterministic test; an alert should name the history (its last commit) it was computed on.

### F8. MongoDB bounds the history at about 500 points, extended back to an already computed change point, and runs with a minimum segment of 3, which puts alerts 5 to 6 days after the commit.
- Quote: "a conscious decision to limit our analysis to results from N recent months, by using up to 500 data points" — S3, §4, PDF p.6
- Quote: "If more than 500 data points are available, we will select data points going back to an already computed change beyond that range. As such, the computation will use more than 500 points." — S3, §4 footnote 9, PDF p.6
- Quote: "our time series typically have 100 to 500 values and 1 to 10 change points" — S3, §4, PDF p.6
- Quote: "The algorithm has a parameter for the minimum cluster size of observations: N. The tests used later in [15] used minimum cluster sizes N=30 and N=10, while we have operated with N=3 to have faster notification. With N=3 we require at least 3 data points after the change (including the change) to detect the change." — S3, §6, PDF p.8
- Quote: "The combined effect of these limitations is that in our primary performance project we typically get alerts of a new regression 5-6 days after it was committed." — S3, §6, PDF p.8
- Conditions/notes: the library's `e_divisive` (S5) has no minimum-size parameter, so the paper's N = 3 is not visible in the code read.
- Relevance: AC-RQ0.7.6: history length and minimum segment are stated parameters with a stated consequence (alert delay); extending the window back to the last change point keeps the segment the test compares against.

### F9. Not every commit is benchmarked, so a change point names a tested commit and the suspect commits are the range back to the previous tested result; the culprit is found by benchmarking the untested commits and rerunning detection.
- Quote: "Note that we do not run the performance tests on every commit to the source repository." — S3, §3.1, PDF p.4
- Quote: "A change point is identified for commits that were built and run. The algorithm cannot determine which commit caused a performance change if the commits have not run. Instead, we can identify the range of commits between the change point and the previous test result as suspect commits." — S3, §3.1, PDF p.4
- Quote: "once a performance change is detected, we need to run the tests on the missing commits to determine the precise commit that introduced the performance change." — S3, §3.2, PDF p.5
- Quote: "When the tests finish, the system will rerun the change point detection algorithm in order to update the change point to the correct commit." — S3, §3.2, PDF p.5
- Quote: "All the change points for a single git hash are grouped together." — S3, §3.1, PDF p.4
- Quote: "Hazard level is computed as the log of the ratio of mean before to the mean after the change point." — S3, §3.1, PDF p.4
- Conditions/notes: the triage page lists git hashes with their change points, sorted or filtered by hazard level; a person acknowledges or hides each.
- Relevance: AC-RQ0.7.6: a nightly detector's alert names the range from the previous nightly's commit (exclusive) to the flagged commit; narrowing it to one commit is a separate, scheduled step.

### F10. The MongoDB paper states no permutation count, threshold or seed; MongoDB's library defaults to 100 permutations and p ≤ 0.05, shuffles with NumPy's global generator, and ships a helper that seeds it so that one input always gives the same change points.
- Quote: "The procedure for termination in the paper involves numerous random permutations of the values in each cluster to determine the statistical significance." — S3, §2.1.2, PDF p.3
- Quote: "pvalue: float = 0.05," — S5, `e_divisive()`, line 228
- Quote: "permutations: int = 100," — S5, `e_divisive()`, line 229
- Quote: "np.random.shuffle(row_indices)" — S5, `_get_next_significant_change_point()`, line 195
- Quote: "probability = better_num / (permutations + 1)" — S5, same function, line 202
- Quote: "return best_candidate[0] if probability <= pvalue else None" — S5, same function, line 203
- Quote: "# E-Divisive's definition requires it to permute change-windows # which leads to non-determinism: we need to always get the # same change-point results when running on the same input." — S6, module comment, lines 3–5
- Quote: "The answer is that a a higher level in MongoDB’s performance CI was designed such that if a point was once flagged as a change point, the system will forever remember it." — S4, §4.2, PDF p.4
- Conditions/notes: S5's `e_divisive` takes no seed; S6 provides context managers that seed Python's and NumPy's global generators and restore them afterwards; whether MongoDB's production used them is not stated. The library accepts at `<=` the threshold where the paper says "less than". The last quote is DataStax's account; Henrik Ingo is an author of both S3 and S4.
- Relevance: AC-RQ0.7.8: a fixed seed makes one implementation repeat itself, but any added point redraws every permutation, so earlier verdicts can still change; MongoDB kept verdicts stable by remembering flagged points outside the statistics.

### F11. MongoDB's measured effect: the threshold system it replaced produced about 100 tickets per useful one, the change-point system about 9 notifications per 4 tracked issues; the detector handles independent noise but not correlated noise, favours the centre of long segments and can merge a quickly reverted change.
- Quote: "For the 5 month period we had 2393 tickets in the first project, and 160 in the second. Of the 160 tickets in the second project, 24 of them were “useful.”" — S3, §5.1, PDF p.7
- Quote: "We went from 100 notifications leading to 1 tracked issue, to 9 notifications leading to 6 worth follow-up and 4 tracked issues." — S3, §5.2, PDF p.7
- Quote: "The algorithm is very good at accounting for IID noise, but not for correlated noise." — S3, §3.2, PDF p.5
- Quote: "Unfortunately, we have some correlated noise in our system – some from time varying behavior on our cloud providers, other due to things as finicky as code alignment issues in compilation and its impact on processor caches." — S3, §3.2, PDF p.5
- Quote: "It also has a bias towards the center of the longest clusters. As such, it cannot tell you anything about the most recent test run, and it will not detect two distinct changes if they are too close together" — S3, §6, PDF p.8
- Quote: "The bias towards the center of clusters is a result of the coefficient mn/(m + n) in Equation 6." — S3, §6, PDF p.8
- Quote: "In most of our tests the worst case run to run variation level is currently less than 10%, but some sensitive tests can fluctuate as much as 20% or more over the course of a small numbers of runs." — S3, §1, PDF p.1
- Conditions/notes: the production figures cover three months of summer 2019 for one project (12,321 change points on 178 git hashes, 126 of them matching a ticket). In the proof of concept a sample of 10 unmatched change points put the false-positive rate between 40% and 80% (PDF p.7).
- Relevance: AC-RQ0.7.6: a person still triages; a code-alignment shift between builds (F30–F32) looks to the detector like a real change, which is a reason to measure each nightly point over several builds (AC-RQ0.7.7).

### C. Hunter and Apache Otava

### F12. Hunter replaced the permutation test with Student's t-test because the permutation results varied from run to run; on MongoDB's public data the permutation version found 0, 3 or 4 change points in one series across 100 runs.
- Quote: "The main change we adopted was to replace the significance test using randomized permutations with a Student’s t-test, as we discovered that the randomized approach did not produce deterministic results, at least not with a reasonable number of iterations." — S4, abstract, PDF p.1
- Quote: "Additionally, because the permutations are random we found that the results of Hunter were non-deterministic and varied from run to run. It is possible to reduce the non-determinism in the results by increasing the number of permutations but this has the negative effect of increasing Hunter’s runtime." — S4, §3.1, PDF p.3
- Quote: "We switched to using Student’s t-test and saw the runtime of Hunter reduce by an order of magnitude as well as providing consistent results when run multiple times on the same data set." — S4, §3.2, PDF p.3
- Quote: "For example for Remove.IntNonIdNoIndex (row 5) it finds 4 change points 43% of the time, 3 points 50% of the time, but 7% of the time it finds zero change points!" — S4, §4.2, PDF p.4
- Quote: "Both the Student’s t-test and weak change points algorithms detected this regression and revert in each of the 30 runs through the data. The permutation based algorithm, only detected these changes for 15 of the 30 runs, or 50% of the time." — S4, §4.1, PDF p.4
- Quote: "Hunter uses a modified E-divisive means algorithm to identify statistically significant changes in normally-distributed performance metrics." — S4, abstract, PDF p.1
- Quote: "While Student’s t-test is not a robust measure of statistical significance for arbitrary data sets, it turned out it works extremely well for our scenario." — S4, §3.2, PDF p.3
- Quote: "But it turned out to not behave very well on small amounts of data, as it requires 30 points to be conclusive. In contrast both the original E-Divisive, and our Student’s t-version, are able to find changes in extremely short time series with only 4-7 points." — S4, §3.2, PDF p.3
- Conditions/notes: "it" in the last quote is the Mann–Whitney U test, tried and rejected. Table 1 (PDF p.5; cells extracted one per line, not quoted): over 30 runs the permutation variant found 16 change points with standard deviation 1.174, the t-test variants 20 and 16 with standard deviation 0. The permutation runs used MongoDB's library with 100 permutations (S7, `compute_change_points_orig`). The paper does not test the normality it assumes.
- Relevance: AC-RQ0.7.8: this is the measured case for a deterministic significance test; its price is a normality assumption that the decision record must either check on nightly latency series or avoid with another deterministic test.

### F13. Hunter runs E-Divisive on overlapping fixed-length windows, so that a regression fixed soon after is not absorbed into one long segment and change points do not vanish as the series grows.
- Quote: "As we began using Hunter on larger and larger data series, we discovered that change points identified in previous runs would suddenly disappear from Hunter’s results. This issue turned out to be caused by performance regressions that were fixed shortly after being introduced." — S4, §3.3, PDF p.3
- Quote: "Our solution to this problem was to split the entire time series into fixed-sized windows and run the E-divisive means algorithm on each window individually." — S4, §3.3, PDF p.3
- Quote: "To address this problem Hunter allows the windows to overlap and care is taken so that a change point isn’t reported multiple times because it exists in multiple windows." — S4, §3.3, PDF p.3
- Quote: "Consecutive windows overlap so that we won't miss changes happening between them." — S7, `split()` docstring, line 203
- Quote: "step = int(window_len / 2)" — S7, `split()`, line 207
- Quote: "start = max(last_new_change_point_index, start + step)" — S7, `split()`
- Conditions/notes: the paper gives no window length; the code at the paper's time calls `split` with `window_len = 50` from `compute_change_points` (F14), so each window of 50 points starts 25 points after the last, or at the last change point found if that is further on.
- Relevance: AC-RQ0.7.6: the window bounds both the cost of a run and how far back a new point can disturb earlier verdicts; its length is a stated parameter of the nightly detector.

### F14. Hunter splits at a looser threshold to keep "weak" change points, then merges bottom-up, removing the weakest point and recomputing its neighbours until every remaining point passes the final threshold; the code's defaults at the paper's time were 0.001 final, ten times that for splitting, and a 5% minimum relative change.
- Quote: "If the algorithm first selects a change point with a p-value above the threshold set by the user (usually 0.05), it will terminate immediately, even if it would have detected change points below the p-value had it continued." — S4, §3.4, PDF p.3
- Quote: "First, we use a larger p-value threshold when splitting so that it allows detection of weak change points. Second, we reevaluate the p-values and merge the split data series in a bottom-up way by removing change points that have a p-value above the smaller, user-specified threshold." — S4, §3.4, PDF p.3
- Quote: "Additionally, we filter out change points that show a small relative change, e.g. change points where the difference in metric value is below 5%." — S4, §3.4, PDF p.4
- Quote: "series: np.array, window_len: int = 50, max_pvalue: float = 0.001, min_magnitude: float = 0.05" — S7, `compute_change_points()`, line 236
- Quote: "change_points = split(series, window_len, max_pvalue * 10)" — S7, `compute_change_points()`, line 238
- Quote: "weakest_cp = max(change_points, key=lambda c: c.stats.pvalue)" — S7, `merge()`
- Quote: "# Recompute the adjacent change point stats:" — S7, `merge()`
- Quote: "first_pass_pvalue = max_pvalue * 10 if max_pvalue < 0.05 else (max_pvalue * 2 if max_pvalue < 0.5 else max_pvalue)" — S8, `compute_change_points()`, line 249
- Conditions/notes: in `merge`, once every p-value is below the threshold, the point with the smallest relative change is removed while that change is at most `min_magnitude`. The paper's "usually 0.05" and the code's 0.001 differ, and the paper does not say which its evaluation used. Otava (S8, 2026-11-11) keeps window 50 and final 0.001, and has `min_magnitude` 0.0.
- Relevance: AC-RQ0.7.6, AC-RQ0.7.8: two thresholds, a window and a magnitude floor make up the parameter set; none of these values is derived in the sources, so each needs its own justification in the decision record.

### F15. Hunter's test compares the two segments either side of a candidate, bounded by the neighbouring change points, with a two-sided equal-variance t-test; at the paper's time it used `np.std` without `ddof`, and Otava now requires at least three points.
- Quote: "Uses two-sided Student's T-test to decide if a candidate change point splits the series into pieces that are significantly different from each other." — S7, `TTestSignificanceTester` docstring
- Quote: "This test is good if the data between the change points have normal distribution. It works well even with tiny numbers of points (<10)." — S7, same docstring, line 87
- Quote: "std_l = np.std(left) if len(left) >= 2 else 0.0" — S7, `compare()`, line 110
- Quote: "mean_l, std_l, len(left), mean_r, std_r, len(right), alternative="two-sided"" — S7, `compare()`, line 115
- Quote: "equal_var=True, alternative="two-sided"," — S8, `TTestSignificanceTester.compare()`, line 94
- Quote: "# We need at least 3 data points to find a statistically significant change point." — S8, same function, line 87
- Conditions/notes: E-Divisive's Q̂ still chooses where to split; only the stopping test changed. Otava's commit of 2026-11-10, "Use sample standard deviations for significance and reporting (#194)", changed the variance estimator (commit message only; the diff was not read).
- Relevance: AC-RQ0.7.8: the verdict is a deterministic function of the series and the parameters; the record should name the variance estimator, because Otava changed it in November 2026.

### F16. Hunter's operators found 14 nightly points too few and default to about 30, report only change points from the last 7 days, and detect changes within 1 to 2 days.
- Quote: "This proved to be far too few data points to get meaningful results from Hunter and we increased it to a month (around 30) by default." — S4, §5.1, PDF p.6
- Quote: "To quieten the output of Hunter’s Slack feature, we capped results to only show change points from the last 7 days." — S4, §5.2, PDF p.7
- Quote: "Additionally, we are able to detect changes sooner, usually within 1-2 days." — S4, §6, PDF p.7
- Quote: "Preferably we like to know the exact commit that caused a regression, not just the general area whereabouts a regression is suspected." — S4, §4.3.8, PDF p.6
- Conditions/notes: 14 and 30 points were judged on DataStax's daily series; no rule for choosing the history length is given.
- Relevance: AC-RQ0.7.6: history length, report window and expected delay are three stated numbers of the nightly detector, here measured on another project's data, not derived.

### D. Instruction counts in CI

### F17. Cachegrind counts the exact number of instructions executed (its `Ir` event), and its manual calls instruction counts highly reproducible where execution time varies by several percent.
- Quote: "Cachegrind is a high-precision tracing profiler. It runs slowly, but collects precise and reproducible profiling data." — S10, §5.1 Overview
- Quote: "Cachegrind measures the exact number of instructions executed by your program, not an approximation." — S10, §5.1
- Quote: "In general, execution time is a better metric than instruction counts because it's what users perceive. However, execution time often has high variability." — S10, §5.1
- Quote: "When running the exact same program on the exact same input multiple times, execution time might vary by several percent. Furthermore, small changes in a program can change its memory layout and have even larger effects on runtime. In contrast, instruction counts are highly reproducible; for some programs they are perfectly reproducible." — S10, §5.1
- Quote: "I cache reads (Ir, which equals the number of instructions executed)" — S10, §5.2.11 Cache and Branch Simulation
- Conditions/notes: "highly reproducible" is the manual's claim; it gives no figure for the variation between identical runs.
- Relevance: AC-RQ0.7.6: `Ir` is the metric for the CI gate; it is a proxy for time, so the gate catches added work, not slower work (memory stalls, contention), which stays with the nightly latency detector.

### F18. Cachegrind's counts differ from hardware counts in a stated way and are perturbed by the size of the executable and its libraries, the length of their file names, and ASLR.
- Quote: "When a REP-prefixed instruction executes each iteration is counted separately. In contrast, hardware counters count each such instruction just once, no matter how many times it iterates." — S10, §5.8.3 Accuracy
- Quote: "Changing the size of the executable being profiled, or the sizes of any of the shared libraries it uses, or even the length of their file names, can perturb the results." — S10, §5.8.3
- Quote: "Many Linux distributions perform address space layout randomisation (ASLR), in which identical runs of the same program have their shared libraries loaded at different locations, as a security measure. This also perturbs the results." — S10, §5.8.3
- Quote: "However, the simulations are basic and unlikely to reflect the behaviour of a modern machine. For this reason they are off by default." — S10, §5.1
- Conditions/notes: the two perturbation sentences follow the list of the cache simulation's shortcomings in the same section; the manual does not say how far they move `Ir` itself.
- Relevance: AC-RQ0.7.6, AC-RQ0.7.7: the gate's tolerance cannot be zero; it should come from each named bench's measured variation over repeated identical runs and builds, and the cache-simulation events should not be gated.

### F19. Valgrind runs the program on a synthetic CPU that simulates every instruction, so Cachegrind reads no hardware counter; the lightest tool already costs about a 4 times slowdown.
- Quote: "Your program is then run on a synthetic CPU provided by the Valgrind core." — S11, §2.1 What Valgrind does with your program
- Quote: "Valgrind simulates every single instruction your program executes." — S11, §2.1
- Quote: "At the other end of the spectrum, the minimal tool, called Nulgrind, adds no instrumentation at all and causes in total "only" about a 4 times slowdown." — S11, §2.1
- Conditions/notes: no source read says in words that Cachegrind needs no privilege; that follows from this mechanism (no `perf_event_open`, so neither F23's sysctl nor F24's seccomp rule applies) and agrees with the iai and Gungraun READMEs (F20). S10 gives Cachegrind's own slowdown only as "It runs slowly".
- Relevance: AC-RQ0.7.6: an instruction-count gate built on Cachegrind does not depend on the runner's perf settings, seccomp profile or capabilities, or on a virtual machine exposing its PMU.

### F20. The iai and Gungraun harnesses run each benchmark once under Valgrind and state that this works in noisy, virtualised CI where wall-clock harnesses do not, with stated limits: system calls and I/O are not measured, the count only correlates with wall time, function order can show as a change, and Windows is not supported.
- Quote: "Iai is an experimental benchmarking harness that uses Cachegrind to perform extremely precise single-shot measurements of Rust code." — S20, introduction
- Quote: "Pro: Iai can work reliably in noisy CI environments or even cloud CI providers like GitHub Actions or Travis-CI, where Criterion-rs cannot." — S20, "Comparison with Criterion-rs"
- Quote: "Mixed: Because Iai can detect such small changes, it may report performance differences from changes to the order of functions in memory and other compiler details." — S20, same section
- Quote: "Con: Iai's measurements merely correlate with wall-clock time (which is usually what you actually care about), where Criterion-rs measures it directly." — S20, same section
- Quote: "Con: Because Cachegrind does not measure system calls, IO time is not accurately measured." — S20, same section
- Quote: "Con: Because Iai runs the benchmark exactly once, it cannot measure variation in the performance such as might be caused by OS thread scheduling or hash-table randomization." — S20, same section
- Quote: "Limitation: Iai can only be used on platforms supported by Valgrind. Notably, this does not include Windows." — S20, same section
- Quote: "Gungraun can take accurate measurements with Valgrind even in virtualized CI environments and make them comparable between different systems completely negating the noise of the environment." — S21, introduction
- Quote: "Gungraun cannot be run on Windows or targets that support neither the Valgrind-based tools nor Linux perf." — S21, "When not to use Gungraun"
- Conditions/notes: these are the tools' own claims; neither README cites a measurement. iai's last commit is from 2021; Gungraun is maintained (v0.20.0, 2026-09-26).
- Relevance: AC-RQ0.7.6: an instruction gate suits benches dominated by computation in user space; benches whose cost is I/O, system calls or scheduling stay with the latency measurements, and the bench list in CI should say which kind each is.

### F21. Valgrind supports Linux on AMD64 (instruction sets up to AVX2) and ARM64, macOS only on x86 up to 13.0, and not Windows.
- Quote: "AMD64/Linux: up to and including AVX2. This is the primary development target and tends to be well supported." — S36, "Current"
- Quote: "ARM64/Linux: supported for ARMv8." — S36, "Current"
- Quote: "X86/Darwin (10.8 to 10.14), AMD64/Darwin (10.8 to 13.0): supported." — S36, "Current"
- Quote: "Windows is not under consideration because porting to it would require so many changes it would almost be a separate project." — S36, "Porting Plans"
- Conditions/notes: the page names valgrind-3.28.0 as the current release. Apple-silicon macOS is not in the list.
- Relevance: AC-RQ0.7.6 and CLAUDE.md §12: the instruction gate runs on Linux amd64 and arm64; on macOS arm64 and Windows the bench tool refuses it with a typed error naming the platform.

### F22. rustc-perf makes instruction counts its default metric because they vary least, tells users who benchmark with `perf` to set `perf_event_paranoid` to −1, and judges a change significant against the history of that benchmark's relative changes.
- Quote: "Instructions is the default because it has the least variation." — S18, "How to benchmark a change using the benchmark server"
- Quote: "There is some non-determinism and natural variation in the measurements." — S18, same paragraph
- Quote: "Benchmarks that are known to have high instructions variance are marked with a '?' in the `compare` page." — S18, same paragraph
- Quote: "For benchmarking using `perf`, you will also need to set `/proc/sys/kernel/perf_event_paranoid` to `-1`." — S18, "Other steps" (setup), line 37
- Quote: "A test result is significant if the relative change percentage is considered an outlier against historical data." — S19, "What makes a test result significant?"
- Quote: "result > Q3 + (interquartile_range * 3)" — S19, same section
- Quote: "whether a value exceeds a threshold equal to the third quartile plus 1.5 times the interquartile range" — S19, same section
- Conditions/notes: the document's prose says 1.5 times the interquartile range and its formula says 3 times; the code that decides was not read.
- Relevance: AC-RQ0.7.6: the CI gate's threshold for each bench can come from that bench's own history of changes rather than a fixed percentage, which meets the house rule against picked constants.

### F23. At the default `perf_event_paranoid` of 2, an unprivileged process may count its own user-space events; kernel events, CPU-wide counting and other users' processes need CAP_PERFMON (or CAP_SYS_ADMIN) or a lower setting.
- Quote: "Controls use of the performance events system by unprivileged users (without CAP_PERFMON). The default value is 2." — S13, `perf_event_paranoid`
- Quote: "Disallow kernel profiling by users without CAP_PERFMON." — S13, same entry (>=2)
- Quote: "Disallow CPU event access by users without CAP_PERFMON." — S13, same entry (>=1)
- Quote: "scope includes per-process performance monitoring only. CPU and system events happened when executing in user space only can be monitored and captured for later analysis." — S12, "Unprivileged users" (>=2)
- Quote: "Unprivileged processes with enabled CAP_PERFMON capability are treated as privileged processes with respect to perf_events performance monitoring and observability operations, thus, bypass scope permissions checks in the kernel." — S12, "perf_events access control"
- Quote: "2 allow only user-space measurements (default since Linux 4.6)." — S14, "perf_event related configuration files"
- Quote: "pid == 0 and cpu == -1 This measures the calling process/thread on any CPU." — S14, arguments
- Quote: "EACCES Returned when the requested event requires CAP_PERFMON (since Linux 5.8) or CAP_SYS_ADMIN permissions (or a more permissive perf_event paranoid setting)." — S14, ERRORS
- Quote: "monitoring all processes on a given CPU (i.e., specifying the pid argument as -1); and not setting exclude_kernel when the paranoid setting requires it." — S14, ERRORS (EACCES)
- Quote: "Retired instructions. Be careful, these can be affected by various issues, most notably hardware interrupt counts." — S14, `PERF_COUNT_HW_INSTRUCTIONS`
- Conditions/notes: at level 2 the counting call must set `exclude_kernel`. The kernel documents only levels −1 to 2; distribution kernels that add stricter levels were not checked. A hardware count also needs a PMU the kernel can see, which a virtual machine may not expose (not checked).
- Relevance: AC-RQ0.7.6: hardware instruction counts of the bench's own process are allowed to an unprivileged user on an upstream-default kernel, but they are noisier than Cachegrind's and depend on the host's sysctl and the container's seccomp profile (F24, F25).

### F24. Docker's default seccomp profile allows `perf_event_open` only to a container holding CAP_PERFMON or CAP_SYS_ADMIN, Docker grants neither by default, and a container cannot change `perf_event_paranoid`; the same profile also refuses the personality flag that turns ASLR off.
- Quote: "In effect, the profile is an allowlist that denies access to system calls by default and then allows specific system calls." — S15, "Pass a profile for a container"
- Quote: "The effect of `SCMP_ACT_ERRNO` is to cause a `Permission Denied` error." — S15, same section
- Quote: "When you run a container, it uses the default profile unless you override it with the `--security-opt` option." — S15, same section
- Quote: "Tracing/profiling syscall, which could leak a lot of information on the host." — S15, "Significant syscalls blocked by the default profile" (`perf_event_open` row)
- Quote: ""names": [ "perf_event_open" ], "action": "SCMP_ACT_ALLOW", "includes": { "caps": [ "CAP_PERFMON" ] }" — S16, last rule of `syscalls`
- Quote: ""defaultAction": "SCMP_ACT_ERRNO", "defaultErrnoRet": 1," — S16, top of file
- Quote: "The next table shows the capabilities which are not granted by default and may be added." — S37, "Runtime privilege and Linux capabilities"
- Quote: "Allow system performance and observability privileged operations using perf_events, i915_perf and other kernel subsystems" — S37, same section (PERFMON row of that table)
- Quote: "Not all sysctls are namespaced. Docker does not support changing sysctls > inside of a container that also modify the host system." — S38, "Configure namespaced kernel parameters (sysctls) at runtime"
- Quote: ""names": [ "personality" ], "action": "SCMP_ACT_ALLOW", "args": [ { "index": 0, "value": 0, "op": "SCMP_CMP_EQ" } ]" — S16, first `personality` rule
- Quote: "ADDR_NO_RANDOMIZE = 0x0040000, /* disable randomization of VA space */" — S39, line 13
- Conditions/notes: S16 also lists `perf_event_open` in the rule that requires CAP_SYS_ADMIN; a refused call fails with error number 1 (`defaultErrnoRet`), which S15 calls `Permission Denied`. S16 has five `personality` rules, allowing the values 0, 8, 131072, 131080 and 4294967295; ADDR_NO_RANDOMIZE is 0x0040000 = 262144 (derived), not among them, so `setarch -R` should fail under the default profile (read from the two files, not run). The sysctl list S38 supports contains only IPC (`kernel.msg*`, `kernel.sem`, `kernel.shm*`, `fs.mqueue.*`) and network (`net.*`) parameters.
- Relevance: AC-RQ0.7.6: in a default Docker CI container `perf stat` fails and ASLR cannot be switched off from inside, so the gate that needs no change to the runner is the Cachegrind one; hardware counting needs the runner to add CAP_PERFMON, which is a request to kiln's runner owners, not something the bench tool can do.

### F25. A Kubernetes pod runs Unconfined, with no seccomp filter, unless the kubelet's `seccompDefault` is on or the pod asks for RuntimeDefault, and the runtimes' default profiles differ.
- Quote: "If you have the seccompDefault configuration enabled, then Pods use the RuntimeDefault seccomp profile whenever no other seccomp profile is specified. Otherwise, the default is Unconfined." — S17, "Create a Pod that uses the container runtime default seccomp profile"
- Quote: "It is possible that the default profiles differ between container runtimes and their release versions, for example when comparing those from CRI-O and containerd." — S17, "Enable the use of RuntimeDefault as the default seccomp profile for all workloads"
- Quote: "Privileged containers always run as Unconfined." — S17, "Before you begin" (note)
- Conditions/notes: an Unconfined pod is still bound by the node's `perf_event_paranoid` and by the capabilities the runtime grants; the containerd and CRI-O default profiles were not read.
- Relevance: AC-RQ0.7.6: whether hardware counting works in a kiln CI leg depends on the executor (docker, Kubernetes, microVM), its profile and the host's sysctl; the bench tool should probe once, record what it found, and use Cachegrind where the probe fails.

### E. The load average

### F26. Linux's load average is an exponentially decaying average, updated every 5 seconds, of runnable plus uninterruptible tasks summed over all CPUs; the 1-, 5- and 15-minute decay factors are exp(−5/60), exp(−5/300) and exp(−5/900) in 11-bit fixed point.
- Quote: "The global load average is an exponentially decaying average of nr_running + * nr_uninterruptible." — S22, header comment, line 18
- Quote: "* for_each_possible_cpu(cpu)" — S22, header comment
- Quote: "* nr_active += cpu_of(cpu)->nr_running + cpu_of(cpu)->nr_uninterruptible;" — S22, header comment, line 25
- Quote: "avenrun[n] = avenrun[0] * exp_n + nr_active * (1 - exp_n)" — S22, header comment, line 27
- Quote: "avenrun[0] = calc_load(avenrun[0], EXP_1, active);" — S22, `calc_global_load()`, line 370
- Quote: "#define LOAD_FREQ (5*HZ+1) /* 5 sec intervals */" — S23, line 20
- Quote: "#define EXP_1 1884 /* 1/exp(5sec/1min) as fixed-point */" — S23, line 21
- Quote: "#define EXP_5 2014 /* 1/exp(5sec/5min) */" — S23, line 22
- Quote: "#define EXP_15 2037 /* 1/exp(5sec/15min) */" — S23, line 23
- Quote: "#define FSHIFT 11 /* nr of bits of precision */" — S23, line 18
- Quote: "newload = load * exp + active * (FIXED_1 - exp);" — S23, `calc_load()`
- Conditions/notes: derived: FIXED_1 = 2048, so 1884/2048 = 0.91992 against exp(−5/60) = 0.92004, 2014/2048 = 0.98340 against 0.98347, and 2037/2048 = 0.99463 against 0.99446. The header comment writes avenrun[0] on the right for every n; the code updates each average from itself ([1] with EXP_5, [2] with EXP_15). Derived from the constants: the 1-minute figure has a time constant of 60 s, so after a load of 8 stops it takes about 60·ln 4 ≈ 83 s to fall below 2, and a load that starts shows 63% of its size after 60 s.
- Relevance: AC-RQ0.7.4: "load average under 2" counts tasks waiting on I/O as well as runnable ones, machine-wide, and lags by about a minute; a rule read only at the start misses load that begins during the run and blames load that ended a minute before, so the tool should sample it through the run and keep the samples in the record.

### F27. The man pages give the same three averages: getloadavg(3) returns up to three samples over 1, 5 and 15 minutes, and /proc/loadavg's first three fields count jobs runnable (R) or in disk wait (D).
- Quote: "The getloadavg() function returns the number of processes in the system run queue averaged over various periods of time." — S25, DESCRIPTION
- Quote: "The system imposes a maximum of 3 samples, representing averages over the last 1, 5, and 15 minutes, respectively." — S25, DESCRIPTION
- Quote: "The first three fields in this file are load average figures giving the number of jobs in the run queue (state R) or waiting for disk I/O (state D) averaged over 1, 5, and 15 minutes." — S26, DESCRIPTION
- Quote: "The fifth field is the PID of the process that was most recently created on the system." — S26, DESCRIPTION
- Conditions/notes: getloadavg(3) speaks of the run queue only; proc_loadavg(5) and the kernel (F26) also count state D. getloadavg(3) lists BSD as its standard; Windows has no counterpart in these sources.
- Relevance: AC-RQ0.7.4: the record should name the figure it used (1-minute average from /proc/loadavg or getloadavg) and say that it includes I/O waits; a Windows host needs its own load source, not researched here.

### F28. /proc/loadavg prints the kernel's global averages, and only its last-PID field depends on the reader's PID namespace, so a container sees the host's load average unless LXCFS's opt-in load-average virtualisation is mounted over it.
- Quote: "get_avenrun(avnrun, FIXED_1/200, 0);" — S24, `loadavg_proc_show()`, line 18
- Quote: "idr_get_cursor(&task_active_pid_ns(current)->idr) - 1);" — S24, same function, line 25
- Quote: "nr_running(), nr_threads," — S24, same function
- Quote: "LXCFS is a small FUSE filesystem written with the intention of making Linux containers feel more like a virtual machine." — S27, Introduction
- Quote: "are container aware such that the values displayed (e.g. in `/proc/uptime`) really reflect how long the container is running and not how long the host is running." — S27, Introduction
- Quote: "-l, --enable-loadavg Enable loadavg virtualization" — S34, usage text, line 762
- Conditions/notes: no kernel document says this in words; it is read from the code (the averages come from the global `avenrun` array of F26). LXCFS's README list of container-aware files does not include /proc/loadavg; the `-l` option turns it on.
- Relevance: AC-RQ0.7.4: in a CI container the load rule measures the host, which is where interfering work runs; the record should say whether its figure was the host's or virtualised.

### F29. Pressure stall information is kept per cgroup: in a container, `cpu.pressure` gives the share of time the container's own tasks waited for CPU, as 10-, 60- and 300-second averages and a running total in microseconds.
- Quote: "In a system with a CONFIG_CGROUPS=y kernel and the cgroup2 filesystem mounted, pressure stall information is also tracked for tasks grouped into cgroups. Each subdirectory in the cgroupfs mountpoint contains cpu.pressure, memory.pressure, and io.pressure files; the format is the same as the /proc/pressure/ files." — S35, "Cgroup2 interface"
- Quote: "The “some” line indicates the share of time in which at least some tasks are stalled on a given resource." — S35, "Pressure interface"
- Quote: "The ratios (in %) are tracked as recent trends over ten, sixty, and three hundred second windows, which gives insight into short term events as well as medium and long term trends. The total absolute stall time (in us) is tracked and exported as well, to allow detection of latency spikes which wouldn’t necessarily make a dent in the time averages, or to average trends over custom time frames." — S35, "Pressure interface"
- Conditions/notes: the kernel document speaks of cgroups, not containers; that a container's tasks form one cgroup whose `cpu.pressure` the container can read is the usual runtime setup, not checked here. LXCFS also virtualises /proc/pressure/* (S27's list). Whether kiln's runners mount cgroup2 with PSI enabled was not checked.
- Relevance: AC-RQ0.7.4: the change in `total` across a run measures contention the benchmark's own tasks suffered over exactly that run, which the host-wide, minute-lagged load average cannot; it is a candidate second signal for "not of record".

### F. Layout and environment bias

### F30. Mytkowicz et al.: the size of the UNIX environment and the link order changed the measured O3 speedup enough to reverse conclusions: link order moved perlbench's between 0.92 and 1.10, environment size moved lbm's between 0.88 and 1.09.
- Quote: "changing a seemingly innocuous aspect of an experimental setup can cause a systems researcher to draw wrong conclusions from an experiment." — S28, abstract, PDF p.1
- Quote: "(i) UNIX environment size (i.e., total number of bytes required to store the environment variables) because it affects the alignment of stack allocated data; and (ii) link order (the order of .o files that we give to the linker) because it affects code and data layout." — S28, §1, PDF p.1
- Quote: "can dramatically (frequently by about 33% and once by almost 300%) change the performance of our program." — S28, §2, PDF p.2
- Quote: "First, depending on link order, the magnitude of our conclusion (height of y-axis) can significantly fluctuate (0.92 vs 1.10)." — S28, §4.1.1, PDF p.5
- Quote: "To generalize these results, the median difference between the minimum and maximum points is 0.02 while on the Pentium 4 the median difference is 0.08." — S28, §4.1.1, PDF p.5
- Quote: "The most extreme is lbm which ranges from 0.88 (i.e., a significant slowdown due to O3 optimizations) to 1.09 (i.e., a healthy O3 speedup)." — S28, §4.2.1, PDF p.6
- Quote: "The median difference between the extreme points on the Core 2 is 0.01 and on the Pentium 4 is 0.04." — S28, §4.2.1, PDF p.6
- Quote: "the best link order on one microprocessor is often not the best link order on another microprocessor" — S28, §1, PDF p.2
- Conditions/notes: the 33% and 300% figures are for a small unoptimised loop on a Core 2 as the environment grows; the median ranges (0.02 and 0.01 on Core 2) are across SPEC CPU2006 C programs. Each plotted point was the mean of five runs with a 95% interval.
- Relevance: AC-RQ0.7.2, AC-RQ0.7.7: a difference between two builds can be a layout accident of the order of percent; one build per side cannot tell the two apart, so builds are a level to be measured.

### F31. Mytkowicz et al. recommend setup randomization, measuring both systems over many generated setups and comparing the distributions; their perlbench example used 484 setups (22 link orders by 22 environment sizes).
- Quote: "In this approach, we generate a large number of experimental setups by varying parameters that we know to cause measurement bias." — S28, §7.1.2, PDF p.9
- Quote: "For this figure we used 484 measurement setups (using a cross product of 22 linking orders and 22 environment variable sizes)." — S28, §7.1.2, PDF p.9
- Quote: "Using this test we found that at the 95% confidence interval the mean speedup ratio was 1.007 ± 0.003; in other words, O3 optimizations speed up the program." — S28, §7.1.2, PDF p.10
- Quote: "if we do not vary the experimental setup adequately then we may still end up with measurement bias for a different bias." — S28, §7.1.2, PDF p.10
- Quote: "Instead, we see that there is a 7% variation in speedup between different experimental setups." — S28, §7.1.1, PDF p.9
- Conditions/notes: three runs were made per setup; the comparison was a t-test on the two distributions. The 7% figure is the spread of the suite-average speedup across 66 setups, showing that a benchmark suite does not average the bias away.
- Relevance: AC-RQ0.7.7: builds (link order, layout) and process starts (environment, ASLR) are levels whose variation the tool measures, and an A|B verdict compares over many of each rather than over many rounds of one build.

### F32. STABILIZER: a binary is one sample of memory layout however many times it runs; re-randomising code, stack and heap during the run (every 500 ms by default) makes execution time approximately normal, and with it -O3 over -O2 was indistinguishable from noise.
- Quote: "A single binary constitutes just one sample from the space of program layouts, regardless of the number of runs." — S29, abstract, PDF p.1
- Quote: "Since compiler optimizations and code changes also alter layout, it is currently impossible to distinguish the impact of an optimization from that of its layout effects." — S29, abstract, PDF p.1
- Quote: "S TABILIZER forces executions to sample the space of memory configurations by repeatedly rerandomizing layouts of code, stack, and heap objects at runtime." — S29, abstract, PDF p.1
- Quote: "we find that simply changing the link order of object files can cause performance to decrease by as much as 57%." — S29, §1, PDF p.1
- Quote: "S TABILIZER re-randomizes functions at regular time intervals (500ms by default)." — S29, §3, PDF p.4
- Quote: "With a sufficient number of randomizations (30 is typical), and no other significant sources of measurement bias, execution time will follow a Gaussian distribution." — S29, §2.2, PDF p.2
- Quote: "We find that, while -O2 has a significant impact relative to -O1, the performance impact of -O3 over -O2 optimizations is indistinguishable from random noise." — S29, abstract, PDF p.1
- Quote: "Unfortunately, even when using current best practices (large numbers of runs and a quiescent system), the conventional approach is unsound." — S29, §1, PDF p.1
- Conditions/notes: STABILIZER is an LLVM compiler pass with a runtime library (§3, PDF p.3); its current state was not checked. Its overhead was a median under 7%. The paper gives 30 randomizations as typical, from the Central Limit Theorem, without deriving the number.
- Relevance: AC-RQ0.7.7: repeating rounds inside one process cannot shrink layout variation; the number of builds (or, for one build, of process starts with different layouts) must come from the variation measured between them.

### G. Interleaved, randomized trials

### F33. Abedi and Brecht's RMIT runs every alternative once per round, in a new random order each round; on EC2 traces, all trials of one alternative followed by all of the other (MCT) reported identical systems as different by up to 37.8% at 95% confidence, and fixed-order interleaving failed when the environment changed periodically.
- Quote: "One trial is conducted using the first alternative, followed by one trial with the second, and so on until each alternative has been run once. When one trial has been conducted using each alternative we say that one round has been completed. Rounds are repeated until the appropriate number of trials has been conducted (Figure 1-C)." — S30, §3, PDF p.2
- Quote: "If the cloud computing environment is affected at regular intervals, and the intervening period coincides with the length of each trial, it is possible that some alternatives are affected more than others. Therefore, the randomized multiple interleaved trials methodology randomly reorders alternatives for each round (Figure 1-D)." — S30, §3, PDF p.2
- Quote: "In essence, a randomized block design [13] is constructed where the blocks are intervals of time (rounds) and within each block all alternatives are tested, with a new random ordering of alternatives being generated for each block." — S30, §3, PDF p.2
- Quote: "In the Multiple Consecutive Trials (MCT) methodology, experimenters often conduct all of the trials for a given alternative, before proceeding to the next." — S30, §4.2, PDF p.3
- Quote: "In the CPU benchmark experiments, the MCT methodology falsely reports a difference of up to 37.8% as being statistically significant (with a confidence level of 0.95) for two identical alternatives. In other words, the MCT methodology determines that a system is 37.8% better than itself!" — S30, §4.2, PDF p.5
- Quote: "Table 3 shows that, except for two disk I/O traces, the ratio of flawed experiments (RFE) is at least 4% when using the MCT methodology, rendering this technique error-prone." — S30, §4.2, PDF p.5
- Quote: "As we saw in Section 4.3, the multiple interleaved trials methodology can not be used if the environment changes occur regularly with a period that overlaps with the length of the trials. Therefore, it is critical to randomize the order of trials to avoid this problem." — S30, §4.4, PDF p.5
- Conditions/notes: the traces are hourly benchmark scores from EC2 over a month (collected by Schad et al. in 2010); the comparisons are simulated by labelling trials of one benchmark A and B, 20 trials per experiment.
- Relevance: AC-RQ0.7.2: the A|B mode runs both builds once per round in a fresh random order, so that drift and periodic interference fall on both sides alike.

### F34. Abedi and Brecht judge a methodology by an A|A comparison, two labels for the same system that must not show non-overlapping 95% intervals; they report that RMIT passes over the whole 30-day traces, with the caveat that large variation hides small differences.
- Quote: "Since in each case A and B are identical benchmarks (two different labels for each alternative), a sound measurement methodology should show no statistical performance differences between them (i.e., no nonoverlapping confidence intervals)." — S30, §4.2, PDF p.4
- Quote: "When comparing the performance of alternatives A and B, we consider them statistically different if the confidence intervals do not overlap." — S30, §4.2, PDF p.4
- Quote: "A comparisons between Figure (c) and (b) shows that the RMIT technique produces no non-overlapping confidence interval and correctly shows that there is no statistical difference between alternatives A and B." — S30, §4.4, PDF p.6
- Quote: "we show than RMIT can be used to obtain repeatable and fair comparisons between multiple alternatives for their entire 30 day traces." — S30, §1, PDF p.1
- Quote: "We note that if the variations are large, it may not be possible to distinguish relatively small differences between two or more alternatives." — S30, §4.4, PDF p.6
- Quote: "The confidence intervals obtained using this technique may provide a false sense of rigor and validity when comparing different alternatives." — S30, §4.2, PDF p.4
- Conditions/notes: their test is the interval-overlap rule of F35 and F36, which is conservative, so "no non-overlapping interval" is a weak pass criterion; the paper does not report RMIT's rate of overlap failures per trace as a number.
- Relevance: AC-RQ0.7.3: the A|A run is the methodology's own test, as in T-RQ0.7.7; the tool reports the A|A rate of "difference" verdicts against the stated level, by a binomial test.

### H. Interval overlap as a test

### F35. Schenker and Gentleman: declaring a difference only when two confidence intervals do not overlap rejects less often than the standard test when there is no difference and misses real differences more often; they advise against it for formal testing.
- Quote: "We compare this technique to the standard method of testing significance under the common assumptions of consistency, asymptotic normality, and asymptotic independence of the estimates." — S31, abstract
- Quote: "Rejection of the null hypothesis by the method of examining overlap implies rejection by the standard method, whereas failure to reject by the method of examining overlap does not imply failure to reject by the standard method." — S31, abstract
- Quote: "As a consequence, the method of examining overlap is more conservative (i.e., rejects the null hypothesis less often) than the standard method when the null hypothesis is true, and it mistakenly fails to reject the null hypothesis more frequently than does the standard method when the null hypothesis is false." — S31, abstract
- Quote: "we conclude that it should not be used for formal significance testing unless the data analyst is aware of its deficiencies and unless the information needed to carry out a more appropriate procedure is unavailable." — S31, abstract
- Conditions/notes: only the abstract was read; the paper's numbers come through S33 (F36) and this note's derivation.
- Relevance: AC-RQ0.7.2, AC-RQ0.7.3: a verdict only on separated intervals is valid (it never claims a difference the standard test would not) but conservative; its actual A|A level is below the intervals' nominal 5%, and its power is lower.

### F36. With equal standard errors, two 95% intervals overlap more than 99% of the time when there is no difference; intervals of about 83 to 84% each make non-overlap a 5% test.
- Quote: "In other words, the 95% confidence intervals will overlap over 99% of the time." — S33, "Confidence intervals and corresponding adjustments for testing hypotheses"
- Quote: "Schenker and Gentleman (2001) showed that for general estimation problems, the interval overlap method tends to be the most conservative when the (true) standard errors are equal." — S33, same section
- Quote: "For example, assuming equal standard errors (k = 1) yields γ = 0.166. In other words, if you wish to use confidence intervals to test equality of two parameters when the standard errors are approximately equal, you would want to use approximately 83% or 84% confidence intervals." — S33, same section
- Quote: "The sizes of the individual confidence intervals necessary to perform a 0.05 test grow as the standard errors become less homogeneous, as illustrated in Table 3." — S33, same section
- Quote: "For instance, in the case where the sample size was 10, the proportion of the 10,000 iterations in which the two intervals constructed by the 95% confidence intervals overlapped was 0.995. The proportion of the 10,000 trials in which the two 84% confidence intervals overlapped for the n = 10 case was 0.949." — S33, same section (simulation)
- Quote: "This probability is equal to 0.843. Thus, examining overlap between standard error intervals to test hypotheses regarding equality of means would be akin to performing a test with a type I error rate of about 15% or 16%." — S33, same section
- Conditions/notes: derived, for normal estimates with standard-error ratio k: non-overlap of two 95% intervals has level 2(1 − Φ(1.96(1 + k)/√(1 + k²))), which is 0.0056 at k = 1, 0.0085 at k = 2 and 0.032 at k = 10, approaching 0.05 as k grows; at k = 1 the per-interval level for a 5% test is 83.4%. S33's general large-sample expression (its Eq. 7, which it attributes to Schenker and Gentleman) and its tables are images in the saved page and are not quoted.
- Relevance: AC-RQ0.7.2, AC-RQ0.7.3: if the A|B rule is "95% intervals separated", the A|A false-difference rate is far below 5% (about 0.6% for normal, equal-error medians) and real differences are often "not separated"; to make 5% the actual level, set each side's interval level from the measured error ratio (about 83% when equal), or report a direct interval for the difference beside both intervals. Either way the A|A test of AC-RQ0.7.3 measures the actual level.

### I. Open versus closed load generation

### F37. Schroeder, Wierman and Harchol-Balter: at the same load a closed-loop generator, which sends a new request only after a completion, measures far lower mean response times than an open-loop one, by an order of magnitude or more at high load, because a closed system can never queue more than its fixed number of clients.
- Quote: "In a closed system model, new job arrivals are only triggered by job completions (followed by think time), as in Figure 1(a). By contrast in an open system model, new jobs arrive independently of job completions, as in Figure 1(b)." — S32, §1, PDF p.1
- Quote: "We find, for example, that for a fixed load, the mean response time for an open system model can exceed that for a closed system model by an order of magnitude or more." — S32, §1, PDF p.2
- Quote: "Principle (i): For a given load, mean response times are significantly lower in closed systems than in open systems." — S32, §5.1, PDF p.8
- Quote: "For fixed high loads, the response time under the closed system is orders of magnitude lower than those for the open system." — S32, §5.1, PDF p.9
- Quote: "Intuitively, this difference in mean response time between open and closed systems is a consequence of the fixed MPL, N , in closed systems, which limits the queue length seen in closed systems to N even under very high load. By contrast, no such limit exists for an open system." — S32, §5.1, PDF p.9
- Quote: "Principle (ii): As the MPL grows, closed systems become open, but convergence is slow for practical purposes." — S32, §5.1, PDF p.9
- Quote: "When the service demand has high variability (C 2 ), a closed system with an MPL of 1000 still has much lower response times then the corresponding open system. Even when the job service demands are lightly variable, an MPL of 500 is required for the closed system to achieve response times comparable to the corresponding open system." — S32, §5.1, PDF p.9
- Quote: "Thus the “choice” of a system model (closed versus open) is often not really a researcher’s choice, but rather is dictated by the availability of the workload generator." — S32, §1, PDF p.1
- Conditions/notes: MPL is the multiprogramming level, the closed system's fixed number of clients; "(C 2 )" is the squared coefficient of variation C² with the superscript lost. The results come from web, e-commerce and auction case studies and from simulation. Research note 06 F48 (wrk2's coordinated omission) is the same effect seen from the load generator's side.
- Relevance: AC-RQ0.7.5 and T-RQ0.7.5: latency is measured from each operation's intended start under an open-loop schedule; a closed-loop control run is expected to show lower latency and hide a stall, and the test should show both.

### J. The design followed, and the environment's range

### F38. Mantle's measurement design fixes the point's rounds at ten to 28 within ±5%, states each state's median with its order-statistic interval and no mean, sizes a round by a count of operations from the quantile, the depth in flight and the operations' spread, discards a first call, and measures latency from each request's intended start.
- Quote: "a point runs at least ten rounds and at most 28, and stops before 28 once its rounds are independent and every state's interval lies within ±5% of its median." — S40, §5
- Quote: "is half of SNIA's 10% steady-state excursion (SNIA PTS 2.0.2 §2.1.24)" — S40, §5
- Quote: "Below ten the runs test has no two-sided 5% rejection region (21 §4), so fewer rounds are reported as untested." — S40, §2
- Quote: "each state is summarized by its median, with the order-statistic interval of Le Boudec's Theorem 2.1, and by its share of the rounds, with the interval of his Theorem 2.4." — S40, §4
- Quote: "a round is a count of transfers, not a length of time." — S40, §5a
- Quote: "runs `max(⌈fewest(q) / 10⌉, ⌈N / (2 × 5%)⌉)` transfers" — S40, §5a
- Quote: "enough that the round's last transfers, which run with fewer than `N` in flight, cost it at most the precision" — S40, §5a
- Quote: "raises the count to `(1.96·cv / 5%)²` where that is more (Jain's sample size for a mean within ±r%; 11 §20.5)" — S40, §5a
- Quote: "one call discarded first (Georges et al., §4.1)" — S40, §5a
- Quote: "No mean or coefficient of variation is reported (21 §9.6)." — S40, §6
- Quote: "open loop with latency from each request's intended start everywhere else" — S40, §10
- Quote: "Each driver reports its own CPU time and, in open loop, its lateness, intended against actual submit time" — S40, §10
- Conditions/notes: an internal design of another hyper-light product, not peer-reviewed; its own sources are Le Boudec's textbook, NIST's e-Handbook, Hartigan and Hartigan, SNIA PTS, Jain and Georges et al. (Mantle's research/21 and /11, cited there). Mantle's code at this revision (`crates/disk/src/rounds.rs`) still holds 30 rounds as the most, where §5 derives 28.
- Relevance: AC-RQ0.7.1 (§2 to §6), AC-RQ0.7.5 (§10), AC-RQ0.7.7 (§5a's round size): the rows of `docs/design/constants.md` for `PRECISION`, `FEWEST_TESTED`, `COVERAGE`, `Z_95`, `DRAIN` and the report's shape rest here.

### F39. Mytkowicz et al. varied the UNIX environment's size up to 4,096 bytes to show it moves the stack's alignment and with it a program's speed.
- Quote: "We varied the UNIX environment size (up to 4096 bytes) and calculated the O3 speedups for each environment size." — S28, §4.2.2, PDF p.10
- Quote: "changing a seemingly innocuous aspect of an experimental setup can cause a systems researcher to draw wrong conclusions from an experiment." — S28, abstract, PDF p.1
- Conditions/notes: the range is the one their causal analysis swept, with the stack's start fixed and then free; Figure 3's points step by 63 bytes from the null environment.
- Relevance: AC-RQ0.7.7: each process start draws its environment's size from 0 to 4,095 bytes added, the range over which the source found the alignment, and the speed, to vary; so the process starts sample that variation rather than one setup's accident.

### F40. Cachegrind's output file names its events on an `events:` line and gives their totals, in that order, on a `summary:` line; the file is named by `--cachegrind-out-file`.
- Quote: "events_line ::= "events:" ws? (event ws)+" — S10, §5.9.2 Cachegrind Output File Format
- Quote: "summary_line ::= "summary:" ws? count (ws+ count)+ ws*" — S10, §5.9.2
- Quote: "but its name can be changed with the --cachegrind-out-file option" — S10, §5.2
- Conditions/notes: the grammar has the summary give one count an event; that the counts follow the `events:` line's order is how cg_annotate reads them, and the grammar's `(ws+ count)+` admits a single event only loosely, as `--cache-sim=no` writes one.
- Relevance: AC-RQ0.7.6: the gate reads `Ir` from the place the `events:` line gives it rather than the first count, so a cache simulation left on cannot make it gate another event.

## Tables

### 1. The detector's parameters in each source
| parameter | Matteson & James (S1) | `ecp` `e.divisive` 3.1.6 (S2) | MongoDB paper (S3) and library (S5, S6) | Hunter code at the paper (S7) | Otava, 2026-11-11 (S8) |
|---|---|---|---|---|---|
| distance exponent α | (0, 2) allowed; 1 in all examples | `alpha=1` | 1 (no exponent applied) | MongoDB's calculator | 1 (S9) |
| within-segment average | over C(n,2) pairs (U-statistic) | not stated | over n² (V-statistic) | MongoDB's C calculator, not read | rewritten in 2025, not read |
| split search | τ and κ | bisection; κ not stated | τ only, end of segment fixed | per window of 50, step 25 | same |
| stopping test | permutation within segments | permutation, Gandy's sequential method | permutation, NumPy global generator | two-sided Student's t on the two neighbouring segments | same; sample standard deviations since 2026-11-10 |
| permutations | R = 499 | at most 199 | 100 | none (100 in the permutation variant) | none (100 in the permutation variant) |
| level | p < 0.05 | 0.05 | p ≤ 0.05 | split 0.01, final 0.001 | final 0.001; split 10× below 0.05, 2× below 0.5 |
| minimum segment | 30 (10 in one study) | 30 | 3 in production; none in the library | t-test needs 3 points in all | 3 points |
| history | whole series | whole series | about 500 points, back to a known change | whole series, in windows | same |
| relative-change floor | none | none | none (hazard level for ranking) | 5% | 0 |
| same verdict every run | no | no | only with a fixed seed (S6) | yes | yes |

### 2. Counting instructions in a Linux CI container
| method | counts | needs | stopped by default in | stated repeatability | platforms |
|---|---|---|---|---|---|
| Cachegrind `Ir` (S10, S11) | every instruction the synthetic CPU runs, user space; REP iterations counted singly | Valgrind in the image; about 4× slowdown or more | nothing found in these sources | "highly reproducible" (S10); perturbed by sizes, file-name lengths, ASLR (S10) | Linux amd64 (to AVX2) and arm64; macOS x86 to 13.0; no Windows (S36) |
| `perf_event_open`, own process, user space (S12–S14) | retired instructions from the PMU, kernel excluded | the syscall allowed; `perf_event_paranoid` ≤ 2; a PMU visible to the kernel | Docker's default seccomp profile without CAP_PERFMON or CAP_SYS_ADMIN (S15, S16, S37) | affected by interrupts (S14) | Linux |
| `perf`, kernel and user (rustc-perf, S18) | the same, kernel included | `perf_event_paranoid` −1, or CAP_PERFMON | the same, and the upstream default of 2 (S13) | high-variance benches flagged with '?' (S18) | Linux |

### 3. Catalogue of constants
| system | constant | value | where | justification given? |
|---|---|---|---|---|
| E-Divisive paper | significance level; permutations | p0 = 0.05; R = 499 | S1 PDF p.8 | no; R "depends on the distribution of the observations" |
| E-Divisive paper | minimum segment | 30 (10 for the genetics data) | S1 PDF p.11, p.20 | partly: larger when the divergence is small |
| E-Divisive paper | α | 1 | S1 PDF p.16 | 0.5 and 1.5 gave similar results |
| `ecp` | sig.lvl, R, min.size, alpha | 0.05, 199, 30, 1 | S2 PDF p.8 | no |
| MongoDB library | pvalue, permutations | 0.05, 100 | S5 lines 228–229 | no |
| MongoDB deployment | minimum segment; history | 3; about 500 points | S3 PDF p.8, p.6 | yes: faster notification; cost and interest in recent commits |
| Hunter, paper-time code | window, step, final p, split p, minimum change | 50, 25, 0.001, 0.01, 5% | S7 lines 207, 236–238 | no; the paper calls 5% changes not actionable |
| Otava | final p, split rule, minimum change | 0.001, 10× or 2×, 0 | S8 lines 220, 249 | no |
| Hunter operations | history; report window | about 30 points; 7 days | S4 PDF p.6, p.7 | partly: 14 points proved too few |
| Linux load average | sampling period; decay constants; precision | 5 s (5·HZ + 1 ticks); 1884, 2014, 2037; 11 bits | S23 lines 18–23 | yes: exp(−5 s / 1, 5, 15 min) in fixed point |
| PSI | averaging windows | 10, 60, 300 s | S35 | no |
| STABILIZER | re-randomization period; randomizations | 500 ms; 30 "typical" | S29 PDF p.4, p.2 | the 30 rests on the Central Limit Theorem, not derived |
| Setup randomization (Mytkowicz) | setups; runs per setup | 22 × 22 = 484; 3 | S28 PDF p.9 | no |
| RMIT evaluation | trials per experiment | 20 (5 and 10 also tried) | S30 PDF p.4 | no |
| Interval overlap | per-interval level for a 5% test, equal errors | 83 to 84% | S33 | yes: derived |
| Docker default seccomp | syscalls disabled | about 44 of 300+ | S15 | no |

## What could not be verified
- Schenker and Gentleman's full text: the publisher answers 403, and no author or archive copy of the PDF was found. Their abstract (S31) was read from an Internet Archive capture of the publisher's page; their numbers are taken from Payton, Greenstone and Schenker (S33), whose equations and tables are images, and from this note's own derivation (F36).
- No source read states in words that Cachegrind runs without privilege or inside Docker's default seccomp profile; F19 rests on the mechanism and on the iai and Gungraun READMEs, which cite no measurement. Valgrind was not run in a container here.
- How much `Ir` varies between identical runs, under ASLR or after a change in executable size or file names, was not measured; the Cachegrind manual gives no figure (F17, F18).
- Whether Valgrind's synthetic CPU reports CPU features (beyond AVX2 on AMD64) that make a program take a different code path than it takes natively was not checked; S36 says only "up to and including AVX2".
- Distribution kernels that add `perf_event_paranoid` levels above 2, and whether the virtual machines behind kiln's runners or hosted CI expose a PMU, were not checked (F23).
- The containerd and CRI-O default seccomp profiles were not read (F25); kiln's executors' actual profiles, capabilities and cgroup setup were not inspected (F24, F29).
- That `setarch -R` fails under Docker's default profile is read from S16 and S39, not run (F24).
- MongoDB's production seed handling and its minimum-size implementation are not in the paper or in the library code read (F8, F10); the code of Otava's 2025 rewrite (`change_point_divisive/`) and of MongoDB's C calculator was not read, so their within-segment averaging is unknown (Table 1).
- rustc-perf's comparison document gives both 1.5 and 3 times the interquartile range; the deciding code was not read (F22).
- The load-average figures for macOS and Windows hosts were not researched (F27); /proc/loadavg's behaviour in a container is read from kernel code, not from a kernel document (F28).
- valgrind.org answered 503 for the whole session; the Valgrind pages are Internet Archive captures and may differ from the live site.
