/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Fx-style hasher for hot maps keyed by guest pointers and integer ids.
//! Not DoS resistant; only use it for keys the guest can't choose freely and
//! for maps whose iteration order is never observable.

use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub struct FxHasher(u64);

pub type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher>>;

// Multiplier from rustc-hash 2. A plain Fx multiply leaves the low bits, which
// hashbrown indexes buckets with, depending only on the low input bits, so
// aligned pointers would share buckets. Finishing with a folded 128-bit
// multiply brings the well-mixed high bits back down.
const K: u64 = 0xf135_7aea_2e62_a9c5;

impl FxHasher {
    #[inline(always)]
    fn add(&mut self, word: u64) {
        self.0 = self.0.wrapping_add(word).wrapping_mul(K);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
    }
    #[inline(always)]
    fn write_u8(&mut self, i: u8) { self.add(i.into()) }
    #[inline(always)]
    fn write_u16(&mut self, i: u16) { self.add(i.into()) }
    #[inline(always)]
    fn write_u32(&mut self, i: u32) { self.add(i.into()) }
    #[inline(always)]
    fn write_u64(&mut self, i: u64) { self.add(i) }
    #[inline(always)]
    fn write_usize(&mut self, i: usize) { self.add(i as u64) }
    #[inline(always)]
    fn finish(&self) -> u64 {
        let product = u128::from(self.0) * u128::from(K);
        (product as u64) ^ ((product >> 64) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{BuildHasher, Hash};

    fn hash<T: Hash>(value: T) -> u64 {
        BuildHasherDefault::<FxHasher>::default().hash_one(value)
    }

    #[test]
    fn aligned_pointers_spread_over_low_bits() {
        // hashbrown indexes buckets with the low bits and tags with the top 7.
        // 1024 random keys would fill about 647 of 1024 buckets.
        for (base, stride) in [(0x1000_0000u32, 16u32), (0x0004_2000, 4096), (0, 1)] {
            let mut low = std::collections::HashSet::new();
            let mut top = std::collections::HashSet::new();
            for i in 0..1024u32 {
                let h = hash(base + i * stride);
                low.insert(h & 1023);
                top.insert(h >> 57);
            }
            assert!(low.len() > 580, "stride {stride}: {}", low.len());
            assert_eq!(top.len(), 128, "stride {stride}");
        }
    }

    // Run explicitly in release mode to compare against std's SipHash.
    #[test]
    #[ignore = "manual release benchmark"]
    fn pointer_lookup_benchmark() {
        use std::{hint::black_box, time::Instant};
        fn run<S: BuildHasher + Default>(name: &str) {
            let mut map = std::collections::HashMap::<u32, u32, S>::default();
            for i in 0..20_000u32 { map.insert(0x10_0000 + i * 48, i); }
            let start = Instant::now();
            let mut sum = 0u32;
            for i in 0..5_000_000u32 { sum = sum.wrapping_add(*map.get(&black_box(0x10_0000 + (i % 20_000) * 48)).unwrap()); }
            println!("{name}: {:.2} ns/lookup ({sum})", start.elapsed().as_nanos() as f64 / 5e6);
        }
        run::<std::collections::hash_map::RandomState>("SipHash");
        run::<BuildHasherDefault<FxHasher>>("FxHasher");
    }

    #[test]
    fn map_round_trips_pointer_and_id_keys() {
        let mut map: FxHashMap<crate::mem::ConstVoidPtr, u64> = Default::default();
        for i in 0..10_000u64 {
            map.insert(crate::mem::Ptr::from_bits(0x2000 + i as u32 * 16), i);
        }
        for i in 0..10_000u64 {
            assert_eq!(map.remove(&crate::mem::Ptr::from_bits(0x2000 + i as u32 * 16)), Some(i));
        }
        assert!(map.is_empty());
        assert_ne!(hash(1u64), hash(2u64));
        assert_ne!(hash([1u8, 2, 3]), hash([1u8, 2, 4]));
    }
}
