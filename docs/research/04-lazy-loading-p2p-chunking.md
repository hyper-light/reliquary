# Lazy loading, chunking, P2P and deltas: source notes
Researched 2026-11-20. Question: what do primary sources say about registry workloads, lazy-loading image formats (incl. block-level microVM images), content-defined chunking and dedup, P2P image distribution, and binary delta updates, and which numbers and constants should drive a new registry's design? Method: each source was downloaded with `curl` (PDFs through `pdftotext`, HTML through a tag stripper, GitHub files raw at a tag or commit) and every quote below was checked mechanically as a whitespace-normalised substring of the downloaded text (one exception, the Apple NBD sentence in F61, was checked by hand against JSON-escaped text); quotes from PDFs avoid passages where `pdftotext` joined a line-end hyphen or dropped a ligature.

Reading conventions:
- A quote is one contiguous passage; line breaks in the source are shown as single spaces. Markup characters (backticks, asterisks) are kept where the source file has them.
- "Derived" marks a number computed here from quoted numbers; it is not in the source.
- "Vendor" numbers are the project's or company's own claims, with no independent reproduction found.
- Kernel version facts were read from `Documentation/filesystems/erofs.rst` at the tags named.

## Sources
| id | type | title | URL | version/date |
|---|---|---|---|---|
| S1 | paper | Anwar et al., "Improving Docker Registry Design Based on Production Workload Analysis", USENIX FAST'18 | https://www.usenix.org/conference/fast18/presentation/anwar (PDF: https://www.usenix.org/system/files/conference/fast18/fast18-anwar.pdf) | Feb 2018 |
| S2 | paper | Zhao et al., "Large-Scale Analysis of the Docker Hub Dataset", IEEE CLUSTER'19 (author copy, NSF PAR) | https://par.nsf.gov/servlets/purl/10167826 | Sep 2019; DOI 10.1109/CLUSTER.2019.8891000 |
| S3 | paper | Zhao et al., "Large-Scale Analysis of Docker Images and Performance Implications for Container Storage Systems", IEEE TPDS 32(4) (accepted manuscript, NSF PAR) | https://par.nsf.gov/servlets/purl/10201736 | Apr 2021 issue; DOI 10.1109/TPDS.2020.3034517 |
| S4 | paper | Zhao et al., "DupHunter: Flexible High-Performance Deduplication for Docker Registries", USENIX ATC'20 | https://www.usenix.org/conference/atc20/presentation/zhao (PDF: https://www.usenix.org/system/files/atc20-zhao.pdf) | Jul 2020 |
| S5 | paper | Zhao et al., "Slimmer: Weight Loss Secrets for Docker Registries", IEEE CLOUD'19 (author copy) | https://par.nsf.gov/servlets/purl/10099559 | Jul 2019 |
| S6 | paper | Littley et al., "Bolt: Towards a Scalable Docker Registry via Hyperconvergence", IEEE CLOUD'19 (author copy) | https://par.nsf.gov/servlets/purl/10099562 | Jul 2019 |
| S7 | paper | Zhao et al., "An End-to-end High-performance Deduplication Scheme for Docker Registries and Docker Container Storage Systems", ACM TOS 20(3) (author copy) | https://par.nsf.gov/servlets/purl/10554549 | Jun 2024; DOI 10.1145/3643819 |
| S8 | paper | Sun et al., "SimEnc: A High-Performance Similarity-Preserving Encryption Approach for Deduplication of Encrypted Docker Images", USENIX ATC'24 | https://www.usenix.org/conference/atc24/presentation/sun | Jul 2024 |
| S9 | paper | Fan et al., "Gear: Enable Efficient Container Storage and Deployment with a New Image Format", IEEE ICDCS'21 (abstract text via OpenAlex) | https://api.openalex.org/works/doi:10.1109/ICDCS51616.2021.00020 | 2021; DOI 10.1109/ICDCS51616.2021.00020 |
| S10 | paper | Harter et al., "Slacker: Fast Distribution with Lazy Docker Containers", USENIX FAST'16 | https://www.usenix.org/conference/fast16/technical-sessions/presentation/harter (PDF: https://www.usenix.org/system/files/conference/fast16/fast16-papers-harter.pdf) | Feb 2016 |
| S11 | docs | eStargz specification (stargz-snapshotter) | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/docs/estargz.md | v0.19.1, 2026-10-27 |
| S12 | code | stargz-snapshotter `estargz/estargz.go` (writer chunk size) | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/estargz/estargz.go#L680-L716 | v0.19.1 |
| S13 | code | stargz-snapshotter `fs/config/config.go` (fetch chunk size, prefetch) | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/fs/config/config.go#L104-L124 | v0.19.1 |
| S14 | docs | stargz-snapshotter README and `docs/overview.md` | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/README.md ; https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/docs/overview.md | v0.19.1 |
| S15 | docs | stargz-snapshotter `docs/passthrough.md` (FUSE passthrough) | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/docs/passthrough.md | v0.19.1 |
| S16 | docs | SOCI snapshotter README | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/README.md | v0.16.1, 2026-09-30 |
| S17 | docs | SOCI glossary | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/glossary.md | v0.16.1 |
| S18 | docs | SOCI Index Manifest v2 | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/soci-index-manifest-v2.md | v0.16.1 |
| S19 | docs | SOCI CLI usage and getting started | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/cli-usage.md ; https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/getting-started.md | v0.16.1 |
| S20 | docs | SOCI considerations | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/considerations.md | v0.16.1 |
| S21 | docs | SOCI parallel pull and unpack mode; snapshotter config | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/parallel-mode.md ; https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/config.md | v0.16.1 |
| S22 | code | SOCI `ztoc/ztoc.go` | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/ztoc/ztoc.go#L40-L62 | v0.16.1 |
| S23 | vendor | AWS News Blog, "AWS Fargate Enables Faster Container Startup using Seekable OCI" | https://aws.amazon.com/blogs/aws/aws-fargate-enables-faster-container-startup-using-seekable-oci/ | 2023-07-17 |
| S24 | vendor | AWS Containers Blog, "Under the hood: Lazy Loading Container Images with Seekable OCI and AWS Fargate" | https://aws.amazon.com/blogs/containers/under-the-hood-lazy-loading-container-images-with-seekable-oci-and-aws-fargate/ | 2023-07-18 |
| S25 | vendor | AWS Containers Blog, "Improving Amazon ECS deployment consistency with SOCI Index Manifest v2" | https://aws.amazon.com/blogs/containers/improving-amazon-ecs-deployment-consistency-with-soci-index-manifest-v2/ | 2025-07-03 |
| S26 | vendor | AWS Containers Blog, "Introducing Seekable OCI Parallel Pull mode for Amazon EKS" | https://aws.amazon.com/blogs/containers/introducing-seekable-oci-parallel-pull-mode-for-amazon-eks/ | 2025-08-27 |
| S27 | preprint | Thompson et al., "Seekable OCI: Lazy-Loading Container Images via Range-Request Indexing" | https://arxiv.org/abs/2607.06868 | v1, 2026-07-07 |
| S28 | docs | AWS Lambda Developer Guide, "Lazy-load container images with Seekable OCI (SOCI)" (Lambda Managed Instances) | https://docs.aws.amazon.com/lambda/latest/dg/lambda-managed-instances-seekable-oci.html | fetched 2026-11-20 |
| S29 | docs | containers/storage, `containers-storage-zstd-chunked.md` | https://github.com/containers/storage/blob/v1.59.1/docs/containers-storage-zstd-chunked.md | v1.59.1, 2025-08-04 (page dated "August 2024") |
| S30 | code | containers/storage chunked compressor (`compressor.go`, `rollsum.go`, `internal/minimal/compression.go`) | https://github.com/containers/storage/blob/v1.59.1/pkg/chunked/compressor/compressor.go#L20-L23 ; https://github.com/containers/storage/blob/v1.59.1/pkg/chunked/compressor/rollsum.go#L28-L36 ; https://github.com/containers/storage/blob/v1.59.1/pkg/chunked/internal/minimal/compression.go#L132-L139 | v1.59.1 |
| S31 | docs | containers/storage, `containers-storage-composefs.md` | https://github.com/containers/storage/blob/v1.59.1/docs/containers-storage-composefs.md | v1.59.1 |
| S32 | docs | Fedora Project Wiki, "Changes/zstd:chunked" | https://fedoraproject.org/wiki/Changes/zstd:chunked | fetched 2026-11-20 |
| S33 | docs | Nydus v2 design (`docs/nydus-design.md`) | https://github.com/dragonflyoss/nydus/blob/v2.4.6/docs/nydus-design.md | v2.4.6, 2026-11-20 |
| S34 | docs | Nydus v2 README | https://github.com/dragonflyoss/nydus/blob/v2.4.6/README.md | v2.4.6 |
| S35 | docs | Nydus v3 README (release candidate) | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/README.md | v3.0.0-rc.0, 2026-11-20 |
| S36 | docs | Nydus v3 design (`docs/nydus.md`) | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/nydus.md | v3.0.0-rc.0 |
| S37 | docs | Nydus v3 UFFD service (`docs/uffd.md`) | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/uffd.md | v3.0.0-rc.0 |
| S38 | docs | Nydus v3 NBD and ublk services (`docs/nbd.md`, `docs/ublk.md`) | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/nbd.md ; https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/ublk.md | v3.0.0-rc.0 |
| S39 | docs | Nydus v3 fanotify pre-content service (`docs/fanotify.md`) | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/fanotify.md | v3.0.0-rc.0 |
| S40 | paper | Li et al., "DADI: Block-Level Image Service for Agile and Elastic Application Deployment", USENIX ATC'20 | https://www.usenix.org/conference/atc20/presentation/li-huiba (PDF: https://www.usenix.org/system/files/atc20-li-huiba.pdf) | Jul 2020 |
| S41 | docs | overlaybd README | https://github.com/containerd/overlaybd/blob/v1.0.18/README.md | v1.0.18, 2026-07-10 |
| S42 | code | overlaybd ZFile format spec and `compressor.h` | https://github.com/containerd/overlaybd/blob/v1.0.18/src/overlaybd/zfile/format_spec.md ; https://github.com/containerd/overlaybd/blob/v1.0.18/src/overlaybd/zfile/compressor.h#L38 | v1.0.18 |
| S43 | docs | accelerated-container-image `PERFORMANCE.md` | https://github.com/containerd/accelerated-container-image/blob/v1.4.5/docs/PERFORMANCE.md | v1.4.5, 2026-09-23 |
| S44 | docs | accelerated-container-image `TURBO_OCI.md` | https://github.com/containerd/accelerated-container-image/blob/v1.4.5/docs/TURBO_OCI.md | v1.4.5 |
| S45 | docs | Linux kernel documentation, "EROFS - Enhanced Read-Only File System" | https://docs.kernel.org/filesystems/erofs.html | latest, fetched 2026-11-20 |
| S46 | code | `Documentation/filesystems/erofs.rst` at kernel tags | https://github.com/torvalds/linux/blob/v6.19/Documentation/filesystems/erofs.rst ; https://github.com/torvalds/linux/blob/v7.0/Documentation/filesystems/erofs.rst ; https://github.com/torvalds/linux/blob/v7.1/Documentation/filesystems/erofs.rst ; https://github.com/torvalds/linux/blob/v7.2/Documentation/filesystems/erofs.rst | v6.19, v7.0, v7.1, v7.2 |
| S47 | docs | EROFS project docs, "Features and Comparison" | https://erofs.docs.kernel.org/en/latest/features.html | "as of Linux 6.17", fetched 2026-11-20 |
| S48 | docs | EROFS project docs, "Build & Mount", "Technical Design", FAQ | https://erofs.docs.kernel.org/en/latest/mkfs.html ; https://erofs.docs.kernel.org/en/latest/design.html ; https://erofs.docs.kernel.org/en/latest/faq.html | fetched 2026-11-20 |
| S49 | code | erofs-utils `man/mkfs.erofs.1` and `ChangeLog` (GitHub mirror) | https://github.com/erofs/erofs-utils/blob/v1.9.4/man/mkfs.erofs.1 ; https://github.com/erofs/erofs-utils/blob/v1.9.4/ChangeLog | v1.9.4, 2026-08-19 |
| S50 | docs | composefs README | https://github.com/composefs/composefs/blob/v1.0.8/README.md | v1.0.8, 2025-01-03 |
| S51 | docs | Linux kernel documentation, "fs-verity: read-only file-based authenticity protection" | https://docs.kernel.org/filesystems/fsverity.html | latest, fetched 2026-11-20 |
| S52 | docs | containerd EROFS snapshotter | https://github.com/containerd/containerd/blob/v2.4.2/docs/snapshotters/erofs.md | v2.4.2, 2026-10-13 |
| S53 | docs | containerd v2.1.0 release notes | https://github.com/containerd/containerd/releases/tag/v2.1.0 | 2025-05-07 |
| S54 | paper | Brooker et al., "On-demand Container Loading in AWS Lambda", USENIX ATC'23 | https://www.usenix.org/conference/atc23/presentation/brooker (PDF: https://www.usenix.org/system/files/atc23-brooker.pdf) | Jul 2023 |
| S55 | paper | Chen et al., "Starlight: Fast Container Provisioning on the Edge and over the WAN", USENIX NSDI'22 | https://www.usenix.org/conference/nsdi22/presentation/chen-jun-lin (PDF: https://www.usenix.org/system/files/nsdi22-paper-chen_jun_lin.pdf) | Apr 2022 |
| S56 | paper | Wang et al., "FaaSNet: Scalable and Fast Provisioning of Custom Serverless Container Runtimes at Alibaba Cloud Function Compute", USENIX ATC'21 | https://www.usenix.org/conference/atc21/presentation/wang-ao (PDF: https://www.usenix.org/system/files/atc21-wang-ao.pdf) | Jul 2021 |
| S57 | docs | PuzzleFS README and `doc/chunking.md` | https://github.com/project-machine/puzzlefs/blob/248f74f22d845b0cfc84629b6f2ded1fc8ba552d/README.md ; https://github.com/project-machine/puzzlefs/blob/248f74f22d845b0cfc84629b6f2ded1fc8ba552d/doc/chunking.md | commit 248f74f, 2025-07-07 |
| S58 | docs | Kata Containers, "How to Use EROFS Snapshotter with Kata Containers (runtime-rs)" and "Kata Containers with Guest Image Management" | https://github.com/kata-containers/kata-containers/blob/4.4.0/docs/how-to/how-to-use-erofs-snapshotter-with-kata.md ; https://github.com/kata-containers/kata-containers/blob/4.4.0/docs/how-to/how-to-pull-images-in-guest-with-kata.md | 4.4.0, 2026-11-17 |
| S59 | docs | gVisor user guide, "Filesystem" | https://gvisor.dev/docs/user_guide/filesystem/ | fetched 2026-11-20 |
| S60 | docs | Firecracker, "Vhost-user block device" | https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/docs/api_requests/block-vhost-user.md | v1.18.0, 2026-11-09 |
| S61 | docs | Firecracker, "Using the Firecracker virtio-pmem device"; CHANGELOG | https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/docs/pmem.md ; https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/CHANGELOG.md | v1.18.0 |
| S62 | docs | Firecracker, "Handling snapshot memory loading" (userfaultfd) and "Block device IO engine" | https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/docs/snapshotting/handling-page-faults-on-snapshot-resume.md ; https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/docs/api_requests/block-io-engine.md | v1.18.0 |
| S63 | docs | firecracker-containerd `docs/remote-snapshotter.md` and `docs/snapshotter.md` | https://github.com/firecracker-microvm/firecracker-containerd/blob/6052d721aea61a8cad3c347b0c07504ac95d869c/docs/remote-snapshotter.md ; https://github.com/firecracker-microvm/firecracker-containerd/blob/6052d721aea61a8cad3c347b0c07504ac95d869c/docs/snapshotter.md | commit 6052d72, 2026-11-10 |
| S64 | blog | Fly.io, "Docker without Docker" | https://fly.io/blog/docker-without-docker/ | 2021-04-08 |
| S65 | blog | Fly.io, "The Design & Implementation of Sprites" | https://fly.io/blog/design-and-implementation/ | 2026-01-14 |
| S66 | blog | CodeSandbox, "How we clone a running VM in 2 seconds" (Wayback Machine capture; the live site refused automated fetches) | https://web.archive.org/web/20250118021818/https://codesandbox.io/blog/how-we-clone-a-running-vm-in-2-seconds | 2022-09-01 |
| S67 | blog | Modal, "Fast, lazy container loading in Modal.com" | https://modal.com/blog/jono-containers-talk | 2024-09-08 |
| S68 | blog | BuildBuddy, "Snapshot, Chunk, Clone: Fast Runners at Scale" | https://www.buildbuddy.io/blog/fast-runners-at-scale | 2026-01-26 |
| S69 | docs | E2B infra `docs/ARCHITECTURE.md` | https://raw.githubusercontent.com/e2b-dev/infra/main/docs/ARCHITECTURE.md | branch `main`, fetched 2026-11-20 |
| S70 | paper | Vangoor et al., "To FUSE or Not to FUSE: Performance of User-Space File Systems", USENIX FAST'17 | https://www.usenix.org/conference/fast17/technical-sessions/presentation/vangoor | Feb 2017 |
| S71 | docs | Apple Developer Documentation, `VZNetworkBlockDeviceStorageDeviceAttachment` and Hypervisor framework (text read from the pages' JSON data endpoints under https://developer.apple.com/tutorials/data/documentation/) | https://developer.apple.com/documentation/virtualization/vznetworkblockdevicestoragedeviceattachment ; https://developer.apple.com/documentation/hypervisor | macOS 14.0+ (NBD attachment); fetched 2026-11-20 |
| S72 | docs | Microsoft Learn, "Windows Hypervisor Platform API Definitions" | https://learn.microsoft.com/en-us/virtualization/api/hypervisor-platform/hypervisor-platform | fetched 2026-11-20 |
| S73 | docs | Linux kernel documentation, "Userspace block device driver (ublk driver)" | https://docs.kernel.org/block/ublk.html | latest, fetched 2026-11-20 |
| S74 | paper | Xia et al., "FastCDC: A Fast and Efficient Content-Defined Chunking Approach for Data Deduplication", USENIX ATC'16 | https://www.usenix.org/conference/atc16/technical-sessions/presentation/xia (PDF: https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf) | Jun 2016 |
| S75 | paper | Xia et al., "The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems", IEEE TPDS 31(9) (author-hosted copy) | https://ranger.uta.edu/~jiang/publication/Journals/2020/2020-IEEE-TPDS(Wen%20Xia).pdf | Sep 2020; DOI 10.1109/TPDS.2020.2984632 |
| S76 | paper | Udayashankar et al., "VectorCDC: Accelerating Data Deduplication with Vector Instructions", USENIX FAST'25 | https://www.usenix.org/conference/fast25/presentation/udayashankar (PDF: https://www.usenix.org/system/files/fast25-udayashankar.pdf) | Feb 2025 |
| S77 | preprint | Gregoriadis et al., "A Thorough Investigation of Content-Defined Chunking Algorithms for Data Deduplication" | https://arxiv.org/abs/2409.06066 (PDF: https://arxiv.org/pdf/2409.06066) | v3, 2024-09-28 |
| S78 | preprint | Gregoriadis et al., "Benchmarking Content-Defined Chunking Algorithms for Data Deduplication" | https://arxiv.org/abs/2610.13704 | v1, 2026-10-09 |
| S79 | paper | "RapidCDC", ACM SoCC'19; Udayashankar et al., "SeqCDC", ACM Middleware'24; Zhang et al., "AE", IEEE INFOCOM'15 (abstract texts via Semantic Scholar and OpenAlex APIs; publisher pages refused automated fetches) | https://api.semanticscholar.org/graph/v1/paper/DOI:10.1145/3357223.3362731?fields=title,abstract ; https://api.openalex.org/works/doi:10.1145/3652892.3700766 ; https://api.openalex.org/works/doi:10.1109/INFOCOM.2015.7218510 | 2019; 2024; 2015 |
| S80 | paper | Meyer and Bolosky, "A Study of Practical Deduplication", USENIX FAST'11 | https://www.usenix.org/conference/fast11/study-practical-deduplication (PDF: https://www.usenix.org/legacy/event/fast11/tech/full_papers/Meyer.pdf) | Feb 2011 |
| S81 | paper | El-Shimi et al., "Primary Data Deduplication - Large Scale Study and System Design", USENIX ATC'12 | https://www.usenix.org/conference/atc12/technical-sessions/presentation/el-shimi (PDF: https://www.usenix.org/system/files/conference/atc12/atc12-final293.pdf) | Jun 2012 |
| S82 | paper | Jin and Miller, "The Effectiveness of Deduplication on Virtual Machine Disk Images", SYSTOR'09 (authors' conference slides; the paper PDF was not reachable) | https://www.systor.org/2009/papers/2_1_2.pdf | May 2009; DOI 10.1145/1534530.1534540 |
| S83 | docs | BLAKE3 README and benchmark chart source | https://github.com/BLAKE3-team/BLAKE3/blob/1.8.7/README.md ; https://github.com/BLAKE3-team/BLAKE3-specs/blob/ac784c9f22a48327782f042ec2f4d8126b3b1744/benchmarks/bar_chart.py | 1.8.7, 2026-08-20 |
| S84 | paper | O'Connor et al., "BLAKE3: one function, fast everywhere" (specification paper) | https://github.com/BLAKE3-team/BLAKE3-specs/blob/ac784c9f22a48327782f042ec2f4d8126b3b1744/blake3.pdf | commit ac784c9 (paper first published 2020) |
| S85 | code | OpenSSL SHA-256/512 assembly, performance tables in file headers | https://github.com/openssl/openssl/blob/openssl-3.5.0/crypto/sha/asm/sha512-x86_64.pl#L84-L108 ; https://github.com/openssl/openssl/blob/openssl-3.5.0/crypto/sha/asm/sha512-armv8.pl#L19-L41 | openssl-3.5.0 |
| S86 | docs | Bao specification and README (BLAKE3 verified streaming) | https://github.com/oconnor663/bao/blob/3466bb34287f10f746d29d899d92429b81cf4302/docs/spec.md ; https://github.com/oconnor663/bao/blob/3466bb34287f10f746d29d899d92429b81cf4302/README.md | commit 3466bb3, 2025-05-07 |
| S87 | paper | Harnik, Pinkas, Shulman-Peleg, "Side channels in cloud services, the case of deduplication in cloud storage" (author copy; IEEE Security & Privacy 8(6), 2010) | https://www.pinkas.net/PAPERS/hps.pdf | 2010 |
| S88 | paper | Bellare, Keelveedhi, Ristenpart, "Message-Locked Encryption and Secure Deduplication" (IACR ePrint 2012/631); "DupLESS: Server-Aided Encryption for Deduplicated Storage", USENIX Security'13 | https://eprint.iacr.org/2012/631 ; https://www.usenix.org/conference/usenixsecurity13/technical-sessions/presentation/bellare | 2012; Aug 2013 |
| S89 | blog | Wilcox-O'Hearn, "Drew Perttula and Attacks on Convergent Encryption" (Tahoe-LAFS project page, Wayback Machine capture; the live site did not answer) | https://web.archive.org/web/20240122020521/https://tahoe-lafs.org/hacktahoelafs/drew_perttula.html | 2008-03-20 |
| S90 | standard | OCI Distribution Specification | https://github.com/opencontainers/distribution-spec/blob/v1.1.1/spec.md | v1.1.1, 2025-01-29 |
| S91 | docs | CNCF Distribution, HTTP API V2 spec (`docs/content/spec/api.md`) | https://github.com/distribution/distribution/blob/v3.0.0/docs/content/spec/api.md | v3.0.0 |
| S92 | docs | Steamworks Documentation, "Uploading to Steam" (SteamPipe) | https://partner.steamgames.com/doc/sdk/uploading | fetched 2026-11-20 |
| S93 | docs | Uber Kraken README | https://github.com/uber/kraken/blob/v0.1.37/README.md | v0.1.37, 2026-11-16 |
| S94 | code | Kraken piece hash, piece length config, blacklist config, piece verification | https://github.com/uber/kraken/blob/v0.1.37/core/piece_hash.go ; https://github.com/uber/kraken/blob/v0.1.37/config/origin/base.yaml#L24-L26 ; https://github.com/uber/kraken/blob/v0.1.37/lib/torrent/scheduler/connstate/config.go ; https://github.com/uber/kraken/blob/v0.1.37/lib/torrent/storage/agentstorage/torrent.go#L192-L193 | v0.1.37 |
| S95 | blog | Uber Engineering, "Introducing Kraken, an Open Source Peer-to-Peer Docker Registry" | https://www.uber.com/blog/introducing-kraken/ | 2019-03-05 |
| S96 | code | Dragonfly Rust client: piece length, minimum piece length, piece digest | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client/src/resource/piece.rs#L295-L319 ; https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-config/src/lib.rs#L35-L36 ; https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client-storage/src/lib.rs#L1035 | v1.5.14, 2026-11-13 |
| S97 | docs | Dragonfly docs, "Architecture" | https://d7y.io/docs/operations/deployment/architecture/ | page updated 2026-06-25 |
| S98 | news | CNCF, "Cloud Native Computing Foundation Announces Dragonfly's Graduation" | https://www.cncf.io/announcements/2026/01/14/cloud-native-computing-foundation-announces-dragonflys-graduation/ | 2026-01-14 |
| S99 | docs | Spegel docs (overview, getting started, FAQ, benchmark), README and `go.mod` | https://spegel.dev/docs/overview/ ; https://spegel.dev/docs/getting-started/ ; https://spegel.dev/docs/faq/ ; https://spegel.dev/docs/benchmark/ ; https://github.com/spegel-org/spegel/blob/v0.8.1/README.md ; https://github.com/spegel-org/spegel/blob/v0.8.1/go.mod | v0.8.1, 2026-10-28 |
| S100 | standard | BEP 52, "The BitTorrent Protocol Specification v2" | https://www.bittorrent.org/beps/bep_0052.html | fetched 2026-11-20 |
| S101 | paper | Wang et al., "FID: A Faster Image Distribution System for Docker Platform", IEEE FAS*W 2017; Nathan et al., "CoMICon", IEEE IC2E 2017; Du et al., "Cider", IEEE HPCC 2017 (abstract texts via OpenAlex; IEEE Xplore refused automated fetches) | https://api.openalex.org/works/doi:10.1109/FAS-W.2017.147 ; https://api.openalex.org/works/doi:10.1109/ic2e.2017.24 ; https://api.openalex.org/works/doi:10.1109/HPCC-SmartCity-DSS.2017.44 | 2017 |
| S102 | paper | Zheng et al., "Wharf: Sharing Docker Images in a Distributed File System", ACM SoCC'18 (abstract text via Semantic Scholar API) | https://api.semanticscholar.org/graph/v1/paper/DOI:10.1145/3267809.3267836?fields=title,abstract | 2018; DOI 10.1145/3267809.3267836 |
| S103 | docs | zsync home page and technical paper | http://zsync.moria.org.uk/ ; http://zsync.moria.org.uk/paper/ | undated, fetched 2026-11-20 |
| S104 | blog | Poettering, "casync - A tool for distributing file system images"; casync `src/cachunker.h` | https://0pointer.net/blog/casync-a-tool-for-distributing-file-system-images.html ; https://github.com/systemd/casync/blob/v2/src/cachunker.h#L8-L12 | 2017-06-20; v2, 2017-07-26 |
| S105 | docs | desync README and `docs/concepts.md` | https://github.com/folbricht/desync/blob/v1.1.5/README.md ; https://github.com/folbricht/desync/blob/v1.1.5/docs/concepts.md | v1.1.5, 2026-10-17 |
| S106 | docs | Percival, bsdiff home page and "Naive differences of executable code" | https://www.daemonology.net/bsdiff/ ; https://www.daemonology.net/papers/bsdiff.pdf | 2003 |
| S107 | docs | Zstandard v1.4.5 release notes; wiki "Zstandard as a patching engine" | https://github.com/facebook/zstd/releases/tag/v1.4.5 ; https://github.com/facebook/zstd/wiki/Zstandard-as-a-patching-engine | May 2020 release; wiki fetched 2026-11-20 |
| S108 | docs | Chromium design document, "Software Updates: Courgette" | https://www.chromium.org/developers/design-documents/software-updates-courgette/ | undated (describes a 2009 update), fetched 2026-11-20 |
| S109 | docs | Chromium `components/zucchini/README.md` | https://chromium.googlesource.com/chromium/src/+/main/components/zucchini/README.md | branch `main`, fetched 2026-11-20 |
| S110 | docs | OSTree `docs/formats.md` (static deltas) | https://github.com/ostreedev/ostree/blob/v2026.7/docs/formats.md | v2026.7, 2026-11-13 |
| S111 | docs | nix-casync README and introduction post; Attic server config template | https://github.com/flokli/nix-casync/blob/v0.5/README.md ; https://flokli.de/posts/2021-12-10-nix-casync-intro/ ; https://github.com/zhaofengli/attic/blob/7a19204df10d606c5070e6bb72615c3461900c05/server/src/config-template.toml#L85-L105 | v0.5, 2022-02-21; 2021-12-10; commit 7a19204, 2026-07-06 |
| S112 | blog | Larsson, "Putting container updates on a diet"; tar-diff README | https://blogs.gnome.org/alexl/2020/05/13/putting-container-updates-on-a-diet/ ; https://github.com/containers/tar-diff/blob/v0.6.2/README.md | 2020-05-13; v0.6.2, 2026-11-14 |
| S113 | docs | balena docs, "Delta updates" | https://docs.balena.io/learn/deploy/delta/ | fetched 2026-11-20 |

## Findings

### A. Registry workload studies

### F1. The only public production registry trace covers 75 days at five IBM data centres (38.22 million requests in total), and both the traces and a replayer are published.
- Quote: "we perform a comprehensive characterization of a large-scale registry workload based on traces that we collected over the course of 75 days from five IBM data centers hosting production-level registries" — S1, abstract
- Quote: "Both the traces and the replayer are open-sourced to facilitate further research." — S1, abstract
- Conditions/notes: Table 1 totals (seven registries: dal, lon, fra, syd, stg, prs, dev): 38.22 million requests, 10.54 TB data ingress, 170.76 TB data egress, 1,289 thousand images pushed, 10,280 thousand images pulled; values read from the `pdftotext` column dump of Table 1 (§3.2). The paper gives the download location as https://dssl.cs.vt.edu/drtp/ (§1, §3); that URL was not fetched here. Traces are from 2017, IBM Cloud only.
- Relevance: copy the replayer-driven evaluation; a new registry's cache and prefetch claims should be replayed against these traces, as Bolt and DupHunter were.

### F2. The workload is read-dominated: 90%–95% of image operations at the mature production registries are pulls.
- Quote: "For dal, lon, and fra, we observe that 90%–95% of requests are pulls (i.e. reads)." — S1, §4.1 "Request type distribution"
- Counter: a newer registry and non-production registries write more — "Syd exhibits a lower pull ratio of 78% because it is a newer installation" — S1, §4.1
- Conditions/notes: non-production registries (stg, prs, dev) show 68–82% pulls (same section).
- Relevance: optimise the read path first; size the write path for the 5–32% push share rather than assuming a fixed ratio.

### F3. By HTTP method, GET is over 60% of requests at every registry and HEAD is 10%–22%; more than half of pull requests fetch layers.
- Quote: "All registries receive more than 60% of GET requests and 10%–22% of HEAD requests." — S1, §4.1
- Quote: "For pulls we observe that, except for syd and fra, 50% or more requests retrieve layers rather than manifests." — S1, §4.1
- Conditions/notes: "PUT requests are 1.9–5.8× more common than PATCH requests" (same paragraph), because manifests and small layers go up in one request.
- Relevance: HEAD (existence checks) is a first-class hot path; single-request (monolithic) uploads dominate chunked uploads.

### F4. Most manifest GETs are not followed by any layer GET, because clients re-check images they already hold.
- Quote: "In most cases, ranging from 96% for dev to 73% for fra, GET manifest requests are not followed by any subsequent request." — S1, §4.4 "Request Correlation"
- Conditions/notes: session threshold 1 minute. The correlation becomes strong only for manifests fetched in the session of a PUT to the same repository (Figure 12(b)); the paper uses that to prefetch freshly pushed layers.
- Relevance: manifest fetch must be nearly free (cache, conditional requests); layer prefetch should key on "recently pushed", not on every manifest GET.

### F5. Requested layers are small: about 65% are under 1 MB and about 80% under 10 MB; a typical manifest is about 1 KB.
- Quote: "about 65% of the layers are smaller than 1 MB and around 80% are smaller than 10 MB" — S1, §4.1 "Request size distribution" (Figure 6(a))
- Conditions/notes: distribution over GET requests, not over stored bytes. Manifests: "the typical manifest size is around 1 KB for all AZs except for lon" (same paragraph).
- Relevance: lazy loading and chunk indexes pay off only for the minority of large layers (compare F25: SOCI skips layers under 10 MiB); small blobs want an inline, one-round-trip path.

### F6. Access is heavily skewed: the top 10 repositories take 25% of requests and the top 10 layers 12%, and 95% of registry time goes to fetching from the backend object store.
- Quote: "we observe that 25% of the total requests are for top 10 repositories and 12% of the requests are for top 10 layers" — S1, §1
- Quote: "95% of the time is spent by the registry in fetching the image content from the backend object store" — S1, §1
- Conditions/notes: per-registry top-10 layer share is 8%–30% (§4.3).
- Relevance: a small hot tier in front of the object store removes most of the latency.

### F7. A small two-level cache captures the workload: a memory cache sized at 2% of ingress gives a 40% hit ratio at the busiest site, and memory plus SSD reaches 100% at 4–6% of ingress.
- Quote: "we estimate that a cache size of 2% of the dataset size can provide 40% and higher hit ratios" — S1, §4.3
- Quote: "we achieve a combined hit ratio of 100% with 6% cache size for dal and 4% cache size for the other four AZs" — S1, §5 (two-level cache, Figure 17)
- Conditions/notes: trace-driven simulation, LRU at both levels, layers under 100 MB in memory, SSD tier 10×/15×/20× the memory tier, cache size as a share of the 75-day data ingress; no benefit at the two development registries.
- Relevance: a target to beat with an admission/eviction policy that is scan-resistant (see F50: Lambda uses LRU-k).

### F8. On Docker Hub (47 TB compressed, 1,792,609 layers, 5,278,465,130 files), layers are small and skewed: half are under 4 MB, 90% under 63 MB compressed.
- Quote: "we analyzed a 47 TB dataset resulting in 1,792,609 layers and 5,278,465,130 files" — S2, §VII
- Quote: "We see that 90% of the layers are smaller than 177 MB in uncompressed format and smaller than 63 MB in compressed format." — S2, §IV
- Conditions/notes: snapshot of `latest` tags, May 2017; "about half of the layers are smaller than 4 MB, independent of the format" (S2 §IV); "0.2% of compressed layers are larger than 1 GB" (S3 §4.1.1). Images: "90% of the images have compressed sizes less than 0.48 GB and uncompressed sizes less than 1.3 GB" and "50% of images are smaller than 17 MB compressed and 94 MB uncompressed" (S3 §4.1.1). Layer count: "half of the images have more than 8 layers while 90% of the images have less than 18 layers" (S3 §4.1.3).
- Relevance: the size distribution to use for workload-derived defaults; the data predates large ML images (compare F30: a 10 GB image in 2025).

### F9. Files inside images are tiny: 90% are under 26 KB and half under 4 KB.
- Quote: "with 90% and 50% of files being smaller than 26 KB and 4 KB, respectively" — S3, §4.1.4
- Conditions/notes: same Docker Hub dataset. "Half of the images have more than 1,090 files and 10% of images have more than 64,780 files" (same section).
- Relevance: any chunk size at or above 64 KiB is larger than almost every file, so small files must be packed into shared chunks (Nydus v3 does this, F35) and per-file metadata must stay compact.

### F10. Layer sharing barely deduplicates: about 90% of layers are referenced by one image.
- Quote: "around 90% of layers are only referenced by a single image, an additional 5% are referenced by 2 images, and less than 1% of the layers are shared by more than 25 images" — S3, §5.1
- Conditions/notes: "without layer sharing, the Docker Hub dataset would grow from 47 TB to 85 TB, implying a 1.8× redundancy ratio provided by layer sharing" (same section).
- Relevance: layer-level content addressing alone leaves most redundancy in place; sub-layer dedup is where the saving is.

### F11. File-level redundancy is large against uncompressed data (only 3.2% of files are unique; 31.5× by count, 6.9× by bytes), but only about 2× against the compressed layers a registry actually stores.
- Quote: "only 3.2% of the files are left, which in total occupy 24 TB. This results in redundancy ratios of 31.5× for file count and 6.9× for space occupied compared to the uncompressed image dataset" — S3, §5.2
- Counter: against compressed layers the saving is far smaller — "Compared to the compressed image dataset, the file-level deduplication ratio is almost 2×." — S3, §5.2
- Conditions/notes: "over 99.4% of files have more than one copy. Around 50% of files have exactly 4 copies and 90% of files have 10 or less copies" (same section). The dedup ratio grows with corpus size: space ratio "increases from 1.9× to 6.9× as the layer dataset grows from 1000 to 1.7 million layers" (§5.3).
- Relevance: the headline 6.9× is not the saving versus today's gzip layers; a design should quote the roughly 2× figure unless it also stores chunks compressed.

### F12. Compression of layers is moderate, and the two versions of the study report different medians.
- Quote: "50% of the layers are smaller than 4 MB which holds both for compressed and uncompressed layers, and the median layer compression ratio is 2.6" — S2, §I
- Counter: the journal version reports a higher figure — "50% of layer compression ratios and image compression ratios are greater than 3.5 and 3.2, respectively" — S3, §4.1.1
- Conditions/notes: same dataset; the papers do not explain the difference.
- Relevance: treat 2.6–3.5× as the range; small layers gain little from compression (S2 argues for storing them uncompressed).

### F13. Docker Hub popularity is extremely skewed: half of images have fewer than 40 pulls.
- Quote: "Only 10% of images are pulled more than 333 times while 50% of images are pulled less than 40 times." — S3, §4.1.2
- Conditions/notes: pull counts scraped from Docker Hub pages in 2017.
- Relevance: most stored bytes are cold; storage cost is set by the cold tail, latency by a small hot set.

### F14. Naive deduplication inside a registry slows pulls badly (up to 98×), because layers must be rebuilt and recompressed; DupHunter recovers this with replicas and preconstruction.
- Quote: "naive deduplication increases layer pull latencies by up to 98× compared to a registry without deduplication" — S4, §1
- Quote: "DupHunter reduces storage space by up to 6.9x compared to the current implementations. In the highest performance mode, DupHunter can reduce the GET layer latency up to 2.8x compared to the state-of-the-art." — S4, abstract
- Conditions/notes: Table 1 of S4: GET latency increase 36× (Jdupes), 60× (VDO), 51× (Btrfs), 50× (ZFS), 98× (Ceph) relative to uncompressed layers. Evaluated on the IBM traces. The time window available for rebuilding: "80% of intervals from London are greater than 1 second, and 60% of the intervals from Sydney are greater than 5 seconds" between manifest GET and layer GET (§3.3). Repull behaviour: "97% of layers from Syd are only fetched once by the same clients" (§3.3).
- Relevance: avoid a design where the bytes served for `GET /blobs/<digest>` must be reconstructed and recompressed on the read path; if chunks are the storage unit, the serving unit must be derivable without recompression.

### F15. Slimmer and Bolt (same group, 2019) are the file-level-dedup and hyperconverged-cache precursors.
- Quote: "Slimmer unpacks layer tarballs into individual files and deduplicates them." — S5, §I
- Quote: "each registry server caches images in its memory, stores images in its local storage, and provides computational resources to process client requests" — S6, abstract
- Conditions/notes: Bolt reports it "improves latency by an order of magnitude and throughput by up to 5×" and serves "up to 35% more requests from its cache" (S6 abstract, IBM traces); Slimmer is a simulation study.
- Relevance: Bolt's consistent-hashing, all-nodes-equal design matches a single-binary registry; copy the idea, measure against the same traces.

### F16. Follow-ups 2021–2024 extend dedup to the client and to encrypted images; none publishes a new production trace.
- Quote: "DupHunter reduces storage space by up to 6.9× and reduces the GET layer latency up to 2.8× compared to the state-of-the-art. Moreover, DupHunter can improve the container I/O performance by up to 93% for reads and 64% for writes." — S7, abstract
- Quote: "SimEnc outperforms both the state-of-the-art encrypted serverless platform and plaintext Docker registry, reducing storage consumption by up to 261.7% and 54.2%, respectively." — S8, abstract
- Quote: "Containers can be launched by pulling a Gear index and on demand retrieving files pointed to by the index." — S9, abstract
- Conditions/notes: S7 is the 2024 journal extension of DupHunter (registry, middle and client tiers). Gear reports 54% registry storage saved and startup up to 5× faster (abstract, authors' numbers). No 2025–2026 peer-reviewed registry workload study was found.
- Relevance: the workload evidence base is still the 2017 IBM and Docker Hub data; a new registry should record and publish its own traces.

### F17. Slacker's measurement that motivates all lazy loading: pull is 76% of start time and 6.4% of the pulled data is read.
- Quote: "pulling packages accounts for 76% of container start time, but only 6.4% of that data is read" — S10, abstract
- Quote: "Slacker speeds up the median container development cycle by 20x and deployment cycle by 5x." — S10, abstract
- Conditions/notes: HelloBench, 57 images. "The average times for push, pull, and run are 72, 20, and 6.1 seconds respectively." (S10 §4.2). Across all images, 27 MB is read on average and the average uncompressed image is 15× larger (§4.1). Global dedup across the 57 images gives rates of 2.6 at file granularity and 2.8 at 4 KB block granularity, against 2.3–2.7 for gzip (§4.1; numbers stated in prose, the passage has dropped ligatures in the text dump).
- Relevance: the 6.4% figure is from 2016 images started and stopped at "ready"; the 2026 SOCI paper (F28) measures the break-even at 80% of content accessed.

### B. Lazy-loading image formats

### F18. eStargz keeps the layer a valid gzip tar: each file (or chunk of a large file) is its own gzip member, a JSON TOC is the last tar entry, and a fixed footer gives the TOC offset.
- Quote: "TOC MUST be a JSON file contained as the last tar entry and MUST be named `stargz.index.json`." — S11, "TOC and TOCEntries"
- Quote: "In an eStargz-formatted blob, each non-empty regular file and each metadata component MUST be separately compressed as gzip." — S11, "The structure"
- Conditions/notes: footer is 51 bytes, an empty gzip member whose Extra field carries the TOC offset (S11 "Footer"). An optional variant moves the TOC to a separate "TOC image" found by a `-esgztoc` tag suffix (S11 "eStargz image with an external TOC"). Index location: inside the layer (default) or a side image (optional).
- Relevance: the "still a valid layer" property is what lets unmodified clients pull it; the cost is a converted (different-digest) layer.

### F19. eStargz chunk granularity is the file, with large files cut at 4 MiB by default; the source gives no justification for the value.
- Quote: "Zero means to use a default, currently 4 MiB." — S12, `Writer.ChunkSize` comment
- Conditions/notes: the writer returns `4 << 20` when `ChunkSize <= 0` (S12 L711-L715). Independently, the snapshotter fetches registry ranges at a configurable granularity: "are fetched from the remote registry. Default is 50000." (S13, `ChunkSize`, bytes). `--estargz-min-chunk-size` packs several small files into one gzip stream (S11 "Details about `innerOffset`").
- Relevance: 4 MiB is an inherited constant (from CRFS stargz), not a measured one; do not treat it as evidence.

### F20. eStargz integrity is a two-step chain: the TOC digest sits in the layer descriptor annotation of the (already verified) manifest, and each chunk carries a `chunkDigest` in the TOC that is checked on every fetch.
- Quote: "The value of this annotation is the digest of the TOC and this MUST be contained in the descriptor that references this eStargz layer." — S11, "Content Verification in eStargz"
- Quote: "Before providing a chunk to the filesystem user, snapshotter recalculates the digest and checks it matches the one recorded in the corresponding TOCEntry." — S11, "Example usecase: Content verification in Stargz Snapshotter"
- Conditions/notes: the annotation key is `containerd.io/snapshot/stargz/toc.digest`. Verification of the manifest itself is out of scope of the format. The whole-layer digest is not checked during lazy use (it cannot be, without the whole blob).
- Relevance: copy the chain manifest → index digest → per-chunk digest; it is the minimum that makes partial fetches verifiable.

### F21. eStargz is compatible with unmodified registries and runtimes, and supports prefetch through file ordering plus landmark files.
- Quote: "eStargz is a *backward-compatible extension* which means that images can be pushed to the extension-agnostic registry and can run on extension-agnostic runtimes." — S11, introduction
- Quote: "Before running the container, stargz snapshotter prefetches and pre-caches the range where prioritized files are contained, by a single HTTP Range Request supported by the registry." — S11, "Example use-case of prioritized files"
- Conditions/notes: prioritized files are recorded by profiling the workload at conversion time (`ctr-remote images optimize`); "Prefetch landmark MUST be named `.prefetch.landmark`." (S11). Runtime requirement: "Stargz snapshotter mounts eStargz layers from registries to the node using FUSE." (S14, overview). Linux 6.9 FUSE passthrough is supported: "FUSE Passthrough allows performing read and write (also via memory maps) on a backing file without incurring the overhead of roundtrips to userspace." (S15).
- Relevance: a single range request for the hot set is the cheapest prefetch a registry can serve; the registry needs correct Range support and nothing else.

### F22. Counter-evidence for eStargz: un-optimised lazy pulling slows the run phase, and on high-latency links it can be slower than a full pull.
- Quote: "But at the same time, we see the performance drawback for `run` operation because each access to files takes extra time for fetching them from the registry." — S14, README
- Counter: on high-latency links lazy pulling can lose to a full pull — "as RTT grows eStargz scales worse than the baseline and can even become slower than baseline containerd, as demonstrated for postgres with RTT of 150ms or above" — S55, §3
- Conditions/notes: Starlight measured eStargz v0.6.3: "eStargz’s provisioning time at RTT=300ms grows by 3.7× when going from RTT of 0ms to 300ms on a 500Mbps network" (S55 §5.2). The stargz README's own benchmark is a histogram image (HelloBench method, GitHub Actions against ghcr.io) with no numbers in text: not found as text.
- Relevance: beat this by bounding round trips per start (batch the hot set, push rather than pull) instead of one request per missed file.

### F23. Starlight replaces pull-per-file with a push-based, cross-layer delta bundle and reports 3.0× faster provisioning than the baseline with 5% storage overhead.
- Quote: "Starlight deploys and starts containers 3.0x faster than the current state-of-the-art implementation while incurring no runtime overhead and little (5%) storage overhead" — S55, abstract
- Conditions/notes: 21 popular containers; 1.9× faster than eStargz on average (S55 §5.2); needs a Starlight proxy next to a standard registry.
- Relevance: the registry (not a proxy) can compute "what this client lacks" itself if it knows the client's chunk set.

### F24. SOCI v1 leaves the image untouched and stores a separate index artifact (an index manifest plus one zTOC per layer) in the registry, found through the OCI referrers API.
- Quote: "Instead of converting the image, it builds a separate index artifact (the "SOCI index"), which lives in the remote registry, right next to the image itself." — S16, "No image conversion"
- Quote: "A zTOC is composed of 2 parts. 1) a table of contents containing file metadata and its offset in the decompressed TAR archive (the "TOC"). 2) A collection of "checkpoints" of the state of the compression engine at various points in the layer." — S17
- Conditions/notes: checkpoints make an ordinary gzip stream seekable; "Each checkpoint in the zInfo corresponds to exactly one span in an image layer." (S17). Runtime: one FUSE mount per layer (S24). SOCI began as a fork of stargz-snapshotter (S16 "Project Origin").
- Relevance: index-as-referrer needs no client or image change, which suits "serve existing OCI images lazily"; a registry can build such indexes server-side at push time.

### F25. SOCI's defaults are a 4 MiB span and a 10 MiB minimum layer size; the stated reason for the minimum is that small layers do not benefit.
- Quote: "Span size that soci index uses to segment layer data. Default is 4MiB" — S19, cli-usage
- Quote: "Minimum layer size to build zTOC for. Smaller layers won't have zTOC and not lazy pulled. Default is 10MiB" — S19, cli-usage
- Conditions/notes: justification given only for the minimum: "because small layers don't benefit much from lazy loading" (S19, getting-started). The span size has no stated derivation. The snapshotter-side `min_layer_size` option defaults to 0 with 10MiB recommended (S21, config).
- Relevance: both constants are unmeasured defaults; F5 (80% of requested layers under 10 MB) supports having a threshold, not this particular value.

### F26. SOCI integrity: the zTOC carries a digest per span; in v1 the index points at the image through a mutable referrer link, which v2 replaces with a binding through an image index.
- Quote: "and other metadata such as all span digests." — S22, `CompressionInfo` comment (field `SpanDigests []digest.Digest`)
- Quote: "The cost for the convenience of the referrers API is a weak, mutable reference between the SOCI index and the container image." — S18, "Introduction"
- Quote: "The SOCI index manifest v2 binds the SOCI index to the container image through an image index." — S25
- Conditions/notes: v1 weakness in the authors' words: "Anyone with write access to the image registry can add or delete a SOCI index at any time which can affect the runtime characteristics of the associated image." (S18). The snapshotter can skip TOC validation with `disable_verification` (S21, config; default false).
- Relevance: an index attached as a referrer must itself be bound to the manifest digest it describes, or deployments become non-reproducible; copy v2's binding.

### F27. SOCI Index Manifest v2 (snapshotter v0.10, Fargate from 2025-07-03) is a conversion step that produces a new image digest while reusing the layers; v1 is off by default since v0.10.0.
- Quote: "The SOCI snapshotter v0.10.0 will no longer consume SOCI Index Manifest v1 by default." — S18, "Migrating from SOCI Index Manifest v1"
- Quote: "Generating a SOCI index manifest v2 modifies the container image manifest as we add an annotation for the SOCI index, resulting in a new container image digest. The contents of the container image’s filesystem layers do not change." — S25, considerations
- Conditions/notes: artifact media type `application/vnd.amazon.soci.index.v2+json` (S25, S28). Copy tools that only take one platform drop the index: "Docker and Podman will only pull the current platform and therefore will not copy the SOCI index." (S20).
- Relevance: what changed in 2025: the "no conversion" promise of v1 was withdrawn for production use; plan for an immutable image-plus-index object.

### F28. Published SOCI numbers: about 50% faster task start on Fargate for one PyTorch image (2023), and 7.4× faster cold-start pull for a 1.3 GB image with a measured break-even at 80% of content accessed (2026 preprint).
- Quote: "the pytorch-without-soci (without SOCI indexes) successfully ran after 129 seconds" — S23
- Quote: "Here, I see my container image with SOCI-enabled — pytorch-soci — was started 60 seconds after being created." — S23
- Quote: "On a 1.3 GB Python web service image, SOCI reduces cold-start pull time from 20 seconds to approximately 2.8 seconds (7.4x speedup), with pull time independent of image size." — S27, abstract
- Quote: "We measure a crossover at 80% access density: below this, lazy loading wins; above, parallel full pull is faster." — S27, abstract
- Conditions/notes: S23 is five Fargate tasks per variant, createdAt to startedAt, one image, one region, vendor blog. S27 is an arXiv preprint by AWS authors (not peer reviewed); it also reports 9.3× on a 2.5 GB image and that Fargate "launched 18.4 million tasks per day during Prime Day 2025". Customer quotes in S23 cite 40–60% improvements.
- Relevance: the 80% crossover is the first published rule for choosing lazy versus eager per image; a registry that records access density per image can choose automatically.

### F29. Counter-evidence for SOCI: it does not help small images, raises filesystem latency during the background pull, and suits I/O-bound workloads badly.
- Quote: "For container images <250MB, the initial lazy loading overhead may be greater than the time taken to pull the full container image using traditional methods." — S24
- Counter: steady-state I/O is slower while the image is still arriving — "SOCI will have higher filesystem latency during image pulls. If your application is sensitive to latency variation, it may not be a good fit for SOCI." — S20, "Performance"
- Conditions/notes: S24 also names metadata-heavy workloads and workloads that read the whole image soon after start as poor fits. S21 introduces the eager mode "for I/O-bound applications that require fast access to the full rootfs".
- Relevance: lazy loading must be a per-image (or per-layer) decision with an eager fallback, not a global mode.

### F30. SOCI's 2025 parallel pull mode is an eager alternative: ranged, concurrent layer download plus parallel unpack cut a roughly 10 GB image pull from 1m52.876s to 45.121s.
- Quote: "In comparison, when using SOCI Parallel Pull Mode we observe a pull time of 45.121s." — S26
- Quote: "for the baseline deployment using the default containerd configuration, we observe a pull time of 1m52.876s" — S26
- Conditions/notes: one Amazon DLC vLLM image (10,778,400,361 bytes), m6i.8xlarge, ECR, settings 10 concurrent downloads per image, 16 MB download chunks, 10 concurrent unpacks. Defaults are conservative (3 downloads per image, chunk = layer size, 1 unpack; S21). The docs recommend at least 64 cores and 128 GB RAM (S21 "Performance Tuning").
- Relevance: for large images the registry's job is also to serve many concurrent ranged GETs per blob without rate limiting; "8mb" or "16mb" ranges are the client-side constants to expect.

### F31. SOCI on Lambda (Lambda Managed Instances) accepts v2 indexes only and never downloads the rest of the image in the background; no numbers are published.
- Quote: "Lambda supports SOCI Index Manifest v2 only (media type application/vnd.amazon.soci.index.v2+json)." — S28
- Quote: "Lambda does not download the remainder of the image in the background." — S28
- Conditions/notes: speed-up numbers for SOCI on Lambda: not found. This is separate from Lambda's own block-level loader (F46–F52).
- Relevance: pure on-demand (no background fill) is a deployed mode; the registry must then serve range reads for the whole life of the workload.

### F32. zstd:chunked puts a table of contents with per-chunk SHA-256 digests in zstd skippable frames, compresses chunks separately, and lets a client fetch only what it lacks by HTTP range.
- Quote: "Additionally chunks are compressed separately. This allows a client to dynamically fetch only content which it doesn't already have using HTTP range requests." — S29, "DESCRIPTION"
- Quote: "ManifestChecksumKey is a hexadecimal sha256 digest of the compressed manifest digest." — S30, `compression.go`
- Conditions/notes: the TOC position and checksum travel as layer annotations (`io.github.containers.zstd-chunked.manifest-position`, `...manifest-checksum`), which binds the TOC to the image manifest. Chunk boundaries inside files come from a bup-style rolling checksum with a 64-byte window and `RollsumBits = 16` (S30), i.e. an expected cut every 2^16 bytes (derived: 64 KiB average; no justification in the source). Its purpose is partial pulls and local reuse, not start-before-download: "When any image is pulled all existing other layers are scanned using `chunked-manifest-cache` to see if they contain a file with a matching digest." (S29).
- Relevance: the only widely shipped format that already does content-defined sub-file chunking inside a standard media type; a chunk-aware registry can serve it without knowing the format.

### F33. zstd:chunked is not standardised, non-supporting clients read it as plain zstd, and the only published speed-up is an unqualified "as much as 90%".
- Quote: "At the current time the format is not officially standardized or documented beyond the comments and code in the reference implementation." — S29, "STANDARDIZATION"
- Quote: "zstd:chunked format has shown itself to speed up the pulling of images by as much as 90%." — S32
- Counter: with composefs, integrity is not enforced yet — "There is currently no support for enforced integrity with composefs;" — S31
- Conditions/notes: "Moby doesn't support zstd:chunked but it handles the container image as a standard zstd layer since Moby 23.0." (S32). The 90% figure has no stated image, baseline or network.
- Relevance: do not depend on its on-disk details; do support its access pattern (many small ranges per blob).

### F34. Nydus v2 (RAFS v5/v6) splits an image into a metadata bootstrap and data blobs of fixed 1 MB chunks, each chunk with a digest; RAFS v6 metadata is EROFS-compatible.
- Quote: "Nydus has splitted a file's data into one or more fixed-length (1MB) chunks." — S33, §5
- Quote: "Data is split into chunks and each chunk has a saved digest in chunk info, the way of metadata digest validation applies to chunks as well." — S33, §3
- Conditions/notes: digests are SHA-256 or BLAKE3 ("adding one of the two digest validations (Sha256 and blake3) to the metadata", S33). Kernel path: "In-kernel EROFS has been fully compatible with RAFS v6 image format since Linux 5.16." (S34); lazy loading in-kernel used fscache from Linux 5.19 (S34), which EROFS has since deprecated (F42). Dedup claim: "Chunk level data de-duplication cross-layer or cross-image to reduce storage, transport, and memory cost." (S34). Index location: a separate bootstrap layer in the image. Not readable by unmodified OCI clients.
- Relevance: the bootstrap-plus-blobs split is the model for "metadata first, data on demand"; the 1 MB constant is unexplained.

### F35. Nydus v3 (release candidate published 2026-11-20) is a ground-up rewrite: a standard EROFS layout, content-defined chunk groups of 2 MiB and up, CRC32C plus optional BLAKE3 per group, and it does not read v2 images.
- Quote: "This repository is the Nydus image format v3 implementation — a ground-up redesign in Rust." — S35
- Quote: "Nydus v3 cuts layer data into chunk groups with content-defined boundaries (2 MiB and up by default)." — S35, "Global cache deduplication with content-defined chunk groups"
- Quote: "Every group carries a CRC32C over its decoded payload." — S36, "Blocks, chunks and chunk groups"
- Counter: its own stated reason for larger units is metadata cost — "OCI and Nydus v2 deduplicate whole layers only (Nydus v2 chunks are too small to deduplicate at an affordable cost): one changed file makes the whole layer different." — S35
- Conditions/notes: `--chunk-size` default 2 MiB, `--chunk-group-minimum-size` default 2 MiB, daemon `storage.fetch_size` default 2 MiB of compressed bytes (S35 "Features"). Files are chunked per file; small files are packed in order into shared groups of one to four times the minimum, closing where a chunk digest's low bits are zero (S36). Hashes: BLAKE3 per group in a digest table, SHA-256 over the data region as blob id, SHA-256 over the whole blob as the OCI layer digest (S36 "Hash and validation summary"). The image format itself lists "Introduce chunk-content or cross-layer global deduplication." under Non-goals (S36); global dedup is done by a cache keyed by group digest. Dragonfly integration: "registry blobs are range-read directly, optionally through a Dragonfly forward proxy or the higher-performance client SDK mode talking straight to a scheduler" (S35). Release candidate, vendor documentation, published on the research date.
- Relevance: the most recent design point for chunk size: group size chosen so that a content-addressed cache can afford the index; compare with Lambda's 512 KiB (F46).

### F36. Nydus v3 vendor numbers: global chunk-group dedup stores about 3 TiB for a 71.7 TiB image set (96%) against 20.8 TiB with layer dedup (71%), and cold start of a 4.04 GB image in a Kata microVM drops from 22.62 s (OCI) to 8.94 s.
- Quote: "The cold-start E2E of Nydus v3 (optimize) is 1/2.5 of the OCI one and 1/2 of the Nydus v2 one." — S35, "OCI vs Nydus v2 FUSE + VirtioFS vs Nydus v3 + PMEM + UFFD"
- Conditions/notes: dedup table: SWE-rebench-v2 set, "120k+ images"; "about 3 TiB" is approximate in the source. Kata table (openclaw image, 4.04 GB, 52 layers; E2E = Pull + Create + Ready): OCI 22.62s; Nydus v2 FUSE + VirtioFS 17.75s; Nydus v3 + PMEM + UFFD 10.78s; with optimize 8.94s; with local cache hit 6.21s. PyTorch image end-to-end startup: Nydus v2 41.2s, v3 24.6s, v3 + Dragonfly 14.9s, v3 + Dragonfly with ondemand blob 9.4s. Hardware and network are not stated.
- Relevance: a target to reproduce on named hardware before citing; the EROFS-over-virtio-pmem path is the microVM-relevant one.

### F37. Nydus v3 measures its five read paths against registry round-trip time: kernel-EROFS paths beat FUSE everywhere, and added RTT dominates everything unless the hot set is prefetched.
- Quote: "The four kernel EROFS modes are faster than FUSE at every point; fanotify with optimize is the fastest." — S35, "FUSE / NBD / ublk / fileio / fanotify"
- Conditions/notes: time to first HTTP 200 of a Next.js app (`node:20`, 1.6 GiB rootfs, 35k files) at RTT ≈ 0 / +30ms / +50ms: Nydus v2 FUSE 1.72s / 5.51s / 8.64s; v3 FUSE 1.44s / 3.48s / 4.90s; NBD 1.15s / 3.28s / 4.77s; ublk 1.04s / 3.09s / 4.62s; fileio 1.17s / 3.21s / 4.83s; fanotify 1.10s / 3.20s / 4.60s. After `nydus optimize` at +50ms: FUSE 1.75s, NBD 1.67s, ublk 1.45s, fileio 1.51s, fanotify 1.38s. Kernel requirements (S36 table): chunk-based files 5.15; device table 5.16; flattened NBD/ublk/PMEM image 6.4 (ublk driver 6.0); fanotify-backed EROFS 6.15+; FUSE path has no kernel EROFS requirement.
- Relevance: confirms F22 with numbers: 50 ms of RTT triples cold start without a prefetch bundle; NBD is within about 10% of ublk and is the portable transport (F61).

### F38. For microVMs Nydus v3 serves the image as an EROFS device over virtio-pmem and fills it on page faults through userfaultfd, with NBD and ublk as block transports for hosts.
- Quote: "The microVM process reserves an anonymous virtio-pmem mapping, registers it with userfaultfd, and sends the userfaultfd to Nydus." — S37, "Overview"
- Quote: "The NBD service supports hosts that mount EROFS directly from a block device." — S38, nbd.md "Overview"
- Conditions/notes: the UFFD protocol is host-local (AF_UNIX socket, fd passing). "For PMEM/RunD, the EROFS requirement applies to the guest mounting the device." (S36). A library entry point (`NydusCore`) exists for hypervisors that embed the read path (S36 "Core (virtio-pmem integration)").
- Relevance: the guest needs only an EROFS-capable Linux kernel; the host-side mechanism (userfaultfd) is Linux-only, the embeddable core is not tied to it.

### F39. DADI (Alibaba, ATC'20) serves images as layered virtual block devices fetched on demand, with a seekable compressed layer format (ZFile) and per-block CRC32 checksum files; it reports cold-starting 10,000 containers on 1,000 hosts within 4 seconds.
- Quote: "ZFile includes the source file compressed in a fixed-sized chunk-by-chunk manner and a compressed index." — S40, §3.3
- Quote: "The checksum file contains the CRC32 value for each fixed-sized block of the layer." — S40, §3.5
- Quote: "DADI can cold start 10,000 containers on 1,000 hosts within 4 seconds." — S40, §1
- Conditions/notes: block granularity for the device is 512 bytes (S40 §3.1); checksum files are "distributed whole to every involved node" and "The data blocks are verified on arrival at each node." (§3.5). Batch startup: "the startup time with DADI remains largely constant at 0.7s as the number of instances increases" (§5, versus 1.5–2.3 s for the Slacker-style baseline). Layer index is small: "the indices have fewer than 4.5K segments" (§3.2, production statistic). CRC32 is an error check, not a cryptographic binding to the manifest.
- Relevance: block-level images work for containers, QEMU guests and microVMs alike; copy the layered block index, replace CRC32 with digests chained to the manifest.

### F40. overlaybd, DADI's open-source implementation, needs the Linux TCMU kernel module on the host; its ZFile default block is 4096 bytes with an optional CRC32 after each compressed block; the TurboOCI variant keeps the OCI image and adds a meta image of about 3%.
- Quote: "Overlaybd provides virtual block devices through TCMU, so the TCMU kernel module is required." — S41, "System Requirements"
- Quote: "whether these exists a 4-byte CRC32 checksum following each compressed block" — S42, ZFile header field `verify`
- Quote: "it builds a separate image with a small amount of data of the ext4 filesystem (which is the "meta image"), which lives in the remote registry and usually has 3% size of the original image" — S44
- Conditions/notes: `DEFAULT_BLOCK_SIZE = 4096` (S42, `compressor.h`); ZFile supports lz4, zstd and others. Vendor benchmark: WordPress service-available time 6.433s (overlaybd) versus 15.341s (OCI tar), prefetch enabled, hardware not stated (S43). The project states applicability beyond containers: "overlaybd is also a widely applicable image format for most runtime, including qemu/kvm and any other runtime supporting block or scsi api" (S41). A ublk backend is announced as future work (overlaybd `docs/README.md`).
- Relevance: the format is host-kernel-independent, the shipped transport (TCMU) is not; for a cross-platform VMM the format logic has to live in the VMM's block backend.

### F41. DADI's own P2P is a tree, not a swarm: each layer has a tree rooted at a per-datacentre root that fetches from the registry, because all peers need the same blocks in the same order.
- Quote: "DADI adopts a tree-structured overlay topology to realize application-level multicast similar to VMThunder [40] instead of the rarest-first policy commonly used in P2P downloading tools" — S40, §3.5
- Conditions/notes: "Notice also that a binary tree for P2P is best when there are fewer than 20,000 participating hosts. A 3-ary or 4-ary tree works better beyond that scale." (§5). Topology information expires "by default after 20 minutes" (§3.5).
- Relevance: for start-up traffic, ordered tree multicast beats rarest-first; see F53 for FaaSNet's criticism of the central root.

### F42. EROFS kernel features by version: chunk-based files (5.15), multiple devices (5.16), fscache on-demand mode (5.19, deprecated in 6.12 and absent from the 7.2 documentation), compressed-data dedup (6.1), file-backed mounts (6.12), page-cache sharing across images (7.0).
- Quote: "a new inode data layout has been supported since Linux v5.15: Files are split in equal-sized data chunks" — S45, "Chunk-based files"
- Quote: "Since Linux v6.1, each pcluster can be used for multiple variable-sized extents, therefore it can be used for compressed data deduplication." — S45
- Quote: "EROFS has supported EROFS over fscache (since Linux 5.19, deprecated in Linux 6.12) and file-backed mounts (since Linux 6.12) to avoid unnecessary loop devices." — S47, note 7
- Quote: "Enable inode page sharing for this filesystem. Inodes with identical content within the same domain ID can share the page cache." — S46, mount option `inode_share` at tag v7.0
- Counter: chunk-based dedup and compression do not combine — "By the way, chunk-based files are all uncompressed for now." — S45
- Conditions/notes: `inode_share` is absent from `erofs.rst` at v6.18 and v6.19 and present at v7.0, v7.1, v7.2; `fsid=` and all Fscache text are present at v7.1 and absent at v7.2 (S46, checked by text search at each tag). Compression algorithms: LZ4, MicroLZMA (5.16), DEFLATE (6.6), Zstandard (6.10) (S47). Multi-device support of 5.16 is taken from the Nydus kernel table (S36) and S34. Compressed-data dedup adjusts cut points "using the Rabin–Karp algorithm" (S48, design).
- Relevance: what changed 2025–2026: fscache-based lazy loading is gone, replaced by file-backed mounts plus fanotify pre-content hooks or a block transport; an EROFS image destined for a microVM guest only needs the guest kernel version, not the host's.

### F43. erofs-utils (1.9 series, 2026) can build EROFS directly from a tarball, an OCI registry or S3, can emit a metadata-only index over the original tar and SOCI-compatible gzip zinfo, and has explicit reproducibility options.
- Quote: "Additionally, --tar=i can be used to generate a minimized EROFS metadata index that references external tar data:" — S48, "From a tarball file"
- Quote: "Specify a UNIX timestamp for image creation time for reproducible builds." — S49, man page `-T`
- Quote: "(mkfs.erofs) Enable image creation from OCI registries (Chengyu Zhu);" — S49, ChangeLog, erofs-utils 1.9 (2026-02-18)
- Counter: global dedup is still single-threaded — "Enable global data deduplication [2]. Note: not supported with multi-threading yet." — S48
- Conditions/notes: `--chunksize #` "Generate chunk-based files with #-byte chunks."; `--blobdev file` "Specify an extra blob device to store chunk-based data."; `-E dedupe` "Enable global compressed data deduplication to minimize duplicated data in the filesystem." (Linux 6.1+); `-U` sets the UUID; `--gzinfo` generates "AWS SOCI-compatible zinfo" (all S49). 1.9 added `--xattr-inode-digest` for page-cache sharing and a mount helper for EROFS-native layers in OCI registries; 1.9.2 (2026-07-02) added "(mount.erofs) Add support for Fanotify (Yifan Zhao) and UBLK (Chengyu Zhu) backends;" and S3-object mounts; 1.9.4 (2026-08-19) fixes a macOS crash in mkfs.erofs (S49). Default block and compression cluster size: 4096 bytes on x86 (S48).
- Relevance: tar-to-EROFS without unpacking, with fixed timestamp and UUID, is the conversion a registry can run at push time on any host OS (mkfs.erofs builds on macOS; Windows not verified).

### F44. composefs splits an image into an EROFS metadata tree and a content-addressed backing store shared on disk and in page cache, with fs-verity making the backing files tamper-evident.
- Quote: "this EROFS filesystem tree is just metadata; the underlying non-empty data files can be shared in a distinct "backing store" directory." — S50
- Quote: "the backing content cannot be changed in any way (by mistake or by malice) without this being detected when the file is used." — S50
- Quote: "fs-verity is essentially a way to hash a file in constant time, subject to the caveat that reads which would violate the hash will fail at runtime." — S51
- Conditions/notes: fs-verity builds a per-file Merkle tree and "all reads from the file are automatically verified against the file’s Merkle tree" (S51); supported on ext4, f2fs and btrfs (S51). Dedup granularity is the whole file. Linux-only (overlayfs + EROFS + fs-verity).
- Relevance: file-level content addressing with kernel-enforced verification is the host-side (container) analogue of per-chunk digests; it does not transfer to a block device handed to a guest, where dm-verity or in-backend verification is needed.

### F45. containerd has had a native EROFS snapshotter and differ since v2.1.0 (2025-05-07); by v2.4.2 it has a tar-index mode whose index can live in the registry, dm-verity per layer, and block-level pass-through to VM runtimes.
- Quote: "The tar index can be stored in a registry alongside image layers, allowing nodes to fetch it directly when needed." — S52, "Tar Index Mode"
- Quote: "Regular mode uses 4096-byte blocks (standard page size), while tar-index mode uses 512-byte blocks (dm-verity logical_block_size constraint)." — S52, "Data Integrity with dm-verity"
- Quote: "This delivers lower overhead, better performance, and smaller memory footprints compared to traditional shared-filesystem approaches." — S58, EROFS snapshotter with Kata
- Counter: verification costs runtime — "it introduces additional runtime overhead since all container image reads from the container will be slower because it needs to verify the Merkle hash tree first." — S52, "Data Integrity with fs-verity"
- Conditions/notes: release note entry "Erofs snapshotter and differ (#10705)" (S53); the same release lists "Update CRI to use transfer service for image pull by default (#8515)". Tar-index mode keeps the original tar bytes, so the layer's diffID can still be checked by hashing them (S52). Setting the immutable flag doubled unpack time in one example: "the unpacking time for tensorflow:2.19.0 increases by 108.86% (from 10.090s to 21.074s) on EXT4" (S52). A `layer_content_caches` option mounts pre-built EROFS blobs keyed by diffID with no conversion and no layer download (S52). Unpack-time benchmark figures are images only: not found as text.
- Relevance: the ecosystem is converging on "EROFS blob per layer, handed to the VM as a block device"; a registry that stores or serves the tar index (and Merkle tree) next to a layer fits this directly.

### F46. AWS Lambda loads container images at block level: layers are flattened deterministically into one ext4 image at function creation, cut into fixed 512 KiB chunks, and served on demand through virtio-blk.
- Quote: "following a deterministic flattening process which applies each tarball in order to create a single ext4 filesystem" — S54, §2
- Quote: "Each fixed-size chunk is 512KiB. Smaller chunks lead to better deduplication by minimizing false-sharing, and can accelerate loading for workloads with highly random access patterns." — S54, §2
- Conditions/notes: the stated trade-off continues: larger chunks "reduce metadata size, reduce the number of requests needed to load data (hence improving throughput), and provide natural read-ahead for sequential workloads", and "The optimal value will change over time as the system evolves" (§2). Reason for block level: filesystem complexity "would unacceptably increase the attack surface of the shared components" (§2). Writes go to a per-MicroVM overlay with a page-granularity bitmap (§2.1). Scale targets: "adding up to 15,000 new containers per second for a single customer" and images up to 10 GiB (abstract). All-zero chunks are elided.
- Relevance: the reference design for block-level microVM images; the chunk size is an engineering judgement stated with its trade-offs, not a measured optimum; the flattening must be deterministic for fixed-size chunks to deduplicate.

### F47. Lambda's dedup measurements: about 80% of uploads add zero new chunks, and the rest add a mean of 4.3% (median 2.5%) unique chunks.
- Quote: "Approximately 80% of newly uploaded Lambda functions result in zero unique chunks, and are just re-uploads of images that had been uploaded in the past." — S54, §3
- Quote: "the mean upload contains 4.3% unique chunks, and the median 2.5% unique chunks" — S54, §3
- Conditions/notes: storage effect in the authors' words: "reducing storage by as much as 23x" for non-trivial uploads and "reducing storage costs by another 5x" from the 80% re-uploads (§3). Production numbers, one provider, images built on common base layers.
- Relevance: the strongest published dedup result for fixed-size chunks; it depends on deterministic flattening and on CI/CD re-uploads, both of which a registry also sees.

### F48. Lambda deduplicates across tenants with convergent encryption: the chunk key is derived from the chunk's SHA-256, a salt limits how far any chunk is shared, the per-customer manifest holds the keys, and chunks are not compressed.
- Quote: "derives a key from it by computing its SHA256 digest, and then encrypts the block using AES-CTR (with the derived key)" — S54, §3.1
- Quote: "we include a varying salt in the key derivation step of our convergent encryption scheme" — S54, §3.3
- Quote: "Our system does not compress chunk plaintexts prior to encryption." — S54, §3.2
- Conditions/notes: manifest size "less than 3MiB for a 16GiB container image, or 0.02% overhead" (§3.1). Only the key table of the manifest is encrypted (AES-GCM, per-customer KMS key) so GC can read the chunk list. Workers verify fetched chunks against the manifest. Reasons for no compression: random access is hard and "compression before encryption allows potential attackers to infer plaintext contents from compressed sizes, a compression side channel" (§3.2). Salt varies in time, with popularity and by placement; "Salt rotation is an operational concern, and is not needed for the security of the deduplication scheme." (§3.3).
- Relevance: copy the salt as an explicit blast-radius and tenancy knob; see F71 and F72 for the attacks convergent encryption still allows.

### F49. Lambda garbage-collects by generations ("roots") instead of reference counts, and keeps retired data in an alarmed "expired" state before deletion.
- Quote: "Our approach to garbage collection is based on the concept of roots." — S54, §3.4
- Quote: "data is still allowed to be read, but any attempt to access data leads to an alarm" — S54, §3.4
- Conditions/notes: a root is "a self-contained manifest and chunk namespace"; new roots are created periodically, live manifests and their chunks are migrated, old roots go active → retired → expired → deleted; the active root's identifier is part of the dedup salt, so chunks never deduplicate across roots (§3.4). The paper says exact reference counting was infeasible without a central chunk directory.
- Relevance: a GC design with a built-in tripwire; costs duplicate storage across roots.

### F50. Lambda's three cache tiers serve a median of 67% of chunk loads from the worker, 32% from an AZ-level cache and 0.06% from S3; the AZ cache uses LRU-k eviction.
- Quote: "a median of 67% of chunks were loaded from the on-worker cache, 32% from the AZ-level distributed cache, and the remaining 0.06% from the backing store" — S54, §4
- Conditions/notes: one week, one large region. AZ-cache hit takes a median of 550µs versus 36ms for S3 (99.9th percentile 3.7ms versus 175ms) as seen by the worker (§4; the sentence has a line-break hyphen in the PDF text, so it is given here in prose). In-AZ cache hit rate: median 99.9%, 10th percentile 99.4% (§4). AZ cache: memory tier about 10% of cache size over flash, chunks placed by consistent hashing, HTTP/2 wire protocol; server-side GET latency "with a median of below 50µs" for 512 kB chunks (§5.1). LRU-k is chosen for scan resistance against periodic cron-style functions (§4.3).
- Relevance: the origin sees well under 1% of reads when a shared regional cache exists; build the regional tier into the registry rather than leaving it to clients.

### F51. Lambda erasure-codes cache entries 4-of-5 to cut tail latency and survive node loss without retries, and warns that a cold cache multiplies origin load by up to 500×.
- Quote: "Our current production deployment uses a 4 of 5 code, achieving 25% storage overhead, and a 25% increase in request rate in exchange for a significant decrease in tail latency." — S54, §4.1
- Quote: "with an end-to-end hit rate typically exceeding 99.8%, this downstream traffic increase could be up to 500 times normal" — S54, §4.2
- Conditions/notes: why tails matter: "a start which fetches 1000 chunks will experience the 99.9th percentile tail latency of the cache on 63% of tasks" (§4.1). Mitigation for the cold-cache case is a concurrency limit and regular cold-start tests from an empty cache (§4.2).
- Relevance: copy "constant work" (ask for 5, use the first 4) instead of retries; state the bound on origin load when caches are empty.

### F52. Counter-evidence from Lambda on FUSE: presenting a FUSE-backed file as a virtio-blk device added latency and jitter, and the team is replacing it.
- Quote: "This introduces inefficiency in steady state, and significant jitter under load. We are moving away from FUSE for this application, primarily due to this effect. Our new implementation uses userfaultfd and mmap, removing two layers from the architecture." — S54, §5.2
- Counter: FUSE overhead depends on workload — "performance degradation caused by FUSE can be completely imperceptible or as high as –83% even when optimized; and relative CPU utilization can increase by 31%" — S70, abstract
- Conditions/notes: the path was guest kernel → Firecracker → host FUSE → local agent, needing "four different threads to be scheduled" per uncached chunk read (S54 §5.2). End-to-end read latency at the agent is multi-modal: below 100µs for local hits, around 2.75ms for AZ-cache hits including decryption (§5.1).
- Relevance: put the lazy block backend in the VMM process (or one hop away), not behind a host filesystem.

### F53. FaaSNet (Alibaba Function Compute) streams 512 KB compressed blocks through per-function trees and criticises both all-to-all P2P and a central root.
- Quote: "finishes provisioning 2,500 function containers on 1,000 virtual machines in 8.3 seconds" — S56, abstract
- Quote: "uses a block size of 512 KB for on-demand fetching and streaming" — S56, §4.1
- Counter: against swarm P2P for bursts — "Kraken takes 100.4 seconds to launch 128 containers." — S56, §4
- Conditions/notes: 758 MB image, VMs with 2 CPUs, 4 GB memory, 1 Gbps. Origin load: the function tree "can completely eliminate the I/O traffic to the registry, as long as a FT has at least one active VM that stores the requested container" (§3.2). Reported 13.4× faster scaling than the previous platform and 16.3× than Kraken (abstract). DADI+P2P is criticised for a root-VM bottleneck (§4). No stated justification for 512 KB.
- Relevance: block size of 512 KB/KiB recurs in two independent production systems (F46); origin load can be bounded to one fetch per tree.

### F54. Firecracker backs a guest block device with a host file (synchronous engine by default, io_uring engine in developer preview) or, since v1.6.0 and still in developer preview, a vhost-user backend process that can implement lazy network fetch.
- Quote: "It can use intelligent algorithms to serve block requests, eg by fetching the block device data over the network or using sophisticated readahead logic." — S60, "Advantages"
- Counter: vhost-user makes guest memory shared, which slows page faults — "It was observed that page faults to a shared memory mapping take significantly longer (up to 24% in our testing)" — S60, "Disadvantages"
- Conditions/notes: vhost-user requires guest memory to be shared through `memfd_create` and a Unix domain socket to the backend (S60). CHANGELOG 1.6.0: "snapshotting is not currently supported for microVMs containing vhost-user block devices" (S61). The host page cache no longer helps: "This benefit is not available in vhost-user block case." (S60). There is no built-in remote or chunked block backend; production users put FUSE (F52, F58) or a kernel block device under the file path.
- Relevance: Firecracker offers no supported lazy block source of its own; a registry client that wants one on Linux must be a vhost-user backend, a FUSE file, NBD/ublk device, or (new) a virtio-pmem source.

### F55. Firecracker added virtio-pmem in v1.14.0 and has userfaultfd-driven snapshot memory loading; together they allow a root device or memory image to be filled lazily by a userspace process.
- Quote: "`virtio-pmem` is backed by a memory mapped file on the host side and is exposed to the guest kernel as a region in the guest physical memory." — S61, pmem.md
- Quote: "Userfaultfd is a mechanism that passes that responsibility of handling page fault events from kernel space to user space." — S62
- Conditions/notes: pmem can be the root device and supports DAX (S61). The UFFD handler is the user's own process (S62). This is the mechanism Nydus v3 targets for Kata (F38).
- Relevance: Linux-only on the host side; relevant as the fastest Linux path, not as the portable one.

### F56. firecracker-containerd uses a device-mapper thin-pool snapshotter for block-device rootfs, and for lazy loading runs the remote snapshotter inside the microVM because Firecracker has no virtio-fs.
- Quote: "Rather than add another layer of indirection in the form of a network filesystem, we opted to move the whole snapshotter inside the microVM." — S63, remote-snapshotter.md
- Conditions/notes: on the devmapper snapshotter, which uses device-mapper thin provisioning: "we can achieve efficient, deduplicated storage between content layers and their associated images or containers" (S63, snapshotter.md); the same document records concerns about its copy-on-write and provisioning performance.
- Relevance: file-level lazy formats (eStargz, SOCI) reach a microVM only by moving the fetcher into the guest; block-level formats do not need that.

### F57. Fly.io: Fly Machines pull OCI images into a host-local containerd on an LVM2 thin pool and hand the guest a block device (no lazy loading); Sprites (2026) drop per-user images and back disks with chunks on object storage behind an NVMe read-through cache.
- Quote: "we pull it from the registry into our server-local containerd, configured to run on an LVM2 thin pool" — S64
- Quote: "Instead, it’s a read-through cache for a blob on object storage." — S65
- Counter: object-storage-backed disks were judged too slow for databases — "but the performance isn’t adequate for a hot Postgres node in production" — S65
- Conditions/notes: S65 on why images are the slow part: "Huge and fussy, they take forever to pull and unpack." Sprites storage follows the JuiceFS model: "Data chunks live on object stores; metadata lives in fast local storage." (S65). No latency numbers beyond "a second or two" for create.
- Relevance: an operator at scale names registry pull-and-unpack as the bottleneck and moved to chunk-on-object-store disks; numbers are absent.

### F58. Other production accounts of lazy microVM state: BuildBuddy chunks Firecracker memory and disk snapshots into a remote cache and loads them lazily (userfaultfd for memory, a FUSE-backed file for disk); E2B restores snapshots with userfaultfd and a copy-on-write rootfs overlay; CodeSandbox resumes mmap'd memory snapshots in about 200–300 ms.
- Quote: "So rather than attaching a monolithic disk image file on our VMs, we attach a FUSE-backed virtual disk." — S68
- Quote: "Later, when a clone tries to read this snapshot data, each chunk is lazily downloaded." — S68
- Quote: "Memory pages are loaded lazily on page-fault (userfaultfd) and the root filesystem is a copy-on-write overlay, so only touched data is ever fetched." — S69
- Quote: "Firecracker will only read the memory that the VM needs to start (as the memory is mmaped), which results in resume timings within ~200-300ms." — S66
- Conditions/notes: BuildBuddy workloads of 20 GB memory and 80 GB disk; "our median CI run is now only 30 seconds" (S68); dirty chunks are copied on write and only dirty chunks are uploaded at the next snapshot. Chunk sizes for BuildBuddy and E2B: not found. Modal is covered in F59.
- Relevance: three independent operators converged on content-addressed chunks plus lazy fetch for both disk and memory; none publishes chunk-size reasoning.

### F59. Modal's lazy container loader is a FUSE filesystem over a roughly 5 MB per-image index of content-addressed files, with page cache, local SSD, an AZ cache server, CDN and blob storage as tiers; tuning raised read throughput from about 800 MB/s to about 2.5 GiB/s.
- Quote: "This index is around five megabytes and can be loaded in one to 100 milliseconds, depending on whether we’re grabbing it from memory, disk, or over the network." — S67
- Quote: "we saw throughput go up to approximately 2.5 GiB/s, enabling us to load a 512 MiB .safetensors file in just 200 milliseconds from disk cache and about 300 milliseconds from the network" — S67
- Counter: over-aggressive read-ahead hurt — "We once shipped a version with a read-ahead set at 1 GB, which resulted in disastrous latency issues in production." — S67
- Conditions/notes: eager baseline in the same post: an 8 GB uncompressed image "can lead to about a minute spent on decompression" because gzip decompression is single-threaded. File-level (not block-level), gVisor/runc containers rather than microVMs.
- Relevance: decompression, not download, is the eager-pull bottleneck (Kraken says the same, F76); bound read-ahead.

### F60. Other 2023–2026 systems: PuzzleFS (FastCDC chunks in an OCI layout, FUSE today), gVisor (EROFS rootfs memory-mapped into the sentry), Kata/Confidential Containers (image pulled inside the guest through the nydus snapshotter).
- Quote: "Reduced duplication is achieved using the content defined chunking algorithm FastCDC." — S57
- Quote: "Currently, puzzlefs is implemented as a userspace filesystem (FUSE). A read-only kernel filesystem driver is underway." — S57
- Quote: "The EROFS image file is memory mapped into the sentry and accessed via memory access by the sentry." — S59
- Quote: "Kata Containers 3.3.0 introduces the guest image management feature, which enables the guest VM to directly pull images using `nydus snapshotter`." — S58
- Conditions/notes: PuzzleFS leaves CDC parameters open ("we leave the choice of hash, parameters, etc. as an exercise to the reader", S57 chunking doc) and argues that fixing them in the spec would increase sharing; last commit seen 2025-07-07. Guest pull exists "to protect the integrity of container images and guard against any tampering by the host" (S58).
- Relevance: for confidential guests the registry is reached from inside the VM, so the lazy client must also run as a small in-guest component.

### F61. Cross-platform: Hypervisor.framework and the Windows Hypervisor Platform are user-space APIs that leave device emulation to the VMM, so a lazy block backend implemented inside the VMM's virtio-blk device needs no host kernel feature; FUSE, fscache, fanotify, ublk, TCMU, userfaultfd and vhost-user are Linux (or Unix) host mechanisms.
- Quote: "Hypervisor provides C APIs so you can interact with virtualization technologies in user space, without writing kernel extensions (KEXTs)." — S71, Hypervisor
- Quote: "The NBD client forwards the guest’s I/O operations to the NBD server, which handles the I/O operations." — S71, `VZNetworkBlockDeviceStorageDeviceAttachment`
- Quote: "WinHvPlatform.dll (located in the System32 folder) exports a set of C-style Windows API functions which return HRESULT error codes." — S72
- Quote: "ublk is a generic framework for implementing block device logic from userspace." — S73
- Conditions/notes: Apple's NBD attachment is available from macOS 14.0 (S71 metadata) and belongs to Virtualization.framework, not Hypervisor.framework. Mechanism requirements from the sources: eStargz and SOCI mount through FUSE (S14, S24); Nydus fanotify needs Linux 6.15+ and ublk 6.0+ (S36); overlaybd needs TCMU (S41); Firecracker's vhost-user needs memfd and a Unix socket (S60); composefs needs overlayfs and fs-verity (S50). Formats whose logic is plain userspace code over ranged reads: Lambda's chunk manifest (S54), overlaybd's LSMT/ZFile (S40, S42), an EROFS image or tar index served as a flat device (S36, S52), casync/desync block indexes (S105). Nydus v2 documents a macOS FUSE port ("Nydus can also run with macfuse(a.k.a osxfuse).", S34), which needs a third-party kernel extension. The statement that WHP and Hypervisor.framework provide no block layer is an inference from their API scope; no source says it in one sentence.
- Relevance: for KVM, Hypervisor.framework and WHP with one code path: a block-level image (flattened or EROFS) plus a chunk index, served by an in-process virtio-blk backend, with NBD as the out-of-process fallback; the guest kernel does the filesystem.

### C. Content-defined chunking and deduplication

### F62. FastCDC (ATC'16) combines a Gear rolling hash, a simplified hash test, skipping of sub-minimum cut points and normalized chunking, and reports about 10× the speed of Rabin CDC and 3× that of Gear and AE at nearly the same dedup ratio.
- Quote: "FastCDC is about 10x faster than the best of open-source Rabin-based CDC, and about 3x faster than the state-of-the-art Gear- and AE-based CDC" — S74, abstract
- Quote: "A common configuration of the average, minimum, and maximum parameters follows that used by LBFS [25], i.e., 8KB, 2KB, and 64KB." — S74, §4.3
- Conditions/notes: chunk-size parameters were inherited from LBFS, not derived from the data. Cut-point skipping alone "increases the chunking speed but decreases the deduplication ratio (about 15% decline in the worst case)" (§1), which normalized chunking repairs by using more mask bits before the expected size and fewer after: "more than 13 effective mask bits are used for hash judgment (e.g., fp & 0x7fff==r) when the current chunking position is smaller than 8KB" (§4.4). Evaluation: seven datasets (tar, Linux sources, web snapshots, two VM image sets, Redis backups, synthetic), Intel i7-4770 at 3.4 GHz and i7-930 at 2.8 GHz, in-memory chunking speed.
- Relevance: the baseline algorithm; its 8 KB average is a backup-system convention, far below what image stores use (64 KiB to 2 MiB).

### F63. The 2020 journal version of FastCDC adds rolling two bytes at a time and reports 3–12× over other CDC approaches and 1.2–3.0× higher end-to-end throughput in the Destor dedup system.
- Quote: "FastCDC is 3-12X faster than the state-of-the-art CDC approaches, while achieving nearly the same and even higher deduplication ratio as the classic Rabin-based CDC" — S75, abstract
- Quote: "FastCDC helps achieve 1.2-3.0X higher throughput than Destor based on state-of-the-art chunkers" — S75, abstract
- Conditions/notes: five techniques, the fifth being "rolling two bytes each time to further speed up CDC" (abstract). The end-to-end gain is much smaller than the chunking gain because fingerprinting and indexing remain.
- Relevance: chunking speed stops mattering once hashing dominates; budget CPU for the hash (F69) first.

### F64. VectorCDC (FAST'25) vectorises hashless chunkers (RAM, AE) with SSE/AVX and reaches 24–26 GB/s, against 2 GB/s for native FastCDC on the same machine, with space savings within 6% of hash-based algorithms.
- Quote: "VRAM, our vector-accelerated RAM implementation, achieves 24-26 GB/s, 21× and 46× faster than SS-GEAR and SS-CRC respectively." — S76, §5.2
- Quote: "The fastest among the native hash-based algorithms are FastCDC [15], Gear [16], and CRC [24] achieving 2 GB/s, 0.95 GB/s, and 0.4 GB/s respectively." — S76, §5.2
- Counter: hash-based FastCDC gains nothing from vectorising, because decoupling the hash phase removes minimum-size skipping — "VectorCDC achieves 12× higher throughput than FastCDC." — S76, §5.2
- Conditions/notes: Intel Ice Lake (two 32-core Xeon Silver 4314, AVX-512), 8 KB chunks, average of 5 runs, datasets DEB (65 Debian VM images, 40 GB), DEV, LNX, RDS, TPCC. "RAM and AE achieve space savings values within 6% of hash-based algorithms on all datasets and chunk sizes" (§5.1). Abstract figure: "21 − 46×" higher throughput than existing vector acceleration. AVX-512 results; SSE-128 and AVX-256 give 8.7× and 12.3× over native RAM. ARM NEON was not evaluated in this paper.
- Relevance: at these rates chunking is free relative to hashing; but see F65 for RAM's failure on low-entropy data and the AE parameter error.

### F65. Two independent benchmark studies (2024 and October 2026, same group) rank Gear with normalized chunking and AE as the practical choices, find RAM unreliable on low-entropy data, and report that the commonly used AE parameterisation is wrong.
- Quote: "we recognize Gear with NC and AE as the most attractive choice for CDC, despite more recent advancements in algorithm development" — S77, conclusion
- Quote: "On datasets with particularly low entropy, such as text-based data (CODE), the algorithm fails to find chunk boundaries, resulting in the generation of pathologically large chunks." — S77, summary (about RAM)
- Quote: "we uncover a longstanding error in the widely-used AE parameterization reproduced across multiple papers and evaluation tools that produces chunks 13-39% smaller than intended and inflates AE's reported deduplication" — S78, abstract
- Conditions/notes: S77 uses four realistic datasets and four metrics; normalized chunking reduces variance at little cost: "we find that the otherwise high variance in chunk size can be mitigated without impairing the deduplication ratio" (S77). Fixed-size chunking in the same harness "unsurprisingly outperforms all CDC contenders at more than 2.5 GiB/s of processed input" (S77 §6.2.4). S78 concludes "AE is the most balanced base chunking algorithm across all four metrics" after correcting the parameters. Both are preprints (S77 submitted to IEEE TCC).
- Relevance: prefer Gear/FastCDC-style with normalized chunking and explicit min/max; any published AE or RAM dedup ratio before October 2026 needs re-checking against S78.

### F66. Other accelerations: RapidCDC skips chunking for data that repeats (up to 33×), AE is the original hashless chunker (3× over prior CDC), and SeqCDC targets the large chunk sizes production systems prefer.
- Quote: "RapidCDC's chunking speedup can be up to 33x higher than regular CDC" — S79, RapidCDC abstract
- Quote: "AE improves the throughput performance of the state-of-the-art CDC algorithms by 3x while attaining comparable or higher deduplication efficiency" — S79, AE abstract
- Quote: "While deduplication systems deployed in production favor larger chunk sizes, existing data chunking algorithms are slow and offer minimal throughput increases with increasing chunk size." — S79, SeqCDC abstract
- Conditions/notes: abstract-level only (publisher pages not fetched); RapidCDC's gain is proportional to the dedup ratio by its own description.
- Relevance: RapidCDC's "next boundary is probably where it was last time" idea fits re-pushed images (F47: 80% of uploads add nothing).

### F67. Fixed versus content-defined chunking depends on whether content shifts: fixed blocks work on disk images whose layout is stable, and fail when bytes move.
- Quote: "The most aggressive chunking algorithm (8K Rabin) reclaimed between 18% and 20% more of the total file size than did whole file deduplication." — S80, results
- Quote: "The 8K fixed block algorithm reclaimed between 10% and 11% more space than whole file." — S80, results
- Quote: "XC achieves a space savings of only 37.39% on the TPCC dataset at 8KB while CDC algorithms achieve 86-87%." — S76, §5 (XC = fixed-size chunking)
- Quote: "Fixed size chunking works well!" — S82, conclusions slide (VM disk images)
- Counter: fixed chunks collapse when offsets shift — "a change to an 8-byte number will result in a new 1MB chunk being created by SteamPipe" — S92
- Conditions/notes: S80: 857 Windows desktops; "by far the largest source of duplicate data is VHD (virtual hard drive) files" among sub-file duplicates, and for whole-file duplicates "dll, lib, pdb, exe, cab, msp, and msi together make up 58% of the saved space"; for compiled outputs "Rabin chunking may find blocks of code (or symbols) that move somewhat in the file due to code changes that affect the length of previous parts of the file." S82: "Deduplication is efficient for virtual machines, often saving 50–80%" with fixed and variable chunks of 512–4096 bytes on OS images. S76 Table 1 fixed-size savings at 8 KB: DEB (VM images) 18.98%, DEV 83.17%, LNX 19.87%, RDS 33.54%, TPCC 37.39%. Lambda gets its dedup with fixed 512 KiB chunks because flattening is deterministic (F46, F47). SteamPipe uses fixed chunks of about 1 MB and tells developers to align pack-file contents to 1 MB.
- Relevance: block-device images (ext4/EROFS built deterministically, files block-aligned) can use fixed chunks, which also give O(1) offset-to-chunk lookup; tar streams and executables, where insertions shift everything after them, need CDC. A registry serving both needs both.

### F68. Chunk size is a three-way trade between dedup, metadata and request count; the one study that derives it from data finds roughly 64–80 KB with compression matches 4 KB chunks, and image systems choose much larger units for index cost.
- Quote: "an average chunk size of about 80KB (and size range 32-128KB) with compression yields savings comparable to 4KB average chunk size with compression" — S81, system design section
- Quote: "High deduplication savings previously obtained using small ∼4KB variable length chunking are achievable with 16-20x larger chunks, after chunk compression is included" — S81, §1
- Counter: cross-server (global) dedup added little in that study — "We find that the additional savings of cross-server deduplication to be no more than 12% above per-server savings." — S81, data analysis section
- Conditions/notes: S81: 15 file servers, over 2000 users, primary (not backup) data; the mechanism is that larger chunks compress better, cancelling the lost duplicates. Lambda at 512 KiB reports a manifest of "less than 3MiB for a 16GiB container image, or 0.02% overhead" (S54), i.e. under 96 bytes per chunk (derived: 16 GiB / 512 KiB = 32,768 chunks). Nydus v3 moved from 1 MB chunks to groups of 2 MiB and more because smaller chunks were "too small to deduplicate at an affordable cost" (S35). Slacker measured 4 KB block dedup at 2.8 versus file dedup at 2.6 across 57 images (S10). F9: half of image files are under 4 KB. No study was found that derives a chunk size from container-image or VM-image data with index cost included.
- Relevance: an open measurement for the new registry: dedup ratio and index bytes per stored byte as a function of chunk size (16 KiB to 4 MiB) on real image corpora, for fixed and CDC cuts; every deployed constant (64 KiB, 512 KiB, 1 MB, 2 MiB, 4 MiB) is a judgement.

### F69. Hashing cost: BLAKE3's official benchmark is 6866 MiB/s single-threaded against 484 MiB/s for OpenSSL SHA-256 on a CPU without SHA extensions; with SHA extensions OpenSSL's own tables put SHA-256 at about 2 cycles per byte on x86 (Ryzen) and ARMv8.
- Quote: "On Intel Cascade Lake-SP, peak single-threaded throughput is 4× that of BLAKE2b, 8× that of SHA-512, and 12× that of SHA-256" — S84, abstract
- Quote: "Performance on AWS c5.metal, 16 KiB input, 1 thread" — S83, chart title in `bar_chart.py` (values: BLAKE3 6866, SHA-256 484, unit "Speed (MiB/s)")
- Quote: "Ryzen 11.0 9.02(+22%) 2.05(+440%) 7.05 5.67(+20%)" — S85, x86-64 table row; columns SHA256, SSSE3, AVX/XOP(*), SHA512, AVX/XOP(*), in cycles per processed byte
- Quote: "(*) whichever best applicable, including SHAEXT;" — S85
- Quote: "Cortex-A53 2.38 15.5 (+115%) 10.0 (+150%(***))" — S85, ARMv8 table row; columns SHA256-hw, SHA256(*), SHA512
- Conditions/notes: the BLAKE3 comparison machine is an Intel Xeon Platinum 8275CL using AVX-512, with SHA-2 "provided by OpenSSL" (S83); that CPU generation has no SHA-NI (background knowledge, not stated in the source), so the 12× gap does not hold on CPUs with SHA extensions. OpenSSL rows: SHA-256 with SHA extensions 2.05 cpb (Ryzen) and 4.16 cpb (Goldmont) against 11.0 and 18.9 scalar; ARMv8 hardware SHA-256 1.92–2.54 cpb across Apple A7, Cortex-A53/A57, Denver, Mongoose, Kryo, ThunderX2. BLAKE3 can also use many cores for one input; SHA-256 cannot. No official single source comparing BLAKE3 and SHA-NI SHA-256 on the same recent x86 and ARM machines was found.
- Relevance: OCI requires SHA-256 for blob digests, so SHA-256 must be computed anyway; whether a second (BLAKE3) per-chunk hash pays off must be measured on the deployment CPUs.

### F70. BLAKE3 is a Merkle tree over 1 KiB chunks, which gives verified streaming of arbitrary ranges at 6.25% hash overhead; Bao specifies combined, outboard and slice encodings for it.
- Quote: "the space overhead of the parent nodes approaches two CVs per chunk, or 6.25%" — S84, §6.4
- Quote: "The outboard encoding format is the same as the combined encoding format, except that all the chunks are omitted." — S86, spec
- Conditions/notes: "BLAKE3 uses 1 KiB chunks." chosen as "a tradeoff between the peak throughput for long inputs" and parallelism (S84 §7.1), with "implementations with 1 KiB chunks can reach at least 90% of that peak throughput". Overhead falls proportionally if the tree is truncated to larger leaf groups (derived). The Bao README names "A BitTorrent-like application" fetching slices from different peers as a use case. BitTorrent v2 does the same with SHA-256 and 16 KiB leaves (F81).
- Relevance: one root hash per blob plus an outboard tree lets any peer or cache serve any byte range verifiably, which per-chunk digest lists (eStargz, SOCI) only do at chunk granularity.

### F71. Cross-user deduplication is a side channel: whether an upload was deduplicated reveals whether someone else already stored the content, and can be used as a covert channel.
- Quote: "We demonstrate how deduplication can be used as a side channel which reveals information about the contents of files of other users." — S87, abstract
- Conditions/notes: the attacks need client-side (source-based) dedup or any observable difference between "new" and "known" uploads. The paper's mitigation is a per-file random threshold of copies before dedup becomes visible: "We therefore propose simple mechanisms that enable cross-user deduplication while greatly reducing the risk of data leakage."
- Relevance: a multi-tenant registry must not let upload behaviour (skipped PUT, mount success, timing) depend on other tenants' content; dedup can stay server-side and invisible.

### F72. Convergent (message-locked) encryption allows confirmation-of-file and learn-the-remaining-information attacks on low-entropy content; the known defences are a secret mixed into the key derivation or a key server.
- Quote: "We show that it suffers also from a learn-the-remaining-information attack." — S89, abstract
- Quote: "instead of symmetric_key = H(plaintext), you have symmetric_key = H(added_secret, plaintext)" — S89, "Defense Against Both Attacks"
- Quote: "it is inherently subject to brute-force attacks that can recover files falling into a known set" — S88, DupLESS abstract
- Conditions/notes: formal treatment as Message-Locked Encryption: "where the key under which encryption and decryption are performed is itself derived from the message" (S88, ePrint 2012/631 abstract). In S89 the set of parties sharing the added secret is exactly the set whose data deduplicates together. Lambda's salt (F48) is such a secret used for blast radius; the paper presents it as operational, with keys held in per-customer manifests. Lambda also avoids compress-then-encrypt for the size side channel.
- Relevance: if chunks are encrypted convergently, scope the convergence secret per tenant or trust domain by default and make wider sharing an explicit choice.

### F73. Cross-repository blob mount is an existence oracle unless the registry checks read access to the source: the OCI spec lets a registry mount without a `from` repository, and success (201) and fallback (202) are distinguishable.
- Quote: "The registry MAY treat the `from` parameter as optional, and it MAY cross-mount the blob if it can be found." — S90, "Mounting a blob from another repository"
- Quote: "A blob may be mounted from another repository that the client has read access to, removing the need to upload a blob already known to the registry." — S91, "Cross Repository Blob Mount"
- Quote: "the registry will fall back to the standard upload behavior and return a `202 Accepted` with the upload URL in the `Location` header" — S91
- Conditions/notes: the OCI text does not mention authorisation for the mount; the access condition appears in the CNCF Distribution text. The oracle itself (a tenant probing digests it cannot read) is an inference from these two texts combined with F71; no primary source documenting an exploited cross-tenant mount probe was found.
- Relevance: mount only when the caller can read the named source repository; never auto-mount across tenants; make the "unknown digest" and "known but unauthorised" responses identical.

### D. Peer-to-peer image distribution

### F74. Uber Kraken: agents on every host, dedicated origin seeders on a hash ring backed by blob storage, a tracker that only builds the connection graph, a proxy for pushes and a build-index for tags; it reports 1 million blobs a day in its busiest cluster and 20K blobs of 100MB–1G distributed in under 30 seconds.
- Quote: "A central component, the tracker, will orchestrate all participants in the network to form a pseudo-random regular graph." — S93, "Design"
- Quote: "In our busiest cluster, Kraken distributes more than 1 million blobs per day, including 100k 1G+ blobs. At its peak production load, Kraken distributes 20K 100MB-1G blobs in under 30 sec." — S93
- Quote: "a 3G Docker image with 2 layers is downloaded by 2600 hosts concurrently (5200 blob downloads), with 300MB/s speed limit on all agents (using 5 trackers and 5 origins)" — S93, "Benchmark"
- Conditions/notes: benchmark result p50 = 10s, p99 = 18s, p99.9 = 22s (S93). Scale claims differ by date: "Supports at least 15k hosts per cluster." (S93, 2026 README) versus "at least 8,000 hosts per cluster" (S95, 2019). Origin role: "Forms a self-healing hash ring to distribute the load" (S93). Theoretical bound: "all participants can reach a minimum of 80% max upload/download speed in theory (60% with current implementation)" with a single seeder (S93). Vendor numbers; the repository is still released (v0.1.37, 2026-11-16).
- Relevance: the tracker-plans, peers-negotiate split keeps the control plane out of the data path; the 5-origins-for-2600-hosts benchmark is the nearest thing to a published origin-load figure.

### F75. Kraken uses 4MB pieces for all blob sizes and verifies each piece with CRC32, not a cryptographic hash; a peer that sends a bad piece is not asked again for it and connections can be blacklisted for 30 seconds by default.
- Quote: "0: 4MB # Use 4MB piece lengths for all file sizes (for now)." — S94, `config/origin/base.yaml`
- Quote: "PieceHash returns the hash used to sum pieces." — S94, `core/piece_hash.go` (the function returns `crc32.NewIEEE()`)
- Quote: "BlacklistDuration is the duration a connection will remain blacklisted." — S94, connstate config (default `30 * time.Second`)
- Conditions/notes: the agent rejects a piece whose CRC32 differs from the metainfo ("invalid piece sum", S94) and marks the request invalid so it is not resent to the same peer (dispatcher code, same tag). The README's security statement is transport-level: "Support uploader authentication and data integrity protection through TLS." (S93). End-to-end protection therefore rests on the image client checking the blob's SHA-256 after assembly and on all peers being trusted hosts; a malicious peer inside the TLS trust domain can forge CRC32-valid pieces (inference from the code, not stated by the project). `max_open_conn` per torrent defaults to 10.
- Relevance: avoid; pieces must be verifiable against the blob digest before they are served onward (Merkle proofs as in F81 or per-chunk digests from a verified manifest).

### F76. Counter-evidence from Kraken itself: P2P does not help when the registry is not the bottleneck, because decompression dominates pulls; blobs are limited to 20G in practice and tags have no consistency guarantee.
- Quote: "If Docker registry throughput is not the bottleneck in your deployment workflow, switching to Kraken will not magically speed up your `docker pull`." — S93, "Limitations"
- Counter: independent measurement in a burst scenario — "Kraken takes 100.4 seconds to launch 128 containers." — S56, §4
- Conditions/notes: "as `docker pull` spends most of the time on data decompression" (S93); "We normally limit max size to 20G for the best performance." (S93); build-index: "No consistency guarantees: the client should use unique tags" (S93).
- Relevance: P2P addresses origin bandwidth, not start-up latency; combine it with lazy loading or it moves the bottleneck to decompression.

### F77. Dragonfly (CNCF graduated, January 2026) has a manager, a scheduler that builds a directed acyclic graph of peers and orders back-to-source downloads, and clients that can act as seed peers.
- Quote: "Build a scheduling directed acyclic graph for the P2P cluster." — S97, "Scheduler"
- Quote: "In the case of scheduling failure, notice peer back-to-source download." — S97, "Scheduler"
- Conditions/notes: seed peer: "which is the root peer for download in the entire cluster" (S97). Kraken's criticism — "Dragonfly cluster has one or a few "supernodes" that coordinates the transfer of every 4MB chunk of data in the cluster." (S93) — describes the older supernode design, not the current scheduler/seed-peer split. Security model (mutual TLS, piece authentication): not verified from primary docs in this pass; the architecture page lists "Support user RBAC and multi-tenant isolation."
- Relevance: scheduler-directed (not gossip) peer selection with an explicit back-to-source path is the design to compare against Kraken's graph.

### F78. Dragonfly's current Rust client sizes pieces from the content length (target at most 500 pieces, power of two, clamped to 4 MiB–64 MiB) and checks pieces with CRC32.
- Quote: "pub const MAX_PIECE_LENGTH: u64 = 64 * 1024 * 1024;" — S96, `piece.rs`
- Quote: "pub const MIN_PIECE_LENGTH: u64 = 4 * 1024 * 1024;" — S96, `dragonfly-client-config/src/lib.rs`
- Quote: "let digest = Digest::new(Algorithm::Crc32, response.hash);" — S96, `dragonfly-client-storage/src/lib.rs`
- Conditions/notes: `const MAX_PIECE_COUNT: u64 = 500;`; piece length = `(content_length / MAX_PIECE_COUNT).next_power_of_two()` clamped to the range (S96). The code comment states the intent (keep the piece count bounded); no measurement is cited. As with Kraken, CRC32 detects corruption, not forgery.
- Relevance: a size-dependent piece length bounds metadata per blob; replace CRC32 with digests tied to the blob.

### F79. Dragonfly's published scale claims are qualitative: tens of millions of container launches per day, up to 90% storage-bandwidth saving, clusters of 10K nodes.
- Quote: "supporting tens of millions of container launches per day in production, saving storage bandwidth by up to 90%, and reducing launch time from minutes to seconds" — S98
- Quote: "the Dragonfly project has accelerated container image and code package delivery across several 10K-node Kubernetes clusters" — S98 (Ant Group statement)
- Conditions/notes: foundation press release with adopter statements; no workload, image size or baseline is given. Nydus v3's own tables (F36) show Dragonfly cutting a PyTorch image start from 24.6s to 14.9s.
- Relevance: no reproducible number; treat "up to 90%" as an upper bound claimed by the project.

### F80. Spegel is a stateless, per-node, pull-only mirror: nodes advertise the layers already in containerd's store in a libp2p Kademlia DHT and serve them to each other, falling back to the upstream registry.
- Quote: "Each instance is a member of a DHT, enabling quick lookups of images." — S99, overview
- Quote: "Spegel acts as a best-effort cache and the worst-case scenario is always that images are pulled from the upstream registry (e.g. Docker Hub)." — S99, FAQ
- Counter: it widens access to private images inside a cluster — "With Spegel that image will be available to all nodes in the cluster." — S99, FAQ
- Conditions/notes: limits stated by the project: "Currently, Spegel only works with Containerd, in the future other container runtime interfaces may be supported."; "Second of all discarding unpacked layers cannot be enabled."; "Currently, spegel advertises images with a TTL of 10 minutes." (a dead node's adverts linger); no garbage collection of its own; README: "We focus on the home lab and individual contributor use case." `go.mod` pins `github.com/libp2p/go-libp2p-kad-dht v0.42.2`. It moves whole blobs (no pieces) and relies on containerd verifying the digest it asked for. Benchmark results are published as images only (10 MB, 100 MB, 1 GB in 1 and 4 layers): numbers not found as text.
- Relevance: the zero-state design is attractive for laptops and small clusters; the authorisation gap (peers serve anything they hold) is the thing a multi-tenant design must close.

### F81. BitTorrent v2 (BEP 52) verifies with per-file SHA-256 Merkle trees over 16KiB blocks, so any block can be checked against the root with a proof and identical files share a root across torrents.
- Quote: "For non-empty files this is the the root hash of a merkle tree with a branching factor of 2, constructed from 16KiB blocks of the file." — S100, `pieces root`
- Quote: "As of meta version 2 SHA2-256 is used as digest function for the merkle tree." — S100
- Conditions/notes: piece length: "It must be a power of two and at least 16KiB." Peers exchange `hash request` / `hashes` messages carrying proof layers (S100).
- Relevance: the standard precedent for piece-level cryptographic verification before forwarding; the tree root is not the OCI SHA-256 blob digest, so a registry must publish and sign the mapping.

### F82. Academic P2P and shared-storage image distribution results: FID (97% less registry traffic), CoMICon (28% faster provisioning), Wharf (up to 12× faster retrieval from a shared file system), Cider (85% and 62% shorter deployment).
- Quote: "FID reduces at least 97% of network traffic for Docker Registry. Furthermore, it reduces 83.50% of distribution time on average when distributing images among 200 nodes" — S101, FID abstract
- Quote: "CoMICon can increase the number of highly available images up to 3× while reducing the application provisioning time by 28% on average" — S101, CoMICon abstract
- Quote: "Wharf can speed up image retrievals by up to 12x" — S102, abstract
- Quote: "Cider can shorten the overall deployment time by 85% and 62% on average when deploying one container and 100 concurrent containers respectively" — S101, Cider abstract
- Conditions/notes: abstract-level only (full texts not fetched); FID also "reduces up to 91.35% of distribution time for the 500M image of Hadoop". Wharf and Cider are shared-storage (not P2P) designs: images live once on network storage and nodes load on demand.
- Relevance: the 97% registry-traffic reduction at 200 nodes is the only explicit origin-load figure in this literature.

### F83. How each system bounds origin load: only partial numbers are published.
- Quote: "can completely eliminate the I/O traffic to the registry, as long as a FT has at least one active VM that stores the requested container" — S56, §3.2 (FaaSNet)
- Conditions/notes: Lambda: 0.06% of chunk loads reach S3 in steady state and up to 500× that on a cold cache (F50, F51). FID: at least 97% less registry traffic at 200 nodes (F82). Kraken: 5 origins seeded 2600 hosts pulling 3G in the README benchmark; origin bytes not stated (F74). DADI: one root per datacentre fetches each block once and its cache is warmed at build time (S40 §3.5). Dragonfly: one seed peer goes back to source; "up to 90%" bandwidth saved (F79). Spegel: best effort, no bound. A figure of the form origin-serves-B-bytes-for-N-simultaneous-pullers of an S-byte image was not found for any system.
- Relevance: a target the new registry can state and test: origin egress for N concurrent cold pullers bounded by a constant multiple of the image size, independent of N.

### E. Delta and differential updates

### F84. zsync runs the rsync algorithm on the client against a precomputed checksum file over plain HTTP range requests, so the server does one-off work per file and none per client.
- Quote: "zsync uses the rsync algorithm, but implemented on the client side, so that only one-off pre-calculations are required on the server, and no special server software or new protocol is required to use zsync" — S103, technical paper
- Conditions/notes: fixed-size blocks with a rolling weak checksum on the client; one `.zsync` metadata file per published file; no dedup across versions on the server. Download-saving figures for executables: not extracted in this pass.
- Relevance: the per-file block-checksum sidecar is the simplest delta scheme a static registry can offer for a single large binary.

### F85. casync and desync publish content-defined chunks in a shared store plus a per-version index; there is no server-side work per version pair, and defaults are 16K/64K/256K (min/avg/max).
- Quote: "The "chunking" algorithm is based on a the buzhash rolling hash function. SHA256 is used as strong hash function to generate digests of the chunks. xz is used to compress the individual chunks." — S104
- Quote: "The tool's default chunk sizes match the defaults used in casync: min 16KB, avg 64KB, max 256KB." — S105, concepts
- Conditions/notes: casync source: `#define CA_CHUNK_SIZE_AVG_DEFAULT ((size_t) (64U*1024U))`, rolling window 48 bytes (S104). Stated motivation for chunk stores over pairwise deltas: "keeping deltas between all version combinations clients might want to run updates between, would suggest keeping an exponentially growing amount of deltas on servers" (S104). desync identifies chunks by SHA512-256, compresses with zstd, and can keep chunks and indexes in an OCI registry (S105). No justification is given for 64K.
- Relevance: the chunk-store model is the same object model as lazy loading (index + content-addressed chunks), so one mechanism can serve both updates and on-demand reads.

### F86. desync's 2026 measurement on a VM disk image: updating a Debian 12 cloud image costs 109.8 MB from a build two days older and 167.8 MB from a build a year older, against 292.2 MB from nothing.
- Quote: "Six builds of the Debian 12 `genericcloud` disk image, 3.2 GB raw with about 1 GB of data, published to one store." — S105, README "What an update costs"
- Quote: "All six rows are served from the same store, which holds every build in 832 MB, against 1,753 MB for six separate copies." — S105
- Counter: chunk reuse disappears when the payload is compressed or encrypted as a whole — "A compressed or encrypted payload, for example, changes throughout when a single byte of its input does." — S105
- Conditions/notes: download sizes are compressed chunks; 7,749 of 11,708 chunks reused for the two-day case; intermediate rows 112.2 MB (3 weeks), 128.5 MB (2 months), 165.4 MB (6 months). Project's own numbers, default chunking, one image family.
- Relevance: about 62% saved for near-consecutive OS images (derived from 109.8 / 292.2) with no per-pair computation; a floor to beat for kernel+rootfs pairs.

### F87. bsdiff needs a per-pair computation and much memory, and gives the smallest generic patches for executables: 58.3× for FreeBSD security updates and 11.6× on average for feature updates in its paper.
- Quote: "BSDiff produces patches totalling 621277 bytes, a reduction by a factor of 58.3" — S106, paper §3
- Quote: "It requires max(17*n,9*n+m)+O(1) bytes of memory, where n is the size of the old file and m is the size of the new file. bspatch requires n+m+O(1) bytes." — S106, home page
- Conditions/notes: corpus for the 58.3× figure: "there are 97 modified binaries, with a total size of 36397575 bytes" (FreeBSD 4.7 security branch, i386); feature updates on 19 pairs of DEC Alpha binaries: "BSDiff gives 11.6-fold compression on average" against 2.8-fold for bzip2 alone. "bsdiff routinely produces binary patches 50-80% smaller than those produced by Xdelta" (home page). Time: "on a 200MHz Pentium Pro, building a binary patch for a 4MB file takes about 90 seconds" (2003 hardware). Client memory is old plus new size.
- Relevance: best ratio for small security updates of a static binary, at the price of one job per (old, new) pair and whole-file memory on the client.

### F88. Executable-aware differs do better still: Courgette shrank one Chrome update from 704,512 bytes (bsdiff) to 78,848 bytes for a 10,385,920-byte full update; Zucchini, its successor, takes about 15–20 minutes and 1 GB of RAM per Chrome patch on the server.
- Quote: "Here are the sizes in bytes for the recent 190.1->190.4 update on the developer channel:" — S108 (followed by: Full update 10,385,920; bsdiff update 704,512; Courgette update 78,848)
- Quote: "This can be expensive (e.g., ~15-20 min for Chrome, using 1 GB of RAM), but since patch generation is a run-once step on the server-side when releasing "new" binaries, the expense is not too critical." — S109
- Conditions/notes: one update pair from 2009; ratios derived here: bsdiff 6.8% and Courgette 0.76% of the full update. Mechanism: disassemble, replace internal pointers by label indexes, diff, reassemble; "Simply converting the program into the assembly language form makes the diff produced by bsdiff about 30% smaller." (S108). Zucchini supports PE, ELF and DEX on x86, x64, ARM and AArch64 (S109); its patch-size results are not in the README: not found.
- Relevance: pointer-aware diffing is where the last order of magnitude is for consecutive releases of one large executable; it requires per-pair server work and format-specific code.

### F89. `zstd --patch-from` uses the old file as a dictionary: patch sizes comparable to bsdiff at level 19, far faster, with client cost equal to a normal zstd decompression plus holding the old file.
- Quote: "--patch-from is >200X faster on level 1 and >100X faster (shown below) on level 3 vs bsdiff while still delivering patch sizes less than 0.5% of the original file size" — S107, v1.4.5 release notes
- Quote: "it also increases the previous maximum limit for dictionaries from 32 MB to 2 GB" — S107
- Conditions/notes: benchmark corpus is source-code tarballs of adjacent versions (zstd, WordPress, LLVM, Linux), not executables; "--patch-from on level 19 (with chainLog=30 and targetLength=4kb) is comparable with bsdiff when comparing patch sizes" (release notes). Wiki: "In terms of memory footprint, `level 19 --patch-from` also significantly outperforms `bsdiff`." Requires a per-pair patch on the server and the long-distance matcher window on both sides.
- Relevance: the cheapest per-pair delta to produce on demand and cache; a reasonable default before format-aware differs.

### F90. OSTree static deltas are per-commit-pair objects (bsdiff and rsync-style inside) generated on the server to keep a static-web-server model.
- Quote: "Effectively, we're spending server-side storage (and one-time compute cost), and gaining efficiency in client network bandwidth." — S110
- Conditions/notes: "These deltas are targeted to be a delta between two specific commit objects, including "bsdiff" and "rsync-style" deltas within a content object." (S110). Without a delta the client falls back to per-object fetches (file-level dedup).
- Relevance: pairwise deltas only for the pairs that matter (N-1 → N), chunk or object fetch for everything else.

### F91. Nix binary caches with chunk stores (nix-casync, Attic) chunk each uncompressed NAR at about 64 KiB average and deduplicate globally; neither publishes saving figures.
- Quote: "Files will be decompressed, chunked, and put in a content-addressed store." — S111, nix-casync README
- Quote: "avg-size = 65536 # 64 KiB" — S111, Attic config template
- Conditions/notes: Attic defaults: `nar-size-threshold = 65536`, `min-size = 16384`, `max-size = 262144`, with the warning that changing them breaks reuse of existing chunks. Attic exposes tenants as "restricted views of the content-addressed NAR Store and Chunk Store" (README), i.e. global dedup under per-tenant visibility. Download or storage reductions: not found.
- Relevance: the tenant-view-over-global-chunk-store model matches a multi-tenant registry; F71–F73 apply to it.

### F92. Container-image deltas: tar-diff (bsdiff + zstd over tar contents, stored as OCI artifacts) reports deltas of 10–15% of layer size in the worst cases; balena computes image-level binary deltas on its servers per release pair.
- Quote: "even for those the deltas are in the realm of 10-15% of the original size. So, even in the worst case deltas are giving around 10x improvement." — S112, blog
- Quote: "In the worst case (i.e. completely replaced image) the binary delta is equal size to the Docker pull (the device needs to download the full image)." — S113
- Conditions/notes: "tar-diff uses knowledge of the tar format, as well as the bsdiff binary diff algorithm and zstd compression to create very small files for typical updates." (S112); the table of per-image results in the blog is an image: numbers not found as text. tar-patch needs the extracted old layer on the client. The proposal never became part of the OCI specs; the shipped alternative in the same ecosystem is zstd:chunked partial pulls (F32), which need no per-pair objects. tar-diff is still maintained (v0.6.2, 2026-11-14).
- Relevance: per-pair layer deltas can be published as referrer artifacts without any client-visible format change; chunk-level reuse covers arbitrary version pairs.

### Recency check

### F93. What changed in 2025–2026: lazy loading moved away from an index beside an unmodified image and from fscache, toward bound indexes, EROFS blobs and block transports, and several long-standing constants were revised.
- Quote: "Erofs snapshotter and differ (#10705)" — S53, containerd v2.1.0 release notes (2025-05-07)
- Quote: "This branch is Nydus v3, a different image format: v3 does not read v2" — S35 (2026-11-20)
- Conditions/notes: dated changes established above: SOCI Index Manifest v2 announced for Fargate on 2025-07-03, with v1 off by default from snapshotter v0.10.0 (F27); SOCI parallel pull (2025-08-27, F30); SOCI design preprint with the 80% crossover (2026-07-07, F28); SOCI on Lambda Managed Instances (date not established, F31); VectorCDC (February 2025, F64) and the AE parameterisation correction (2026-10-09, F65); Dragonfly CNCF graduation (2026-01-14, F77); Fly.io Sprites and BuildBuddy chunked snapshots (January 2026, F57, F58); erofs-utils 1.9 building from OCI registries and S3 (2026-02-18) and mounting through fanotify and ublk (2026-07-02, F43); EROFS page-cache sharing in Linux 7.0 and the fscache mode gone from the 7.2 documentation (F42); Firecracker virtio-pmem (v1.14.0) with vhost-user block still in developer preview at v1.18.0 (F54, F55); Kata 4.4.0 documenting EROFS block pass-through (F45); Nydus v3 release candidate with content-defined 2 MiB chunk groups (F35).
- Relevance: designs built on fscache, on SOCI v1 referrer discovery or on AE/RAM dedup figures published before late 2026 rest on superseded ground.

## Comparison tables

### 1. Lazy formats

The cross-platform column asks whether the read path can run inside a userspace VMM block backend on macOS and Windows without host kernel features; a cell saying format means the on-disk logic is plain userspace code even if the shipped transport is Linux-only. Speed-ups are the sources' own, under the conditions in the finding.

| Format | Index location | Chunk granularity (default) | Integrity | Unmodified client compat | Kernel/runtime needs | Cross-platform? | Published speedup |
|---|---|---|---|---|---|---|---|
| eStargz (F18–F22) | TOC JSON as last tar entry in the layer; optional external TOC image | file; large files cut at 4 MiB; fetch unit 50000 bytes | TOC digest in layer descriptor annotation; SHA-256 `chunkDigest` per chunk checked on fetch | yes (valid gzip tar; converted layer has a new digest) | FUSE (passthrough from Linux 6.9); stargz snapshotter | no as shipped (FUSE); file-level, so a guest needs the snapshotter inside it | not found as text; counter: slower than full pull at RTT ≥ 150 ms (S55) |
| SOCI v1 / v2 (F24–F31) | separate OCI artifact (index manifest + zTOC per layer); v1 by referrers API, v2 bound through an image index | span of compressed stream, 4 MiB; layers under 10 MiB not indexed | digest per span in zTOC; v1 link mutable, v2 immutable binding | v1 yes (image untouched); v2 yields a new image digest, layers unchanged | FUSE per layer; SOCI snapshotter | no as shipped (FUSE); index is format-only and reusable (mkfs.erofs `--gzinfo`) | 129 s → 60 s one PyTorch image on Fargate (S23); 7.4× on 1.3 GB image, crossover at 80% access (S27, preprint) |
| zstd:chunked (F32–F33) | TOC in zstd skippable frames at end of layer; position and checksum in layer annotations | file, with rolling-checksum cuts (2^16 expected, derived) | SHA-256 per file and chunk in TOC; TOC checksum annotation | yes, read as plain zstd | none for partial pull; composefs optional | n/a (partial pull, not start-before-download) | "as much as 90%" pull speed-up, unqualified (S32) |
| Nydus v2 / RAFS v6 (F34) | separate bootstrap (metadata) layer | fixed 1 MB chunks | per-chunk SHA-256 or BLAKE3 in metadata | no | FUSE/virtiofs, or EROFS + fscache (5.19–6.12, deprecated) | FUSE path only (macFUSE port documented) | vendor figure only (image), not found as text |
| Nydus v3 rc (F35–F38) | bootstrap embedded in each layer blob (zstd) plus optional standalone bootstrap | content-defined chunk groups, 2 MiB minimum; fetch 2 MiB | CRC32C per group, optional BLAKE3 per group, SHA-256 of blob = layer digest | no | FUSE; or kernel EROFS via NBD (6.4), ublk (6.0/6.4), file-backed (6.12), fanotify (6.15); virtio-pmem + userfaultfd for microVMs | core library is userspace; NBD transport is portable; uffd/ublk/fanotify are Linux | Kata cold start 22.62 s → 8.94 s, 4.04 GB image (vendor, rc) |
| DADI / overlaybd (F39–F41) | per-layer block index (LSMT) + ZFile jump table inside the layer blob; TurboOCI: separate meta image (~3%) | 512-byte device blocks; ZFile compression block 4096 bytes | CRC32 per block (checksum file / ZFile); no digest chain | no (TurboOCI keeps the OCI image and adds an artifact) | TCMU kernel module; ublk planned | format yes (block-level, userspace library); transport no | 10,000 containers on 1,000 hosts within 4 s (S40); 6.433 s vs 15.341 s WordPress (S43) |
| EROFS layer / tar index (F42–F45) | EROFS metadata; tar-index mode: small index + original tar, index storable in registry | block 4096 (512 in tar-index mode); `--chunksize` for chunk-based files | fs-verity or dm-verity Merkle tree per layer blob; tar diffID still checkable in tar-index mode | tar-index mode yes (original tar kept); native EROFS layers no | Linux ≥ 5.4 guest or host; file-backed mounts 6.12; fscache gone by 7.2 | yes for guests (image is a block device; guest kernel mounts it); mkfs.erofs runs on macOS | unpack benchmark as images only, not found as text |
| composefs (F44) | EROFS metadata image + content-addressed file store | whole file | fs-verity digest per file in overlay xattrs | via zstd:chunked or on-the-fly conversion | overlayfs + EROFS + fs-verity | no | none found |
| Lambda block loader (F46–F52) | encrypted per-function chunk manifest, outside the image | fixed 512 KiB of a flattened ext4 image | SHA-256 per chunk in manifest; convergent AES-CTR; AES-GCM over manifest | no (server-side conversion; users push ordinary images) | virtio-blk; agent via FUSE (being replaced by userfaultfd + mmap) | yes in principle (pure userspace chunk map behind virtio-blk) | no speed-up ratio; 67% / 32% / 0.06% tier split, cache hit > 99.8% |
| Starlight (F23) | proxy-computed delta bundle | file | not examined here | needs Starlight proxy and snapshotter | FUSE-based snapshotter | no | 3.0× vs baseline, 1.9× vs eStargz |
| FaaSNet block format (F53) | offset table per layer | fixed 512 KB compressed blocks | not stated | no | Alibaba-internal | n/a | 2,500 containers on 1,000 VMs in 8.3 s |

### 2. P2P systems

| System | Topology | Piece size | Integrity | Origin load bound | Published scale |
|---|---|---|---|---|---|
| Kraken (F74–F76) | tracker forms a pseudo-random regular graph; origins on a hash ring seed | 4MB for all sizes | CRC32 per piece; TLS between components; blacklist 30 s | not stated; benchmark used 5 origins for 2600 hosts | 1 million blobs/day; ≥ 15k hosts/cluster (2026 README); 3G image to 2600 hosts p50 10s, p99 18s |
| Dragonfly (F77–F79) | scheduler builds a DAG; seed peer is the root and goes back to source | content length / 500 rounded to a power of two, clamped 4 MiB–64 MiB | CRC32 per piece | one back-to-source per task via seed peer (design); "up to 90%" bandwidth saved (claim) | "tens of millions of container launches per day"; 10K-node clusters |
| Spegel (F80) | Kademlia DHT (libp2p) of per-node mirrors | whole blob | containerd's digest check of the requested blob | none (best effort, fall back to upstream) | numbers as images only |
| DADI P2P (F41) | per-layer tree under a per-datacentre root | on-demand blocks | CRC32 per block | root fetches each block once; cache warmed at build | 10,000 containers / 1,000 hosts in 4 s; tree projected to 100,000 containers |
| FaaSNet (F53) | per-function tree (FT), no central root | 512 KB blocks | not stated | zero registry I/O while one VM in the tree holds the image | 2,500 containers / 1,000 VMs in 8.3 s |
| FID (F82) | BitTorrent-based swarm | not found | not found | ≥ 97% less registry traffic at 200 nodes | 200 nodes |
| BitTorrent v2 (F81) | swarm (protocol only) | power of two ≥ 16KiB; Merkle leaves 16KiB | SHA-256 Merkle tree per file, proofs per block | n/a | n/a |
| Lambda cache (F50–F51), for contrast | no P2P: worker cache → AZ cache → S3 | 512 KiB chunks, erasure-coded 4-of-5 in cache | SHA-256 per chunk from manifest | 0.06% of loads steady state; up to 500× on cold cache | 15,000 container starts/s per customer |

### 3. Delta methods

| Method | Server precompute? | Typical saving (source's conditions) | Client cost |
|---|---|---|---|
| Chunk store: casync/desync (F85–F86) | none per pair; chunk once per version | Debian cloud image: 109.8 MB vs 292.2 MB from the build 2 days older; store 832 MB vs 1,753 MB for six builds | chunk local seed (CDC + hash), fetch missing chunks; index per version |
| Chunk store: zstd:chunked (F32), Nix caches (F91), SteamPipe (F67) | none per pair | zstd:chunked "as much as 90%" (unqualified); Nix: not found; SteamPipe: not found | TOC or manifest lookup, range or chunk fetches |
| zsync (F84) | one checksum file per published file | not extracted | rolling checksum over the old file; HTTP range requests |
| bsdiff (F87) | per pair; memory max(17n, 9n+m) | 58.3× on 97 FreeBSD security-update binaries; 11.6× average on feature updates | memory n+m; bzip2 decompress |
| zstd `--patch-from` (F89) | per pair; fast (>100× faster than bsdiff at level 3) | patches < 0.5% of file size on adjacent source tarballs; comparable to bsdiff at level 19 | zstd decompress with old file as dictionary (window up to 2 GB) |
| Courgette / Zucchini (F88) | per pair; ~15–20 min and 1 GB RAM for Chrome (Zucchini) | one Chrome update: 78,848 bytes vs 704,512 (bsdiff) vs 10,385,920 full | disassemble old, apply patch, reassemble; format-specific |
| OSTree static deltas (F90) | per commit pair | not found | apply delta "program"; falls back to per-object fetch |
| tar-diff / balena (F92) | per layer pair (tar-diff) or per release pair, on demand (balena) | tar-diff: 10–15% of changed-layer size in the worst cases | needs the extracted old layer; bsdiff apply + zstd |

## Catalogue of constants
| system | constant | value | where (URL) | justification given? |
|---|---|---|---|---|
| eStargz writer | chunk size for large files | 4 MiB (`4 << 20`) | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/estargz/estargz.go#L711-L715 | no |
| stargz snapshotter | fetch granularity `chunk_size` | 50000 bytes | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/fs/config/config.go#L108-L110 | no |
| stargz snapshotter | prefetch timeout | 10 s | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/fs/config/config.go#L54-L55 | no |
| stargz snapshotter | passthrough merge buffer / workers | 400MB (419430400) / 10 | https://github.com/containerd/stargz-snapshotter/blob/v0.19.1/docs/passthrough.md | no |
| SOCI | span size | 4MiB | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/cli-usage.md | no |
| SOCI | minimum layer size for a zTOC | 10MiB | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/cli-usage.md | yes, qualitative ("small layers don't benefit much from lazy loading") |
| SOCI | background fetch period / silence period / queue | 500 ms / 30000 ms / 300 entries | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/config.md | no |
| SOCI parallel pull | per-image downloads / unpacks (default) | 3 / 1 | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/parallel-mode.md | "mimic containerd's default settings" |
| SOCI parallel pull | recommended download chunk for ECR | "8mb" or "16mb", 10–20 downloads per image | https://github.com/awslabs/soci-snapshotter/blob/v0.16.1/docs/parallel-mode.md | vendor recommendation, one benchmark (S26) |
| SOCI on Fargate | image size where lazy loading starts to help | > 250 MB compressed | https://aws.amazon.com/blogs/containers/under-the-hood-lazy-loading-container-images-with-seekable-oci-and-aws-fargate/ | internal testing, no data shown |
| SOCI (preprint) | lazy vs eager crossover | 80% access density | https://arxiv.org/abs/2607.06868 | measured (preprint) |
| zstd:chunked | `RollsumBits` / rolling window / holes threshold | 16 / 64 bytes / `1 << 10` | https://github.com/containers/storage/blob/v1.59.1/pkg/chunked/compressor/compressor.go#L20-L23 ; https://github.com/containers/storage/blob/v1.59.1/pkg/chunked/compressor/rollsum.go#L28-L36 | no |
| Nydus v2 | chunk size | 1 MB (fixed) | https://github.com/dragonflyoss/nydus/blob/v2.4.6/docs/nydus-design.md | no |
| Nydus v3 | `--chunk-size` / `--chunk-group-minimum-size` / `storage.fetch_size` | 2 MiB / 2 MiB / 2 MiB compressed | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/README.md | partly (smaller chunks "too small to deduplicate at an affordable cost"; EBS dedups fixed 2 MiB blocks) |
| Nydus v3 | store group plain if compression leaves more than | 70% of payload | https://github.com/dragonflyoss/nydus/blob/v3.0.0-rc.0/docs/nydus.md | no |
| overlaybd ZFile | compression block size / max read | 4096 bytes / 65536 bytes | https://github.com/containerd/overlaybd/blob/v1.0.18/src/overlaybd/zfile/compressor.h#L38 ; https://github.com/containerd/overlaybd/blob/v1.0.18/src/overlaybd/zfile/zfile.h#L20 | no |
| DADI | device block size / P2P topology expiry | 512 bytes / 20 minutes | https://www.usenix.org/system/files/atc20-li-huiba.pdf (§3.1, §3.5) | block size "similar to real block devices"; expiry because startup is short |
| DADI | tree arity | binary below 20,000 hosts, 3-ary or 4-ary above | https://www.usenix.org/system/files/atc20-li-huiba.pdf (§5) | projected from measurement |
| EROFS | block size / default compression cluster | 4096 bytes on x86 / 1 block | https://erofs.docs.kernel.org/en/latest/mkfs.html | yes, qualitative (random-access performance) |
| containerd EROFS + dm-verity | hash block | 4096 bytes; 512 bytes in tar-index mode | https://github.com/containerd/containerd/blob/v2.4.2/docs/snapshotters/erofs.md | yes (page size; dm-verity logical block constraint) |
| AWS Lambda | chunk size | 512KiB (fixed) | https://www.usenix.org/system/files/atc23-brooker.pdf (§2) | yes, as a stated trade-off; "optimal value will change over time" |
| AWS Lambda | cache erasure code | 4 of 5 (25% overhead) | https://www.usenix.org/system/files/atc23-brooker.pdf (§4.1) | yes (tail latency vs replication cost; measured CDF) |
| AWS Lambda | AZ cache memory tier | about 10% of cache size | https://www.usenix.org/system/files/atc23-brooker.pdf (§5.1) | no |
| AWS Lambda | manifest overhead | < 3MiB per 16GiB image (0.02%) | https://www.usenix.org/system/files/atc23-brooker.pdf (§3.1) | measured |
| FaaSNet | block size | 512 KB | https://www.usenix.org/system/files/atc21-wang-ao.pdf (§4.1) | no |
| Firecracker | default block IO engine | `Sync` | https://github.com/firecracker-microvm/firecracker/blob/v1.18.0/docs/api_requests/block-io-engine.md | backwards compatibility |
| FastCDC | average / min / max chunk | 8KB / 2KB / 64KB | https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf (§4.3) | inherited from LBFS |
| Windows Server 2012 dedup (El-Shimi) | average chunk / range | about 80KB / 32–128KB | https://www.usenix.org/system/files/conference/atc12/atc12-final293.pdf | yes, derived from data (savings with compression vs chunk size) |
| casync, desync | min / avg / max chunk; window | 16KB / 64KB / 256KB; 48 bytes | https://github.com/systemd/casync/blob/v2/src/cachunker.h#L8-L12 ; https://github.com/folbricht/desync/blob/v1.1.5/docs/concepts.md | no |
| Attic | NAR threshold / min / avg / max chunk | 64 KiB / 16 KiB / 64 KiB / 256 KiB | https://github.com/zhaofengli/attic/blob/7a19204df10d606c5070e6bb72615c3461900c05/server/src/config-template.toml#L85-L105 | no |
| SteamPipe | chunk size | about 1 MB (fixed) | https://partner.steamgames.com/doc/sdk/uploading | no |
| BLAKE3 | leaf chunk | 1 KiB | https://github.com/BLAKE3-team/BLAKE3-specs/blob/ac784c9f22a48327782f042ec2f4d8126b3b1744/blake3.pdf (§7.1) | yes (≥ 90% of peak throughput, parallelism for medium inputs) |
| BitTorrent v2 | Merkle leaf block / minimum piece | 16KiB / 16KiB, power of two | https://www.bittorrent.org/beps/bep_0052.html | no |
| Kraken | piece length | 4MB for all sizes "(for now)" | https://github.com/uber/kraken/blob/v0.1.37/config/origin/base.yaml#L24-L26 | no |
| Kraken | max connections per torrent / blacklist duration | 10 / 30 s | https://github.com/uber/kraken/blob/v0.1.37/lib/torrent/scheduler/connstate/config.go | no |
| Kraken | practical blob size limit | 20G | https://github.com/uber/kraken/blob/v0.1.37/README.md | "for the best performance" |
| Dragonfly client | min / max piece length; max piece count | 4 MiB / 64 MiB; 500 | https://github.com/dragonflyoss/client/blob/v1.5.14/dragonfly-client/src/resource/piece.rs#L295-L319 | intent stated in comment (bound piece count), no measurement |
| Spegel | advertisement TTL / default memory limit | 10 minutes / 128Mi | https://spegel.dev/docs/faq/ | memory limit explained (VFS cache accounting) |
| zstd `--patch-from` | max dictionary (old file) size | 2 GB (was 32 MB) | https://github.com/facebook/zstd/releases/tag/v1.4.5 | no |
| bsdiff | memory to diff / to patch | max(17*n,9*n+m)+O(1) / n+m+O(1) bytes | https://www.daemonology.net/bsdiff/ | algorithmic |
| IBM registry cache study | memory-tier layer cutoff / SSD:memory ratio | 100 MB / 10×, 15×, 20× | https://www.usenix.org/system/files/conference/fast18/fast18-anwar.pdf (§5) | simulation parameters |

## Not verified / open questions
- Trace availability: the IBM trace download URL given in S1 (https://dssl.cs.vt.edu/drtp/) and the replayer repository were not fetched; whether they are still online in 2026 is unverified.
- No peer-reviewed registry workload study newer than the 2017 IBM and Docker Hub data was found; nothing characterises ML-era images (tens of GB) at registry scale.
- S2 and S3 give different median layer compression ratios (2.6 vs "greater than 3.5") for the same dataset; unexplained.
- eStargz, Nydus v2, containerd EROFS and Spegel publish their benchmarks as images; no numbers could be quoted as text.
- zstd:chunked "as much as 90%" (S32) has no stated conditions; no primary measurement of partial-pull savings was found.
- Nydus v3 material is a release candidate published on the research date; all its numbers are the project's own and name no hardware.
- SOCI on Lambda Managed Instances: no performance numbers; the date the feature launched was not established.
- Chunk sizes used by BuildBuddy, E2B, Fly.io Sprites and Modal: not found. CodeSandbox posts were read from Wayback Machine captures only.
- Linux 5.16 as the version for EROFS multi-device support comes from Nydus documents, not from kernel documentation.
- That Intel Cascade Lake-SP lacks SHA extensions (F69) is background knowledge, not from a fetched source; no official same-machine comparison of BLAKE3 with SHA-NI or ARMv8-CE SHA-256 was found. Throughput in bytes per second for the OpenSSL cycles-per-byte rows depends on clock speed and was not computed.
- VectorCDC reports x86 (SSE/AVX) only; ARM NEON chunking throughput was not found.
- RapidCDC, AE, SeqCDC, FID, CoMICon, Cider and Wharf were read at abstract level through the Semantic Scholar and OpenAlex APIs (publisher sites refused automated fetches); Jin and Miller through the authors' slides. The abstract texts are assumed to match the publishers' pages.
- Dragonfly's security model (mutual TLS, piece authentication between peers) and Kraken's behaviour when a peer forges CRC32-valid pieces were not verified from documentation; the statement that both rely on trusted peers plus end-of-download digest checks is an inference from code.
- Cross-tenant blob-mount probing (F73) is an inference from the specs; no advisory or paper documenting it was found.
- No published figure of the form origin-serves-B-bytes-for-N-simultaneous-pullers exists for any P2P system examined.
- Typical download reduction for consecutive releases of one large executable: only single examples exist (Chrome 2009 via Courgette; FreeBSD 4.7 via bsdiff). zsync's and Zucchini's own result tables were not extracted. No source measures CDC chunk-store savings on executables specifically.
- Whether mkfs.erofs builds and runs on Windows was not verified (macOS is supported per its ChangeLog).
- That Hypervisor.framework and the Windows Hypervisor Platform provide no storage layer is inferred from their documented scope.
- Open design measurement (F68): dedup ratio and index size versus chunk size on current container and microVM image corpora, fixed versus content-defined, with compression; no source provides it.
