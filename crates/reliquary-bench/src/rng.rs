//! The one pseudorandom generator the tool draws from: SplitMix64 (Steele, Lea and Flood, "Fast
//! Splittable Pseudorandom Number Generators", OOPSLA 2014), the generator hyper-measure's open loop
//! draws its arrivals from. Every draw is a function of a stated seed, so an A|B order, a permutation
//! test and a synthetic workload replay exactly from their record.

/// SplitMix64's increment, the odd integer nearest 2^64 over the golden ratio (Steele, Lea and Flood
/// §3; Vigna's `splitmix64.c`).
pub const GOLDEN: u64 = 0x9e37_79b9_7f4a_7c15;

/// A SplitMix64 stream.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A stream from `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A draw in `[0, n)`, zero for `n = 0`: the top 64 bits of the 128-bit product of a draw and `n`
    /// (Lemire, "Fast Random Integer Generation in an Interval", ACM TOMACS 29(1), 2019, §2), whose
    /// bias toward any value is at most `n / 2^64`.
    pub fn below(&mut self, n: u64) -> u64 {
        let wide = u128::from(self.next_u64()).wrapping_mul(u128::from(n)) >> 64;
        u64::try_from(wide).unwrap_or(0)
    }

    /// A uniform draw on [0, 1): the top 53 bits as a fraction.
    pub fn unit(&mut self) -> f64 {
        crate::rounds::u64_as_f64(self.next_u64() >> 11) / 9_007_199_254_740_992.0
    }

    /// Shuffles `items` in place (Fisher and Yates; Knuth's Algorithm P, TAOCP vol. 2, §3.4.2).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let mut i = items.len();
        while i > 1 {
            let j = usize::try_from(self.below(u64::try_from(i).unwrap_or(u64::MAX))).unwrap_or(0);
            i = i.saturating_sub(1);
            items.swap(i, j.min(i));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SplitMix64's first outputs for seed 0, as Vigna's reference `splitmix64.c` gives them, the values
    /// hyper-measure's own test holds its arrivals to.
    /// (AC-RQ0.7.2)
    #[test]
    fn the_draws_are_splitmix64s() {
        // AC-RQ0.7.2: the A|B order replays from its seed.
        let mut r = SplitMix64::new(0);
        for expected in [
            0xe220_a839_7b1d_cdafu64,
            0x6e78_9e6a_a1b9_65f4,
            0x06c4_5d18_8009_454f,
        ] {
            assert_eq!(r.next_u64(), expected);
        }
    }

    /// (AC-RQ0.7.2)
    #[test]
    fn draws_below_n_stay_below_n_and_cover_it() {
        // AC-RQ0.7.2: a coin flip of the A|B order takes both faces.
        let mut r = SplitMix64::new(5);
        let mut seen = [0u32; 7];
        for _ in 0..7_000 {
            let v = r.below(7);
            assert!(v < 7);
            seen[v as usize] += 1;
        }
        assert!(seen.iter().all(|&c| c > 800), "{seen:?}");
        assert_eq!(r.below(0), 0);
        for _ in 0..1_000 {
            let u = r.unit();
            assert!((0.0..1.0).contains(&u));
        }
    }

    /// (AC-RQ0.7.8)
    #[test]
    fn a_shuffle_is_a_permutation_and_replays() {
        // AC-RQ0.7.8: a permutation drawn from a seed is the same permutation every time.
        let mut a: Vec<u32> = (0..50).collect();
        let mut b = a.clone();
        SplitMix64::new(9).shuffle(&mut a);
        SplitMix64::new(9).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
        assert_ne!(a, sorted);
    }
}
