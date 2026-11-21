//! What the world's tests share: the topologies they run (a laptop: one machine with the records
//! store, the bytes store and a node; a fleet: the stores on machines of their own and nodes on
//! others), the bounds and budgets they state, and the clients a generated plan becomes. Every
//! figure here is a test's statement of the run it wants, said where it is set; none is a tuning
//! of the registry, whose own figures come from plan RQ0.8.

#![allow(dead_code)]

use std::rc::Rc;
use std::time::Duration;

use hyper_sim::net::{NetLimits, Path};
use hyper_sim::{Clock, Discipline, Lateness, Limits, Random, Source};
use reliquary_api::generate::Mix;
use reliquary_api::history::Plan;
use reliquary_api::model::Config;
use reliquary_api::request::Request;
use reliquary_core::seams::bytes::Plan as UploadPlan;
use reliquary_core::seams::bytes::sim::SimLimits as BytesLimits;
use reliquary_core::seams::records::SimLimits as RecordsLimits;
use reliquary_sim::client::Policy;
use reliquary_sim::model_node::{Budgets, ModelNode};
use reliquary_sim::nemesis::Schedule;
use reliquary_sim::node::{Core, NodeLimits};
use reliquary_sim::store::StoreLimits;
use reliquary_sim::world::{self, ClientSpec, Machine, Outcome, Role, Spec};

/// 2026-11-20T00:00:00Z in nanoseconds since the Unix epoch: the wall clocks' start.
pub const WALL: u64 = 1_795_132_800_000_000_000;

pub const MS: Duration = Duration::from_millis(1);

/// A machine's clocks: no offset, no drift, timers on time.
pub fn clock() -> Clock {
    Clock {
        offset_ns: 0,
        rate_ppm: 0,
        wall_ns: WALL,
        lateness: Lateness::default(),
    }
}

/// The model node's budgets: a call may take 50 ms (a hundred LAN round trips), a log read asks
/// 64 rows, pieces of 4 KiB (the bytes store's plan), 16 tries, 32 requests at once.
pub fn budgets(grant_life: Option<Duration>) -> Budgets {
    Budgets {
        wait: MS * 50,
        page: 64,
        piece: 4096,
        tries: 16,
        jobs: 32,
        grant_life,
    }
}

/// A client waits 500 ms for an answer (ten times the node's wait for any one call, more than a
/// restarted node takes to read every log these tests make), tries an operation 8 times, and pauses
/// 10 ms between.
pub fn policy() -> Policy {
    Policy {
        patience: MS * 500,
        attempts: 8,
        pause: MS * 10,
    }
}

/// A small mix: two repositories, two principals, three tags, four contents of up to 300 bytes,
/// one request in five broken.
pub fn mix() -> Mix {
    Mix {
        repositories: 2,
        principals: 2,
        tags: 3,
        contents: 4,
        largest_blob: 300,
        invalid_one_in: 5,
        manifest_bound: None,
        staging_budget: None,
    }
}

/// Where the processes are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// One machine holds the records store, the bytes store and the node.
    Laptop,
    /// The records store, the bytes store and each of this many nodes on a machine of its own.
    Fleet(usize),
}

impl Shape {
    /// The machines that hold nodes.
    pub fn nodes(self) -> Vec<usize> {
        match self {
            Self::Laptop => vec![0],
            Self::Fleet(n) => (2..2 + n).collect(),
        }
    }

    /// The machines that hold the stores: records, then bytes.
    pub fn stores(self) -> (usize, usize) {
        match self {
            Self::Laptop => (0, 0),
            Self::Fleet(_) => (0, 1),
        }
    }

    /// The machines a nemesis may touch: every one but the clients'.
    pub fn servers(self) -> Vec<usize> {
        match self {
            Self::Laptop => vec![0],
            Self::Fleet(n) => (0..2 + n).collect(),
        }
    }
}

/// A client running `script` against the nodes of `shape`, starting on node `first` among them,
/// at once or after client `after`.
pub fn client(
    script: Vec<Request>,
    shape: Shape,
    first: usize,
    after: Option<usize>,
) -> ClientSpec {
    let mut nodes = shape.nodes();
    let len = nodes.len();
    nodes.rotate_left(first % len);
    ClientSpec {
        script,
        nodes,
        policy: policy(),
        start: Duration::ZERO,
        after,
    }
}

/// A plan's clients: its setup's client first, then each of the others once the setup is done.
pub fn clients(plan: &Plan, shape: Shape) -> Vec<ClientSpec> {
    let mut scripts: Vec<Vec<Request>> = Vec::new();
    for planned in plan.operations() {
        let k = usize::from(planned.client.0);
        if scripts.len() <= k {
            scripts.resize(k + 1, Vec::new());
        }
        scripts[k].push(planned.request.clone());
    }
    scripts
        .into_iter()
        .enumerate()
        .map(|(k, script)| client(script, shape, k, if k == 0 { None } else { Some(0) }))
        .collect()
}

/// A plan's clients, all starting at once, its setup's client among them: requests may meet a
/// repository the setup has not yet made, which the model answers like any other.
pub fn clients_at_once(plan: &Plan, shape: Shape) -> Vec<ClientSpec> {
    clients(plan, shape)
        .into_iter()
        .map(|c| ClientSpec { after: None, ..c })
        .collect()
}

/// The world for a registry of `config` in `shape`, its clients `clients`, its nemesis `nemesis`,
/// its nodes the model node, grants living `grant_life` where given.
pub fn spec(
    config: &Config,
    shape: Shape,
    clients: Vec<ClientSpec>,
    nemesis: Schedule,
    grant_life: Option<Duration>,
) -> Spec<ModelNode> {
    let config = config.clone();
    let budgets = budgets(grant_life);
    spec_with(
        shape,
        clients,
        nemesis,
        Rc::new(move || ModelNode::new(config.clone(), budgets)),
    )
}

/// The world in `shape` with clients `clients`, nemesis `nemesis`, nodes running what `core` makes.
pub fn spec_with<C: Core>(
    shape: Shape,
    clients: Vec<ClientSpec>,
    nemesis: Schedule,
    core: Rc<dyn Fn() -> C>,
) -> Spec<C> {
    let (records, bytes) = shape.stores();
    let mut machines = match shape {
        Shape::Laptop => vec![Machine {
            clock: clock(),
            roles: vec![Role::Records, Role::Bytes, Role::Node { records, bytes }],
        }],
        Shape::Fleet(n) => {
            let mut machines = vec![
                Machine {
                    clock: clock(),
                    roles: vec![Role::Records],
                },
                Machine {
                    clock: clock(),
                    roles: vec![Role::Bytes],
                },
            ];
            for _ in 0..n {
                machines.push(Machine {
                    clock: clock(),
                    roles: vec![Role::Node { records, bytes }],
                });
            }
            machines
        }
    };
    for c in clients {
        machines.push(Machine {
            clock: clock(),
            roles: vec![Role::Client(c)],
        });
    }
    Spec {
        machines,
        path: Path::LAN,
        duplicate_ppm: 0,
        discipline: Discipline::Ordered,
        limits: Limits {
            events: 1 << 14,
            nodes: 64,
            streams: 1 << 14,
            steps: 2_000_000,
            trace_words: 1 << 23,
        },
        net: NetLimits {
            flows: 1 << 12,
            links: 0,
            nats: 0,
            link_messages: 0,
            messages: 1 << 12,
            bytes: 64 << 20,
        },
        records: RecordsLimits {
            rows: 1 << 20,
            changes: 1 << 20,
        },
        bytes: BytesLimits {
            plan: UploadPlan {
                piece: 4096,
                run: 4096,
            },
            objects: 1 << 20,
            uploads: 1 << 16,
            bytes: 1 << 30,
            key: 128,
        },
        grant_key: b"reliquary-sim's bytes store".to_vec(),
        stores: StoreLimits {
            queue: 1 << 12,
            readings: 1 << 10,
        },
        nodes: NodeLimits {
            jobs: 64,
            calls: 256,
        },
        sleeps: 1 << 10,
        core,
        nemesis,
        drawn: None,
        downtime: MS * 20,
        keep: 200,
    }
}

/// Runs `spec` from `seed`, ties broken at random.
pub fn run<C: Core + 'static>(spec: &Spec<C>, seed: u64) -> Outcome {
    world::run(spec, Source::Seed(seed), &mut Random).expect("the run")
}

// ------------------------------------------------------------------------------------ pushes

use reliquary_api::grammar::{Algorithm, Digest};
use reliquary_api::model::{Action, Grants};
use reliquary_api::request::{
    ChunkRange, Descriptor, ImageManifest, Manifest, ManifestBody, Op, Principal, SessionTag,
};

/// The principal the tests push as.
pub const OWNER: Principal = Principal::Named(0);

pub const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub const LAYER: &str = "application/vnd.oci.image.layer.v1.tar+gzip";

/// A registry where the owner may pull, push and delete in each of `repositories`.
pub fn owned(repositories: &[&str]) -> Config {
    let mut grants = Grants::default();
    for name in repositories {
        for action in [Action::Pull, Action::Push, Action::Delete] {
            grants.grant(0, name, action);
        }
    }
    Config {
        grants,
        ..Config::default()
    }
}

pub fn request(op: Op) -> Request {
    Request {
        principal: OWNER,
        op,
    }
}

pub fn digest(bytes: &[u8]) -> Digest {
    Digest::of(Algorithm::Sha256, bytes)
}

/// `n` bytes that differ by `seed`.
pub fn content(seed: u8, n: usize) -> Vec<u8> {
    (0..n)
        .map(|i| u8::try_from((i * 7 + usize::from(seed) * 13) % 251).unwrap())
        .collect()
}

/// An image manifest of `config` and `layers`, as structure and as the JSON bytes it is.
pub fn image(config: &[u8], layers: &[&[u8]]) -> Box<Manifest> {
    let descriptor = |media_type: &str, bytes: &[u8]| Descriptor {
        media_type: media_type.to_owned(),
        digest: digest(bytes).to_string(),
        size: u64::try_from(bytes.len()).unwrap(),
    };
    let json = |d: &Descriptor| {
        format!(
            "{{\"mediaType\":\"{}\",\"digest\":\"{}\",\"size\":{}}}",
            d.media_type, d.digest, d.size
        )
    };
    let manifest = ImageManifest {
        media_type: Some(IMAGE_MANIFEST.to_owned()),
        artifact_type: None,
        config: descriptor(CONFIG, config),
        layers: layers.iter().map(|l| descriptor(LAYER, l)).collect(),
        subject: None,
        annotations: Vec::new(),
    };
    let bytes = format!(
        "{{\"schemaVersion\":2,\"mediaType\":\"{IMAGE_MANIFEST}\",\"config\":{},\"layers\":[{}]}}",
        json(&manifest.config),
        manifest
            .layers
            .iter()
            .map(json)
            .collect::<Vec<_>>()
            .join(",")
    );
    Box::new(Manifest {
        content_type: IMAGE_MANIFEST.to_owned(),
        body: ManifestBody::Image(manifest),
        bytes: bytes.into_bytes().into(),
    })
}

/// A push to `name`: a layer of `layer` in two chunks through session `session`, a config in one
/// `POST`, and a manifest of both tagged `tag`.
pub fn push(name: &str, session: u32, layer: &[u8], config: &[u8], tag: &str) -> Vec<Request> {
    let half = layer.len() / 2;
    let chunk = |first: usize, bytes: &[u8]| Op::PatchUpload {
        session: SessionTag(session),
        range: ChunkRange::Span {
            first: u64::try_from(first).unwrap(),
            last: u64::try_from(first + bytes.len() - 1).unwrap(),
        },
        body: bytes.to_vec().into(),
    };
    vec![
        request(Op::StartUpload {
            name: name.to_owned(),
            session: SessionTag(session),
        }),
        request(chunk(0, &layer[..half])),
        request(chunk(half, &layer[half..])),
        request(Op::PutUpload {
            session: SessionTag(session),
            digest: Some(digest(layer).to_string()),
            chunk: None,
        }),
        request(Op::PostBlob {
            name: name.to_owned(),
            digest: digest(config).to_string(),
            body: config.to_vec().into(),
        }),
        request(Op::PutManifest {
            name: name.to_owned(),
            reference: tag.to_owned(),
            manifest: image(config, &[layer]),
        }),
    ]
}

/// Reads of everything a push of `layer`, `config` and `tag` made in `name`.
pub fn read_back(name: &str, layer: &[u8], config: &[u8], tag: &str) -> Vec<Request> {
    let manifest = image(config, &[layer]);
    vec![
        request(Op::GetBlob {
            name: name.to_owned(),
            digest: digest(layer).to_string(),
            range: None,
        }),
        request(Op::GetBlob {
            name: name.to_owned(),
            digest: digest(config).to_string(),
            range: None,
        }),
        request(Op::HeadBlob {
            name: name.to_owned(),
            digest: digest(layer).to_string(),
        }),
        request(Op::GetManifest {
            name: name.to_owned(),
            reference: tag.to_owned(),
        }),
        request(Op::GetManifest {
            name: name.to_owned(),
            reference: digest(&manifest.bytes).to_string(),
        }),
        request(Op::ListTags {
            name: name.to_owned(),
            n: None,
            last: None,
        }),
    ]
}
