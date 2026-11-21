#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::disallowed_macros
    )
)]
//! Reliquary's sans-IO cores, and the seams it reaches other products through (architecture D24).
//!
//! The cores come with the phases that need them. What is here now: the seams ([`seams`], plan
//! RQ0.4), each a trait, a simulator that can inject every fault the trait can meet, and a
//! conformance suite the simulator passes now and the real implementation passes when it lands
//! (`docs/design/seams.md` states each contract clause by clause); and what Reliquary is observed
//! by ([`observe`], plan RQ0.9).

pub mod observe;
pub mod seams;
