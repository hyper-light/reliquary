//! The crash sweep (docs/design/sim.md, W7; plan AC-RQ0.6.3, T-RQ0.6.2): a run once as it is, to
//! learn every write it makes to records and to bytes, then once for each of those writes with
//! power cut before it and once with power cut after it, each judged as any run is. The runs are
//! of one seed, so each is the first run up to its cut: the write it cuts at is the one the first
//! run made there.

use std::collections::BTreeMap;

use hyper_sim::{Source, Strategy};
use reliquary_api::check::Budget;
use reliquary_api::model::Config;

use crate::call::Seam;
use crate::ledger::{Side, Write, WriteCut};
use crate::node::Core;
use crate::verdict::{self, Verdict};
use crate::world::{self, Spec};

/// One run of a sweep: where power was cut, at which write, whether the cut landed, and the
/// run's verdict.
#[derive(Debug)]
pub struct Cut {
    pub cut: WriteCut,
    pub write: Write,
    pub landed: bool,
    pub verdict: Verdict,
}

/// A sweep: the writes of the run as it is, and a run for each cut.
#[derive(Debug)]
pub struct Sweep {
    pub writes: Vec<Write>,
    pub uncut: Verdict,
    pub cuts: Vec<Cut>,
}

impl Sweep {
    /// The runs that failed their verdict, or whose cut did not land.
    pub fn failures(&self) -> impl Iterator<Item = &Cut> {
        self.cuts
            .iter()
            .filter(|c| !c.landed || !c.verdict.passed())
    }

    /// The cuts that landed, by the seam of their write and their side.
    #[must_use]
    pub fn covered(&self) -> BTreeMap<(Seam, Side), u64> {
        let mut counts = BTreeMap::new();
        for c in self.cuts.iter().filter(|c| c.landed) {
            let n = counts.entry((c.write.seam, c.cut.side)).or_insert(0u64);
            *n = n.saturating_add(1);
        }
        counts
    }
}

/// Sweeps `spec` from `seed`: each run's strategy from `strategy`, each verdict against a model of
/// `config` within `budget`.
pub fn sweep<C: Core + 'static, S: Strategy>(
    spec: &Spec<C>,
    seed: u64,
    mut strategy: impl FnMut() -> S,
    config: &Config,
    budget: Budget,
) -> Result<Sweep, world::Error> {
    let first = world::run(spec, Source::Seed(seed), &mut strategy())?;
    let uncut = verdict::judge(&first, config, budget);
    let mut cuts = Vec::new();
    for write in &first.writes {
        for side in [Side::Before, Side::After] {
            let cut = WriteCut {
                write: write.index,
                side,
            };
            let outcome = world::run_cut(spec, Some(cut), Source::Seed(seed), &mut strategy())?;
            let landed = outcome
                .writes
                .get(usize::try_from(write.index).unwrap_or(usize::MAX))
                .is_some_and(|w| w.method == write.method && w.seam == write.seam)
                && outcome
                    .faults
                    .get(crate::nemesis::POWER_CUT)
                    .copied()
                    .unwrap_or(0)
                    > 0;
            let verdict = verdict::judge(&outcome, config, budget);
            cuts.push(Cut {
                cut,
                write: write.clone(),
                landed,
                verdict,
            });
        }
    }
    Ok(Sweep {
        writes: first.writes,
        uncut,
        cuts,
    })
}
