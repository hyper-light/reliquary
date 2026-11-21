//! The event log (docs/design/sim.md, W8): one line for everything the world did, in the order it
//! did it, each line the virtual time, the process and what happened, in words that hold no host
//! address, no host time and no unordered iteration. Two runs of one seed on any two targets give
//! the same bytes (plan T-RQ0.6.8), which the log's SHA-256 shows without keeping the log: the log
//! keeps only the last lines its maker asks for, for a failure's report, so it is bounded however
//! long a run is (CLAUDE.md §2).

use std::collections::VecDeque;
use std::fmt::{self, Write as _};

use aws_lc_rs::digest::{Context, SHA256};

/// The event log of a run.
pub struct Log {
    hash: Context,
    lines: u64,
    tail: VecDeque<String>,
    keep: usize,
    line: String,
}

/// A log's digest: its SHA-256 and how many lines it had.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Digest {
    pub sha256: String,
    pub lines: u64,
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{} over {} lines", self.sha256, self.lines)
    }
}

impl Log {
    /// A log keeping its last `keep` lines.
    #[must_use]
    pub fn new(keep: usize) -> Self {
        Self {
            hash: Context::new(&SHA256),
            lines: 0,
            tail: VecDeque::new(),
            keep,
            line: String::new(),
        }
    }

    /// Writes a line: `at` nanoseconds into the run, `who` did `what`. The line's text is folded
    /// into the digest with its ending, and kept if it is among the last the log keeps.
    pub fn line(&mut self, at: u64, who: &str, what: fmt::Arguments<'_>) {
        self.line.clear();
        // Writing into a String cannot fail.
        let _ = write!(self.line, "{at} {who} {what}");
        self.line.push('\n');
        self.hash.update(self.line.as_bytes());
        self.lines = self.lines.saturating_add(1);
        if self.keep == 0 {
            return;
        }
        if self.tail.len() >= self.keep {
            self.tail.pop_front();
        }
        self.tail.push_back(self.line.trim_end().to_owned());
    }

    /// The log's digest so far.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut sha256 = String::new();
        for byte in self.hash.clone().finish().as_ref() {
            let _ = write!(sha256, "{byte:02x}");
        }
        Digest {
            sha256,
            lines: self.lines,
        }
    }

    /// The last lines kept, oldest first.
    #[must_use]
    pub fn tail(&self) -> Vec<String> {
        self.tail.iter().cloned().collect()
    }

    /// How many lines were written.
    #[must_use]
    pub fn lines(&self) -> u64 {
        self.lines
    }
}
