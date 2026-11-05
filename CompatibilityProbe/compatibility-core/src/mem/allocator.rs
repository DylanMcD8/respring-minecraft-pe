/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::{GuestUSize, VAddr, PAGE_SIZE};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

/// iPhone OS's allocator always aligns to 16 bytes at minimum, and this
/// is also the minimum allocation size.
pub const MIN_CHUNK_SIZE: GuestUSize = 16;

/// A non-empty range of bytes in virtual address space.
///
/// Similar to [`RangeInclusive<u32>`][std::ops::RangeInclusive] but with a
/// more convenient representation.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Chunk {
    pub(super) base: VAddr,
    pub(super) size: NonZeroU32,
}

impl Chunk {
    pub fn new(base: VAddr, size: GuestUSize) -> Chunk {
        Chunk {
            base,
            size: NonZeroU32::new(size).unwrap(),
        }
    }

    fn merge(self, other: Chunk) -> Chunk {
        assert!(
            self.last_byte() + 1 == other.base || other.last_byte() + 1 == self.base,
            "Chunks must be adjacent to merge"
        );
        Chunk::new(
            self.base.min(other.base),
            self.size.get() + other.size.get(),
        )
    }

    #[inline(always)]
    fn last_byte(&self) -> VAddr {
        self.base + (self.size.get() - 1)
    }

    #[inline(always)]
    fn contains(&self, addr: VAddr) -> bool {
        self.base <= addr && addr <= self.last_byte()
    }

    #[inline(always)]
    fn trisect_by(&self, middle: Chunk) -> Option<(Option<Chunk>, Option<Chunk>)> {
        if !self.contains(middle.base) || !self.contains(middle.last_byte()) {
            return None;
        }

        Some(self.difference(middle))
    }

    #[inline(always)]
    /// Returns parts of `self` that don't overlap with other
    fn difference(&self, other: Chunk) -> (Option<Chunk>, Option<Chunk>) {
        if other.last_byte() < self.base {
            return (None, Some(*self));
        }

        if other.base > self.last_byte() {
            return (Some(*self), None);
        }

        let left = match other.base.checked_sub(self.base) {
            None | Some(0) => None,
            Some(size) => Some(Chunk::new(self.base, size)),
        };

        let right = match self.last_byte().checked_sub(other.last_byte()) {
            None | Some(0) => None,
            Some(size) => Some(Chunk::new(other.last_byte() + 1, size)),
        };

        (left, right)
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chunk ({:#x}–{:#x}; {:#x} bytes)",
            self.base,
            self.base + (self.size.get() - 1),
            self.size.get()
        )
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::Chunk;
    #[test]
    fn contains() {
        assert!(Chunk::new(2, 4).contains(2));
        assert!(Chunk::new(2, 4).contains(5));
        assert!(!Chunk::new(2, 4).contains(6));
    }

    #[test]
    fn trisect() {
        assert_eq!(
            Chunk::new(2, 4).trisect_by(Chunk::new(3, 2)),
            Some((Some(Chunk::new(2, 1)), Some(Chunk::new(5, 1))))
        );
        assert_eq!(
            Chunk::new(2, 4).trisect_by(Chunk::new(2, 2)),
            Some((None, Some(Chunk::new(4, 2))))
        );
        assert_eq!(
            Chunk::new(2, 4).trisect_by(Chunk::new(4, 2)),
            Some((Some(Chunk::new(2, 2)), None))
        );
        assert_eq!(Chunk::new(2, 4).trisect_by(Chunk::new(1, 2)), None);
        assert_eq!(Chunk::new(2, 4).trisect_by(Chunk::new(5, 2)), None);
    }

    #[test]
    fn merge() {
        let a = Chunk::new(0, 10);
        let b = Chunk::new(10, 10);
        assert_eq!(a.merge(b), Chunk::new(0, 20));
        assert_eq!(b.merge(a), Chunk::new(0, 20));
    }

    #[test]
    #[should_panic]
    fn merge_non_adjacent() {
        let a = Chunk::new(0, 10);
        let b = Chunk::new(20, 10);
        a.merge(b);
    }

    #[test]
    fn difference() {
        let test_chunk = Chunk::new(10, 10);
        assert_eq!(
            test_chunk.difference(Chunk::new(0, 5)),
            (None, Some(Chunk::new(10, 10)))
        );

        assert_eq!(
            test_chunk.difference(Chunk::new(20, 10)),
            (Some(Chunk::new(10, 10)), None)
        );

        assert_eq!(test_chunk.difference(Chunk::new(10, 10)), (None, None));

        assert_eq!(test_chunk.difference(Chunk::new(9, 15)), (None, None));

        assert_eq!(
            test_chunk.difference(Chunk::new(5, 10)),
            (None, Some(Chunk::new(15, 5)))
        );

        assert_eq!(
            test_chunk.difference(Chunk::new(15, 10)),
            (Some(Chunk::new(10, 5)), None)
        );

        assert_eq!(
            test_chunk.difference(Chunk::new(13, 3)),
            (Some(Chunk::new(10, 3)), Some(Chunk::new(16, 4)))
        );
    }
}

/// Specialized collection types. They're kept in their own module so the
/// allocator can only access them via their public methods, so that there's
/// less places inconsistencies between the sub-collections could happen.
mod collections {
    use super::*;

    #[derive(Default, Debug)]
    pub struct ChunkMap {
        chunks: BTreeMap<VAddr, NonZeroU32>,
    }
    impl ChunkMap {
        #[inline(always)]
        pub fn insert(&mut self, Chunk { base, size }: Chunk) {
            assert!(self.chunks.insert(base, size).is_none());
        }
        #[inline(always)]
        pub fn remove_with_base(&mut self, base: VAddr) -> Option<Chunk> {
            self.chunks.remove(&base).map(|size| Chunk { base, size })
        }
        #[inline(always)]
        pub fn remove_with_end(&mut self, end: VAddr) -> Option<Chunk> {
            let (&base, &size) = self.chunks.range(..end).next_back()?;
            let chunk = Chunk { base, size };
            if chunk.last_byte() + 1 != end {
                return None;
            }
            Some(self.remove_with_base(chunk.base).unwrap())
        }
        #[inline(always)]
        pub fn get_size_with_base(&self, base: VAddr) -> Option<NonZeroU32> {
            self.chunks.get(&base).copied()
        }

        pub fn overlapping_chunks(&self, chunk: Chunk) -> impl Iterator<Item = Chunk> + '_ {
            let start = self
                .chunks
                .range(..=chunk.base)
                .next_back()
                .filter(|(&base, &size)| chunk.base < base + size.get())
                .map(|(&base, _)| base)
                .unwrap_or(chunk.base);

            self.chunks
                .range(start..=chunk.last_byte())
                .map(|(&base, size)| Chunk::new(base, size.get()))
        }

        #[cfg(test)]
        pub fn iter(&self) -> impl Iterator<Item = Chunk> + '_ {
            self.chunks.iter().map(|(&base, &size)| Chunk { base, size })
        }

        /// Remove all chunks overlapping a provided Chunk. In the case
        /// of overlap the non overlapping portion is left
        pub fn remove_range(&mut self, remove: Chunk) {
            let bases: Vec<VAddr> = self
                .overlapping_chunks(remove)
                .map(|chunk| chunk.base)
                .collect();

            for base in bases {
                let chunk = self.remove_with_base(base).unwrap();
                let (left, right) = chunk.difference(remove);
                if let Some(left) = left {
                    self.insert(left);
                }

                if let Some(right) = right {
                    self.insert(right);
                }
            }
        }
    }

    impl IntoIterator for ChunkMap {
        type Item = Chunk;
        type IntoIter = std::iter::Map<
            std::collections::btree_map::IntoIter<VAddr, NonZeroU32>,
            fn((VAddr, NonZeroU32)) -> Self::Item,
        >;

        fn into_iter(self) -> Self::IntoIter {
            self.chunks
                .into_iter()
                .map(|(base, size)| Chunk { base, size })
        }
    }

    #[derive(Debug)]
    pub struct SizeBucketedChunkMap {
        min_chunk_size: u32,
        chunks: ChunkMap,
        chunks_by_log2_size: Vec<Vec<Chunk>>,
    }
    impl SizeBucketedChunkMap {
        pub fn new(min_chunk_size: u32) -> Self {
            Self {
                min_chunk_size,
                chunks: Default::default(),
                chunks_by_log2_size: vec![
                    Vec::new();
                    (u32::MAX.ilog2() - min_chunk_size.ilog2()) as usize + 1
                ],
            }
        }

        /// Get log2 size bucket for chunk.
        fn bucket_for(&self, size: GuestUSize) -> usize {
            (size.ilog2() - self.min_chunk_size.ilog2()) as usize
        }

        pub fn insert(&mut self, chunk: Chunk) {
            assert!(chunk.size.get() >= self.min_chunk_size);
            self.chunks.insert(chunk);
            let bucket_size = self.bucket_for(chunk.size.get());
            self.chunks_by_log2_size[bucket_size].push(chunk);
        }

        #[inline(always)]
        fn remove_from_bucket(&mut self, chunk: Chunk) {
            let bucket_size = self.bucket_for(chunk.size.get());
            let bucket = &mut self.chunks_by_log2_size[bucket_size];
            // Search from the end (recent frees are usually at the end, so
            // following the generational hypothesis, that's a better place to
            // start)
            let idx = bucket
                .iter()
                .rposition(|chunk2| chunk.base == chunk2.base)
                .unwrap();
            assert_eq!(chunk, bucket.swap_remove(idx));
        }

        pub fn remove_with_base(&mut self, base: VAddr) -> Option<Chunk> {
            let chunk = self.chunks.remove_with_base(base)?;
            self.remove_from_bucket(chunk);
            Some(chunk)
        }

        pub fn remove_with_end(&mut self, end: VAddr) -> Option<Chunk> {
            let chunk = self.chunks.remove_with_end(end)?;
            self.remove_from_bucket(chunk);
            Some(chunk)
        }

        /// Remove all chunks overlapping a provided Chunk. In the case
        /// of overlap the non overlapping portion is carved out
        pub fn remove_range(&mut self, remove: Chunk) {
            let bases: Vec<VAddr> = self
                .chunks
                .overlapping_chunks(remove)
                .map(|chunk| chunk.base)
                .collect();

            for base in bases {
                let chunk = self.remove_with_base(base).unwrap();
                let (left, right) = chunk.difference(remove);
                if let Some(left) = left {
                    self.insert(left);
                }

                if let Some(right) = right {
                    self.insert(right);
                }
            }
        }

        fn allocate_in_bucket(&mut self, size: GuestUSize, bucket: usize) -> Option<Chunk> {
            let (idx, _) = {
                let mut best_chunk: Option<(usize, GuestUSize)> = None;

                // Search from end because we should prefer recently-freed
                // allocations that might be the right size.
                for (idx, chunk) in self.chunks_by_log2_size[bucket]
                    .iter_mut()
                    .enumerate()
                    .rev()
                {
                    if chunk.size.get() >= size
                        && (best_chunk.is_none() || best_chunk.unwrap().1 > chunk.size.get())
                    {
                        best_chunk = Some((idx, chunk.size.get()));
                        if chunk.size.get() == size {
                            break;
                        }
                    }
                }

                best_chunk
            }?;

            let existing = self.chunks_by_log2_size[bucket].swap_remove(idx);
            let existing2 = self.chunks.remove_with_base(existing.base);
            assert_eq!(Some(existing), existing2);

            if existing.size.get() == size {
                return Some(existing);
            }

            let alloc = Chunk::new(existing.base, size);
            let rump = Chunk::new(existing.base + size, existing.size.get() - size);
            self.insert(rump);

            Some(alloc)
        }

        pub fn allocate(&mut self, size: GuestUSize) -> Option<Chunk> {
            assert!(size >= self.min_chunk_size);

            // Look in the smallest bucket first. This is the only bucket where
            // an exact match can be found.

            let bucket = self.bucket_for(size);
            if let Some(alloc) = self.allocate_in_bucket(size, bucket) {
                return Some(alloc);
            }

            // Exact match has been ruled out, find the smallest chunk in the
            // next largest non-empty bucket.

            let bucket = self.chunks_by_log2_size[bucket + 1..]
                .iter()
                .position(|bucket| !bucket.is_empty())?
                + bucket
                + 1;
            self.allocate_in_bucket(size, bucket)
        }

        pub fn iter(&self) -> impl Iterator<Item = Chunk> + '_ {
            self.chunks_by_log2_size
                .iter()
                .flat_map(|chunks| chunks.iter())
                .copied()
        }

        /// Every bucket entry is in the right bucket and in the address map,
        /// and nothing else is in the map. Returns the chunks by address.
        #[cfg(test)]
        pub fn checked_chunks(&self) -> Vec<Chunk> {
            let mut count = 0;
            for (bucket, chunks) in self.chunks_by_log2_size.iter().enumerate() {
                for chunk in chunks {
                    assert_eq!(self.bucket_for(chunk.size.get()), bucket);
                    assert_eq!(self.chunks.get_size_with_base(chunk.base), Some(chunk.size));
                    count += 1;
                }
            }
            assert_eq!(count, self.chunks.iter().count());
            self.chunks.iter().collect()
        }
    }
}
use collections::{ChunkMap, SizeBucketedChunkMap};

/// Tracks which memory is in use and makes allocations from it.
#[derive(Debug)]
pub struct HeapAllocator {
    used_chunks: ChunkMap,
    unused_chunks: SizeBucketedChunkMap,
    // These are chunks that are managed by an external allocator
    external_chunks: ChunkMap,
    backing_chunks: Vec<Chunk>,
}

impl HeapAllocator {
    /// Size of chunks requested by the heap from the VM allocator.
    /// This is set to 2MiB based on the original jemalloc paper.
    pub const HEAP_CHUNK_SIZE: GuestUSize = 2 * 1024 * 1024;

    /// This is the maximum allocation for the heap. Anything else is deferred
    /// to the vm allocator. This is set to 15 KiB according to:
    /// <https://www.cocoawithlove.com/2010/05/look-at-how-malloc-works-on-mac.html>
    pub const HEAP_ALLOCATION_THRESHOLD: GuestUSize = (15 * 1024) - 1;

    pub fn new(vm: &mut VMAllocator, size: GuestUSize) -> HeapAllocator {
        let mut unused_chunks = SizeBucketedChunkMap::new(MIN_CHUNK_SIZE);
        let mut backing_chunks = Vec::new();

        if size > 0 {
            let base_chunk = vm
                .allocate(None, size)
                .expect("Failed to allocate heap space");
            backing_chunks.push(base_chunk);
            unused_chunks.insert(base_chunk);
        }

        HeapAllocator {
            used_chunks: Default::default(),
            unused_chunks,
            external_chunks: Default::default(),
            backing_chunks,
        }
    }

    pub fn alloc(&mut self, vm: &mut VMAllocator, size: GuestUSize) -> Option<Chunk> {
        let size = size.max(MIN_CHUNK_SIZE);
        let size = Self::align(size, MIN_CHUNK_SIZE);

        let alloc = if size > Self::HEAP_ALLOCATION_THRESHOLD {
            let alloc = vm.allocate(None, size).ok()?;
            self.external_chunks.insert(alloc);
            alloc
        } else {
            let alloc = self.unused_chunks.allocate(size).or_else(|| {
                self.grow(vm);
                self.unused_chunks.allocate(size)
            })?;
            self.used_chunks.insert(alloc);
            alloc
        };

        Some(alloc)
    }

    /// Special case for valloc().
    /// This does not respect [Self::HEAP_ALLOCATION_THRESHOLD]!
    pub fn valloc(&mut self, vm: &mut VMAllocator, size: GuestUSize) -> Option<Chunk> {
        assert!(size > 0); // TODO
        let size = size.next_multiple_of(PAGE_SIZE);
        let alloc = vm.allocate(None, size).ok()?;
        self.external_chunks.insert(alloc);
        Some(alloc)
    }

    fn align(size: GuestUSize, align: GuestUSize) -> GuestUSize {
        if !size.is_multiple_of(align) {
            size + align - (size % align)
        } else {
            size
        }
    }

    /// This is used for realloc
    pub fn find_allocated_size(&mut self, base: VAddr) -> GuestUSize {
        if let Some(size) = self.external_chunks.get_size_with_base(base) {
            return size.get();
        }
        let Some(size) = self.used_chunks.get_size_with_base(base) else {
            panic!("Can't find {base:#x}, unknown allocation!");
        };
        size.get()
    }

    /// Returns the size of the freed chunk so it can be zeroed if desired
    #[must_use]
    pub fn free(&mut self, vm: &mut VMAllocator, base: VAddr) -> GuestUSize {
        if let Some(freed) = self.external_chunks.remove_with_base(base) {
            vm.deallocate(freed.base, freed.size.get());
            return freed.size.get();
        }

        let Some(freed) = self.used_chunks.remove_with_base(base) else {
            log!("Can't free {:#x}, unknown allocation!", base);
            return 0;
        };

        self.insert_unused(freed);

        freed.size.get()
    }

    // Merge with both free neighbours, like VMAllocator::deallocate, so two
    // free chunks are never left adjacent and unusable as one span.
    fn insert_unused(&mut self, chunk: Chunk) {
        let mut combined = chunk;
        if let Some(adjacent) = self.unused_chunks.remove_with_base(chunk.last_byte() + 1) {
            combined = combined.merge(adjacent);
        }
        if let Some(adjacent) = self.unused_chunks.remove_with_end(chunk.base) {
            combined = combined.merge(adjacent);
        }
        self.unused_chunks.insert(combined);
    }

    /// Consume the allocator returning an iterator over the managed
    /// virtual memory chunks
    pub fn into_vm_chunks(self) -> impl Iterator<Item = Chunk> {
        self.external_chunks.into_iter().chain(self.backing_chunks)
    }

    fn grow(&mut self, vm: &mut VMAllocator) {
        log!("Attempting to grow heap.");
        let chunk = vm
            .allocate(None, Self::HEAP_CHUNK_SIZE)
            .expect("Failed to allocate memory for heap.");

        self.backing_chunks.push(chunk);
        // The VM allocator can place this right next to an earlier backing
        // chunk, so its free edge joins this one.
        self.insert_unused(chunk);
    }
}

#[derive(Debug)]
pub enum VMAllocError {
    AddressUnavailable,
    NoSpace,
}

/// Virtual Memory Allocator which handles allocation with page granularity
#[derive(Debug)]
pub struct VMAllocator {
    used_chunks: ChunkMap,
    unused_chunks: SizeBucketedChunkMap,
}

impl VMAllocator {
    /// Read-only coverage query. Adjacent live reservations may cover one span;
    /// a hole, deallocated page, zero length, or address wrap rejects it.
    pub(super) fn is_mapped_range(&self, address: VAddr, size: GuestUSize) -> bool {
        let end = u64::from(address) + u64::from(size);
        if size == 0 || end > (1u64 << 32) { return false; }
        let mut cursor = u64::from(address);
        for chunk in self.used_chunks.overlapping_chunks(Chunk::new(address, size)) {
            if u64::from(chunk.base) > cursor { return false; }
            cursor = cursor.max(u64::from(chunk.base) + u64::from(chunk.size.get()));
            if cursor >= end { return true; }
        }
        false
    }

    pub fn new(base: VAddr, size: GuestUSize) -> VMAllocator {
        let allocation_space = Chunk::new(base, size);

        let mut unused_chunks = SizeBucketedChunkMap::new(PAGE_SIZE);
        unused_chunks.insert(allocation_space);

        VMAllocator {
            used_chunks: Default::default(),
            unused_chunks,
        }
    }

    pub fn allocate(
        &mut self,
        address: Option<VAddr>,
        size: GuestUSize,
    ) -> Result<Chunk, VMAllocError> {
        let size = size.next_multiple_of(PAGE_SIZE);
        match address {
            Some(address) => {
                let address = address & !(PAGE_SIZE - 1);
                self.allocate_at(address, size)
            }
            None => self.allocate_any(size),
        }
    }

    pub fn deallocate(&mut self, address: VAddr, size: GuestUSize) -> Chunk {
        // From testing vm_deallocate you can deallocate anything as long as
        // the memory is not protected. Since we have no permissions we can
        // always succeed
        let size = size.next_multiple_of(PAGE_SIZE);
        let address = address & !(PAGE_SIZE - 1);
        let freed = Chunk::new(address, size);

        self.used_chunks.remove_range(freed);
        self.unused_chunks.remove_range(freed);

        let left_adjacent = self.unused_chunks.remove_with_base(freed.last_byte() + 1);
        let right_adjacent = self.unused_chunks.remove_with_end(freed.base);

        let mut combined = freed;

        if let Some(adjacent) = left_adjacent {
            combined = combined.merge(adjacent);
        }

        if let Some(adjacent) = right_adjacent {
            combined = combined.merge(adjacent);
        }

        self.unused_chunks.insert(combined);
        freed
    }

    fn allocate_at(&mut self, address: VAddr, size: GuestUSize) -> Result<Chunk, VMAllocError> {
        assert!(address.is_multiple_of(PAGE_SIZE));
        assert!(size.is_multiple_of(PAGE_SIZE) && size >= PAGE_SIZE);
        let chunk = Chunk::new(address, size);

        let to_trisect = self
            .unused_chunks
            .iter()
            .find(|unused_chunk| unused_chunk.contains(address))
            .ok_or(VMAllocError::AddressUnavailable)?;

        let (before, after) = to_trisect.trisect_by(chunk).ok_or(VMAllocError::NoSpace)?;
        self.unused_chunks.remove_with_base(to_trisect.base);
        if let Some(before) = before {
            self.unused_chunks.insert(before);
        }
        if let Some(after) = after {
            self.unused_chunks.insert(after);
        }
        self.used_chunks.insert(chunk);

        Ok(chunk)
    }

    fn allocate_any(&mut self, size: GuestUSize) -> Result<Chunk, VMAllocError> {
        assert!(size.is_multiple_of(PAGE_SIZE) && size >= PAGE_SIZE);

        let alloc = self
            .unused_chunks
            .allocate(size)
            .ok_or(VMAllocError::NoSpace)?;

        self.used_chunks.insert(alloc);

        Ok(alloc)
    }
}

#[cfg(test)]
mod heap_tests {
    use super::*;

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
        fn below(&mut self, n: u64) -> u64 { self.next() % n }
    }

    // Mostly small objects with a tail of buffers, roughly what the game's
    // C++ containers and chunk meshes ask for.
    fn churn_size(rng: &mut XorShift) -> GuestUSize {
        match rng.below(100) {
            0..=69 => 1 + rng.below(128) as GuestUSize,
            70..=89 => 128 + rng.below(896) as GuestUSize,
            90..=97 => 1024 + rng.below(14 * 1024) as GuestUSize,
            _ => 16 * 1024 + rng.below(48 * 1024) as GuestUSize,
        }
    }

    // Same order of operations as Mem::realloc_in_heap when growing.
    fn realloc(heap: &mut HeapAllocator, vm: &mut VMAllocator, base: VAddr, size: GuestUSize) -> Chunk {
        let new = heap.alloc(vm, size).unwrap();
        let _ = heap.free(vm, base);
        new
    }

    // Runs of address-contiguous backing chunks, which merges may span.
    fn backing_runs(heap: &HeapAllocator) -> Vec<Chunk> {
        let mut runs: Vec<Chunk> = Vec::new();
        let mut backing = heap.backing_chunks.clone();
        backing.sort_by_key(|chunk| chunk.base);
        for chunk in backing {
            match runs.last_mut() {
                Some(run) if run.last_byte() + 1 == chunk.base => *run = run.merge(chunk),
                _ => runs.push(chunk),
            }
        }
        runs
    }

    // Free and used heap chunks tile the backing chunks exactly, no two free
    // chunks touch, and the live set matches the reference model.
    fn check_heap(heap: &HeapAllocator, live: &BTreeMap<VAddr, GuestUSize>) {
        let free = heap.unused_chunks.checked_chunks();
        for pair in free.windows(2) {
            assert!(pair[0].last_byte() + 1 < pair[1].base, "adjacent free chunks {pair:?}");
        }
        let mut tiles: Vec<Chunk> = heap.used_chunks.iter().chain(free.iter().copied()).collect();
        tiles.sort_by_key(|chunk| chunk.base);
        for pair in tiles.windows(2) {
            assert!(pair[0].last_byte() < pair[1].base, "overlap {pair:?}");
        }
        let runs = backing_runs(heap);
        for tile in &tiles {
            assert!(runs.iter().any(|run| run.contains(tile.base) && run.contains(tile.last_byte())));
        }
        let size_of = |chunks: &mut dyn Iterator<Item = &Chunk>| chunks.map(|c| u64::from(c.size.get())).sum::<u64>();
        assert_eq!(size_of(&mut tiles.iter()), size_of(&mut heap.backing_chunks.iter()));

        let external: Vec<Chunk> = heap.external_chunks.iter().collect();
        for chunk in &external {
            assert!(chunk.base.is_multiple_of(PAGE_SIZE));
            assert!(!heap.backing_chunks.iter().any(|b| b.contains(chunk.base) || chunk.contains(b.base)));
        }
        let mut allocated: Vec<Chunk> = heap.used_chunks.iter().chain(external).collect();
        allocated.sort_by_key(|chunk| chunk.base);
        assert_eq!(allocated.len(), live.len());
        for (chunk, (&base, &requested)) in allocated.iter().zip(live) {
            assert_eq!(chunk.base, base);
            assert!(base.is_multiple_of(MIN_CHUNK_SIZE) && chunk.size.get().is_multiple_of(MIN_CHUNK_SIZE));
            assert!(chunk.size.get() >= requested.max(MIN_CHUNK_SIZE));
        }
    }

    #[test]
    fn heap_randomized_stress_against_model() {
        let mut vm = VMAllocator::new(0x1000, 0xf000_0000);
        let mut heap = HeapAllocator::new(&mut vm, HeapAllocator::HEAP_CHUNK_SIZE);
        let mut rng = XorShift(0x2545_f491_4f6c_dd1d);
        let mut live = BTreeMap::<VAddr, GuestUSize>::new();
        let mut order = Vec::<VAddr>::new();
        for step in 0..60_000u32 {
            // Drift between growing and shrinking phases so merges of all
            // shapes happen, including frees between two free chunks.
            let grow = (step / 5_000) % 2 == 0;
            match rng.below(10) {
                0..=4 if grow || order.is_empty() => {
                    let size = churn_size(&mut rng);
                    let chunk = heap.alloc(&mut vm, size).unwrap();
                    assert!(live.insert(chunk.base, size).is_none());
                    order.push(chunk.base);
                }
                0..=7 if !order.is_empty() => {
                    let base = order.swap_remove(rng.below(order.len() as u64) as usize);
                    let size = live.remove(&base).unwrap();
                    assert!(heap.free(&mut vm, base) >= size);
                }
                _ if !order.is_empty() => {
                    let idx = rng.below(order.len() as u64) as usize;
                    let size = live[&order[idx]] + churn_size(&mut rng);
                    if heap.find_allocated_size(order[idx]) >= size {
                        live.insert(order[idx], size);
                    } else {
                        let new = realloc(&mut heap, &mut vm, order[idx], size);
                        live.remove(&order[idx]);
                        assert!(live.insert(new.base, size).is_none());
                        order[idx] = new.base;
                    }
                }
                _ => {}
            }
            if step % 500 == 0 {
                check_heap(&heap, &live);
            }
        }
        check_heap(&heap, &live);

        for base in order.drain(..) {
            live.remove(&base);
            let _ = heap.free(&mut vm, base);
        }
        check_heap(&heap, &live);
        assert!(heap.external_chunks.iter().next().is_none());

        // With everything freed, each run of contiguous backing chunks is one
        // free chunk, and all of it can be handed out again without growing.
        let runs = backing_runs(&heap);
        assert_eq!(heap.unused_chunks.checked_chunks(), runs);
        let largest = HeapAllocator::HEAP_ALLOCATION_THRESHOLD & !(MIN_CHUNK_SIZE - 1);
        let fits: u32 = runs.iter().map(|run| run.size.get() / largest).sum();
        let backing_count = heap.backing_chunks.len();
        for _ in 0..fits {
            let chunk = heap.alloc(&mut vm, largest).unwrap();
            live.insert(chunk.base, largest);
        }
        assert_eq!(heap.backing_chunks.len(), backing_count);
        check_heap(&heap, &live);
    }

    // grow() can be handed VM space directly above or below an existing
    // backing chunk whose edge is free; the two must merge.
    #[test]
    fn heap_grow_merges_with_adjacent_backing_chunk() {
        let size = HeapAllocator::HEAP_CHUNK_SIZE;
        // Above: the next VM chunk follows the initial one.
        let mut vm = VMAllocator::new(0x1000, 0xf000_0000);
        let mut heap = HeapAllocator::new(&mut vm, PAGE_SIZE);
        let mut live = BTreeMap::new();
        live.insert(heap.alloc(&mut vm, PAGE_SIZE - 16).unwrap().base, PAGE_SIZE - 16);
        let grown = heap.alloc(&mut vm, 32).unwrap();
        live.insert(grown.base, 32);
        assert_eq!(heap.backing_chunks[1].base, 0x2000);
        assert_eq!(grown.base, 0x2000 - 16);
        check_heap(&heap, &live);

        // Below: a hole of exactly one heap chunk ends at the backing chunk,
        // whose first bytes are free.
        let mut vm = VMAllocator::new(0x1000, 0xf000_0000);
        let hole = vm.allocate(None, size).unwrap();
        let mut heap = HeapAllocator::new(&mut vm, PAGE_SIZE);
        let head = heap.alloc(&mut vm, 16).unwrap();
        let mut live = BTreeMap::new();
        live.insert(heap.alloc(&mut vm, PAGE_SIZE - 16).unwrap().base, PAGE_SIZE - 16);
        let _ = heap.free(&mut vm, head.base);
        vm.deallocate(hole.base, size);
        live.insert(heap.alloc(&mut vm, 32).unwrap().base, 32);
        assert_eq!(heap.backing_chunks[1], hole);
        check_heap(&heap, &live);
        assert_eq!(heap.unused_chunks.checked_chunks(), vec![Chunk::new(hole.base + 32, size - 32 + 16)]);
    }

    // Run explicitly in release mode to time a malloc/free/realloc churn.
    #[test]
    #[ignore = "manual release benchmark"]
    fn heap_churn_benchmark() {
        use std::{hint::black_box, time::Instant};
        let mut vm = VMAllocator::new(0x1000, 0xf000_0000);
        let mut heap = HeapAllocator::new(&mut vm, HeapAllocator::HEAP_CHUNK_SIZE);
        let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
        let mut live: Vec<VAddr> = Vec::new();
        for _ in 0..30_000 { live.push(heap.alloc(&mut vm, churn_size(&mut rng)).unwrap().base); }
        const OPS: u32 = 2_000_000;
        let start = Instant::now();
        for _ in 0..OPS {
            match rng.below(20) {
                0..=8 => live.push(heap.alloc(&mut vm, churn_size(&mut rng)).unwrap().base),
                9..=17 if !live.is_empty() => {
                    let idx = rng.below(live.len() as u64) as usize;
                    black_box(heap.free(&mut vm, live.swap_remove(idx)));
                }
                _ if !live.is_empty() => {
                    let idx = rng.below(live.len() as u64) as usize;
                    let size = heap.find_allocated_size(live[idx]) + churn_size(&mut rng);
                    live[idx] = realloc(&mut heap, &mut vm, live[idx], size).base;
                }
                _ => {}
            }
        }
        let elapsed = start.elapsed();
        println!("heap churn: {OPS} ops in {elapsed:?} = {:.1} ns/op, {} live, {} backing chunks",
            elapsed.as_nanos() as f64 / OPS as f64, live.len(), heap.backing_chunks.len());
    }
}
