//! Four consecutive updates of the verified game's MT state using ARM64 NEON.
//! This is a prefix only: the caller executes the last scalar iteration to
//! restore the original scratch registers, flags and precise instruction cost.
use std::arch::aarch64::*;

pub(super) fn prefix(data: &mut [u8], index: usize, start: u32, count: u32,
                     carry: &mut u32, mask: u32, table: [u32; 2]) -> u32 {
    let (next, other, dest, limit) = match index {
        0 => (8, 0x638, 4, 227),
        1 => (0x394, 4, 0x390, 396),
        _ => return 0,
    };
    if count < 5 || data.len() < 0x9c4 || start > limit || count > limit - start { return 0; }
    let n = (count - 1) & !3;
    // ARM64 Apple targets require NEON and are little-endian. Original guest
    // allocations need not be aligned. For each group both loads precede the
    // store. The overlapping input is one word *ahead* of the output; all
    // other feedback distances are at least 227 words, so grouping four
    // preserves the original read/write dependencies across both MT phases.
    unsafe {
        let base = data.as_mut_ptr().add(start as usize * 4);
        let mut prev = vdupq_n_u32(*carry);
        let high = vdupq_n_u32(0x80000000);
        let mask = vdupq_n_u32(mask);
        let lo = vdupq_n_u32(table[0]);
        let delta = vdupq_n_u32(table[0] ^ table[1]);
        for i in (0..n as usize * 4).step_by(16) {
            let b = base.add(i);
            let cur = vld1q_u32(b.add(next).cast());
            let far = vld1q_u32(b.add(other).cast());
            let combined = vorrq_u32(vandq_u32(vextq_u32::<3>(prev, cur), high), vandq_u32(cur, mask));
            let odd = vtstq_u32(cur, vdupq_n_u32(1));
            let twist = veorq_u32(lo, vandq_u32(odd, delta));
            let value = veorq_u32(veorq_u32(far, twist), vshrq_n_u32::<1>(combined));
            vst1q_u32(b.add(dest).cast(), value);
            prev = cur;
        }
        *carry = vgetq_lane_u32::<3>(prev);
    }
    n
}
