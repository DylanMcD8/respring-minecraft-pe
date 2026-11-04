//! Scheduling experiment for the verified Minecraft PE 0.9.0 image.
//! Observe the engine's actual readiness test and substitute the order of
//! two homogeneous, still-pending jobs before the worker removes its root.
use super::{InterpreterCpu, Mem, Ptr};
use std::collections::BTreeMap;

pub(super) const OBSERVE: u32 = 0x176b3c;
pub(super) const SELECT: u32 = 0xa8f1a;
const TASK_BYTES: u32 = 56;
const GENERATE: u32 = 0x2408c8;
const COMPLETE: u32 = 0x240898;
type Cell = (i32, i32);

#[derive(Default)]
pub(super) struct Planner {
    owner: u32,
    serial: u64,
    // Only coordinates survive a worker turn. Every guest reference is read
    // afresh from an original, live task when it is considered.
    hints: BTreeMap<Cell, (Cell, u64)>,
    eye: Option<(f32, f32)>,
    heading: (f32, f32),
    pub(super) draw_assist: bool,
    in_terrain_pass: bool,
    // The original renderer exits its grid scan on this exact column. Store
    // coordinates only. No guest pointer is retained after the observation.
    blockers: BTreeMap<Cell, Blocker>,
    observations: u64,
    exact: u64,
    issued: u64,
}

fn word(mem: &Mem, base: u32, offset: u32) -> Option<u32> {
    let addr = base.checked_add(offset)?;
    let bytes = mem.get_bytes_fallible(Ptr::from_bits(addr), 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}
fn stage(mem: &Mem, chunk: u32) -> Option<u8> {
    Some(mem.get_bytes_fallible(Ptr::from_bits(chunk.checked_add(0x44)?), 1)?[0])
}
fn cell(mem: &Mem, chunk: u32) -> Option<Cell> {
    Some((
        word(mem, chunk, 0x1c)? as i32,
        word(mem, chunk, 0x20)? as i32,
    ))
}

// A null pointer and a stage-zero chunk are both missing. Infer a null
// neighbor by elimination, independently of the game's grid storage order.
fn last_missing(center: Cell, ready: u16) -> Option<Cell> {
    let mask = (!ready) & 0x1ff;
    if ready & (1 << 4) == 0 || mask.count_ones() != 1 {
        return None;
    }
    let i = mask.trailing_zeros() as i32;
    Some((
        center.0.checked_add(i % 3 - 1)?,
        center.1.checked_add(i / 3 - 1)?,
    ))
}
fn eye(mem: &Mem, owner: u32) -> Option<(f32, f32)> {
    let source = word(mem, owner, 12)?;
    let node = word(mem, source, 0x44)?;
    if word(mem, node, 0)? != 0 {
        return None;
    } // one player only
    let position = word(mem, node, 8)?;
    let x = f32::from_bits(word(mem, position, 20)?);
    let z = f32::from_bits(word(mem, position, 28)?);
    (x.is_finite() && z.is_finite()).then_some((x, z))
}
#[derive(Clone, Copy)]
struct Blocker {
    section: Cell,
    needs_neighbors: bool,
    observed_at: u64,
}

// A generated column cannot be decorated until its neighboring columns have
// generated. For an absent or ungenerated column, only the cell itself helps.
fn render_rank(blockers: &BTreeMap<Cell, Blocker>, key: Cell, rank: &impl Fn(Cell) -> Option<f32>) -> Option<f32> {
    let mut best = None::<f32>;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (Some(x), Some(z)) = (key.0.checked_add(dx), key.1.checked_add(dz)) else { continue };
            let Some(blocker) = blockers.get(&(x, z)) else { continue };
            let neighbor = dx != 0 || dz != 0;
            if neighbor && !blocker.needs_neighbors { continue; }
            if let Some(score) = rank(blocker.section).filter(|r| *r < 70.) {
                let score = score + if neighbor { 4. } else { 0. };
                best = Some(best.map_or(score, |old| old.min(score)));
            }
        }
    }
    best
}
// A visible section is within 96 blocks, and its blocker and support each
// add at most one chunk. Score each blocker once for a large heap scan.
const RANK_SIDE: usize = 19;
struct RenderRanks {
    origin: Cell,
    scores: [f32; RANK_SIDE * RANK_SIDE],
}

impl RenderRanks {
    fn index(&self, key: Cell) -> Option<usize> {
        let x = key.0.checked_sub(self.origin.0)? as usize;
        let z = key.1.checked_sub(self.origin.1)? as usize;
        if x < RANK_SIDE && z < RANK_SIDE { Some(z * RANK_SIDE + x) } else { None }
    }

    fn build(blockers: &BTreeMap<Cell, Blocker>, eye: (f32, f32), rank: &impl Fn(Cell) -> Option<f32>) -> Option<Self> {
        let origin = |v: f32| ((v / 16.).floor() as i32).saturating_sub(9);
        let mut grid = Self { origin: (origin(eye.0), origin(eye.1)), scores: [f32::INFINITY; RANK_SIDE * RANK_SIDE] };
        for (&cell, blocker) in blockers {
            let Some(score) = rank(blocker.section).filter(|r| *r < 70.) else { continue };
            let radius = if blocker.needs_neighbors { 1 } else { 0 };
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let (Some(x), Some(z)) = (cell.0.checked_add(dx), cell.1.checked_add(dz)) else { continue };
                    // A distant or imprecise coordinate takes the original full
                    // lookup path. Never publish a partially populated grid.
                    let slot = grid.index((x,z))?;
                    let value = if dx == 0 && dz == 0 { score } else { score + 4. };
                    grid.scores[slot] = grid.scores[slot].min(value);
                }
            }
        }
        Some(grid)
    }

    fn lookup(&self, key: Cell) -> Option<f32> {
        let value = self.scores[self.index(key)?];
        value.is_finite().then_some(value)
    }
}
impl Planner {
    fn observe(
        &mut self,
        mem: &Mem,
        region: u32,
        chunk: u32,
        owner: u32,
        trace: bool,
    ) -> Option<()> {
        if self.owner != owner {
            *self = Self {
                owner,
                draw_assist: self.draw_assist,
                ..Self::default()
            };
        }
        let begin = word(mem, region, 168)?;
        let end = word(mem, region, 172)?;
        let count = end.checked_sub(begin)? / 4;
        self.observations += 1;
        if trace && self.observations.is_power_of_two() {
            crate::diagnostics::event(&format!(
                r#"{{"event":"queue_observe","count":{},"size":{},"exact":{},"issued":{}}}"#,
                self.observations, count, self.exact, self.issued
            ));
        }
        if end.checked_sub(begin)? != 36 || stage(mem, chunk)? != 2 {
            return None;
        }
        let center = cell(mem, chunk)?;
        let mut mask = 0u16;
        for i in 0..9 {
            let other = word(mem, begin, 4 * i)?;
            if other == 0 || stage(mem, other)? < 2 {
                continue;
            }
            let (x, z) = cell(mem, other)?;
            let dx = x.checked_sub(center.0)?.checked_add(1)?;
            let dz = z.checked_sub(center.1)?.checked_add(1)?;
            if !(0..3).contains(&dx) || !(0..3).contains(&dz) {
                return None;
            }
            let bit = 1 << (dz * 3 + dx);
            if mask & bit != 0 {
                return None;
            }
            mask |= bit;
        }
        let missing = last_missing(center, mask)?;
        self.exact += 1;
        if self.hints.len() >= 128 && !self.hints.contains_key(&missing) {
            let oldest = self
                .hints
                .iter()
                .min_by_key(|(_, v)| v.1)
                .map(|(k, _)| *k)?;
            self.hints.remove(&oldest);
        }
        self.hints.insert(missing, (center, self.serial));
        Some(())
    }
    fn select(&mut self, mem: &mut Mem, base: u32, end: u32, trace: bool) -> Option<()> {
        let size = end.checked_sub(base)?;
        if size % TASK_BYTES != 0 || !(2..=8192).contains(&(size / TASK_BYTES)) {
            return None;
        }
        let (owner, root_chunk) = terrain_task(mem, base)?;
        if self.owner != owner {
            return None;
        }
        self.serial += 1;
        // Keep at least half the original generation choices. Priority keys,
        // all other task types, callbacks and queue links stay under game control.
        if self.serial % 2 != 0 || (self.hints.is_empty() && self.blockers.is_empty()) {
            return None;
        }
        let pos = eye(mem, owner)?;
        if let Some(prev) = self.eye {
            let (dx, dz) = (pos.0 - prev.0, pos.1 - prev.1);
            let distance = dx.hypot(dz);
            if distance > 96.0 {
                self.hints.clear();
                self.blockers.clear();
                self.heading = (0., 0.);
            } else if distance >= 1.0 {
                self.heading = (dx / distance, dz / distance);
            }
            if distance >= 1.0 {
                self.eye = Some(pos);
            }
        } else {
            self.eye = Some(pos);
        }
        let rank = |c: Cell| {
            let (dx, dz) = (
                c.0 as f32 * 16.0 + 8.0 - pos.0,
                c.1 as f32 * 16.0 + 8.0 - pos.1,
            );
            let dist = dx.hypot(dz);
            let forward = dx * self.heading.0 + dz * self.heading.1;
            (dist <= 96.0 && forward >= -20.0).then_some(dist - 0.5 * forward)
        };
        self.blockers.retain(|_, b| self.serial.saturating_sub(b.observed_at) <= 20);
        let root_cell = cell(mem, root_chunk)?;
        self.hints
            .retain(|_, (_, serial)| self.serial - *serial <= 128);
        if self.hints.contains_key(&root_cell) || rank(root_cell).is_some_and(|r| r < 15.) {
            return None;
        }
        let grid = if self.draw_assist && !self.blockers.is_empty() && size / TASK_BYTES >= 16 {
            RenderRanks::build(&self.blockers, pos, &rank)
        } else { None };
        let blocker_rank = |key| match &grid {
            Some(grid) => grid.lookup(key),
            None => render_rank(&self.blockers, key, &rank),
        };
        let root_block=if self.draw_assist {blocker_rank(root_cell)} else {None};
        let original = word(mem, base, 48)? as i32;
        let mut best = None;
        let mut score = f32::INFINITY;
        for i in 1..(size / TASK_BYTES).min(128) {
            let slot = base + i * TASK_BYTES;
            let Some((src, c)) = terrain_task(mem, slot) else {
                continue;
            };
            if src != owner || (word(mem, slot, 48)? as i32).saturating_sub(original) > 80 {
                continue;
            }
            let key = cell(mem, c)?;
            let dependency=self.hints.get(&key).and_then(|(center,_)|rank(*center));
            let visible=if self.draw_assist {
                blocker_rank(key).filter(|n|root_block.map_or(true,|r|*n+8.<r))
            } else {None};
            if let Some(r) = visible.map(|r|r-128.).or(dependency) {
                if r < score {
                    score = r;
                    best = Some((slot, c, key));
                }
            }
        }
        let (slot, promoted, key) = best?;
        // Both closures contain only a vptr, the source and the chunk.
        // Move their two matching chunk arguments together. Validate the
        // entire span before the first store and leave self-pointers intact.
        let bytes = mem.get_bytes_fallible_mut(Ptr::from_bits(base), slot - base + TASK_BYTES)?;
        for offset in [8usize, 32] {
            bytes[offset..offset + 4].copy_from_slice(&promoted.to_le_bytes());
            let other = (slot - base) as usize + offset;
            bytes[other..other + 4].copy_from_slice(&root_chunk.to_le_bytes());
        }
        self.hints.remove(&key);
        self.issued += 1;
        if trace {
            crate::diagnostics::event(&format!(
                r#"{{"event":"queue_promote","x":{},"z":{},"displaced_x":{},"displaced_z":{},"ahead_score":{},"slot":{}}}"#,
                key.0,
                key.1,
                root_cell.0,
                root_cell.1,
                score,
                (slot - base) / TASK_BYTES
            ));
        }
        Some(())
    }
}
fn terrain_task(mem: &Mem, slot: u32) -> Option<(u32, u32)> {
    if word(mem, slot, 16)? != slot
        || word(mem, slot, 40)? != slot.checked_add(24)?
        || word(mem, slot, 0)? != GENERATE
        || word(mem, slot, 24)? != COMPLETE
    {
        return None;
    }
    let owner = word(mem, slot, 4)?;
    let chunk = word(mem, slot, 8)?;
    if owner != word(mem, slot, 28)? || chunk != word(mem, slot, 32)? || stage(mem, chunk)? != 0 {
        return None;
    }
    Some((owner, chunk))
}
impl InterpreterCpu {
    pub(super) fn queue_point(&mut self, mem: &mut Mem) {
        if !self.is_thumb() || self.in_it_block() || self.excl_addr.is_some() {
            return;
        }
        let trace = self.minecraft_worldgen.chunk_trace;
        let Some(planner) = self.minecraft_worldgen.queue.as_mut() else {
            return;
        };
        match self.regs[15] {
            // The verified terrain pass translates by the negated camera.
            // Do not change the worker's startup choices before entering the
            // world. The first observation of a different owner resets this.
            0x994c8 if planner.draw_assist && planner.owner!=0 && word(mem,0x994c8,0)==Some(0xeb6ef196) => {
                planner.in_terrain_pass=true;
            }
            // BLX lookup at 0x91aaa left r10/r11 at the first missing
            // (chunk x,z); r5 still points to the original render section.
            pc @ (0x91ad4|0x91ad8) if planner.draw_assist && planner.in_terrain_pass && planner.owner!=0 => {
                let expected=if pc==0x91ad4 {0xe0002100} else {0xf8dd2101};
                if word(mem,pc,0)!=Some(expected){return;}
                let Some(x)=word(mem,self.regs[5],4) else{return};
                let Some(z)=word(mem,self.regs[5],12) else{return};
                let center=((x as i32).div_euclid(16),(z as i32).div_euclid(16));
                let key=(self.regs[10] as i32,self.regs[11] as i32);
                if center.0.abs_diff(key.0)>1 || center.1.abs_diff(key.1)>1{return;}
                // r0 is the stage for the below-4 case, zero for null.
                let needs_neighbors=pc==0x91ad8 && self.regs[0]==2;
                if planner.blockers.len()>=128 && !planner.blockers.contains_key(&key){return;}
                planner.blockers.insert(key, Blocker { section: center, needs_neighbors, observed_at: planner.serial });
            }
            OBSERVE if word(mem, OBSERVE, 0) == Some(0x012ae9da) => {
                if let Some(owner) = word(mem, self.regs[13], 0x34) {
                    planner.observe(mem, self.regs[10], self.regs[8], owner, trace);
                }
            }
            SELECT if word(mem, SELECT, 0) == Some(0x94041b41) => {
                planner.select(mem, self.regs[5], self.regs[0], trace);
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minecraft_probe_queue_moves_both_callbacks_and_preserves_keys() {
        let mut mem = Mem::new();
        mem.set_null_segment_size(0x4000);
        let base = mem.alloc(8192).to_bits();
        let owner = base + 512;
        let level = base + 768;
        let node = base + 1024;
        let player = base + 1280;
        let mut put = |a, v: u32| {
            mem.get_bytes_fallible_mut(Ptr::from_bits(a), 4)
                .unwrap()
                .copy_from_slice(&v.to_le_bytes())
        };
        put(owner + 12, level);
        put(level + 0x44, node);
        put(node, 0);
        put(node + 8, player);
        put(player + 20, 0f32.to_bits());
        put(player + 28, 0f32.to_bits());
        let chunks = [base + 1536, base + 1792];
        for (i, c) in chunks.into_iter().enumerate() {
            let slot = base + TASK_BYTES * i as u32;
            for (offset, value) in [
                (0, GENERATE),
                (4, owner),
                (8, c),
                (16, slot),
                (24, COMPLETE),
                (28, owner),
                (32, c),
                (40, slot + 24),
                (48, 64 + i as u32),
            ] {
                put(slot + offset, value);
            }
            put(c + 0x1c, if i == 0 { 5 } else { 3 });
            put(c + 0x20, 0);
        }
        let mut plan = Planner {
            owner,
            serial: 1,
            ..Planner::default()
        };
        plan.hints.insert((3, 0), ((2, 0), 1));
        let end = base + 2 * TASK_BYTES;
        let before = mem
            .get_bytes_fallible(Ptr::from_bits(base), 2 * TASK_BYTES)
            .unwrap()
            .to_vec();
        // A mismatched completion callback cannot be selected.
        mem.get_bytes_fallible_mut(Ptr::from_bits(base + TASK_BYTES + 32), 4)
            .unwrap()
            .copy_from_slice(&chunks[0].to_le_bytes());
        assert!(plan.select(&mut mem, base, end, false).is_none());
        mem.get_bytes_fallible_mut(Ptr::from_bits(base), 2 * TASK_BYTES)
            .unwrap()
            .copy_from_slice(&before);
        plan.serial = 1;
        assert!(plan.select(&mut mem, base, end, false).is_some());
        let mut wanted = before;
        for i in 0..2 {
            for off in [8, 32] {
                let off = i * TASK_BYTES as usize + off;
                wanted[off..off + 4].copy_from_slice(&chunks[1 - i].to_le_bytes());
            }
        }
        assert_eq!(
            mem.get_bytes_fallible(Ptr::from_bits(base), 2 * TASK_BYTES)
                .unwrap(),
            wanted
        );
        assert_eq!(plan.issued, 1);
    }
    #[test]
    fn minecraft_probe_render_dependencies() {
        let cell = (-4, 7);
        let mut blocked = BTreeMap::new();
        blocked.insert(cell, Blocker { section: (-3,7), needs_neighbors:false, observed_at: 1 });
        let rank = |c| (c==(-3,7)).then_some(12.);
        assert_eq!(render_rank(&blocked,cell,&rank),Some(12.));
        assert_eq!(render_rank(&blocked,(-5,7),&rank),None);
        blocked.get_mut(&cell).unwrap().needs_neighbors=true;
        assert_eq!(render_rank(&blocked,(-5,6),&rank),Some(16.));
        assert_eq!(render_rank(&blocked,(-6,7),&rank),None);
        assert_eq!(render_rank(&blocked,cell,&|_|Some(76.)),None);
        assert_eq!(render_rank(&blocked,(i32::MAX,i32::MIN),&rank),None);
    }
    #[test]
    fn minecraft_probe_terrain_rank_grid() {
        let mut blockers = BTreeMap::new();
        for x in -3i32..=4 {
            for z in -4i32..=3 {
                blockers.insert((x,z), Blocker {
                    section: (x-1,z), needs_neighbors: (x+z)%3==0, observed_at: 1,
                });
            }
        }
        let rank = |(x,z): Cell| Some((x * x + z * z) as f32);
        let grid = RenderRanks::build(&blockers, (8., -8.), &rank).unwrap();
        for x in -14..=14 { for z in -14..=14 {
            assert_eq!(grid.lookup((x,z)), render_rank(&blockers,(x,z),&rank));
        }}
        blockers.insert((i32::MAX,0),Blocker {section:(0,0), needs_neighbors:true, observed_at:1});
        assert!(RenderRanks::build(&blockers,(8.,-8.),&rank).is_none());
    }
    #[test]
    fn minecraft_probe_last_missing() {
        for bit in 0..9 {
            let expected = (bit != 4).then_some((-40 + bit % 3 - 1, 17 + bit / 3 - 1));
            assert_eq!(last_missing((-40, 17), 0x1ff ^ (1 << bit)), expected);
        }
        assert!(last_missing((0, 0), 0x1ff).is_none());
        assert!(last_missing((0, 0), 0x1fc).is_none());
    }
}
