//! Hardware prefix for the verified LevelDB CRC32C loop. The caller executes
//! its last 16 bytes through the translated instructions to restore every
//! temporary register and the exact guest flags.
use super::{Mem, Ptr};

pub(super) const ENTRY: u32 = 0x1eb864;
const TABLE_BASE: u32 = 0x213028;

const fn expected_tables() -> [u8; 4096] {
    let mut bytes = [0; 4096];
    let mut byte = 0;
    while byte < 256 {
        let mut crc = byte as u32;
        let mut slice = 0;
        while slice < 4 {
            let mut bit = 0;
            while bit < 8 {
                crc = (crc >> 1) ^ (0x82f63b78 & 0u32.wrapping_sub(crc & 1));
                bit += 1;
            }
            // The original image stores the slicing tables in order 0,3,2,1.
            let offset = ((4 - slice) % 4) * 1024 + byte * 4;
            let value = crc.to_le_bytes();
            let mut i = 0;
            while i < 4 {
                bytes[offset + i] = value[i];
                i += 1;
            }
            slice += 1;
        }
        byte += 1;
    }
    bytes
}

pub(super) static TABLES: [u8; 4096] = expected_tables();

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "crc")]
unsafe fn extend(mut crc: u32, data: &[u8]) -> u32 {
    for word in data.chunks_exact(8) {
        let word = u64::from_le_bytes(word.try_into().unwrap());
        crc = core::arch::aarch64::__crc32cd(crc, word);
    }
    crc
}

pub(super) fn prefix(mem: &Mem, registers: &mut [u32; 16], visits: u64) -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let start = registers[0];
        let Some(length) = registers[9].checked_sub(start) else { return 0 };
        let groups = visits.min(u64::from(length / 16));
        if length > i32::MAX as u32 || groups < 4 || start < mem.null_segment_size()
            || !std::arch::is_aarch64_feature_detected!("crc")
        {
            return 0;
        }
        // Protect against a modified guest lookup table, even after a prior
        // successful call. Both the table read and the input are read-only.
        if [registers[10], registers[12], registers[14], registers[8]]
            != [TABLE_BASE, TABLE_BASE + 1024, TABLE_BASE + 2048, TABLE_BASE + 3072]
            || mem.get_bytes_fallible(Ptr::from_bits(TABLE_BASE), 4096) != Some(TABLES.as_slice())
        {
            return 0;
        }
        let count = (groups - 1) as u32 * 16;
        // Prove the last translated iteration cannot fault halfway through,
        // since the hardware prefix omits its overwritten scratch registers.
        let Some(data) = mem.get_bytes_fallible(Ptr::from_bits(start), count + 16) else { return 0 };
        // Feature detection above covers both device and host builds.
        registers[3] = unsafe { extend(registers[3], &data[..count as usize]) };
        registers[0] += count;
        return groups - 1;
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (mem, registers, visits); 0 }
}
