// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand_core::impls::{fill_bytes_via_next, next_u64_via_u32};
use rand_core::le::read_u32_into;
use rand_core::{RngCore, SeedableRng};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A xoshiro128+ random number generator.
///
/// The xoshiro128+ algorithm is not suitable for cryptographic purposes, but
/// is very fast and has good statistical properties, besides a low linear
/// complexity in the lowest bits.
///
/// The algorithm used here is translated from [the `xoshiro128starstar.c`
/// reference source code](http://xoshiro.di.unimi.it/xoshiro128starstar.c) by
/// David Blackman and Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Xoshiro128Plus {
    s: [u32; 4],
}

impl Xoshiro128Plus {
    /// Jump forward, equivalently to 2^64 calls to `next_u32()`.
    ///
    /// This can be used to generate 2^64 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoroshiro128StarStar;
    ///
    /// let rng1 = Xoroshiro128StarStar::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(u32, self, [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b]);
    }

    /// Jump forward, equivalently to 2^96 calls to `next_u32()`.
    ///
    /// This can be used to generate 2^32 starting points, from each of which
    /// `jump()` will generate 2^32 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(u32, self, [0xb523952e, 0x0b6f099f, 0xccf5a0ef, 0x1c580662]);
    }
}

impl SeedableRng for Xoshiro128Plus {
    type Seed = [u8; 16];

    /// Create a new `Xoshiro128Plus`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    #[inline]
    fn from_seed(seed: [u8; 16]) -> Xoshiro128Plus {
        deal_with_zero_seed!(seed, Self, 16);
        let mut state = [0; 4];
        read_u32_into(&seed, &mut state);
        Xoshiro128Plus { s: state }
    }

    /// Seed a `Xoshiro128Plus` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoshiro128Plus {
        from_splitmix!(seed)
    }
}

impl RngCore for Xoshiro128Plus {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let result_plus = self.s[0].wrapping_add(self.s[3]);
        impl_xoshiro_u32!(self);
        result_plus
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        next_u64_via_u32(self)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mut rng = Xoshiro128Plus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plus.c
        let expected = [
            5, 12295, 25178119, 27286542, 39879690, 1140358681, 3276312097, 4110231701, 399823256,
            2144435200,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    fn test_jump() {
        let mut rng = Xoshiro128Plus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        rng.jump();
        // These values were produced by instrumenting the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plus.c
        assert_eq!(rng.s[0], 2843103750);
        assert_eq!(rng.s[1], 2038079848);
        assert_eq!(rng.s[2], 1533207345);
        assert_eq!(rng.s[3], 44816753);
    }

    #[test]
    fn test_long_jump() {
        let mut rng = Xoshiro128Plus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        rng.long_jump();
        // These values were produced by instrumenting the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plus.c
        assert_eq!(rng.s[0], 1611968294);
        assert_eq!(rng.s[1], 2125834322);
        assert_eq!(rng.s[2], 966769569);
        assert_eq!(rng.s[3], 3193880526);
    }
}
