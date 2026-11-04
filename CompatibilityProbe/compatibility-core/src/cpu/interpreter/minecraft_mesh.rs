//! Original section mesh builder scratch initialization, 0xe340c..0xe3418.
//! Each iteration stores D10 then R0, advances R2, compares to R1 and branches.
use super::{CpuState, InterpreterCpu, Mem, Ptr, MESH_INDEX, PC, report};

pub(super) const ENTRY: u32 = 0xe340c;
const CODE: &[u8] = &[130, 237, 0, 171, 144, 96, 12, 50, 138, 66, 249, 209];
const RECORD: u32 = 12;
const COST: u64 = 5;

impl InterpreterCpu {
    pub(super) fn minecraft_mesh_fill(&mut self, mem: &mut Mem, budget: u64) -> Option<(CpuState, u64)> {
        let state = &self.minecraft_worldgen;
        if !state.mesh_native || state.disabled[MESH_INDEX] || budget < COST
            || !self.is_thumb() || self.in_it_block() || self.excl_addr.is_some()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRY), CODE.len() as u32)? != CODE {
            return None;
        }
        let start = self.regs[2];
        let limit = self.regs[1];
        let remaining = limit.checked_sub(start)?;
        if start < self.null_segment_size || start % 4 != 0 || remaining == 0 || remaining % RECORD != 0 {
            return None;
        }
        let call = state.calls[MESH_INDEX] + 1;
        let shadow = state.shadow_only || state.checked[MESH_INDEX] < 128 || call % 4096 == 0;
        let records = (budget / COST).min(u64::from(remaining / RECORD)).min(if shadow { 8 } else { 1024 }) as u32;
        let bytes = records * RECORD;
        let end = start.checked_add(bytes)?;
        if start < ENTRY + CODE.len() as u32 && end > ENTRY { return None; }
        let mut pattern = [0u8; 12];
        for (dst, value) in pattern.chunks_exact_mut(4).zip([self.extregs[20], self.extregs[21], self.regs[0]]) {
            dst.copy_from_slice(&value.to_le_bytes());
        }
        let target = mem.get_bytes_fallible_mut(Ptr::from_bits(start), bytes)?;
        let mut regs = self.regs;
        regs[2] = end;
        regs[PC] = if end == limit { ENTRY + CODE.len() as u32 } else { ENTRY };
        let (result, carry, overflow) = Self::add_with_carry(end, !limit, true);
        let flags = (self.cpsr & 0x0fff_ffff) | (result & 0x8000_0000)
            | (u32::from(result == 0) << 30) | (u32::from(carry) << 29) | (u32::from(overflow) << 28);
        let retired = u64::from(records) * COST;
        self.minecraft_worldgen.calls[MESH_INDEX] = call;
        if shadow {
            for n in 1..=retired {
                let outcome = self.step_one(mem);
                if !matches!(outcome, CpuState::Normal) {
                    self.minecraft_worldgen.disabled[MESH_INDEX] = true;
                    return Some((outcome, n));
                }
            }
            if self.regs == regs && self.cpsr == flags && mem.get_bytes_fallible(Ptr::from_bits(start), bytes)?
                .chunks_exact(RECORD as usize).all(|r| r == pattern) {
                self.minecraft_worldgen.checked[MESH_INDEX] += 1;
                if self.minecraft_worldgen.checked[MESH_INDEX] == 128 { report("shadow_gate_passed", MESH_INDEX, 128); }
            } else {
                self.minecraft_worldgen.disabled[MESH_INDEX] = true;
                report("disabled_after_mismatch", MESH_INDEX, call);
            }
        } else {
            // Doubling the initialized prefix also handles nonzero patterns.
            target[..pattern.len()].copy_from_slice(&pattern);
            let mut filled = pattern.len();
            while filled < target.len() {
                let copy = filled.min(target.len() - filled);
                target.copy_within(..copy, filled);
                filled += copy;
            }
            self.regs = regs;
            self.cpsr = flags;
            if self.minecraft_worldgen.fast[MESH_INDEX] == 0 { report("native_mesh_records", MESH_INDEX, retired); }
            self.minecraft_worldgen.fast[MESH_INDEX] += retired;
        }
        Some((CpuState::Normal, retired))
    }
}
