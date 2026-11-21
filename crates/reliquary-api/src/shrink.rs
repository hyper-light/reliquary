//! Shrinking a failing input to a 1-minimal one (docs/design/model.md, H13), in the manner of
//! Zeller and Hildebrandt's delta debugging: remove runs of items while the failure stays, the runs
//! halving in length, until a pass removing each single item in turn keeps none of the removals.
//! What is left fails, and removing any one of its items makes the failure go (1-minimality).
//! Each try runs the caller's predicate once, on a budget the caller states.

/// What shrinking left, and whether it got to a 1-minimal input within its budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shrunk<T> {
    pub items: Vec<T>,
    /// A final pass removed no single item: the input is 1-minimal.
    pub minimal: bool,
    /// How many times the predicate ran.
    pub tries: u64,
}

/// Shrinks `items`, which `fails`, keeping each removal after which it still `fails`. At most
/// `budget` tries.
pub fn shrink<T: Clone>(
    items: &[T],
    mut fails: impl FnMut(&[T]) -> bool,
    budget: u64,
) -> Shrunk<T> {
    let mut current = items.to_vec();
    let mut tries: u64 = 0;
    let mut run = current.len();
    while run > 0 {
        let mut start = 0;
        let mut removed = false;
        while start < current.len() {
            if tries >= budget {
                return Shrunk {
                    items: current,
                    minimal: false,
                    tries,
                };
            }
            let end = start.saturating_add(run).min(current.len());
            let candidate: Vec<T> = current
                .iter()
                .take(start)
                .chain(current.iter().skip(end))
                .cloned()
                .collect();
            // Tries never pass the budget, a u64 the caller chose.
            tries = tries.saturating_add(1);
            if fails(&candidate) {
                current = candidate;
                removed = true;
            } else {
                start = end;
            }
        }
        if !removed {
            if run == 1 {
                break;
            }
            run = run.checked_shr(1).unwrap_or(0).max(1);
        }
    }
    Shrunk {
        items: current,
        minimal: true,
        tries,
    }
}
