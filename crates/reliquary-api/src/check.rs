//! Linearizability checking (docs/design/model.md, H12). A concurrent history is right when its
//! operations can be put in one order that keeps real time (an operation answered before another
//! was invoked comes first) and in which the model, given the requests in that order, gives every
//! answered operation the answer its client got (Herlihy and Wing 1990). An operation whose answer
//! was lost may have taken effect anywhere after its invocation, or not at all.
//!
//! The search is Wing and Gong's, as Lowe improved it and Porcupine implements it: walk the
//! history's calls and returns in time order, linearize a call whose answer the model agrees with,
//! and back up when a return arrives whose operation has not been placed; a cache of (operations
//! placed, model state) pairs already explored cuts repeated work. Deciding linearizability is
//! NP-complete in general (Gibbons and Korach), so the search counts what it explores against a
//! budget its caller states, and says so when the budget runs out. The search keeps its own stack:
//! nothing here calls itself (CLAUDE.md §11).

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::answer::Answer;
use crate::history::History;
use crate::model::Model;
use crate::request::Request;

/// The most configurations a search may explore: the caller's statement of what it will spend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget(pub u64);

/// What the checker found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// An order that explains every answer: indices into the history's operations, those whose
    /// answers were lost included where they took effect.
    Linearizable { order: Vec<usize> },
    /// No order explains the history. `operation` is the first, by completion, whose answer no
    /// order of what was invoked before it explains (the index into the history's operations).
    NotLinearizable { operation: usize },
    /// The budget ran out after `explored` configurations, with no verdict.
    Exhausted { explored: u64 },
}

/// Whether `history` is linearizable against the model, from the state `initial`.
pub fn check(initial: &Model, history: &History, budget: Budget) -> Verdict {
    let whole: Vec<Candidate<'_>> = history
        .operations
        .iter()
        .enumerate()
        .map(|(index, op)| Candidate {
            index,
            request: &op.request,
            invoked: op.invoked,
            answer: match &op.outcome {
                crate::history::Outcome::Answered { at, answer } => Some((*at, answer)),
                crate::history::Outcome::Lost => None,
            },
        })
        .collect();
    match search(initial, &whole, budget) {
        Search::Found(order) => Verdict::Linearizable { order },
        Search::Exhausted(explored) => Verdict::Exhausted { explored },
        Search::Impossible => locate(initial, &whole, budget),
    }
}

/// An operation as the search sees it: its request, invocation, and answer with its completion
/// time, if it was answered.
#[derive(Clone, Copy)]
struct Candidate<'h> {
    index: usize,
    request: &'h Request,
    invoked: u64,
    answer: Option<(u64, &'h Answer)>,
}

enum Search {
    Found(Vec<usize>),
    Impossible,
    Exhausted(u64),
}

/// The first operation, by completion, that no order explains. Cut the history at the k-th
/// completion: operations invoked by then stay, those not answered by then counted as lost. A cut
/// that no order explains stays unexplained as the cut moves later, so a binary search finds the
/// first.
fn locate(initial: &Model, whole: &[Candidate<'_>], budget: Budget) -> Verdict {
    let mut completions: Vec<(u64, usize)> = whole
        .iter()
        .filter_map(|c| c.answer.map(|(at, _)| (at, c.index)))
        .collect();
    completions.sort_unstable();
    let cut = |k: usize| -> Vec<Candidate<'_>> {
        let Some(&(time, _)) = completions.get(k) else {
            return Vec::new();
        };
        whole
            .iter()
            .filter(|c| c.invoked <= time)
            .map(|c| Candidate {
                answer: c.answer.filter(|(at, _)| *at <= time),
                ..*c
            })
            .collect()
    };
    let Some(mut high) = completions.len().checked_sub(1) else {
        return Verdict::Linearizable { order: Vec::new() };
    };
    let mut low = 0;
    while low < high {
        let middle = high
            .checked_sub(low)
            .and_then(|span| span.checked_shr(1))
            .and_then(|half| low.checked_add(half))
            .unwrap_or(low);
        match search(initial, &cut(middle), budget) {
            Search::Impossible => high = middle,
            Search::Found(_) => low = middle.saturating_add(1),
            Search::Exhausted(explored) => return Verdict::Exhausted { explored },
        }
    }
    match completions.get(low) {
        Some(&(_, operation)) => Verdict::NotLinearizable { operation },
        None => Verdict::Linearizable { order: Vec::new() },
    }
}

/// A call or a return, by the candidate's position.
#[derive(Clone, Copy)]
enum Entry {
    Call(usize),
    Return(usize),
}

/// The entries still to place, as a doubly linked list over their positions, with a head before
/// the first. Lifting an operation takes out its call and its return; putting it back reverses
/// that, last lifted first.
struct List {
    next: Vec<Option<usize>>,
    prev: Vec<Option<usize>>,
    head: usize,
}

impl List {
    fn new(len: usize) -> Self {
        let head = len;
        let next = (0..=len)
            .map(|x| {
                if x == head {
                    if len > 0 { Some(0) } else { None }
                } else {
                    x.checked_add(1).filter(|y| *y < len)
                }
            })
            .collect();
        let prev = (0..=len)
            .map(|x| {
                if x == head {
                    None
                } else if x == 0 {
                    Some(head)
                } else {
                    x.checked_sub(1)
                }
            })
            .collect();
        Self { next, prev, head }
    }

    fn after(&self, x: usize) -> Option<usize> {
        self.next.get(x).copied().flatten()
    }

    fn unlink(&mut self, x: usize) {
        let before = self.prev.get(x).copied().flatten();
        let after = self.after(x);
        if let Some(slot) = before.and_then(|b| self.next.get_mut(b)) {
            *slot = after;
        }
        if let Some(slot) = after.and_then(|a| self.prev.get_mut(a)) {
            *slot = before;
        }
    }

    fn relink(&mut self, x: usize) {
        let before = self.prev.get(x).copied().flatten();
        let after = self.after(x);
        if let Some(slot) = before.and_then(|b| self.next.get_mut(b)) {
            *slot = Some(x);
        }
        if let Some(slot) = after.and_then(|a| self.prev.get_mut(a)) {
            *slot = Some(x);
        }
    }
}

/// Which candidates are placed: one bit each.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Placed(Vec<u64>);

impl Placed {
    fn width() -> usize {
        usize::try_from(u64::BITS).unwrap_or(usize::MAX)
    }

    fn new(len: usize) -> Self {
        Self(vec![0; len.div_ceil(Self::width())])
    }

    fn bit(i: usize) -> Option<(usize, u64)> {
        let word = i.checked_div(Self::width())?;
        let shift = u32::try_from(i.checked_rem(Self::width())?).ok()?;
        Some((word, 1u64.checked_shl(shift)?))
    }

    fn set(&mut self, i: usize, on: bool) {
        if let Some((word, mask)) = Self::bit(i)
            && let Some(bits) = self.0.get_mut(word)
        {
            if on {
                *bits |= mask;
            } else {
                *bits &= !mask;
            }
        }
    }
}

/// A state's fingerprint for the cache: two 64-bit hashes of differently prefixed input. Hashing
/// is deterministic (SipHash with fixed keys), so a run's verdict is a function of its history.
fn fingerprint(model: &Model) -> (u64, u64) {
    let mut first = DefaultHasher::new();
    false.hash(&mut first);
    model.hash(&mut first);
    let mut second = DefaultHasher::new();
    true.hash(&mut second);
    model.hash(&mut second);
    (first.finish(), second.finish())
}

/// Lowe's search over `candidates`.
fn search(initial: &Model, candidates: &[Candidate<'_>], budget: Budget) -> Search {
    let mut keyed: Vec<((u64, bool), Entry)> = Vec::new();
    for (i, c) in candidates.iter().enumerate() {
        // At one instant calls come before returns: operations that touch are concurrent.
        keyed.push(((c.invoked, false), Entry::Call(i)));
        let end = c.answer.map_or(u64::MAX, |(at, _)| at);
        keyed.push(((end, true), Entry::Return(i)));
    }
    keyed.sort_by_key(|(key, _)| *key);
    let entries: Vec<Entry> = keyed.into_iter().map(|(_, entry)| entry).collect();
    let mut call_at = vec![0; candidates.len()];
    let mut return_at = vec![0; candidates.len()];
    for (position, entry) in entries.iter().enumerate() {
        let (Entry::Call(i) | Entry::Return(i)) = *entry;
        let slot = match entry {
            Entry::Call(_) => call_at.get_mut(i),
            Entry::Return(_) => return_at.get_mut(i),
        };
        if let Some(slot) = slot {
            *slot = position;
        }
    }
    let mut list = List::new(entries.len());
    let mut unplaced = candidates.iter().filter(|c| c.answer.is_some()).count();
    let mut placed = Placed::new(candidates.len());
    let mut seen: HashSet<(Placed, (u64, u64))> = HashSet::new();
    let mut explored: u64 = 0;
    let mut stack: Vec<(usize, Model)> = Vec::new();
    let mut state = initial.clone();
    let mut cursor = list.after(list.head);
    // The counts below never pass the number of candidates, so their saturating steps never
    // saturate.
    loop {
        if unplaced == 0 {
            let order = stack
                .iter()
                .filter_map(|(i, _)| candidates.get(*i).map(|c| c.index))
                .collect();
            return Search::Found(order);
        }
        let call = cursor.and_then(|at| match entries.get(at) {
            Some(Entry::Call(i)) => Some((at, *i)),
            _ => None,
        });
        if let Some((at, i)) = call {
            let Some(candidate) = candidates.get(i) else {
                return Search::Impossible;
            };
            let decision = state.decide(candidate.request);
            let agrees = candidate
                .answer
                .is_none_or(|(_, answer)| decision.answer == *answer);
            if agrees {
                let mut after = state.clone();
                after.apply(decision.change);
                let mut next_placed = placed.clone();
                next_placed.set(i, true);
                if seen.insert((next_placed.clone(), fingerprint(&after))) {
                    explored = explored.saturating_add(1);
                    if explored > budget.0 {
                        return Search::Exhausted(explored);
                    }
                    placed = next_placed;
                    stack.push((i, std::mem::replace(&mut state, after)));
                    if candidate.answer.is_some() {
                        unplaced = unplaced.saturating_sub(1);
                    }
                    list.unlink(at);
                    if let Some(&ret) = return_at.get(i) {
                        list.unlink(ret);
                    }
                    cursor = list.after(list.head);
                    continue;
                }
            }
            cursor = list.after(at);
            continue;
        }
        // A return whose operation is not placed (or the end of the list): undo the last placing
        // and try what follows its call.
        let Some((i, before)) = stack.pop() else {
            return Search::Impossible;
        };
        state = before;
        placed.set(i, false);
        if candidates.get(i).is_some_and(|c| c.answer.is_some()) {
            unplaced = unplaced.saturating_add(1);
        }
        let at = call_at.get(i).copied().unwrap_or(list.head);
        if let Some(&ret) = return_at.get(i) {
            list.relink(ret);
        }
        list.relink(at);
        cursor = list.after(at);
    }
}
