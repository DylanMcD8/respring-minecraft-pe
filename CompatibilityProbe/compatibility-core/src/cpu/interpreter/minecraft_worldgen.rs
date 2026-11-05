/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Exact native translations of profiled Minecraft PE 0.9.0 loading work:
//! random numbers, terrain noise/classification, ore, lighting, and atlas scans.
//! No change to seeds, random values, generation quality, or guest scheduling.
//! Each block retires exactly the original instruction count. Short budgets,
//! IT blocks, exclusive reservations, changed code, and faults use the interpreter.
use super::{InterpreterCpu, CpuState, Mem, Ptr, PC};

const ENTRIES: [u32; 9] = [0x1a002c, 0x1a0062, 0x6e414, 0xad2b4, 0x25b1c, 0xbf294, 0x1a3828, 0x1a3858, 0x19ffb8];
const CODE: [&[u8]; 9] = [
    &[0x00,0xeb,0x83,0x02,0x01,0x33,0x91,0x68,0xd2,0xf8,0x38,0x56,0xe3,0x2b,0x01,0xf0,0x01,0x04,0x59,0xf8,0x24,0x40,0x84,0xea,0x05,0x06,0x0e,0xf0,0x00,0x44,0x01,0xea,0x0c,0x05,0x8e,0x46,0x45,0xea,0x04,0x05,0x86,0xea,0x55,0x06,0x56,0x60,0xe7,0xd1],
    &[0x00,0xeb,0x83,0x02,0x06,0xf0,0x00,0x46,0x01,0x33,0xd2,0xf8,0x94,0x13,0x54,0x68,0xb3,0xf5,0xc6,0x7f,0x01,0xf0,0x01,0x05,0x59,0xf8,0x25,0x50,0x85,0xea,0x04,0x05,0x01,0xea,0x0c,0x04,0x46,0xea,0x04,0x06,0x85,0xea,0x56,0x06,0xc2,0xf8,0x90,0x63,0x0e,0x46,0xe5,0xd1],
    &[0x04,0xeb,0x83,0x03,0x5b,0x68,0x83,0xea,0x93,0x73,0x03,0xfb,0x01,0x23,0x04,0xeb,0x82,0x02,0x53,0x60,0xd4,0xf8,0xc4,0x39,0x5a,0x1c,0xc4,0xf8,0xc4,0x29,0xb2,0xf5,0x1c,0x7f,0xed,0xdb],
    &[0x04,0xeb,0x81,0x01,0x49,0x68,0x81,0xea,0x91,0x71,0x01,0xfb,0x0a,0x01,0x04,0xeb,0x80,0x00,0x41,0x60,0xd4,0xf8,0xc4,0x19,0x48,0x1c,0xc4,0xf8,0xc4,0x09,0xb0,0xf5,0x1c,0x7f,0xed,0xdb],
    &[0x11,0x98,0xa4,0xeb,0x0a,0x08,0x08,0xfb,0x06,0xf2,0x82,0x42,0x22,0xdb,0x9c,0x46,0x58,0x46,0xe1,0x46,0x55,0x46,0x99,0xf8,0x00,0xe0,0xbe,0xf1,0x00,0x0f,0x16,0xd0,0x01,0x35,0x09,0xf1,0x01,0x09,0xa5,0x42,0xf5,0xdb,0x10,0x9d,0x01,0x30,0x88,0x42,0xac,0x44,0xee,0xdb,0xcd,0xf8,0x38,0x80,0x0c,0x96,0xcd,0xf8,0x2c,0xa0,0xcd,0xf8,0x28,0xb0,0xcd,0xf8,0x24,0xa0,0xcd,0xf8,0x20,0xb0,0x0d,0x96,0x11,0x92,0x01,0x3c,0x54,0x45,0xd5,0xdc],
    &[0xf0,0xb5,0x03,0xaf,0x4d,0xf8,0x04,0x8d,0x82,0xb0,0x15,0x46,0x06,0x46,0x88,0x46,0x68,0x68,0x80,0x28,0x06,0xd3,0xd8,0xf8,0x00,0x00,0xc0,0xb2,0x02,0xb0,0x5d,0xf8,0x04,0x8b,0xf0,0xbd,0x28,0x68,0x02,0x11,0x00,0x92,0xa8,0x68,0x01,0x11,0x01,0x91,0xb0,0x6a,0x28,0xb1,0xc3,0x69,0x93,0x42,0x04,0xbf,0x02,0x6a,0x8a,0x42,0x1b,0xd0,0x30,0x69,0x34,0x7a,0x01,0x68,0x8a,0x68,0x69,0x46,0x90,0x47,0x54,0xb1,0x00,0x28,0x2c,0xd0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x01,0x29,0x0a,0xd8,0x00,0x20,0xda,0xe7,0x00,0x28,0x34,0xd0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x04,0x29,0x2f,0xd3,0xb0,0x62,0x00,0x28,0x15,0xd0,0x2b,0x68,0xaa,0x68,0x69,0x68,0x4c,0xf2,0x8e,0x36,0xc0,0xf2,0x1c,0x06,0x7e,0x44,0x06,0xf2,0x6c,0x65,0x02,0xf0,0x0f,0x02,0x03,0xf0,0x0f,0x03,0xa8,0x45,0x09,0xd0,0x06,0xf2,0x74,0x66,0xb0,0x45,0x1a,0xd0,0x00,0x20,0xb8,0xe7,0x00,0x20,0xb6,0xe7,0x00,0x20,0xb4,0xe7,0x01,0xf0,0xfe,0x06,0x11,0xf0,0x01,0x0f,0x46,0xea,0xc3,0x23,0x43,0xea,0xc2,0x12,0x00,0xeb,0x52,0x00,0x4c,0xf2,0x4c,0x02,0x80,0x5c,0x16,0xd1,0x00,0xf0,0x0f,0x00,0xa3,0xe7,0x00,0x20,0xa1,0xe7,0x00,0x20,0x9f,0xe7,0x01,0xf0,0xfe,0x06,0x11,0xf0,0x01,0x0f,0x46,0xea,0xc3,0x23,0x43,0xea,0xc2,0x12,0x00,0xeb,0x52,0x00,0x40,0xf2,0x4c,0x02,0xc0,0xf2,0x01,0x02,0x80,0x5c,0x01,0xd0,0x00,0x09,0x8d,0xe7,0x00,0xf0,0x0f,0x00,0x8a,0xe7],
    &[0x00,0x29,0x1a,0xbf,0x08,0x70,0x61,0x68,0x00,0x21,0x01,0x3d,0x01,0xf1,0x01,0x01,0x61,0x60,0xf5,0xd1],
    &[0x00,0x2e,0x14,0xbf,0x30,0x70,0x00,0x26,0x01,0x3d,0x06,0xf1,0x01,0x06,0xf7,0xd1],
    &[0xf0,0xb5,0xd0,0xf8,0xc4,0x19,0x03,0xaf,0xb1,0xf5,0x1c,0x7f,0xc0,0xf2,0x7d,0x80,0x40,0xf2,0x71,0x22,0x91,0x42,0x21,0xd1,0x4e,0xf6,0x96,0x63,0x02,0x21,0x41,0xf2,0x71,0x52,0xc4,0xf6,0x98,0x53,0xc0,0xf8,0xc4,0x19,0xc0,0xe9,0x01,0x23,0x48,0xf6,0x65,0x12,0xc6,0xf6,0x07,0x42,0x02,0xe0,0x00,0xeb,0x83,0x03,0x5b,0x68,0x83,0xea,0x93,0x73,0x03,0xfb,0x02,0x13,0x00,0xeb,0x81,0x01,0x4b,0x60,0xd0,0xf8,0xc4,0x39,0x59,0x1c,0xc0,0xf8,0xc4,0x19,0xb1,0xf5,0x1c,0x7f,0xed,0xdb,0xd0,0xf8,0x04,0xe0,0x42,0xf2,0xea,0x59,0x4f,0xf6,0xfe,0x7c,0x00,0x23,0xc0,0xf2,0x07,0x09,0xc7,0xf6,0xff,0x7c,0xf9,0x44,0x00,0xeb,0x83,0x02,0x01,0x33,0x91,0x68,0xd2,0xf8,0x38,0x56,0xe3,0x2b,0x01,0xf0,0x01,0x04,0x59,0xf8,0x24,0x40,0x84,0xea,0x05,0x06,0x0e,0xf0,0x00,0x44,0x01,0xea,0x0c,0x05,0x8e,0x46,0x45,0xea,0x04,0x05,0x86,0xea,0x55,0x06,0x56,0x60,0xe7,0xd1,0xd0,0xf8,0x90,0x63,0x00,0x23,0x00,0xeb,0x83,0x02,0x06,0xf0,0x00,0x46,0x01,0x33,0xd2,0xf8,0x94,0x13,0x54,0x68,0xb3,0xf5,0xc6,0x7f,0x01,0xf0,0x01,0x05,0x59,0xf8,0x25,0x50,0x85,0xea,0x04,0x05,0x01,0xea,0x0c,0x04,0x46,0xea,0x04,0x06,0x85,0xea,0x56,0x06,0xc2,0xf8,0x90,0x63,0x0e,0x46,0xe5,0xd1,0xd0,0xf8,0xc0,0x39,0x41,0x68,0x01,0xea,0x0c,0x02,0x01,0xf0,0x01,0x01,0x03,0xf0,0x00,0x43,0x1a,0x43,0x59,0xf8,0x21,0x10,0xd0,0xf8,0x34,0x36,0x59,0x40,0x81,0xea,0x52,0x02,0x00,0x21,0xc0,0xf8,0xc0,0x29,0xc0,0xf8,0xc4,0x19,0x4a,0x1c,0xc0,0xf8,0xc4,0x29,0x00,0xeb,0x81,0x00,0x45,0xf2,0x80,0x61,0x40,0x68,0xc9,0xf6,0x2c,0x51,0x80,0xea,0xd0,0x20,0x01,0xea,0xc0,0x11,0x48,0x40,0x00,0x21,0xce,0xf6,0xc6,0x71,0x01,0xea,0xc0,0x31,0x48,0x40,0x80,0xea,0x90,0x40,0xf0,0xbd],
];
// One indexed bit test keeps native dispatch out of ordinary instructions.
// The table covers only the verified main image, never the relocated dylibs.
const FRONTIER_SEED:u32=0x1c7e56;
const FRONTIER_CODE:&[u8]=&[3,235,130,2,82,104,130,234,146,114,2,251,0,18,3,235,129,1,74,96,214,248,208,41,81,28,198,248,208,25,177,245,28,127,237,219];
const FRONTIER_INDEX:usize=36+super::minecraft_loading::BLOCKS.len();
#[path = "minecraft_mesh.rs"]
mod mesh;
const MESH_INDEX: usize = FRONTIER_INDEX + 1;
const BLOCK_COUNT: usize = MESH_INDEX + 1;
const DISPATCH_PCS:&[u32]=&[0x1a002c,0x1a0062,0x6e414,0xad2b4,0x25b1c,0xbf294,0xbf310,0x1a3828,0x1a3858,0x19ffb8,0x4fb92,0x1947b8,0x12d924,0x48214,0x482b2,0x25b32,0x47b08,0x47b44,0x483aa,0x483c6,0x483e2,0x483fe,0x48418,0x48430,0x4e55a,0x176d70,0x1642c2,0x1767a8,0x1767e4,0x176804,super::minecraft_queue::OBSERVE,super::minecraft_queue::SELECT,0x176f9c,0x91cc6,0x91ad4,0x91ad8,0x994c8,mesh::ENTRY,FRONTIER_SEED];
const DISPATCH_LEN:usize={
    let mut max=0;let mut i=0;
    while i<DISPATCH_PCS.len(){if DISPATCH_PCS[i]>max{max=DISPATCH_PCS[i];}i+=1;}
    i=0;while i<super::minecraft_loading::BLOCKS.len(){let pc=super::minecraft_loading::BLOCKS[i].0;if pc>max{max=pc;}i+=1;}
    ((max as usize)>>7)+1
};
const fn dispatch_words()->[u64;DISPATCH_LEN]{
    let mut words=[0;DISPATCH_LEN];let mut i=0;
    while i<DISPATCH_PCS.len(){let pc=DISPATCH_PCS[i];words[(pc>>7) as usize]|=1u64<<((pc>>1)&63);i+=1;}
    i=0;while i<super::minecraft_loading::BLOCKS.len(){let pc=super::minecraft_loading::BLOCKS[i].0;words[(pc>>7) as usize]|=1u64<<((pc>>1)&63);i+=1;}words
}
static DISPATCH:[u64;DISPATCH_LEN]=dispatch_words();
// Chained islands skip the dispatcher, so no island may start at a PC that
// it routes elsewhere or that the bitmap test excludes.
const _:()={
    let mut i=0;while i<super::minecraft_loading::BLOCKS.len(){
        let pc=super::minecraft_loading::BLOCKS[i].0;assert!(pc>=0x1bd38);
        let mut j=0;while j<DISPATCH_PCS.len(){assert!(DISPATCH_PCS[j]!=pc);j+=1;}i+=1;
    }
};
#[inline(always)] fn native_entry(pc:u32)->bool{
    pc>=0x1bd38 && (pc>>7)<DISPATCH_LEN as u32 && DISPATCH[(pc>>7) as usize]&(1u64<<((pc>>1)&63))!=0
}
pub(super) struct State {
    pub enabled: bool,
    neighbors_enabled: bool,
    hot_loops_enabled: bool,
    frontier_enabled: bool,
    pub(super) queue: Option<Box<super::minecraft_queue::Planner>>,
    pub(super) chunk_trace: bool,
    mesh_native: bool,
    loading_enabled: bool,
    terrain_enabled: bool,
    streaming_enabled: bool,
    calls: [u64; BLOCK_COUNT],
    checked: [u64; BLOCK_COUNT],
    fast: [u64; BLOCK_COUNT],
    disabled: [bool; BLOCK_COUNT],
    shadow_only: bool,
    light_fallback: [u64; 8],
    census: Option<Box<Census>>,
    // Retired instructions in finished batches.
    retired: u64,
    window: Option<Box<Window>>,
    #[cfg(test)] fault: bool,
    #[cfg(test)] single_island: bool,
}
impl Default for State {
    fn default()->Self {Self {enabled:false,neighbors_enabled:true,hot_loops_enabled:true,frontier_enabled:true,queue:None,chunk_trace:false,mesh_native:true,loading_enabled:true,terrain_enabled:true,streaming_enabled:true,calls:[0;BLOCK_COUNT],checked:[0;BLOCK_COUNT],fast:[0;BLOCK_COUNT],disabled:[false;BLOCK_COUNT],shadow_only:false,light_fallback:[0;8],census:None,retired:0,window:None,#[cfg(test)]fault:false,#[cfg(test)]single_island:false}}
}
// Counter snapshot for the five-second native coverage event.
struct Window { next:u64, last:std::time::Instant, retired:u64, calls:[u64;BLOCK_COUNT], checked:[u64;BLOCK_COUNT], fast:[u64;BLOCK_COUNT] }
#[derive(Default)]
struct Census { remaining:u32, random:u32, samples:u64, pcs:std::collections::BTreeMap<u32,u64> }
impl Census {
    fn sample(&mut self,pc:u32) {
        if self.remaining>0{self.remaining-=1;return;}
        self.random^=self.random<<13;self.random^=self.random>>17;self.random^=self.random<<5;
        self.remaining=512+(self.random&1023);
        *self.pcs.entry(pc&!63).or_default()+=1;self.samples+=1;
        if self.samples%16384==0 {
            let mut top:Vec<_>=self.pcs.iter().map(|(&pc,&count)|(pc,count)).collect();top.sort_by_key(|x|std::cmp::Reverse(x.1));top.truncate(40);
            let entries=top.iter().map(|(p,c)|format!("[{},{}]",p,c)).collect::<Vec<_>>().join(",");
            crate::diagnostics::event(&format!("{{\"event\":\"minecraft_interpreted_census\",\"samples\":{},\"top\":[{}]}}",self.samples,entries));
            self.pcs.clear();
        }
    }
}
const ORE_CODE:&[u8]=&[0x06,0x46,0xb0,0xee,0x4f,0x2a,0x00,0xee,0x10,0x6a,0xb8,0xee,0xc0,0x0a,0x30,0xee,0x29,0x0a,0x30,0xee,0x6f,0x0a,0x80,0xee,0x2b,0x0a,0x00,0xee,0x00,0x2a,0xb4,0xee,0xee,0x2a,0xf1,0xee,0x10,0xfa,0x56,0xda,0xb8,0xf1,0x7f,0x0f,0x19,0xd8,0x0f,0x98,0x31,0x11,0x11,0x90,0x10,0x98,0x12,0x91,0x80,0x6a,0x30,0xb1,0xc2,0x69,0x0f,0x9b,0x9a,0x42,0x04,0xbf,0x02,0x6a,0x8a,0x42,0x22,0xd0,0x10,0x98,0x04,0x7a,0x00,0x69,0x01,0x68,0x8a,0x68,0x11,0xa9,0x90,0x47,0x00,0x2c,0x04,0xd1,0x30,0xb9,0x00,0x20,0x22,0xe0,0x00,0x20,0x20,0xe0,0x50,0xb9,0x00,0x20,0x1d,0xe0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x04,0x29,0x08,0xd2,0x00,0x20,0x14,0xe0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x01,0x29,0x0c,0xd9,0x10,0x99,0x88,0x62,0x08,0xb9,0x00,0x20,0x08,0xe0,0x0a,0x9a,0x0a,0xf4,0xf0,0x61,0x11,0x43,0x08,0x44,0x90,0xf8,0x4c,0x00,0x00,0xe0,0x00,0x20,0x46,0xf6,0x46,0x61,0xc0,0xf2,0x0f,0x01,0x79,0x44,0xd1,0xf8,0x80,0x10,0x91,0xf8,0x44,0x10,0x81,0x42,0x0b,0xd1,0x0d,0x98,0x11,0xa9,0x04,0x23,0xb0,0xf8,0x05,0x20,0x10,0x98,0x11,0x95,0xcd,0xf8,0x48,0x80,0x13,0x96,0xdb,0xf6,0x84,0xf9,0x0a,0xf1,0x80,0x0a,0x70,0x1c,0x5e,0x45,0x8f,0xdb];
const RESOLVER_CODE:&[u8]=&[0x80,0xb5,0xd0,0xf8,0x78,0xe0,0x0b,0x68,0x6f,0x46,0x73,0x45,0x15,0xdb,0xd0,0xf8,0x84,0x20,0x93,0x42,0xdc,0xbf,0xd0,0xf8,0x7c,0x90,0xb9,0xf1,0x00,0x0f,0x0c,0xdc,0xd0,0xf8,0x88,0x20,0x00,0x2a,0x08,0xdb,0x49,0x68,0xd0,0xf8,0x80,0xc0,0x61,0x45,0x03,0xdb,0xd0,0xf8,0x8c,0x20,0x91,0x42,0x01,0xdd,0x00,0x20,0x80,0xbd,0xa3,0xeb,0x0e,0x0e,0xd0,0xf8,0x90,0x30,0xd0,0xf8,0x9c,0x20,0xd0,0xf8,0xa8,0x00,0xa1,0xeb,0x0c,0x01,0x03,0xfb,0x01,0xe1,0x02,0xfb,0x19,0x11,0x50,0xf8,0x21,0x00,0x80,0xbd];
const CELL_CODE:&[u8]=&[0x2a,0x98,0xd8,0xf8,0x1c,0x50,0x04,0x93,0x02,0x11,0x2f,0x92,0x2c,0x98,0x01,0x11,0x30,0x91,0xa8,0x6a,0x28,0xb1,0xc3,0x69,0x93,0x42,0x04,0xbf,0x02,0x6a,0x8a,0x42,0x1f,0xd0,0x28,0x69,0x2e,0x7a,0x01,0x68,0x8a,0x68,0x2f,0xa9,0x90,0x47,0x00,0x2e,0x0b,0xd1,0x00,0x28,0x00,0xf0,0x94,0x80,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x04,0x29,0xc0,0xf0,0x8c,0x80,0x0a,0xe0,0x00,0x28,0x00,0xf0,0x88,0x80,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x01,0x29,0x40,0xf2,0x80,0x80,0xa8,0x62,0x00,0x28,0x00,0xf0,0x7c,0x80,0x2a,0x99,0x2c,0x9a,0x01,0xf0,0x0f,0x01,0x62,0xf3,0x07,0x11,0x08,0x44,0x44,0xf2,0x4c,0x41,0xc0,0xf2,0x01,0x01,0x40,0x5c,0x9d,0xf8,0xac,0x10,0x81,0x42,0x80,0xf0,0x6b,0x80,0x04,0x9b,0x00,0x20,0x7a,0xe0,0xd8,0xf8,0x00,0x10,0xd8,0xf8,0x1c,0x00,0x2a,0xaa,0x76,0xf0,0xf1,0xff,0x04,0x46,0x2b,0x98,0x7f,0x28,0x1a,0xd8,0x2a,0x98,0xd8,0xf8,0x1c,0x50,0x02,0x11,0x2d,0x92,0x2c,0x98,0x01,0x11,0x2e,0x91,0xa8,0x6a,0x28,0xb1,0xc3,0x69,0x93,0x42,0x04,0xbf,0x02,0x6a,0x8a,0x42,0x20,0xd0,0x28,0x69,0x2e,0x7a,0x01,0x68,0x8a,0x68,0x2d,0xa9,0x90,0x47,0x00,0x2e,0x04,0xd1,0x30,0xb9,0x00,0x20,0x2b,0xe0,0x00,0x20,0x29,0xe0,0x50,0xb9,0x00,0x20,0x26,0xe0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x04,0x29,0x08,0xd2,0x00,0x20,0x1d,0xe0,0x90,0xf8,0x44,0x10,0xbf,0xf3,0x5b,0x8f,0xc9,0xb2,0x01,0x29,0x15,0xd9,0xa8,0x62,0x08,0xb9,0x00,0x20,0x12,0xe0,0x2c,0x99,0x4f,0xf4,0xf0,0x62,0x4f,0xf4,0xf0,0x43,0x02,0xea,0xc1,0x11,0x2a,0x9a,0x03,0xea,0xc2,0x22,0x9d,0xf8,0xac,0x30,0x1a,0x43,0x11,0x43,0x08,0x44,0x90,0xf8,0x4c,0x00,0x00,0xe0,0x00,0x20];
const COLUMN_CODE:&[u8]=&[0x53,0xfa,0x81,0xf6,0x6,0x44,0x96,0xf8,0x4c,0x60,0x2,0xeb,0x86,0x6,0xd6,0xf8,0x0,0x64,0x26,0xb9,0x1,0x39,0x11,0xf0,0xff,0xf,0xf1,0xd1,0x0,0x21,0x49,0xea,0xb,0x3,0x44,0xf2,0x4c,0x46,0x3,0x44,0xc0,0xf2,0x1,0x6,0x99,0x55,0x1,0x68,0x91,0xf8,0x2d,0x1a,0x61,0xbb,0xf,0x21,0x4f,0xf0,0x7f,0x9,0x54,0xfa,0x89,0xf3,0x3,0x44,0x93,0xf8,0x4c,0x30,0x2,0xeb,0x83,0x3,0xd3,0xf8,0x0,0x34,0xc9,0x1a,0x1,0x29,0x1d,0xdb,0x5f,0xfa,0x89,0xf6,0x46,0xea,0x5,0x3,0x16,0xf0,0x1,0xf,0x0,0xeb,0x53,0x3,0x3,0xeb,0xe,0xc,0x13,0xf8,0xe,0x30,0x4,0xd1,0x1,0xf0,0xf,0x6,0x3,0xf0,0xf0,0x3,0x3,0xe0,0xa,0xea,0x1,0x16,0x3,0xf0,0xf,0x3,0xa9,0xf1,0x1,0x9,0x33,0x43,0x8c,0xf8,0x0,0x30,0x19,0xf0,0xff,0xf,0xd5,0xd1];
const NEIGHBOR_CODE:&[u8]=&[0x2a,0xae,0x3,0x90,0xd8,0xf8,0x0,0x10,0xd8,0xf8,0x1c,0x0,0x27,0xac,0x4c,0xce,0x1,0x3a,0x84,0xe8,0x4c,0x0,0x27,0xaa,0x76,0xf0,0x67,0xff,0x2a,0xae,0x82,0x46,0xd8,0xf8,0x0,0x10,0xd8,0xf8,0x1c,0x0,0x24,0xac,0x4c,0xce,0x1,0x32,0x84,0xe8,0x4c,0x0,0x24,0xaa,0x76,0xf0,0x59,0xff,0x2a,0xae,0x83,0x46,0xd8,0xf8,0x0,0x10,0xd8,0xf8,0x1c,0x0,0x4c,0xce,0x21,0x92,0x5a,0x1c,0x22,0x92,0x21,0xaa,0x23,0x96,0x76,0xf0,0x4b,0xff,0x46,0x46,0x2a,0xad,0x80,0x46,0x31,0x68,0xf0,0x69,0x2c,0xcd,0x1e,0x92,0x5a,0x1e,0x1f,0x92,0x1e,0xaa,0x20,0x95,0x76,0xf0,0x3e,0xff,0x2a,0xac,0x5,0x46,0x31,0x68,0xf0,0x69,0x1c,0xcc,0x1b,0x92,0x62,0x1c,0x1c,0x93,0x1d,0x92,0x1b,0xaa,0x76,0xf0,0x32,0xff,0x31,0x68,0xf2,0x69,0x2a,0xae,0xda,0x45,0x58,0xce,0x18,0x93,0xa6,0xf1,0x1,0x3,0x19,0x94,0x1a,0x93,0x38,0xbf,0xda,0x46,0xc2,0x45,0x38,0xbf,0xc2,0x46,0xaa,0x45,0x38,0xbf,0xaa,0x46,0x82,0x45,0x38,0xbf,0x82,0x46,0x10,0x46,0x18,0xaa,0x76,0xf0,0x19,0xff];
const SURFACE_CODE:&[u8]=&[0xa0,0xfb,0x6,0x12,0x91,0x8,0x1,0xeb,0x81,0x1,0x40,0x1a,0x80,0x45,0x10,0xd9,0x23,0x46,0x45,0xea,0x8,0x4,0x55,0x46,0x5,0xeb,0x4,0xa,0x1a,0xf8,0x4c,0x1f,0x0,0x29,0x10,0xd1,0xaa,0x46,0x7,0x9d,0x4f,0xf0,0xff,0x30,0x1c,0x46,0x6,0x90,0xe8,0xe0,0xd4,0xf8,0x64,0x14,0x45,0xea,0x8,0x0,0x50,0x44,0x91,0xf8,0x44,0x10,0x80,0xf8,0x4c,0x10,0xde,0xe0,0x4f,0xf6,0xb4,0x40,0xc0,0xf2,0x23,0x0,0x78,0x44,0x50,0xf8,0x21,0x20,0x18,0x46,0x53,0xf8,0xc8,0x9f,0x0,0x2a,0x18,0xbf,0x2,0xf1,0x6c,0x3,0x1a,0x68,0x4a,0x45,0x6,0xd1,0xaa,0x46,0x7,0x9d,0x4f,0xf0,0xff,0x31,0x4,0x46,0x6,0x91,0xc6,0xe0,0xd0,0xf8,0x80,0x20,0x92,0xf8,0x44,0x20,0x91,0x42,0x40,0xf0,0xb9,0x80];
const SURFACE_RESET:&[u8]=&[0xaa,0x46,0x7,0x9d,0x4f,0xf0,0xff,0x31,0x4,0x46,0x6,0x91,0xc6,0xe0];
const SURFACE_TAIL:&[u8]=&[0xaa,0x46,0x7,0x9d,0x4c,0xf6,0x50,0x74,0xc0,0xf2,0x23,0x4,0x7c,0x44,0xa8,0xf1,0x1,0x0,0x18,0xf0,0xff,0xf,0x7f,0xf4,0xf4,0xae];
const SURFACE_HEAD:&[u8]=&[0x80,0x46,0x58,0x46,0x51,0xf1,0x2f,0xfd];
struct Prediction { regs: [u32; 16], cpsr: u32, address: u32, value: u32 }
fn report(kind: &str, index: usize, count: u64) {
    crate::diagnostics::event(&format!("{{\"event\":\"minecraft_worldgen\",\"kind\":\"{kind}\",\"block\":{index},\"count\":{count}}}"));
}
impl InterpreterCpu {
    pub(crate) fn configure_minecraft_worldgen(&mut self, verified: bool, background_terrain: bool) {
        self.minecraft_worldgen = State {
            enabled: verified && std::env::var("MINECRAFT_NATIVE_WORLDGEN").as_deref() != Ok("0"),
            neighbors_enabled:std::env::var("MINECRAFT_NATIVE_NEIGHBORS").as_deref()!=Ok("0"),
            hot_loops_enabled:std::env::var("MINECRAFT_NATIVE_LOOPS").as_deref()!=Ok("0"),
            frontier_enabled:std::env::var("MINECRAFT_NATIVE_FRONTIER").as_deref()!=Ok("0"),
            queue: (verified && background_terrain && std::env::var("MINECRAFT_QUEUE_ASSIST").as_deref() != Ok("0")).then(Box::default),
            mesh_native: std::env::var("MINECRAFT_EXTRA_NATIVE").as_deref() != Ok("0"),
            chunk_trace:verified && std::env::var("MINECRAFT_CHUNK_TRACE").as_deref()==Ok("1"),
            loading_enabled: std::env::var("MINECRAFT_NATIVE_LOADING").as_deref()!=Ok("0"),
            terrain_enabled: std::env::var("MINECRAFT_NATIVE_TERRAIN").as_deref()!=Ok("0"),
            streaming_enabled: std::env::var("MINECRAFT_NATIVE_STREAMING").as_deref()!=Ok("0"),
            census: self.pc_sampling.as_ref().filter(|_|std::env::var("MINECRAFT_CENSUS").as_deref()==Ok("1")).map(|_|Box::new(Census{remaining:1024,random:0xabcdef01,..Census::default()})),
            shadow_only: std::env::var("MINECRAFT_WORLDGEN_SHADOW_ONLY").as_deref() == Ok("1"),
            // MINECRAFT_PROFILE runs turn on diagnostics and guest PC sampling together.
            window: crate::diagnostics::pc_sampling_enabled().then(||Box::new(Window{next:0,last:std::time::Instant::now(),retired:0,calls:[0;BLOCK_COUNT],checked:[0;BLOCK_COUNT],fast:[0;BLOCK_COUNT]})),
            ..State::default()
        };
        if let Some(planner) = self.minecraft_worldgen.queue.as_mut() {
            planner.draw_assist = std::env::var("MINECRAFT_DRAW_ASSIST").as_deref() != Ok("0");
        }
        if self.minecraft_worldgen.enabled { report("image_verified", 0, 0); }
    }
    // Diagnostic only: right after each original successful STREXB. The game
    // reads the same state byte at 0x91ab0 and requires state >=4 to mesh.
    #[cold]
    fn trace_chunk_transition(&self,mem:&Mem) {
        let (address,status,stage)=match self.regs[PC] {
            0x176d70=>(1,2,1),0x1642c2=>(0,3,2),0x1767a8=>(0,1,3),
            0x1767e4=>(1,3,4),0x176804=>(1,3,2),_=>return,
        };
        if self.regs[status]!=0{return;}
        let Some(chunk)=self.regs[address].checked_sub(0x44) else{return};
        if self.data_r_u8(mem,chunk+0x44)!=Some(stage){return;}
        if let (Some(x),Some(z))=(self.data_r_u32(mem,chunk+0x1c),self.data_r_u32(mem,chunk+0x20)) {
            let time=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
            crate::diagnostics::event(&format!("{{\"event\":\"frontier_chunk\",\"stage\":{stage},\"x\":{},\"z\":{},\"unix_time_ms\":{time}}}",x as i32,z as i32));
        }
    }
    fn predict_minecraft_twist(&self, mem: &Mem, index: usize) -> Option<Prediction> {
        if !self.is_thumb() || self.in_it_block() || self.excl_addr.is_some()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[index]), CODE[index].len() as u32)? != CODE[index] {
            return None;
        }
        let mut r = self.regs;
        r[2] = r[0].wrapping_add(r[3].wrapping_shl(2));
        r[3] = r[3].wrapping_add(1);
        let (limit, address, value) = if index == 0 {
            r[1] = self.data_r_u32(mem, r[2].wrapping_add(8))?;
            r[5] = self.data_r_u32(mem, r[2].wrapping_add(0x638))?;
            r[4] = self.data_r_u32(mem, r[9].wrapping_add((r[1] & 1) * 4))?;
            r[6] = r[4] ^ r[5];
            r[4] = r[14] & 0x80000000;
            r[5] = (r[1] & r[12]) | r[4];
            r[14] = r[1];
            r[6] ^= r[5] >> 1;
            (0xe3, r[2].wrapping_add(4), r[6])
        } else {
            r[6] &= 0x80000000;
            r[1] = self.data_r_u32(mem, r[2].wrapping_add(0x394))?;
            r[4] = self.data_r_u32(mem, r[2].wrapping_add(4))?;
            r[5] = self.data_r_u32(mem, r[9].wrapping_add((r[1] & 1) * 4))? ^ r[4];
            r[4] = r[1] & r[12];
            r[6] = r[5] ^ ((r[6] | r[4]) >> 1);
            let value = r[6];
            r[6] = r[1];
            (0x18c, r[2].wrapping_add(0x390), value)
        };
        // Validate the sole store before committing any state. Reads all precede
        // that store, so aliases (including the lookup table) retain ARM ordering.
        self.data_r_u32(mem, address)?;
        let (result, carry, overflow) = Self::add_with_carry(r[3], !limit, true);
        let cpsr = (self.cpsr & 0x0fff_ffff) | (result & 0x80000000)
            | (u32::from(result == 0) << 30) | (u32::from(carry) << 29)
            | (u32::from(overflow) << 28);
        r[PC] = if result != 0 { ENTRIES[index] } else { ENTRIES[index] + CODE[index].len() as u32 };
        Some(Prediction { regs: r, cpsr, address, value })
    }
    fn minecraft_twist(&mut self, mem: &mut Mem, budget: u64) -> Option<(CpuState, u64)> {
        let index = match self.regs[PC] { 0x1a002c => 0, 0x1a0062 => 1, _ => return None };
        if budget < 15 || self.minecraft_worldgen.disabled[index] { return None; }
        let p = self.predict_minecraft_twist(mem, index)?;
        self.minecraft_worldgen.calls[index] += 1;
        let s = &self.minecraft_worldgen;
        if s.checked[index] < 128 || s.calls[index] % 4096 == 0 || s.shadow_only {
            let before_fp = self.extregs;
            let before_fpscr = self.fpscr;
            #[cfg(test)] let mut p = p;
            #[cfg(test)] if self.minecraft_worldgen.fault { p.regs[8] ^= 1; }
            let mut state = CpuState::Normal;
            let mut used = 0;
            for _ in 0..15 {
                state = self.step_one(mem); used += 1;
                if !matches!(state, CpuState::Normal) { break; }
            }
            let equal = matches!(state, CpuState::Normal) && self.regs == p.regs
                && self.cpsr == p.cpsr && self.extregs == before_fp && self.fpscr == before_fpscr
                && self.data_r_u32(mem, p.address) == Some(p.value);
            if equal {
                self.minecraft_worldgen.checked[index] += 1;
                if self.minecraft_worldgen.checked[index] == 128 { report("shadow_gate_passed", index, 128); }
            } else {
                self.minecraft_worldgen.disabled[index] = true;
                report("disabled_after_mismatch", index, self.minecraft_worldgen.calls[index]);
            }
            // Preserve the interpreter result on every shadow, including failure.
            Some((state, used))
        } else {
            let iterations = if let Some(iterations) = self.minecraft_twist_batch(mem, index, budget) {
                iterations
            } else {
                assert!(self.data_w_u32(mem, p.address, p.value));
                self.regs = p.regs; self.cpsr = p.cpsr;
                1
            };
            let previous = self.minecraft_worldgen.fast[index];
            self.minecraft_worldgen.fast[index] += iterations;
            let count = self.minecraft_worldgen.fast[index];
            if previous == 0 || previous / 65536 != count / 65536 { report("native_blocks", index, count); }
            Some((CpuState::Normal, iterations * 15))
        }
    }
    // Once validated, fuse consecutive iterations within the current CPU
    // quantum. The small single-iteration predictor remains the fallback for
    // unusual indices/aliases; the interpreter remains the shadow oracle.
    fn minecraft_twist_batch(&mut self, mem: &mut Mem, index: usize, budget: u64) -> Option<u64> {
        let base = self.regs[0];
        let end = base.checked_add(0x9c4)?;
        let table = self.regs[9];
        let table_end = table.checked_add(8)?;
        let limit = if index == 0 { 227 } else { 396 };
        if self.regs[3] >= limit || base < self.null_segment_size
            || (table < end && table_end > base)
            || ENTRIES.iter().zip(CODE).any(|(&a, code)| base < a + code.len() as u32 && end > a) {
            return None;
        }
        let lookup = [self.data_r_u32(mem, table)?, self.data_r_u32(mem, table + 4)?];
        let data = mem.get_bytes_fallible_mut(Ptr::from_bits(base), 0x9c4)?;
        let iterations = (budget / 15).min(u64::from(limit - self.regs[3]));
        #[inline(always)] fn read(data: &[u8], offset: usize) -> u32 {
            u32::from_le_bytes(data[offset..offset+4].try_into().unwrap())
        }
        let r = &mut self.regs;
        #[cfg(all(target_arch="aarch64",target_endian="little"))]
        let done=if self.minecraft_worldgen.neighbors_enabled {
            let mask=r[12];let start=r[3];
            let n=super::minecraft_twist::prefix(data,index,start,iterations as u32,&mut r[if index==0 {14} else {6}],mask,lookup);
            r[3]+=n;u64::from(n)
        } else {0};
        #[cfg(not(all(target_arch="aarch64",target_endian="little")))]
        let done=0;
        for _ in done..iterations {
            let offset = r[3] as usize * 4;
            r[2] = base + offset as u32;
            r[3] += 1;
            let (at, value) = if index == 0 {
                r[1] = read(data, offset + 8);
                r[5] = read(data, offset + 0x638);
                r[4] = lookup[(r[1] & 1) as usize];
                r[6] = r[4] ^ r[5];
                r[4] = r[14] & 0x80000000;
                r[5] = (r[1] & r[12]) | r[4];
                r[14] = r[1];
                r[6] ^= r[5] >> 1;
                (offset + 4, r[6])
            } else {
                r[1] = read(data, offset + 0x394);
                r[4] = read(data, offset + 4);
                r[5] = lookup[(r[1] & 1) as usize] ^ r[4];
                r[4] = r[1] & r[12];
                let value = r[5] ^ (((r[6] & 0x80000000) | r[4]) >> 1);
                r[6] = r[1];
                (offset + 0x390, value)
            };
            data[at..at+4].copy_from_slice(&value.to_le_bytes());
        }
        let (result, carry, overflow) = Self::add_with_carry(r[3], !limit, true);
        r[PC] = if result != 0 { ENTRIES[index] } else { ENTRIES[index] + CODE[index].len() as u32 };
        self.set_nzcv(result, carry, overflow);
        Some(iterations)
    }
    fn minecraft_seed(&mut self, mem: &mut Mem, budget: u64) -> Option<(CpuState, u64)> {
        // The inlined frontier generator keeps the same MT layout on the stack,
        // with a separate register pointing twelve bytes before its state.
        let pc=self.regs[PC];
        let (index,previous,next,multiplier,base,code)=match pc {
            0x6e414=>(2,3,2,1,self.regs[4],CODE[2]),
            0xad2b4=>(3,1,0,10,self.regs[4],CODE[3]),
            FRONTIER_SEED if self.minecraft_worldgen.frontier_enabled && self.minecraft_worldgen.streaming_enabled && self.regs[6].checked_add(12)==Some(self.regs[3]) =>
                (FRONTIER_INDEX,2,1,0,self.regs[3],FRONTIER_CODE),
            _=>return None,
        };
        let end = base.checked_add(0x9c8)?;
        let start = self.regs[next];
        if budget < 11 || self.minecraft_worldgen.disabled[index] || !self.is_thumb()
            || self.in_it_block() || self.excl_addr.is_some() || base < self.null_segment_size
            || !(1..624).contains(&start) || self.regs[previous] != start - 1
            || self.data_r_u32(mem, base + 0x9c4)? != start
            || mem.get_bytes_fallible(Ptr::from_bits(pc), code.len() as u32)? != code
            || (base < pc + code.len() as u32 && end > pc)
            || ENTRIES.iter().zip(CODE).any(|(&a, code)| base < a + code.len() as u32 && end > a) {
            return None;
        }
        let iterations = (budget / 11).min(u64::from(624 - start));
        let finish = start + iterations as u32;
        let factor = self.regs[multiplier];
        fn seed(data: &mut [u8], start: u32, finish: u32, factor: u32) {
            let at = start as usize * 4;
            let mut value = u32::from_le_bytes(data[at..at+4].try_into().unwrap());
            for i in start..finish {
                value = (value ^ (value >> 30)).wrapping_mul(factor).wrapping_add(i);
                let at = (i as usize + 1) * 4;
                data[at..at+4].copy_from_slice(&value.to_le_bytes());
            }
            data[0x9c4..0x9c8].copy_from_slice(&finish.to_le_bytes());
        }
        let mut regs = self.regs;
        regs[previous] = finish - 1; regs[next] = finish;
        regs[PC] = if finish < 624 { pc } else { pc + code.len() as u32 };
        let (result, carry, overflow) = Self::add_with_carry(finish, !624, true);
        let cpsr = (self.cpsr & 0x0fff_ffff) | (result & 0x80000000)
            | (u32::from(result == 0) << 30) | (u32::from(carry) << 29) | (u32::from(overflow) << 28);
        self.minecraft_worldgen.calls[index] += 1;
        let s = &self.minecraft_worldgen;
        if s.checked[index] < 128 || s.calls[index] % 4096 == 0 || s.shadow_only {
            let mut expected = mem.get_bytes_fallible(Ptr::from_bits(base),0x9c8)?.to_vec();
            seed(&mut expected, start, finish, factor);
            let before_fp = self.extregs; let before_fpscr = self.fpscr;
            let mut state = CpuState::Normal; let mut used = 0;
            for _ in 0..iterations*11 {
                state = self.step_one(mem); used += 1;
                if !matches!(state,CpuState::Normal) { break; }
            }
            if matches!(state,CpuState::Normal) && self.regs == regs && self.cpsr == cpsr
                && self.extregs == before_fp && self.fpscr == before_fpscr
                && mem.get_bytes_fallible(Ptr::from_bits(base),0x9c8)? == expected {
                self.minecraft_worldgen.checked[index] += 1;
                if self.minecraft_worldgen.checked[index] == 128 { report("shadow_gate_passed",index,128); }
            } else {
                self.minecraft_worldgen.disabled[index] = true;
                report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);
            }
            Some((state,used))
        } else {
            seed(mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x9c8)?,start,finish,factor);
            self.regs=regs; self.cpsr=cpsr;
            let previous=self.minecraft_worldgen.fast[index];
            self.minecraft_worldgen.fast[index]+=iterations;
            let count=self.minecraft_worldgen.fast[index];
            if previous==0 || previous/65536!=count/65536 { report("native_blocks",index,count); }
            Some((CpuState::Normal,iterations*11))
        }
    }
    // Profiled loading-time rectangle scan: an empty first cell rejects every
    // candidate width. Translate only that read-only path, preserving the
    // original overflow/area tests and exact 5/15-instruction branch costs.
    fn minecraft_empty_rect(&mut self, mem: &mut Mem, budget: u64) -> Option<(CpuState,u64)> {
        let index=4;
        if budget<15 || self.minecraft_worldgen.disabled[index] || !self.is_thumb()
            || self.in_it_block() || self.excl_addr.is_some()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[index]),CODE[index].len() as u32)? != CODE[index]
            || self.data_r_u8(mem,self.regs[3])? != 0 {return None;}
        let area=self.data_r_u32(mem,self.regs[13].wrapping_add(0x44))?;
        let mut r=self.regs;let mut cpsr=self.cpsr;let mut used=0;
        fn flags(old:u32,a:u32,b:u32)->u32 {
            let (v,c,o)=InterpreterCpu::add_with_carry(a,!b,true);
            (old&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28)
        }
        while budget-used>=15 && used<61440 {
            r[0]=area;r[8]=r[4].wrapping_sub(r[10]);r[2]=r[8].wrapping_mul(r[6]);
            if (r[2] as i32)<(area as i32) {
                cpsr=flags(cpsr,r[2],area);r[PC]=0x25b70;used+=5;break;
            }
            r[12]=r[3];r[0]=r[11];r[9]=r[3];r[5]=r[10];r[14]=0;
            r[4]=r[4].wrapping_sub(1);cpsr=flags(cpsr,r[4],r[10]);used+=15;
            r[PC]=if (r[4] as i32)>(r[10] as i32) {ENTRIES[index]} else {0x25b70};
            if r[PC]!=ENTRIES[index] {break;}
        }
        self.minecraft_worldgen.calls[index]+=1;
        let s=&self.minecraft_worldgen;
        if s.checked[index]<128 || s.calls[index]%4096==0 || s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;
            let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used {state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal) && self.regs==r && self.cpsr==cpsr && self.extregs==fp && self.fpscr==fpscr {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128 {report("shadow_gate_passed",index,128);}
            } else {self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        } else {
            self.regs=r;self.cpsr=cpsr;
            let previous=self.minecraft_worldgen.fast[index];self.minecraft_worldgen.fast[index]+=used;
            if previous==0 {report("native_scan_enabled",index,used);}
            Some((CpuState::Normal,used))
        }
    }
    // libc++ vector growth's original per-byte initialization loops. Only
    // non-aliasing contiguous ranges are fused, within the existing quantum.
    fn minecraft_fill(&mut self, mem:&mut Mem, budget:u64) -> Option<(CpuState,u64)> {
        let (block,index,pointer,cost)=match self.regs[PC] {
            0x1a3828=>(6,9,1,9),0x1a3858=>(7,10,6,7),_=>return None,
        };
        if budget<cost || self.minecraft_worldgen.disabled[index] || !self.is_thumb()
            || self.in_it_block() || self.excl_addr.is_some() || self.regs[5]==0
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[block]),CODE[block].len() as u32)?!=CODE[block] {return None;}
        // Bound original-path verification work per visit. Validating128 huge
        // vector fills in full can itself stall first-time terrain streaming.
        // Still compare128 complete chunks and every4096th visit, with exact
        // retired instructions; validated chunks keep the original large batch.
        let validating=self.minecraft_worldgen.checked[index]<128
            || (self.minecraft_worldgen.calls[index]+1)%4096==0 || self.minecraft_worldgen.shadow_only;
        let limit=if validating && self.minecraft_worldgen.streaming_enabled {64} else {16384};
        let count=(budget/cost).min(u64::from(self.regs[5])).min(limit) as u32;
        let start=self.regs[pointer];let end=start.checked_add(count)?;
        if start<self.null_segment_size
            || ENTRIES.iter().zip(CODE).any(|(&a,c)| start<a+c.len() as u32 && end>a) {return None;}
        mem.get_bytes_fallible(Ptr::from_bits(start),count)?;
        let header=if block==6 {
            let a=self.regs[4].checked_add(4)?;let e=a.checked_add(4)?;
            if (a<end && e>start) || self.data_r_u32(mem,a)?!=start
                || ENTRIES.iter().zip(CODE).any(|(&b,c)| a<b+c.len() as u32 && e>b) {return None;}
            Some(a)
        } else {None};
        let mut r=self.regs;r[pointer]=end;r[5]-=count;
        r[PC]=if r[5]==0 {ENTRIES[block]+CODE[block].len() as u32} else {ENTRIES[block]};
        let (v,c,o)=Self::add_with_carry(r[5].wrapping_add(1),!1,true);
        let cpsr=(self.cpsr&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);
        self.minecraft_worldgen.calls[index]+=1;let state=&self.minecraft_worldgen;
        let retired=u64::from(count)*cost;
        if state.checked[index]<128 || state.calls[index]%4096==0 || state.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;
            let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..retired {state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal) && self.regs==r && self.cpsr==cpsr && self.extregs==fp && self.fpscr==fpscr
                && mem.get_bytes_fallible(Ptr::from_bits(start),count)?.iter().all(|&v|v==r[0] as u8)
                && header.is_none_or(|a|self.data_r_u32(mem,a)==Some(end)) {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128 {report("shadow_gate_passed",index,128);}
            } else {self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        } else {
            mem.get_bytes_fallible_mut(Ptr::from_bits(start),count)?.fill(r[0] as u8);
            if let Some(a)=header {assert!(self.data_w_u32(mem,a,end));}
            self.regs=r;self.cpsr=cpsr;self.minecraft_worldgen.fast[index]+=u64::from(count);
            if self.minecraft_worldgen.fast[index]==u64::from(count) {report("native_fill_enabled",index,u64::from(count));}
            Some((CpuState::Normal,retired))
        }
    }
    // MT19937 non-reseed call, including the original pushed frame, scratch
    // registers and flags. The once-per-624 twist path remains separate.
    fn minecraft_random(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let index=11;let base=self.regs[0];let sp=self.regs[13];let stack=sp.checked_sub(20)?;
        if budget<20 || self.minecraft_worldgen.disabled[index] || !self.is_thumb()
            || self.in_it_block() || self.excl_addr.is_some() || stack<self.null_segment_size
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[8]),CODE[8].len() as u32)?!=CODE[8] {return None;}
        let cursor=base.checked_add(0x9c4)?;let next=self.data_r_u32(mem,cursor)?;
        if next>=624 {return None;}
        let word=base.checked_add(4+next*4)?;
        let disjoint=|a:u32,n:u32|a.checked_add(n).is_some_and(|e|e<=stack || a>=sp);
        if !disjoint(word,4)||!disjoint(cursor,4)||!disjoint(ENTRIES[8],CODE[8].len() as u32)
            || ENTRIES.iter().zip(CODE).any(|(&a,c)|cursor<a+c.len() as u32 && cursor+4>a) {return None;}
        let mut value=self.data_r_u32(mem,word)?;
        mem.get_bytes_fallible(Ptr::from_bits(stack),20)?;
        value^=value>>11;let mask=(value<<7)&0x9d2c5680;value^=mask;
        let mask=(value<<15)&0xefc60000;let unshifted=value^mask;value=unshifted^(unshifted>>18);
        let mut r=self.regs;r[0]=value;r[1]=mask;r[2]=next+1;r[PC]=r[14]&!1;
        // Last flag writer is EORS before the final non-S tempering shift.
        let cpsr=(self.cpsr&0x0fffffdf)|(unshifted&0x80000000)|(u32::from(unshifted==0)<<30)|((r[14]&1)<<5);
        let mut frame=[0u8;20];for (i,v) in [r[4],r[5],r[6],r[7],r[14]].iter().enumerate(){frame[i*4..i*4+4].copy_from_slice(&v.to_le_bytes());}
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128 || s.calls[index]%4096==0 || s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..20 {state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal) && self.regs==r && self.cpsr==cpsr && self.extregs==fp && self.fpscr==fpscr
                && self.data_r_u32(mem,cursor)==Some(next+1) && mem.get_bytes_fallible(Ptr::from_bits(stack),20)?==frame {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128 {report("shadow_gate_passed",index,128);}
            } else {self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        } else {
            assert!(self.data_w_u32(mem,cursor,next+1));mem.get_bytes_fallible_mut(Ptr::from_bits(stack),20)?.copy_from_slice(&frame);
            self.regs=r;self.cpsr=cpsr;self.minecraft_worldgen.fast[index]+=1;
            if self.minecraft_worldgen.fast[index]==1 {report("native_random_enabled",index,1);}
            Some((CpuState::Normal,20))
        }
    }
    // Cache misses return through the same nibble calculation. Accelerate
    // this read-only suffix too, after the guest resolves/validates the chunk.
    fn minecraft_light_tail(&mut self, mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let (offset,instructions,layer)=match self.regs[8] {0x28bd1c=>(0xc04c,25,0),0x28bd24=>(0x1004c,29,1),_=>return None};
        if budget<instructions || !self.is_thumb() || self.in_it_block() || self.excl_addr.is_some()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[5]),CODE[5].len() as u32)?!=CODE[5] {return None;}
        let pos=self.regs[5];let x=self.data_r_u32(mem,pos)?;
        let z=self.data_r_u32(mem,pos.checked_add(8)?)?;let y=self.data_r_u32(mem,pos.checked_add(4)?)?;
        let index=12+layer*2+(y&1) as usize;
        if self.minecraft_worldgen.disabled[index] {return None;}
        let partial=(y&0xfe)|((x&15)<<11);
        let address=self.regs[0].checked_add((partial|((z&15)<<7))>>1)?.checked_add(offset)?;
        let byte=u32::from(self.data_r_u8(mem,address)?);let value=if y&1==0{byte&15}else{byte>>4};
        let sp=self.regs[13];let end=sp.checked_add(32)?;
        let mut r=self.regs;r[0]=value;r[1]=y;r[2]=offset;r[3]=partial;
        for (reg,at) in [(8,8),(4,12),(5,16),(6,20),(7,24)] {r[reg]=self.data_r_u32(mem,sp.checked_add(at)?)?;}
        let target=self.data_r_u32(mem,sp.checked_add(28)?)?;r[13]=end;r[PC]=target&!1;
        let (zero,carry)=if y&1==0{(true,true)}else{(value==0,byte&8!=0)};
        let cpsr=(self.cpsr&0x0fffffdf)|(u32::from(zero)<<30)|(u32::from(carry)<<29)|((target&1)<<5);
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128 || s.calls[index]%4096==0 || s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..instructions {state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal) && self.regs==r && self.cpsr==cpsr && self.extregs==fp && self.fpscr==fpscr {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        }else{
            self.regs=r;self.cpsr=cpsr;self.minecraft_worldgen.fast[index]+=1;
            if self.minecraft_worldgen.fast[index]==1{report("native_light_tail_enabled",index,1);}
            Some((CpuState::Normal,instructions))
        }
    }
    // Translate the cache-miss prologue up to (and including) its BLX.
    // Chunk resolution still runs the original virtual method, with exactly
    // the same arguments, stack, scratch registers, flags and return address.
    fn minecraft_light_prefix(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        if budget<24 || !self.is_thumb() || self.in_it_block() || self.excl_addr.is_some()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[5]),CODE[5].len() as u32)?!=CODE[5] {return None;}
        let sp=self.regs[13];let stack=sp.checked_sub(32)?;
        if stack<self.null_segment_size{return None;}
        let disjoint=|a:u32,n:u32|a.checked_add(n).is_some_and(|e|e<=stack||a>=sp);
        let source=self.regs[0];let pos=self.regs[2];
        if !disjoint(source,44)||!disjoint(pos,12)||!disjoint(ENTRIES[5],CODE[5].len() as u32){return None;}
        mem.get_bytes_fallible(Ptr::from_bits(stack),32)?;
        let y=self.data_r_u32(mem,pos.checked_add(4)?)?;if y>=128{return None;}
        let x=self.data_r_u32(mem,pos)?;let z=self.data_r_u32(mem,pos.checked_add(8)?)?;
        let cx=((x as i32)>>4) as u32;let cz=((z as i32)>>4) as u32;
        let chunk=self.data_r_u32(mem,source.checked_add(40)?)?;
        let mut r=self.regs;let mut comparison=(y,128);
        let instructions=if chunk==0{24}else{
            let coords=chunk.checked_add(28)?;if !disjoint(coords,8){return None;}
            r[3]=self.data_r_u32(mem,coords)?;comparison=(r[3],cx);
            if r[3]==cx {
                let chunk_z=self.data_r_u32(mem,coords.checked_add(4)?)?;
                if chunk_z==cz{return None;}comparison=(chunk_z,cz);
            }
            30
        };
        let index=if chunk==0{16}else{17};
        if budget<instructions||self.minecraft_worldgen.disabled[index]{return None;}
        let object=self.data_r_u32(mem,source.checked_add(16)?)?;
        if !disjoint(object,4){return None;}
        let vtable=self.data_r_u32(mem,object)?;let method=vtable.checked_add(8)?;
        if !disjoint(method,4){return None;}
        let target=self.data_r_u32(mem,method)?;let flag=self.data_r_u8(mem,source.checked_add(8)?)?;
        let mut frame=[0u8;32];
        for (i,v) in [cx,cz,r[8],r[4],r[5],r[6],r[7],r[14]].iter().enumerate(){frame[i*4..i*4+4].copy_from_slice(&v.to_le_bytes());}
        r[0]=object;r[1]=stack;r[2]=target;r[4]=u32::from(flag);r[5]=pos;r[6]=source;r[7]=sp-8;r[8]=self.regs[1];r[13]=stack;r[14]=0xbf2e1;r[PC]=target&!1;
        let(v,c,o)=Self::add_with_carry(comparison.0,!comparison.1,true);
        let nzcv=if chunk==0{(cz&0x80000000)|(u32::from(cz==0)<<30)|((z&8)<<26)}else{(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28)};
        let cpsr=(self.cpsr&0x0fffffdf)|nzcv|((target&1)<<5);
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..instructions{state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==cpsr&&self.extregs==fp&&self.fpscr==fpscr
                &&mem.get_bytes_fallible(Ptr::from_bits(stack),32)?==frame {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        }else{
            mem.get_bytes_fallible_mut(Ptr::from_bits(stack),32)?.copy_from_slice(&frame);self.regs=r;self.cpsr=cpsr;
            self.minecraft_worldgen.fast[index]+=1;if self.minecraft_worldgen.fast[index]==1{report("native_light_prefix_enabled",index,1);}
            Some((CpuState::Normal,instructions))
        }
    }
    fn minecraft_noise(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        use super::minecraft_noise as noise;
        let index=18;if budget<noise::MAX_INSTRUCTIONS||self.minecraft_worldgen.disabled[index]{return None;}
        let p=noise::predict(self,mem)?;self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..p.used{state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==p.regs&&self.cpsr==p.cpsr&&self.extregs==p.fp&&self.fpscr==fpscr{
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        }else{
            self.regs=p.regs;self.cpsr=p.cpsr;self.extregs=p.fp;self.minecraft_worldgen.fast[index]+=1;
            if self.minecraft_worldgen.fast[index]==1{report("native_noise_enabled",index,1);}
            Some((CpuState::Normal,p.used))
        }
    }
    // Broader profiled islands around the original noise, ore and lighting
    // operations. Predictions record writes transactionally, including aliases.
    fn minecraft_loading(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        use super::minecraft_loading as loading;
        use crate::pipeline_native::{Machine,Predict,Outcome};
        let which=self.loading_island(mem,budget)?;
        let (lo,hi,_,_)=loading::BLOCKS[which];let index=36+which;
        let context=crate::cpu::CpuContext{regs:self.regs,extregs:self.extregs,cpsr:self.cpsr,fpscr:self.fpscr};
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let mut m=Machine{c:context,a:Predict::new(mem),retired:0};
            loading::execute(which,&mut m)?;
            if m.a.writes[..m.a.nw].iter().any(|w|u64::from(w.a)<u64::from(hi)&&u64::from(lo)<u64::from(w.a)+u64::from(w.n)){return None;}
            let p=Outcome{context:m.c,retired:m.retired,writes:m.a.writes,nw:m.a.nw,touches:m.a.touches,nt:m.a.nt};
            let used=u64::from(p.retired);if used>budget{return None;}
            let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==p.context.regs&&self.extregs==p.context.extregs
                &&self.cpsr==p.context.cpsr&&self.fpscr==p.context.fpscr&&p.compare_memory(mem){
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            let mut used=self.loading_fast(mem,which,budget)?;
            #[cfg(test)] if self.minecraft_worldgen.single_island {return Some((CpuState::Normal,used));}
            // Continue into the next island exactly as the dispatcher would at
            // that PC, but only while it is also due a fast visit; a shadow
            // visit or a failed guard goes back through the dispatcher.
            while let Some(which)=self.loading_island(mem,budget-used) {
                let index=36+which;let s=&self.minecraft_worldgen;
                if s.checked[index]<128||(s.calls[index]+1)%4096==0||s.shadow_only{break;}
                self.minecraft_worldgen.calls[index]+=1;
                let Some(more)=self.loading_fast(mem,which,budget-used) else {self.minecraft_worldgen.calls[index]-=1;break};
                used+=more;
            }
            Some((CpuState::Normal,used))
        }
    }
    fn loading_island(&self,mem:&Mem,budget:u64)->Option<usize> {
        use super::minecraft_loading as loading;
        let s=&self.minecraft_worldgen;
        if !s.loading_enabled{return None;}
        let which=loading::entry_index(self.regs[PC])?;
        if which >= 90 && !s.mesh_native {return None;}
        if which>=78 && !s.neighbors_enabled {return None;}
        if which>=62 && !s.hot_loops_enabled {return None;}
        if which>=48&&!s.frontier_enabled{return None;}
        if which>=9&&!s.terrain_enabled{return None;}
        if which>=29&&!s.streaming_enabled{return None;}
        let (lo,_,max,code)=loading::BLOCKS[which];
        if budget<max||s.disabled[36+which]||!self.is_thumb()
            ||self.in_it_block()||self.excl_addr.is_some()
            ||mem.get_bytes_fallible(Ptr::from_bits(lo),code.len() as u32)?!=code{return None;}
        Some(which)
    }
    // Keep repeated loop iterations in one native context, but stop before
    // the next scheduled shadow check and at the original guest budget.
    fn loading_fast(&mut self,mem:&mut Mem,which:usize,budget:u64)->Option<u64> {
        let index=36+which;
        let context=crate::cpu::CpuContext{regs:self.regs,extregs:self.extregs,cpsr:self.cpsr,fpscr:self.fpscr};
        let max_visits=(4096-self.minecraft_worldgen.calls[index]%4096).min(64);
        let (c,used,visits)=super::minecraft_loading::fast_batch(mem,context,which,budget.min(16384),max_visits);
        self.minecraft_worldgen.calls[index]+=visits.saturating_sub(1);
        self.regs=c.regs;self.extregs=c.extregs;self.cpsr=c.cpsr;self.fpscr=c.fpscr;
        if used==0{return None;}
        self.minecraft_worldgen.fast[index]+=used;if self.minecraft_worldgen.fast[index]==used{report("native_loading_enabled",index,used);}
        Some(used)
    }
    // Ore feature's inner distance/rejection loop. Preserve the guest's f32
    // rounding sequence and FPSCR comparison flags; accepted cells return to
    // the original block lookup/update code. No chunk contents are changed here.
    fn minecraft_ore(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let index=19;
        if budget<10||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()
            ||mem.get_bytes_fallible(Ptr::from_bits(0x1947b6),ORE_CODE.len() as u32)?!=ORE_CODE{return None;}
        let mut r=self.regs;let mut f=self.extregs;let mut cpsr=self.cpsr;let mut fpscr=self.fpscr;let mut used=0;
        let [offset,center,radius,partial,threshold]=[19,31,23,30,29].map(|i|f32::from_bits(f[i]));
        if ![offset,center,radius,partial,threshold].iter().all(|v|v.is_finite())||radius==0.0{return None;}
        while budget-used>=10&&used<61440 {
            let sum=(r[6] as i32 as f32)+offset;let delta=sum-center;let coordinate=delta/radius;
            let product=coordinate*coordinate;let distance=partial+product;
            if ![sum,delta,coordinate,product,distance].iter().all(|v|v.is_finite()){return None;}
            f[0]=coordinate.to_bits();f[4]=distance.to_bits();
            let flags=if distance<threshold{0x80000000}else if distance==threshold{0x60000000}else{0x20000000};
            fpscr=(fpscr&0x0fffffff)|flags;cpsr=(cpsr&0x0fffffff)|flags;used+=10;
            r[PC]=if distance>=threshold{0x19488c}else{0x1947de};
            if distance<threshold||budget-used<5{break;}
            r[10]=r[10].wrapping_add(128);r[0]=r[6].wrapping_add(1);
            let(v,c,o)=Self::add_with_carry(r[6],!r[11],true);
            cpsr=(cpsr&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);used+=4;
            if (r[6] as i32)<(r[11] as i32){r[6]=r[0];r[PC]=0x1947b8;used+=1;}else{r[PC]=0x194896;break;}
        }
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==cpsr&&self.extregs==f&&self.fpscr==fpscr {
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            self.regs=r;self.extregs=f;self.cpsr=cpsr;self.fpscr=fpscr;
            self.minecraft_worldgen.fast[index]+=used;if self.minecraft_worldgen.fast[index]==used{report("native_ore_enabled",index,used);}
            Some((CpuState::Normal,used))
        }
    }
    // The observed virtual resolver's in-bounds chunk-grid lookup. Out-of-
    // bounds/custom implementations keep their original guest paths.
    fn minecraft_chunk_grid(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let index=20;let object=self.regs[0];let pos=self.regs[1];let sp=self.regs[13];let stack=sp.checked_sub(8)?;
        if budget<31||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()||stack<self.null_segment_size
            ||mem.get_bytes_fallible(Ptr::from_bits(0x12d924),RESOLVER_CODE.len() as u32)?!=RESOLVER_CODE{return None;}
        let disjoint=|a:u32,n:u32|a.checked_add(n).is_some_and(|e|e<=stack||a>=sp);
        let bounds=object.checked_add(0x78)?;
        if !disjoint(pos,8)||!disjoint(bounds,0x34)||!disjoint(0x12d924,RESOLVER_CODE.len() as u32){return None;}
        mem.get_bytes_fallible(Ptr::from_bits(stack),8)?;
        let get=|at:u32|self.data_r_u32(mem,object.checked_add(at)?);
        let x=self.data_r_u32(mem,pos)?;let z=self.data_r_u32(mem,pos.checked_add(4)?)?;
        let min_x=get(0x78)?;let max_x=get(0x84)?;let min_y=get(0x7c)?;let max_y=get(0x88)?;let min_z=get(0x80)?;let max_z=get(0x8c)?;
        if (x as i32)<(min_x as i32)||(x as i32)>(max_x as i32)||(min_y as i32)>0||(max_y as i32)<0||(z as i32)<(min_z as i32)||(z as i32)>(max_z as i32){return None;}
        let width=get(0x90)?;let stride_y=get(0x9c)?;let table=get(0xa8)?;let dx=x.wrapping_sub(min_x);
        let at=width.wrapping_mul(z.wrapping_sub(min_z)).wrapping_add(dx).wrapping_sub(stride_y.wrapping_mul(min_y));
        let address=table.wrapping_add(at.wrapping_mul(4));if !disjoint(address,4){return None;}
        let chunk=self.data_r_u32(mem,address)?;let target=self.regs[14];
        let mut r=self.regs;r[0]=chunk;r[1]=at;r[2]=stride_y;r[3]=width;r[9]=min_y;r[12]=min_z;r[14]=dx;r[PC]=target&!1;
        let(v,c,o)=Self::add_with_carry(z,!max_z,true);
        let cpsr=(self.cpsr&0x0fffffdf)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28)|((target&1)<<5);
        let mut frame=[0u8;8];frame[..4].copy_from_slice(&r[7].to_le_bytes());frame[4..].copy_from_slice(&target.to_le_bytes());
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut used=0;
            for _ in 0..31{state=self.step_one(mem);used+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==cpsr&&self.extregs==fp&&self.fpscr==fpscr
                &&mem.get_bytes_fallible(Ptr::from_bits(stack),8)?==frame {
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,used))
        }else{
            mem.get_bytes_fallible_mut(Ptr::from_bits(stack),8)?.copy_from_slice(&frame);self.regs=r;self.cpsr=cpsr;
            self.minecraft_worldgen.fast[index]+=1;if self.minecraft_worldgen.fast[index]==1{report("native_chunk_grid_enabled",index,1);}
            Some((CpuState::Normal,31))
        }
    }
    // Lighting's inlined cached block-id/height-map reads. These callers
    // were still interpreting the same cache checks after the light getter
    // itself became native. Cache misses retain original guest resolution.
    fn minecraft_cached_cell(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let height=self.regs[PC]==0x48214;let index=if height{22}else{21};
        if budget<32||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()
            ||mem.get_bytes_fallible(Ptr::from_bits(0x48214),CELL_CODE.len() as u32)?!=CELL_CODE{return None;}
        let sp=self.regs[13];let pos=sp.checked_add(0xa8)?;
        let x=self.data_r_u32(mem,pos)?;let original_y=self.data_r_u32(mem,pos.checked_add(4)?)?;let z=self.data_r_u32(mem,pos.checked_add(8)?)?;
        if !height&&original_y>=128{return None;}
        let y=original_y&255;let cx=((x as i32)>>4) as u32;let cz=((z as i32)>>4) as u32;
        let owner=self.regs[8].checked_add(28)?;let source=self.data_r_u32(mem,owner)?;let cache=source.checked_add(40)?;
        let chunk=self.data_r_u32(mem,cache)?;if chunk==0{return None;}
        let coords=chunk.checked_add(28)?;
        if self.data_r_u32(mem,coords)?!=cx||self.data_r_u32(mem,coords.checked_add(4)?)?!=cz{return None;}
        let writes=if height{[(sp.checked_add(16)?,self.regs[3]),(sp.checked_add(0xbc)?,cx),(sp.checked_add(0xc0)?,cz)]}
            else{[(sp.checked_add(0xb4)?,cx),(sp.checked_add(0xb8)?,cz),(0,0)]};
        let writes=&writes[..if height{3}else{2}];
        let packed=if height{(x&15)|((z&15)<<4)}else{((x&15)<<11)|((z&15)<<7)|y};
        let address=chunk.checked_add(packed)?.checked_add(if height{0x1444c}else{0x4c})?;
        for &(a,_) in writes {
            let end=a.checked_add(4)?;self.data_r_u32(mem,a)?;
            for (read,len) in [(pos,12),(owner,4),(cache,4),(coords,8),(address,1),(0x48214,CELL_CODE.len() as u32)] {
                if a<read.checked_add(len)?&&end>read{return None;}
            }
        }
        let value=u32::from(self.data_r_u8(mem,address)?);let mut r=self.regs;r[5]=source;
        let (cpsr,used)=if height {
            r[0]=value;r[1]=y;r[2]=z;r[3]=cx;
            let(v,c,o)=Self::add_with_carry(y,!value,true);
            let flags=(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);
            if y>=value{r[PC]=0x48374;((self.cpsr&0x0fffffff)|flags,29)}
            else{r[0]=0;r[3]=self.regs[3];r[PC]=0x4839a;((self.cpsr&0x0fffffff)|0x40000000,32)}
        }else{
            r[4]=self.regs[0];r[0]=value;r[1]=packed;r[2]=((x&15)<<11)|y;r[3]=y;r[PC]=0x48348;
            ((self.cpsr&0x0fffffff)|0x20000000|(u32::from(packed==0)<<30),32)
        };
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==cpsr&&self.extregs==fp&&self.fpscr==fpscr
                &&writes.iter().all(|&(a,v)|self.data_r_u32(mem,a)==Some(v)){
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            for &(a,v)in writes{assert!(self.data_w_u32(mem,a,v));}self.regs=r;self.cpsr=cpsr;
            self.minecraft_worldgen.fast[index]+=1;if self.minecraft_worldgen.fast[index]==1{report("native_cached_cell_enabled",index,1);}
            Some((CpuState::Normal,used))
        }
    }
    // The complementary nonempty scan in atlas packing. Searching contiguous
    // bytes natively replaces seven interpreted instructions per nonzero byte.
    fn minecraft_byte_scan(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let index=23;let start=self.regs[9];let column=self.regs[5];let end_column=self.regs[4];
        if budget<7||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()
            ||column>=end_column||end_column>0x7fffffff||start<self.null_segment_size
            ||mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[4]),CODE[4].len() as u32)?!=CODE[4]{return None;}
        let count=(budget/7).min(u64::from(end_column-column)).min(16384) as u32;
        let data=mem.get_bytes_fallible(Ptr::from_bits(start),count)?;
        let first_zero=data.iter().position(|&v|v==0);let nonzero=first_zero.unwrap_or(data.len()) as u32;
        let mut r=self.regs;r[5]+=nonzero;r[9]=start.checked_add(nonzero)?;
        let(cpsr,used)=if first_zero.is_some(){r[14]=0;r[PC]=0x25b6a;((self.cpsr&0x0fffffff)|0x60000000,u64::from(nonzero)*7+3)}
        else{r[14]=u32::from(*data.last()?);r[PC]=if r[5]<r[4]{0x25b32}else{0x25b46};let(v,c,o)=Self::add_with_carry(r[5],!r[4],true);
            ((self.cpsr&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28),u64::from(nonzero)*7)};
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==cpsr&&self.extregs==fp&&self.fpscr==fpscr {
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            self.regs=r;self.cpsr=cpsr;self.minecraft_worldgen.fast[index]+=used;
            if self.minecraft_worldgen.fast[index]==used{report("native_byte_scan_enabled",index,used);}
            Some((CpuState::Normal,used))
        }
    }
    // Initial chunk height and skylight columns. A whole run of air cells is
    // translated together; opacity, nibble packing and scheduling are unchanged.
    fn minecraft_column(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let sky=self.regs[PC]==0x47b44;let index=if sky{25}else{24};let quota=if sky{23}else{10};
        if budget<quota||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()
            ||mem.get_bytes_fallible(Ptr::from_bits(0x47b08),COLUMN_CODE.len() as u32)?!=COLUMN_CODE{return None;}
        let mut r=self.regs;let mut flags=self.cpsr;let mut used=0;let mut writes=[(0u32,0u8);128];let mut nw=0;
        let chunk=r[0];let table=r[2].checked_add(0x400)?;
        if chunk<self.null_segment_size||table<self.null_segment_size{return None;}
        if sky {
            if r[9]==0||r[9]>127||r[5]&!0x7f80!=0||r[4]!=r[5]||r[14]!=0xc04c||r[10]!=0xff0||r[1]>15{return None;}
            let lo=chunk.checked_add(0xc04c)?;let hi=lo.checked_add(0x4000)?;
            if lo<table.checked_add(1024)?&&hi>table || lo<0x47b98&&hi>0x47b08{return None;}
            mem.get_bytes_fallible(Ptr::from_bits(lo),0x4000)?;
            while budget-used>=23 {
                let block=self.data_r_u8(mem,chunk.checked_add(r[4])?.checked_add(r[9])?.checked_add(0x4c)?)?;
                let opacity=self.data_r_u32(mem,table.checked_add(u32::from(block)*4)?)?;
                if opacity>255{return None;}
                r[3]=opacity;r[1]=r[1].wrapping_sub(opacity);
                let(v,c,o)=Self::add_with_carry(r[1],!1,true);
                flags=(flags&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);used+=8;
                if (r[1] as i32)<1 {r[PC]=0x47b98;break;}
                let y=r[9];let a=lo.checked_add((r[5]|y)>>1)?;
                let mut prior=self.data_r_u8(mem,a)?;
                if nw>0&&writes[nw-1].0==a{prior=writes[nw-1].1;}
                r[6]=if y&1==0{r[1]&15}else{r[10]&(r[1]<<4)};
                r[3]=u32::from(prior)&if y&1==0{0xf0}else{15};r[3]|=r[6];r[12]=a;
                r[9]-=1;writes[nw]=(a,r[3] as u8);nw+=1;
                let v=r[9]&255;flags=(flags&0x3fffffff)|(v&0x80000000)|(u32::from(v==0)<<30);
                used+=if y&1==0{15}else{14};r[PC]=if v!=0{0x47b44}else{0x47b98};if v==0{break;}
            }
        }else{
            if r[1]==0||r[1]>127{return None;}
            while budget-used>=10 {
                let a=chunk.checked_add(r[3].wrapping_add(r[1]&255))?.checked_add(0x4c)?;
                let block=self.data_r_u8(mem,a)?;r[6]=self.data_r_u32(mem,table.checked_add(u32::from(block)*4)?)?;used+=6;
                if r[6]!=0{r[PC]=0x47b26;break;}
                r[1]-=1;let v=r[1]&255;flags=(flags&0x0fffffff)|0x20000000|(u32::from(v==0)<<30);used+=3;
                r[PC]=if v==0{used+=1;0x47b26}else{0x47b08};if v==0{break;}
            }
        }
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            let memory_ok=(0..nw).all(|i|writes[i+1..nw].iter().any(|&(a,_)|a==writes[i].0)||self.data_r_u8(mem,writes[i].0)==Some(writes[i].1));
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==flags&&self.extregs==fp&&self.fpscr==fpscr&&memory_ok {
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            for &(a,v)in &writes[..nw]{assert!(self.data_w_u8(mem,a,v));}self.regs=r;self.cpsr=flags;
            self.minecraft_worldgen.fast[index]+=used;if self.minecraft_worldgen.fast[index]==used{report("native_column_enabled",index,used);}
            Some((CpuState::Normal,used))
        }
    }
    // Prepare the six neighboring positions for the original light lookup.
    // The lookup itself still runs through its independently checked paths.
    fn minecraft_neighbors(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        let (which,used,next)=match self.regs[PC]{0x483aa=>(0,10,0x483c6),0x483c6=>(1,10,0x483e2),0x483e2=>(2,11,0x483fe),0x483fe=>(3,12,0x48418),0x48418=>(4,11,0x48430),0x48430=>(5,23,0x48462),_=>return None};
        let index=26+which;
        if budget<used||self.minecraft_worldgen.disabled[index]||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()
            ||mem.get_bytes_fallible(Ptr::from_bits(0x483aa),NEIGHBOR_CODE.len() as u32)?!=NEIGHBOR_CODE{return None;}
        let mut r=self.regs;let mut flags=self.cpsr;let sp=r[13];let pos=sp.checked_add(0xa8)?;
        let x=self.data_r_u32(mem,pos)?;let y=self.data_r_u32(mem,pos.checked_add(4)?)?;let z=self.data_r_u32(mem,pos.checked_add(8)?)?;
        let owner=if which<4{r[8]}else{r[6]};let source_addr=owner.checked_add(28)?;
        let light=self.data_r_u32(mem,owner)?;let source=self.data_r_u32(mem,source_addr)?;
        let dst=sp.checked_add([0x9c,0x90,0x84,0x78,0x6c,0x60][which])?;
        let mut xyz=[x,y,z];let mut writes=[(0u32,0u32);4];let mut nw=3;
        let cmp=|a:u32,b:u32|{let(v,c,o)=Self::add_with_carry(a,!b,true);(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28)};
        match which {
            0=>{let a=sp.checked_add(12)?;for p in [owner,source_addr]{if a<p.checked_add(4)?&&a.checked_add(4)?>p{return None;}}
                writes[3]=(a,r[0]);nw=4;r[4]=dst;r[6]=z;r[3]=y;xyz[0]=x.wrapping_sub(1);flags=(flags&0x0fffffff)|cmp(x,1);}
            1=>{r[10]=r[0];r[4]=dst;r[6]=z;r[3]=y;xyz[0]=x.wrapping_add(1);let(v,c,o)=Self::add_with_carry(x,1,false);flags=(flags&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);}
            2=>{r[11]=r[0];r[6]=z;r[3]=y;xyz[1]=y.wrapping_add(1);let(v,c,o)=Self::add_with_carry(y,1,false);flags=(flags&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);}
            3=>{r[6]=r[8];r[8]=r[0];r[5]=z;r[3]=y;xyz[1]=y.wrapping_sub(1);flags=(flags&0x0fffffff)|cmp(y,1);}
            4=>{r[5]=r[0];r[4]=z;r[3]=y;xyz[2]=z.wrapping_add(1);let(v,c,o)=Self::add_with_carry(z,1,false);flags=(flags&0x0fffffff)|(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28);}
            5=>{r[4]=y;r[6]=z;xyz[2]=z.wrapping_sub(1);r[3]=xyz[2];r[10]=r[10].max(r[11]).max(r[8]).max(r[5]);flags=(flags&0x0fffffff)|cmp(r[10],r[0]);r[10]=r[10].max(r[0]);}
            _=>unreachable!()
        }
        for i in 0..3{writes[i]=(dst.checked_add(4*i as u32)?,xyz[i]);}
        for &(a,_)in &writes[..nw]{self.data_r_u32(mem,a)?;if a<0x48462&&a.checked_add(4)?>0x483aa{return None;}}
        r[0]=source;r[1]=light;r[2]=dst;r[14]=next|1;r[PC]=0xbf294;
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==flags&&self.extregs==fp&&self.fpscr==fpscr&&writes[..nw].iter().all(|&(a,v)|self.data_r_u32(mem,a)==Some(v)){
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            for &(a,v)in &writes[..nw]{assert!(self.data_w_u32(mem,a,v));}self.regs=r;self.cpsr=flags;
            self.minecraft_worldgen.fast[index]+=1;if self.minecraft_worldgen.fast[index]==1{report("native_neighbors_enabled",index,1);}
            Some((CpuState::Normal,used))
        }
    }
    // Surface-column classification after the original random draw. Keep the
    // bedrock branch and material-changing operations in the original engine.
    fn minecraft_surface(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        if budget<40||!self.is_thumb()||self.in_it_block()||self.excl_addr.is_some()||self.regs[6]!=0xcccccccd
            ||self.regs[8]==0||self.regs[8]>127{return None;}
        for(a,code)in[(0x4e55a,SURFACE_CODE),(0x4e5c2,SURFACE_RESET),(0x4e750,SURFACE_TAIL),(0x4e552,SURFACE_HEAD)] {
            if mem.get_bytes_fallible(Ptr::from_bits(a),code.len() as u32)?!=code{return None;}
        }
        let mut r=self.regs;let mut flags=self.cpsr;let product=u64::from(r[0])*u64::from(r[6]);r[2]=(product>>32) as u32;r[1]=(r[2]>>2)*5;
        r[0]=r[0].wrapping_sub(r[1]);if r[8]<=r[0]{return None;}
        let sp=r[13];let saved_column=sp.checked_add(28)?;let reset=sp.checked_add(24)?;let mut write=None;
        let cmp=|a:u32,b:u32|{let(v,c,o)=Self::add_with_carry(a,!b,true);(v&0x80000000)|(u32::from(v==0)<<30)|(u32::from(c)<<29)|(u32::from(o)<<28)};
        r[3]=r[4];r[4]=r[5]|r[8];r[5]=r[10];r[10]=r[5].checked_add(r[4])?.checked_add(0x4c)?;
        r[1]=u32::from(self.data_r_u8(mem,r[10])?);flags=(flags&0x0fffffff)|cmp(r[1],0);
        let (index,used)=if r[1]==0 {
            r[10]=r[5];r[5]=self.data_r_u32(mem,saved_column)?;r[0]=u32::MAX;r[4]=r[3];write=Some((reset,r[0]));(32,25)
        }else{
            r[0]=0x28e260;r[2]=self.data_r_u32(mem,r[0].checked_add(r[1]*4)?)?;r[0]=r[3];r[3]=r[3].checked_add(200)?;r[9]=self.data_r_u32(mem,r[3])?;
            if r[2]!=0{r[3]=r[2].checked_add(0x6c)?;}r[2]=self.data_r_u32(mem,r[3])?;flags=(flags&0x0fffffff)|cmp(r[2],r[9]);
            if r[2]==r[9]{
                r[10]=r[5];r[5]=self.data_r_u32(mem,saved_column)?;r[1]=u32::MAX;r[4]=r[0];write=Some((reset,r[1]));(33,37)
            }else{
                r[2]=self.data_r_u32(mem,r[0].checked_add(0x80)?)?;r[2]=u32::from(self.data_r_u8(mem,r[2].checked_add(0x44)?)?);flags=(flags&0x0fffffff)|cmp(r[1],r[2]);
                if r[1]!=r[2]{r[10]=r[5];r[5]=self.data_r_u32(mem,saved_column)?;r[4]=0x28b6b0;(34,40)}else{(35,29)}
            }
        };
        if self.minecraft_worldgen.disabled[index]{return None;}
        if let Some((a,_))=write{self.data_r_u32(mem,a)?;if a<0x4e76a&&a.checked_add(4)?>0x4e552{return None;}}
        if index!=35{flags&=0x3fffffff;r[8]-=1;r[0]=r[11];r[14]=0x4e55b;r[PC]=0x19ffb8;}else{r[PC]=0x4e5de;}
        self.minecraft_worldgen.calls[index]+=1;let s=&self.minecraft_worldgen;
        if s.checked[index]<128||s.calls[index]%4096==0||s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..used{state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal)&&self.regs==r&&self.cpsr==flags&&self.extregs==fp&&self.fpscr==fpscr&&write.is_none_or(|(a,v)|self.data_r_u32(mem,a)==Some(v)){
                self.minecraft_worldgen.checked[index]+=1;if self.minecraft_worldgen.checked[index]==128{report("shadow_gate_passed",index,128);}
            }else{self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        }else{
            if let Some((a,v))=write{assert!(self.data_w_u32(mem,a,v));}self.regs=r;self.cpsr=flags;
            self.minecraft_worldgen.fast[index]+=1;if self.minecraft_worldgen.fast[index]==1{report("native_surface_enabled",index,1);}
            Some((CpuState::Normal,used))
        }
    }
    fn light_fallback(&mut self, reason:usize) -> Option<(CpuState,u64)> {
        if self.pc_sampling.is_some() {
            let n=&mut self.minecraft_worldgen.light_fallback[reason];*n+=1;
            if n.is_power_of_two() {
                crate::diagnostics::event(&format!("{{\"event\":\"minecraft_light_fallback\",\"reason\":{reason},\"count\":{n},\"args\":{:?},\"sp\":{},\"exclusive\":{:?}}}", &self.regs[..3],self.regs[13],self.excl_addr.unwrap_or(0)));
            }
        }
        None
    }
    // Cache-hit light nibble lookup. Misses still execute the guest virtual
    // calls. Preserve scratch registers, flags, stack writes and retire counts,
    // including ARM/Thumb return state; validate each layer/parity independently.
    fn minecraft_light_lookup(&mut self, mem: &mut Mem, budget: u64) -> Option<(CpuState,u64)> {
        let (offset, instructions, layer) = match self.regs[1] {
            0x28bd1c => (0xc04cu32,51,0),
            0x28bd24 => (0x1004cu32,55,1),
            _ => return self.light_fallback(0),
        };
        if budget<instructions {return self.light_fallback(1);}
        if self.excl_addr.is_some() {return self.light_fallback(2);}
        if !self.is_thumb() || self.in_it_block()
            || mem.get_bytes_fallible(Ptr::from_bits(ENTRIES[5]),CODE[5].len() as u32)? != CODE[5] {return self.light_fallback(3);}
        let sp=self.regs[13];let stack=sp.checked_sub(32)?;
        if stack<self.null_segment_size {return None;}
        let disjoint=|a:u32,n:u32| a.checked_add(n).is_some_and(|end| end<=stack || a>=sp);
        // Original pushes occur before argument reads. Exclude aliases and code
        // overlap so pre-reading inputs and committing the frame stays exact.
        if !disjoint(ENTRIES[5],CODE[5].len() as u32) || !disjoint(self.regs[2],12) {return self.light_fallback(7);}
        mem.get_bytes_fallible(Ptr::from_bits(stack),32)?;
        let pos=self.regs[2];let y=self.data_r_u32(mem,pos.checked_add(4)?)?;
        if y>=128 {return self.light_fallback(4);}
        let index=5+layer*2+(y&1) as usize;
        if self.minecraft_worldgen.disabled[index] {return None;}
        let x=self.data_r_u32(mem,pos)?;let z=self.data_r_u32(mem,pos.checked_add(8)?)?;
        let cx=((x as i32)>>4) as u32;let cz=((z as i32)>>4) as u32;
        let cache=self.regs[0].checked_add(0x28)?;
        if !disjoint(cache,4) {return None;}
        let chunk=self.data_r_u32(mem,cache)?;
        if chunk<self.null_segment_size {return self.light_fallback(6);}
        let coords=chunk.checked_add(0x1c)?;
        if !disjoint(coords,8) || self.data_r_u32(mem,coords)?!=cx
            || self.data_r_u32(mem,coords.checked_add(4)?)?!=cz {
            if self.pc_sampling.is_some() && self.minecraft_worldgen.light_fallback[5]==0 {
                let object=self.data_r_u32(mem,self.regs[0].wrapping_add(16)).unwrap_or(0);
                let vtable=self.data_r_u32(mem,object).unwrap_or(0);
                let target=self.data_r_u32(mem,vtable.wrapping_add(8)).unwrap_or(0);
                crate::diagnostics::event(&format!("{{\"event\":\"minecraft_chunk_resolver\",\"object\":{object},\"vtable\":{vtable},\"target\":{target}}}"));
            }
            return self.light_fallback(5);
        }
        let partial=(y&0xfe)|((x&15)<<11);
        let address=chunk.checked_add((partial|((z&15)<<7))>>1)?.checked_add(offset)?;
        if !disjoint(address,1) {return None;}
        let byte=u32::from(self.data_r_u8(mem,address)?);
        let value=if y&1==0 {byte&15} else {byte>>4};
        let mut r=self.regs;r[0]=value;r[1]=y;r[2]=offset;r[3]=partial;r[PC]=r[14]&!1;
        let (zero,carry)=if y&1==0 {(true,true)} else {(value==0,byte&8!=0)};
        let cpsr=(self.cpsr&0x0fffffdf)|(u32::from(zero)<<30)|(u32::from(carry)<<29)|((r[14]&1)<<5);
        let words=[cx,cz,r[8],r[4],r[5],r[6],r[7],r[14]];
        let mut frame=[0u8;32];
        for (i,w) in words.iter().enumerate() {frame[i*4..i*4+4].copy_from_slice(&w.to_le_bytes());}
        self.minecraft_worldgen.calls[index]+=1;
        let s=&self.minecraft_worldgen;
        if s.checked[index]<128 || s.calls[index]%4096==0 || s.shadow_only {
            let fp=self.extregs;let fpscr=self.fpscr;
            let mut state=CpuState::Normal;let mut retired=0;
            for _ in 0..instructions {state=self.step_one(mem);retired+=1;if !matches!(state,CpuState::Normal){break;}}
            if matches!(state,CpuState::Normal) && self.regs==r && self.cpsr==cpsr
                && self.extregs==fp && self.fpscr==fpscr
                && mem.get_bytes_fallible(Ptr::from_bits(stack),32)?==frame {
                self.minecraft_worldgen.checked[index]+=1;
                if self.minecraft_worldgen.checked[index]==128 {report("shadow_gate_passed",index,128);}
            } else {self.minecraft_worldgen.disabled[index]=true;report("disabled_after_mismatch",index,self.minecraft_worldgen.calls[index]);}
            Some((state,retired))
        } else {
            mem.get_bytes_fallible_mut(Ptr::from_bits(stack),32)?.copy_from_slice(&frame);
            self.regs=r;self.cpsr=cpsr;
            self.minecraft_worldgen.fast[index]+=1;
            if self.minecraft_worldgen.fast[index]==1 {report("native_light_enabled",index,1);}
            Some((CpuState::Normal,instructions))
        }
    }
    #[inline(always)]
    pub(super) fn run_minecraft_batch(&mut self, mem: &mut Mem, budget: &mut u64) -> CpuState {
        let start=*budget;
        let state=if self.minecraft_worldgen.census.is_some(){self.run_minecraft_batch_inner::<true>(mem,budget)}
        else{self.run_minecraft_batch_inner::<false>(mem,budget)};
        let s=&mut self.minecraft_worldgen;s.retired=s.retired.wrapping_add(start-*budget);
        if s.window.as_ref().is_some_and(|w|s.retired>=w.next){self.native_window();}
        state
    }
    #[inline(always)]
    fn run_minecraft_batch_inner<const CENSUS:bool>(&mut self,mem:&mut Mem,budget:&mut u64)->CpuState {
        loop {
            // Only the bitmap test stays in the per-instruction loop; every
            // native entry, observer and hook sits behind one cold call.
            let translated=if native_entry(self.regs[PC]) {self.minecraft_native(mem,*budget)} else {None};
            let (state, used) = translated.unwrap_or_else(|| {
                if CENSUS {self.minecraft_worldgen.census.as_mut().unwrap().sample(self.regs[PC]);}
                (self.step_one(mem), 1)
            });
            *budget = budget.saturating_sub(used);
            match state {
                CpuState::Normal if *budget > 0 => (),
                other => return other,
            }
        }
    }
    #[cold]
    #[inline(never)]
    fn minecraft_native(&mut self,mem:&mut Mem,budget:u64)->Option<(CpuState,u64)> {
        match self.regs[PC] {
            0x1a002c|0x1a0062=>self.minecraft_twist(mem,budget),
            mesh::ENTRY=>self.minecraft_mesh_fill(mem,budget),
            0x6e414|0xad2b4|FRONTIER_SEED=>self.minecraft_seed(mem,budget),
            0x25b1c=>self.minecraft_empty_rect(mem,budget),
            0xbf294=>self.minecraft_light_lookup(mem,budget).or_else(||self.minecraft_light_prefix(mem,budget)),
            0x1a3828|0x1a3858=>self.minecraft_fill(mem,budget),
            0x19ffb8=>self.minecraft_random(mem,budget),
            0xbf310=>self.minecraft_light_tail(mem,budget),
            super::minecraft_noise::ENTRY=>self.minecraft_noise(mem,budget),
            0x1947b8=>self.minecraft_ore(mem,budget),
            0x12d924=>self.minecraft_chunk_grid(mem,budget),
            0x48214|0x482b2=>self.minecraft_cached_cell(mem,budget),
            0x25b32=>self.minecraft_byte_scan(mem,budget),
            0x47b08|0x47b44=>self.minecraft_column(mem,budget),
            0x483aa|0x483c6|0x483e2|0x483fe|0x48418|0x48430=>self.minecraft_neighbors(mem,budget),
            0x4e55a=>self.minecraft_surface(mem,budget),
            0x176f9c|0x91cc6=>{
                if self.minecraft_worldgen.chunk_trace {
                    let gen=self.regs[PC]==0x176f9c;
                    let ptr=self.regs[if gen{4}else{11}];
                    let offsets=if gen{[0x1c,0x20]}else{[4,12]};
                    if let (Some(x),Some(z))=(self.data_r_u32(mem,ptr.wrapping_add(offsets[0])),self.data_r_u32(mem,ptr.wrapping_add(offsets[1]))) {
                        let t=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
                        let name=if gen{"terrain_request"}else{"mesh_request"};
                        let y=if gen {0} else {self.data_r_u32(mem,ptr.wrapping_add(8)).unwrap_or(u32::MAX)};
                        crate::diagnostics::event(&format!(r#"{{"event":"{name}","x":{},"y":{y},"z":{},"unix_time_ms":{t}}}"#,x as i32,z as i32));
                    }
                }
                None
            }
            super::minecraft_queue::OBSERVE|super::minecraft_queue::SELECT|0x91ad4|0x91ad8|0x994c8=>{self.queue_point(mem);None}
            0x176d70|0x1642c2|0x1767a8|0x1767e4|0x176804=>{
                if self.minecraft_worldgen.chunk_trace {self.trace_chunk_transition(mem);}
                None
            }
            _=>self.minecraft_loading(mem,budget),
        }
    }
    // Per-block deltas every five seconds of diagnostics. `fast` counts native
    // calls for whole-call kernels and retired instructions for batched ones.
    #[cold]
    fn native_window(&mut self) {
        let s=&mut self.minecraft_worldgen;let Some(w)=s.window.as_mut() else {return};
        w.next=s.retired.wrapping_add(1<<22);
        let elapsed=w.last.elapsed().as_secs_f64();if elapsed<5.0{return;}
        let mut blocks=Vec::new();
        for i in 0..BLOCK_COUNT {
            let (calls,fast,checked)=(s.calls[i].wrapping_sub(w.calls[i]),s.fast[i].wrapping_sub(w.fast[i]),s.checked[i].wrapping_sub(w.checked[i]));
            if calls|fast|checked!=0||s.disabled[i] {blocks.push(format!("[{i},{calls},{fast},{checked},{}]",u8::from(s.disabled[i])));}
        }
        crate::diagnostics::event(&format!("{{\"event\":\"minecraft_native_window\",\"duration_s\":{elapsed:.3},\"retired\":{},\"blocks\":[{}]}}",
            s.retired.wrapping_sub(w.retired),blocks.join(",")));
        w.last=std::time::Instant::now();w.retired=s.retired;w.calls=s.calls;w.checked=s.checked;w.fast=s.fast;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Mem, Box<InterpreterCpu>, u32, u32) {
        let mut mem = Mem::new();
        mem.set_null_segment_size(0x4000);
        let cpu = InterpreterCpu::new(mem.null_segment_size()/4096);
        for i in 0..9 { mem.get_bytes_fallible_mut(Ptr::from_bits(ENTRIES[i]), CODE[i].len() as u32).unwrap().copy_from_slice(CODE[i]); }
        let base = mem.alloc(4096).to_bits();
        let table = mem.alloc(8).to_bits();
        mem.write(Ptr::<u32, true>::from_bits(table), 0);
        mem.write(Ptr::<u32, true>::from_bits(table+4), 0x9908b0df);
        (mem, cpu, base, table)
    }
    #[test]
    fn minecraft_probe_mesh_record_fill() {
        let (mut mem,mut cpu,_,_)=fixture();
        let code = [130,237,0,171,144,96,12,50,138,66,249,209];
        mem.get_bytes_fallible_mut(Ptr::from_bits(mesh::ENTRY),12).unwrap().copy_from_slice(&code);
        let base=mem.alloc(8192).to_bits()+4;
        let bytes=12*39;
        for n in 0..160u32 {
            mem.get_bytes_fallible_mut(Ptr::from_bits(base-4),bytes+8).unwrap().fill(0x9a);
            cpu.regs[2]=base;cpu.regs[1]=base+bytes;cpu.regs[0]=n.wrapping_mul(0x478abcde);
            cpu.extregs[20]=!n;cpu.extregs[21]=n.rotate_right(1);cpu.cpsr=0x30;cpu.regs[PC]=mesh::ENTRY;
            let budget=u64::from(5*(1+n%39)+2);
            let (_,used)=cpu.minecraft_mesh_fill(&mut mem,budget).unwrap();
            let records=(budget/5).min(if n<128 {8}else{39}) as u32;
            assert_eq!(used,5*u64::from(records));
            let end=base+records*12;
            assert_eq!((cpu.regs[2],cpu.regs[PC]),(end,if end==base+bytes {mesh::ENTRY+12}else{mesh::ENTRY}));
            let buffer=mem.get_bytes_fallible(Ptr::from_bits(base-4),bytes+8).unwrap();
            assert_eq!(&buffer[..4],&[0x9a;4]);
            for record in buffer[4..4+12*records as usize].chunks_exact(12) {
                assert_eq!(&record[..4],&(!n).to_le_bytes());assert_eq!(&record[4..8],&n.rotate_right(1).to_le_bytes());
                assert_eq!(&record[8..],&cpu.regs[0].to_le_bytes());
            }
            assert!(buffer[4+12*records as usize..].iter().all(|&b|b==0x9a));
            assert!(!cpu.minecraft_worldgen.disabled[MESH_INDEX]);
        }
        assert!(cpu.minecraft_worldgen.fast[MESH_INDEX]>0);
        cpu.regs[2]=mesh::ENTRY;cpu.regs[1]=mesh::ENTRY+12;
        assert!(cpu.minecraft_mesh_fill(&mut mem,5).is_none());
    }

    #[test]
    fn minecraft_probe_neighbor_stack_alias_and_fault() {
        use super::super::minecraft_loading as loading;
        let (mut mem,mut cpu,_,_)=fixture();
        let which=loading::entry_index(0x39884).unwrap();
        let (pc,_,_,code)=loading::BLOCKS[which];
        mem.get_bytes_fallible_mut(Ptr::from_bits(pc),code.len() as u32).unwrap().copy_from_slice(code);
        for sp in [0x400c,pc+12,pc+4] {
            // A PUSH beginning below the null segment or overlapping its own
            // code cannot write a prefix before the ordinary interpreter runs.
            cpu.regs[PC]=pc; cpu.regs[13]=sp;cpu.cpsr=0x30;
            let c=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let (after,used)=loading::fast(&mut mem,c,which);
            assert_eq!(used,0);assert_eq!(after.regs,c.regs);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(pc),code.len() as u32).unwrap(),code);
        }
    }

    #[test]
    #[ignore] // release timing; run explicitly
    fn minecraft_neighbor_release_benchmark() {
        use super::super::minecraft_loading as loading;
        let (mut mem,mut cpu,_,_)=fixture();
        let data=mem.alloc(0x10000).to_bits();let stack=data+0x8000;
        let object=data+0x1000;let xyz=data+0x2400;let mask=xyz+4;
        let which=loading::entry_index(0x39884).unwrap();
        let (entry,_,budget,code)=loading::BLOCKS[which];let ret=0x2010u32;
        mem.get_bytes_fallible_mut(Ptr::from_bits(entry),code.len() as u32).unwrap().copy_from_slice(code);
        let word=|mem:&mut Mem,off:u32,v|mem.write(Ptr::<u32,true>::from_bits(object+off),v);
        word(&mut mem,0x1040,data+0x3000);word(&mut mem,0x1044,data+0x3004);
        mem.write(Ptr::<u32,true>::from_bits(data+0x3000),data+0x4000);
        for native in [false,true,true,false] {
            let start=std::time::Instant::now();let mut checksum=0u32;let mut total=0u64;
            for n in 0..200_000u32 {
                let x=if n%5==0{255}else{n&15};let y=(n>>4)&15;let z=(n>>8)&15;
                mem.get_bytes_fallible_mut(Ptr::from_bits(xyz),3).unwrap().copy_from_slice(&[x as u8,y as u8,z as u8]);
                mem.write(Ptr::<u8,true>::from_bits(mask),0);
                mem.write(Ptr::<u8,true>::from_bits(object+0x3c+((x&15)<<8)+(y<<4)+z),(n&1) as u8);
                word(&mut mem,0x1050,0);
                cpu.regs=[0;16];cpu.regs[0]=object;cpu.regs[1]=xyz;cpu.regs[2]=mask;cpu.regs[13]=stack;cpu.regs[14]=ret|1;cpu.regs[PC]=entry;cpu.cpsr=0x30;
                if native {
                    // Use the shipping byte checks, periodic shadow and dispatch.
                    let (_,used)=cpu.minecraft_loading(&mut mem,budget).unwrap();total+=used;
                }else{
                    while cpu.regs[PC]!=ret {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));total+=1;}
                }
                assert_eq!(cpu.regs[PC],ret);
                checksum+=cpu.data_r_u32(&mem,object+0x1050).unwrap()+u32::from(cpu.data_r_u8(&mem,mask).unwrap());
            }
            assert_eq!(checksum,720_000);
            eprintln!("neighbor native={native} visits=200000 retired={total} seconds={:.6}",start.elapsed().as_secs_f64());
        }
    }

    #[test]
    #[ignore] // release timing; run explicitly
    fn minecraft_dispatch_loop_release_benchmark() {
        // adds/eors/subs/bne at a main-image PC with no native entry, so this
        // times only the bitmap test, step_one and the budget check.
        let (mut mem,mut cpu,_,_)=fixture();let pc=0x30000;
        mem.get_bytes_fallible_mut(Ptr::from_bits(pc),8).unwrap().copy_from_slice(&[0x01,0x30,0x41,0x40,0x01,0x3a,0xfb,0xd1]);
        cpu.minecraft_worldgen.enabled=true;
        for _ in 0..3 {
            cpu.regs=[0;16];cpu.regs[2]=u32::MAX;cpu.regs[PC]=pc;cpu.cpsr=0x30;
            let start=std::time::Instant::now();let mut total=0u64;
            for _ in 0..400 {let mut budget=100_000;assert!(matches!(cpu.run_minecraft_batch(&mut mem,&mut budget),CpuState::Normal));total+=100_000-budget;}
            assert_eq!((total,cpu.regs[0]),(40_000_000,10_000_000));
            eprintln!("dispatch loop retired={total} ns_per_instruction={:.3}",start.elapsed().as_secs_f64()*1e9/total as f64);
        }
    }

    #[test]
    fn minecraft_probe_tessellator_vertex_differential() {
        // Whole Tessellator::vertex calls through the dispatcher against the
        // interpreter alone: first an exact budget that stops at a varying
        // point, then spare budget up to an ARM SVC at the return address.
        // Covers chained islands, the zero-fill kernel and both visit kinds.
        use super::super::minecraft_loading as loading;
        let (mut mem,mut cpu,_,_)=fixture();
        for &(lo,_,_,code) in loading::BLOCKS {mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
        let ret=0x30000u32;mem.write(Ptr::<u32,true>::from_bits(ret),0xef000000);
        let base=mem.alloc(0x20000).to_bits();let owner=base+0x1000;let buffer=base+0x8000;let sp=base+0x18000;
        let put=|mem:&mut Mem,a:u32,v:u32|mem.write(Ptr::<u32,true>::from_bits(a),v);
        let region=|mem:&Mem|mem.get_bytes_fallible(Ptr::from_bits(base),0x20000).unwrap().to_vec();
        cpu.minecraft_worldgen.enabled=true;
        for n in 0..3000u32 {
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x20000).unwrap().fill(0);
            let (uv,color,normal)=(n&1!=0,n&2!=0,n&4!=0);
            let stride=12+8*u32::from(uv)+4*u32::from(color)+4*u32::from(normal);
            let offsets=[0,if uv {12} else {255},if color {12+8*u32::from(uv)} else {255},if normal {stride-4} else {255}];
            let count=(n/8)%64;let end=buffer+stride*count;
            put(&mut mem,owner+0x18,buffer);put(&mut mem,owner+0x1c,end);put(&mut mem,owner+0x20,end+stride*(1+n%3));
            put(&mut mem,owner+0xb8,count);put(&mut mem,owner+0xbc,n);put(&mut mem,owner+0x14,owner+0x200);
            mem.write(Ptr::<u8,true>::from_bits(owner+0x204),stride as u8);
            for (k,&o) in offsets.iter().enumerate() {mem.write(Ptr::<u8,true>::from_bits(owner+0x30+k as u32),o as u8);}
            mem.write(Ptr::<u8,true>::from_bits(owner+0x34),stride as u8);
            if count>0 {
                // Pointers into the previous vertex, advanced incrementally.
                let last=end-stride;put(&mut mem,owner+4,last);
                for (k,off) in [(1,8),(2,0xc),(3,0x10)] {if offsets[k]!=255 {put(&mut mem,owner+off,last+offsets[k]);}}
            }
            for i in 0..13 {put(&mut mem,owner+0x3c+i*4,((n as i32%23-11+i as i32) as f32/5.0).to_bits());}
            for i in 0..4 {put(&mut mem,owner+0x60+i*4,n.wrapping_mul(0x9e3779b9).rotate_left(i));}
            cpu.regs=std::array::from_fn(|i|n.wrapping_mul(0x91abcdef).rotate_left(i as u32));
            cpu.regs[0]=owner;cpu.regs[13]=sp;cpu.regs[14]=ret;cpu.regs[PC]=0x3999c;cpu.cpsr=(n<<28)|0x30;cpu.fpscr=0;
            for i in 0..64 {cpu.extregs[i]=(((n+i as u32)%29) as f32/7.0-2.0).to_bits();}
            let (regs,extregs,cpsr)=(cpu.regs,cpu.extregs,cpu.cpsr);
            let original=region(&mem);
            let mut steps=0u64;let split=1+u64::from(n.wrapping_mul(7919))%340;let mut middle=None;
            loop {
                if steps==split {middle=Some((cpu.regs,cpu.extregs,cpu.cpsr,region(&mem)));}
                if cpu.regs[PC]==ret {break;}
                assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));steps+=1;
            }
            let expected=(cpu.regs,cpu.extregs,cpu.cpsr,region(&mem));
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x20000).unwrap().copy_from_slice(&original);
            cpu.regs=regs;cpu.extregs=extregs;cpu.cpsr=cpsr;cpu.fpscr=0;
            let done=if middle.is_some() {split} else {0};
            if let Some(middle)=middle {
                let mut budget=split;assert!(matches!(cpu.run_minecraft_batch(&mut mem,&mut budget),CpuState::Normal));
                assert_eq!(budget,0);assert!((cpu.regs,cpu.extregs,cpu.cpsr,region(&mem))==middle,"split {split} case {n}");
            }
            let mut budget=1000;assert!(matches!(cpu.run_minecraft_batch(&mut mem,&mut budget),CpuState::Svc(0)));
            assert_eq!(1000-budget,steps-done+1,"case {n}");
            assert_eq!(cpu.regs[PC],ret+4);cpu.regs[PC]=ret;
            assert_eq!(cpu.regs,expected.0,"case {n}");assert_eq!(cpu.extregs,expected.1,"case {n}");assert_eq!(cpu.cpsr,expected.2,"case {n}");
            assert!(region(&mem)==expected.3,"memory case {n}");
        }
        for which in 96..102 {
            let i=36+which;let s=&cpu.minecraft_worldgen;
            assert!(!s.disabled[i]&&s.fast[i]>0,"island {which}");
        }
    }

    #[test]
    fn minecraft_probe_loading_chain_counters() {
        // Chaining must leave every counter where the dispatcher alone would,
        // across the 4096th-visit shadow checks it hands back, and when a
        // chained island declines: last, __append's push faults below 0x4000.
        use super::super::minecraft_loading as loading;
        let mut results=Vec::new();let mut depth=None;
        for single in [None,Some(false),Some(true)] {
            let (mut mem,mut cpu,_,_)=fixture();
            for &(lo,_,_,code) in loading::BLOCKS {mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
            let ret=0x30000u32;mem.write(Ptr::<u32,true>::from_bits(ret),0xef000000);
            let base=mem.alloc(0x100000).to_bits();let owner=base+0x1000;let buffer=base+0x10000;let sp=base+0x8000;
            let put=|mem:&mut Mem,a:u32,v:u32|mem.write(Ptr::<u32,true>::from_bits(a),v);
            put(&mut mem,owner+0x18,buffer);put(&mut mem,owner+0x1c,buffer);put(&mut mem,owner+0x20,base+0xf0000);put(&mut mem,owner+0x14,owner+0x200);
            mem.write(Ptr::<u8,true>::from_bits(owner+0x204),28);
            for (k,o) in [0u8,12,20,24,28].into_iter().enumerate() {mem.write(Ptr::<u8,true>::from_bits(owner+0x30+k as u32),o);}
            for i in 0..13 {put(&mut mem,owner+0x3c+i*4,(i as f32/4.0+1.0).to_bits());}
            let Some(single)=single else {
                // Interpreted, for the stack depth at __append's entry.
                cpu.regs[0]=owner;cpu.regs[13]=sp;cpu.regs[14]=ret;cpu.regs[PC]=0x3999c;cpu.cpsr=0x30;
                while cpu.regs[PC]!=0x1a37d0 {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
                depth=Some(sp-cpu.regs[13]);continue;
            };
            cpu.minecraft_worldgen=State{enabled:true,single_island:single,..State::default()};
            for n in 0..5001u32 {
                cpu.regs[0]=owner;cpu.regs[13]=if n<5000 {sp} else {0x4004+depth.unwrap()};cpu.regs[14]=ret;cpu.regs[PC]=0x3999c;cpu.cpsr=0x30;
                for i in 0..3 {cpu.extregs[i]=((n%97+i as u32) as f32).to_bits();}
                let mut budget=100_000;let state=cpu.run_minecraft_batch(&mut mem,&mut budget);
                // The interpreter replays the push and faults on it.
                assert!(if n<5000 {matches!(state,CpuState::Svc(0))} else {matches!(state,CpuState::Error(_))&&cpu.regs[PC]==0x1a37d2},"{n}");
            }
            let s=&cpu.minecraft_worldgen;
            assert!(s.calls[36+96..36+102].iter().all(|&c|c>4096),"{:?}",&s.calls[36+96..36+102]);
            results.push((s.calls,s.checked,s.fast,s.disabled,s.retired,cpu.regs,mem.get_bytes_fallible(Ptr::from_bits(base),0x100000).unwrap().to_vec()));
        }
        assert!(results[0]==results[1]);
    }

    #[test]
    #[ignore] // release timing; run explicitly
    fn minecraft_tessellator_vertex_release_benchmark() {
        // 100k appends of a position/uv/color/normal vertex: interpreted, then
        // through the dispatcher with only the store tail and zero fill native
        // (islands 96-101 off), then with every island and chaining. Each call
        // returns to an ARM SVC, as a quantum's budget is rarely the limit.
        use super::super::minecraft_loading as loading;
        let (mut mem,mut cpu,_,_)=fixture();
        for &(lo,_,_,code) in loading::BLOCKS {mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
        let ret=0x30000u32;mem.write(Ptr::<u32,true>::from_bits(ret),0xef000000);
        let base=mem.alloc(0x400000).to_bits();let owner=base+0x1000;let buffer=base+0x10000;let sp=base+0x8000;
        let mut expected=None;
        for mode in [0,1,2,2,1,0] {
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x400000).unwrap().fill(0);
            let put=|mem:&mut Mem,a:u32,v:u32|mem.write(Ptr::<u32,true>::from_bits(a),v);
            put(&mut mem,owner+0x18,buffer);put(&mut mem,owner+0x1c,buffer);put(&mut mem,owner+0x20,base+0x3f0000);put(&mut mem,owner+0x14,owner+0x200);
            mem.write(Ptr::<u8,true>::from_bits(owner+0x204),28);
            for (k,o) in [0u8,12,20,24,28].into_iter().enumerate() {mem.write(Ptr::<u8,true>::from_bits(owner+0x30+k as u32),o);}
            for i in 0..13 {put(&mut mem,owner+0x3c+i*4,(i as f32/4.0+1.0).to_bits());}
            cpu.minecraft_worldgen=State{enabled:mode>0,..State::default()};
            if mode==1 {cpu.minecraft_worldgen.disabled[36+96..36+102].fill(true);}
            let start=std::time::Instant::now();let mut total=0;
            for n in 0..100_000u32 {
                cpu.regs[0]=owner;cpu.regs[13]=sp;cpu.regs[14]=ret;cpu.regs[PC]=0x3999c;cpu.cpsr=0x30;
                for i in 0..3 {cpu.extregs[i]=((n%97+i as u32) as f32).to_bits();}
                if mode>0 {
                    let mut budget=100_000;assert!(matches!(cpu.run_minecraft_batch(&mut mem,&mut budget),CpuState::Svc(0)));total+=99_999-budget;
                } else {
                    while cpu.regs[PC]!=ret {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));total+=1;}
                    cpu.regs[PC]+=4;
                }
                assert_eq!(cpu.regs[PC],ret+4);
            }
            let seconds=start.elapsed().as_secs_f64();
            let bytes=mem.get_bytes_fallible(Ptr::from_bits(base),0x400000).unwrap().to_vec();
            if let Some(w)=&expected {assert!(&bytes==w);}else{expected=Some(bytes);}
            eprintln!("vertex mode={mode} calls=100000 retired={total} ns_per_call={:.1}",seconds*1e4);
        }
    }

    #[test]
    fn minecraft_probe_loading_partial_progress() {
        use super::super::minecraft_loading as loading;
        let(mut mem,mut cpu,_,_)=fixture();let base=mem.alloc(4096).to_bits();
        for &(lo,_,_,code) in loading::BLOCKS{mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
        for which in [0,1,4,37] {
            cpu.regs.fill(base);cpu.regs[13]=base+1024;cpu.regs[PC]=loading::BLOCKS[which].0;cpu.cpsr=0x30;cpu.extregs.fill(1.0f32.to_bits());cpu.fpscr=0;
            if which==0{cpu.regs[0]=0xfffffff8;}
            if which==37{cpu.regs[4]=0xfffffff0;cpu.regs[0]=1.0f32.to_bits();}
            if which==1{cpu.regs[10]=loading::BLOCKS[1].0;}
            if which==4{
                cpu.regs[8]=0;cpu.regs[6]=0;
                mem.write(Ptr::<u32,true>::from_bits(base+1024+0x40),base);
                mem.write(Ptr::<u32,true>::from_bits(base+1024+0x3c),0);
                mem.write(Ptr::<u32,true>::from_bits(base+0x28),0xffffffe0);
                mem.write(Ptr::<u32,true>::from_bits(0xfffffffc),0);
            }
            let before=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let(c,used)=loading::fast(&mut mem,before,which);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,c.regs,"island {which}");assert_eq!(cpu.extregs,c.extregs);assert_eq!(cpu.cpsr,c.cpsr);assert_eq!(cpu.fpscr,c.fpscr);
            if which==4{assert_eq!(cpu.regs[PC],0x1947fa);assert!(cpu.in_it_block());}
            if which==1{assert_eq!(cpu.regs[PC],0x4fe36);}
        }
    }

    #[test]
    fn minecraft_probe_loading_loop_batches() {
        use super::super::minecraft_loading as loading;
        let (mut mem,mut cpu,_,_)=fixture();
        let base=mem.alloc(0x20000).to_bits();
        for &(lo,_,_,code) in loading::BLOCKS {mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
        for which in [9,11] {for n in 1..65u32 {
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x20000).unwrap().fill(0);
            cpu.regs.fill(0);cpu.regs[PC]=loading::BLOCKS[which].0;cpu.cpsr=0x30;cpu.fpscr=n;
            cpu.extregs.fill(0.25f32.to_bits());
            if which==9 {
                cpu.regs[1]=base;cpu.regs[5]=n.min(4);cpu.regs[6]=n;cpu.regs[10]=base+0x4000;cpu.regs[11]=base+0x100;
                mem.write(Ptr::<u32,true>::from_bits(base+0x180),base);
                mem.write(Ptr::<u8,true>::from_bits(base+0x44),7);
            } else {
                cpu.regs[0]=base;cpu.regs[1]=base+n*4;cpu.extregs[0]=0x437effff;
                for i in 0..n*4{mem.write(Ptr::<u8,true>::from_bits(base+i),(i.wrapping_mul(n)%256)as u8);}
            }
            let before=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let original=mem.get_bytes_fallible(Ptr::from_bits(base),0x20000).unwrap().to_vec();
            let budget=loading::BLOCKS[which].2*u64::from(1+n%17);
            let (result,used,visits)=loading::fast_batch(&mut mem,before,which,budget,8);
            assert!(used<=budget);assert!(visits<=8);if n>2&&n%17>1{assert!(visits>1);}
            let expected=mem.get_bytes_fallible(Ptr::from_bits(base),0x20000).unwrap().to_vec();
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x20000).unwrap().copy_from_slice(&original);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,result.regs);assert_eq!(cpu.extregs,result.extregs);assert_eq!(cpu.cpsr,result.cpsr);assert_eq!(cpu.fpscr,result.fpscr);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),0x20000).unwrap(),expected);
        }}
    }

    #[test]
    fn minecraft_probe_loading_islands_differential() {
        use super::super::minecraft_loading as loading;
        use crate::pipeline_native::{Machine,Predict};
        let(mut mem,mut cpu,_,_)=fixture();
        mem.reserve(0x28b000,0x5000);
        mem.reserve(0x211000,0x1000); // original traversal direction table
        mem.reserve(0x24a000,0x4000);
        mem.reserve(0x213000,0x2000);
        let base=mem.alloc(0x40000).to_bits();let owner=base+0x1000;let chunk=base+0x4000;let sp=base+0x18000;
        for &(lo,_,_,code) in loading::BLOCKS {mem.get_bytes_fallible_mut(Ptr::from_bits(lo),code.len() as u32).unwrap().copy_from_slice(code);}
        let put=|mem:&mut Mem,a:u32,v:u32|mem.write(Ptr::<u32,true>::from_bits(a),v);
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x213028),4096).unwrap()
            .copy_from_slice(&super::super::minecraft_crc::TABLES);
        let mut counts=[0;loading::BLOCKS.len()];cpu.minecraft_worldgen.single_island=true;
        for which in 0..loading::BLOCKS.len() {for n in 0..1024u32 {
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x20000).unwrap().fill(0);
            cpu.regs.fill(base);cpu.regs[13]=sp;cpu.regs[15]=loading::BLOCKS[which].0;
            cpu.cpsr=(n<<28)|0x30;cpu.fpscr=n;
            for i in 0..64{cpu.extregs[i]=(((n as i32%17)-8) as f32/3.0+(i as f32)/17.0).to_bits();}
            cpu.regs[4]=n%8;cpu.regs[5]=n%32;cpu.regs[6]=n%48;cpu.regs[8]=owner;cpu.regs[10]=base+0x800;cpu.regs[11]=n%32;
            put(&mut mem,sp+4,if n%2==0{base}else{0});put(&mut mem,sp+8,owner);
            put(&mut mem,sp+0xc,n%16);put(&mut mem,sp+0x10,n%16);put(&mut mem,sp+0x14,n%16);
            put(&mut mem,sp+0x1c,n%32);put(&mut mem,sp+0x20,n*128);put(&mut mem,sp+0x24,n%16);
            put(&mut mem,sp+0x28,n%16);put(&mut mem,sp+0x2c,n%48);put(&mut mem,sp+0x34,owner);
            put(&mut mem,sp+0x38,n%48);put(&mut mem,sp+0x3c,cpu.regs[5]>>4);put(&mut mem,sp+0x40,owner);
            put(&mut mem,sp+0xac,if n%3==0{u32::MAX}else{n%140});
            put(&mut mem,owner,base);put(&mut mem,owner+0x10,base);put(&mut mem,owner+0x1c,chunk);
            put(&mut mem,owner+0x28,if n%8==0{0}else{chunk});
            put(&mut mem,base,base+0x100);put(&mut mem,base+0x108,0x12d925);
            put(&mut mem,chunk+0x1c,cpu.regs[5]>>4);put(&mut mem,chunk+0x20,cpu.regs[6]>>4);
            // Original PC-relative block property tables and stone singleton.
            for i in 0..256{put(&mut mem,0x28e260+0x400+i*4,n%17);put(&mut mem,0x28e260+0x800+i*4,n%16);}
            put(&mut mem,0x28b730,owner);mem.write(Ptr::<u8,true>::from_bits(owner+0x44),(n%16) as u8);
            match which {
                0=>{cpu.regs[3]=n%256;put(&mut mem,base+8,(n as f32/7.0).to_bits());}
                1=>{cpu.regs[2]=n%8;put(&mut mem,base+0x800,0.25f32.to_bits());}
                2=>{cpu.regs[5]=n%64;}
                3=>{cpu.regs[8]=n%140;}
                4=>{cpu.regs[8]=n%140;cpu.regs[10]=n*128;}
                5=>{cpu.regs[0]=n%256;}
                6=>{cpu.regs[0]=n%17;cpu.regs[10]=n%16;}
                7=>{for i in 0..13{cpu.regs[i]=n.wrapping_mul(0x91abcdef).rotate_left(i as u32);}cpu.regs[14]=!n;put(&mut mem,sp+0x4c,n.wrapping_mul(0xabcdef01));put(&mut mem,sp+0x48,!n);put(&mut mem,sp+0x5c,n<<23);put(&mut mem,sp+0x70,n);}
                8=>{cpu.regs[0]=n%32;cpu.regs[1]=n%128;cpu.regs[3]=n%64;cpu.regs[5]=n%2;cpu.regs[6]=chunk;}
                9=>{cpu.regs[1]=owner;cpu.regs[3]=(n%4)*2048;cpu.regs[5]=1+n%4;cpu.regs[6]=n%128;cpu.regs[9]=0;cpu.regs[10]=chunk;cpu.regs[11]=base;cpu.regs[12]=n%128;put(&mut mem,base+0x80,owner);}
                10=>{cpu.regs[0]=chunk;cpu.regs[3]=(n%256)*128;cpu.regs[6]=n%128;cpu.regs[9]=base+0x800;cpu.regs[5]=n;mem.write(Ptr::<u8,true>::from_bits(chunk+cpu.regs[3]+cpu.regs[6]+0x4c),(n%4) as u8);put(&mut mem,0x28b778,owner);put(&mut mem,owner,base+0x200);for i in 0..4{put(&mut mem,0x28e260+i*4,if i%2==0{0}else{owner});}put(&mut mem,owner+0x6c,base+0x200);mem.write(Ptr::<u8,true>::from_bits(base+0x20c),(n%2)as u8);mem.write(Ptr::<u8,true>::from_bits(base+0x204),(n%3)as u8);}
                11=>{cpu.regs[0]=chunk;cpu.regs[1]=chunk+4*(1+n%4);cpu.extregs[0]=f32::from_bits(0x437effff).to_bits();for i in 0..4{mem.write(Ptr::<u8,true>::from_bits(chunk+i),((n>>(i*2))&255)as u8);}}
                12=>{cpu.regs[0]=if n%3==0{0}else{chunk};cpu.regs[4]=n%2;cpu.regs[6]=owner;mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%8)as u8);}
                13=>{cpu.regs[5]=n%16;cpu.regs[10]=chunk;put(&mut mem,sp+0xc0,0);put(&mut mem,sp+0xb0,17);put(&mut mem,sp+0x9c,34);put(&mut mem,sp+0xa0,51);put(&mut mem,sp+0xe0,(n%4)*2048);for i in 0..80{put(&mut mem,sp+0x284+i*4,((i as f32-40.0)/8.0).to_bits());}}
                14=>{cpu.regs[0]=chunk;cpu.regs[6]=n%16;cpu.regs[10]=n%256;let at=chunk+((cpu.regs[10]|((n%16)<<4))*2)+0x1455c;mem.write(Ptr::<u16,true>::from_bits(at),if n%2==0{0xfc19}else{n as u16});put(&mut mem,sp+0x50,(n%16)<<11);}
                15=>{cpu.regs[8]=8+(n%8)*4;for off in [0x50,0x54,0x58]{put(&mut mem,sp+off,chunk);}for i in 0..16{put(&mut mem,chunk+i*4,(n+i)%7);}cpu.regs[4]=n.wrapping_mul(0x92345671);cpu.regs[11]=!n;}
                16|17=>{for i in 0..13{cpu.regs[i]=n.wrapping_mul(0x91abcdef).rotate_left(i as u32);}cpu.regs[14]=!n;put(&mut mem,sp+0x38,n.wrapping_mul(0xabcdef01));put(&mut mem,sp+0x40,!n);}
                18=>{cpu.regs[8]=n;put(&mut mem,sp+0x1c,n.wrapping_mul(0x12345678));}
                19=>{cpu.regs[1]=n%64;cpu.regs[4]=n%128;cpu.regs[10]=n%16;cpu.regs[11]=n%32;}
                20=>{cpu.regs[1]=n%64;cpu.regs[4]=n%128;cpu.regs[10]=n%16;cpu.regs[11]=n%32;put(&mut mem,sp+0x40,n%32);}
                21..=27=>{for i in 0..13{cpu.regs[i]=n.wrapping_mul(0x91abcdef).rotate_left(i as u32);}cpu.regs[6]=if n%2==0{0}else{cpu.regs[6]};cpu.regs[14]=!n;put(&mut mem,sp+0x30,n%2);put(&mut mem,sp+0x34,n%3);put(&mut mem,sp+0x38,n.wrapping_mul(0xabcdef01));put(&mut mem,sp+0x40,!n);put(&mut mem,sp+0x64,n%5);}
                28=>{cpu.regs[0]=n%7;cpu.regs[6]=owner;cpu.regs[8]=4*(n%16);cpu.regs[10]=n.wrapping_mul(0x12345678);put(&mut mem,sp+0x3c,owner);put(&mut mem,owner,chunk);put(&mut mem,sp+0x44,n%32);put(&mut mem,sp+0x64,n%8);}
                29=>{cpu.regs[8]=owner;cpu.regs[10]=base+0x200;cpu.regs[11]=n;put(&mut mem,base+0x200,n%64);put(&mut mem,base+0x204,n%140);put(&mut mem,base+0x208,n%64);put(&mut mem,chunk+0x1c,(n%64)>>4);put(&mut mem,chunk+0x20,(n%64)>>4);}
                30=>{cpu.regs[8]=owner;cpu.regs[6]=if n%8==0{0}else{chunk};cpu.regs[5]=n%2;mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%8)as u8);mem.write(Ptr::<u8,true>::from_bits(chunk+0x24),(n%2)as u8);}
                31=>{cpu.regs[0]=n%256;cpu.regs[4]=chunk;cpu.regs[5]=n%256;cpu.regs[11]=n.wrapping_mul(17);}
                32=>{cpu.regs[4]=n;cpu.regs[5]=n%4;cpu.regs[6]=chunk;cpu.regs[11]=n%8;for i in 0..8{put(&mut mem,0x28e260+i*4,if i%2==0{0}else{owner});}put(&mut mem,owner+0x5c,n%3);put(&mut mem,owner,base+0x100);put(&mut mem,base+0x16c,0x12345);put(&mut mem,sp+0x38,0);}
                33=>{cpu.regs[4]=owner;cpu.regs[5]=n%64;cpu.regs[10]=n%32;cpu.regs[11]=n%32;put(&mut mem,sp+0x2c,n%16);put(&mut mem,chunk+0x1c,(n%64+n%32)>>4);put(&mut mem,chunk+0x20,(n%32+n%16)>>4);}
                34=>{cpu.regs[0]=if n%8==0{0}else{chunk};cpu.regs[4]=owner;cpu.regs[10]=n%2;cpu.regs[8]=n;put(&mut mem,sp+0x3c,base+0x200);put(&mut mem,base+0x200,owner);put(&mut mem,sp+0x4c,n);put(&mut mem,sp+0x54,n);mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%8)as u8);put(&mut mem,owner,base+0x100);put(&mut mem,base+0x140,0x12345);}
                35=>{cpu.regs[0]=chunk;cpu.regs[6]=n%16;put(&mut mem,sp+0x38,n%140);put(&mut mem,sp+0x30,n%16);}
                36=>{cpu.regs[0]=n.wrapping_mul(0x91234567);cpu.regs[5]=if n%2==0{8}else{0};put(&mut mem,sp+0x34,n%4);}
                37=>{cpu.regs[0]=if n%8==0{[0,0x80000000,0x7f800000,0xff800000,0x7fc00000,1,0x3f800000,0x3f000000][(n/8%8)as usize]}else{((n as i32%32-8)as f32/16.0).to_bits()};cpu.regs[4]=owner;put(&mut mem,owner+76,((n as i32%64-16)as f32/32.0).to_bits());put(&mut mem,0xd9cf0,0);put(&mut mem,0xd9cf4,255.0f32.to_bits());}
                38=>{cpu.regs[0]=n%256;put(&mut mem,0x24afd8+0x10,chunk);mem.write(Ptr::<u8,true>::from_bits(0x24afd8+8),(n%2)as u8);for i in 0..256{put(&mut mem,chunk+i*4,n.wrapping_mul(0x12345678)^i);}}
                39=>{cpu.regs[5]=n%32;cpu.regs[6]=n%128;cpu.regs[8]=n%128;cpu.regs[10]=n;cpu.regs[11]=n%140;put(&mut mem,sp+0x38,n%140);put(&mut mem,sp+0x30,n%32);}
                40|41|42=>{cpu.regs[8]=owner;cpu.regs[6]=n%16;for off in [0x10,0x14,0x18]{put(&mut mem,owner+off,n%8);}for off in [0xa8,0xac,0xb0]{put(&mut mem,sp+off,n%9);}put(&mut mem,owner+0x1c,chunk);}
                43=>{cpu.regs[0]=owner;cpu.regs[1]=n;cpu.regs[2]=base+0x200;cpu.regs[3]=n;put(&mut mem,owner+12,chunk);mem.write(Ptr::<u8,true>::from_bits(chunk+0xa2d),(n%2)as u8);put(&mut mem,base+0x200,n%64);put(&mut mem,base+0x208,n%64);put(&mut mem,owner+0x28,if n%4==0{0}else{chunk});put(&mut mem,chunk+0x1c,(n%64)>>4);put(&mut mem,chunk+0x20,if n%2==0{(n%64)>>4}else{99});put(&mut mem,owner+0x10,base);put(&mut mem,base,base+0x100);put(&mut mem,base+0x108,0x12345);}
                44=>{cpu.regs[0]=if n%8==0{0}else{chunk};cpu.regs[4]=n%2;cpu.regs[6]=owner;cpu.regs[10]=n;mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%8)as u8);mem.write(Ptr::<u8,true>::from_bits(chunk+0x24),(n%2)as u8);}
                45=>{cpu.regs[0]=chunk;cpu.regs[1]=n%128;cpu.regs[2]=n%16;cpu.regs[3]=n%16;let at=chunk+(((n%128&0xfe)|((n%16)<<11)|((n%16)<<7))>>1)+0x1004c;mem.write(Ptr::<u8,true>::from_bits(at),(n%256)as u8);}
                46=>{cpu.regs[0]=chunk;cpu.regs[4]=(n%256)*128;cpu.regs[2]=1+n%127;cpu.regs[5]=cpu.regs[2];mem.write(Ptr::<u8,true>::from_bits(chunk+cpu.regs[4]+cpu.regs[2]+0x4c),if n%3==0{(1+n%255)as u8}else{0});}
                47=>{cpu.regs[1]=n%256;cpu.regs[5]=n.wrapping_mul(31);cpu.regs[6]=n%17;cpu.regs[9]=n%17;}
                48..=50=>{cpu.regs[12]=n%8;cpu.regs[14]=n%140;cpu.regs[9]=n%4;cpu.regs[5]=n%2;for off in [0x17c,0x7c,0x5c,0x58,0x54,0x3c]{put(&mut mem,sp+off,n%20);}for off in [0x6c,0x30,0x24]{put(&mut mem,sp+off,chunk);}put(&mut mem,sp+0x1c,n%16);mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%3)as u8);}
                51|52=>{cpu.regs[0]=owner;for i in 0..320{put(&mut mem,owner-8+i*4,(((n as i32%31)-15+i as i32%5)as f32/17.0).to_bits());}}
                53=>{cpu.regs[4]=n%256;cpu.regs[10]=n%8;cpu.regs[12]=1;put(&mut mem,sp+0x80,owner);mem.write(Ptr::<u8,true>::from_bits(owner),n as u8);}
                54=>{cpu.regs[8]=(n%6)*12;cpu.regs[11]=owner;cpu.regs[14]=chunk;cpu.regs[9]=base;for i in 0..3{put(&mut mem,owner+cpu.regs[8]+156+i*4,((i as i32-1)as f32).to_bits());put(&mut mem,chunk+cpu.regs[8]+0x7c0+i*4,((n as i32%5-2)as f32).to_bits());}}
                55=>{cpu.regs[0]=owner;cpu.regs[5]=base;put(&mut mem,owner,if n%8==0{0}else{chunk});put(&mut mem,base+0xf8,n%8);put(&mut mem,chunk+0xe8,n%3);}
                56=>{cpu.regs[8]=owner;cpu.regs[6]=chunk;put(&mut mem,owner,n*7);put(&mut mem,owner+4,n%200);put(&mut mem,owner+8,!n);put(&mut mem,chunk+0xac,16);}
                57=>{cpu.regs[6]=chunk;cpu.regs[0]=n%8;cpu.regs[10]=n%7;cpu.regs[4]=32;put(&mut mem,chunk+0xb0,owner);put(&mut mem,chunk+0xa4,2);put(&mut mem,owner+(32u32.wrapping_add(2u32.wrapping_mul(cpu.regs[0].wrapping_sub(cpu.regs[10])))).wrapping_mul(4),n);}
                58..=60=>{cpu.regs[0]=if n%7==0{0}else{chunk};cpu.regs[5]=owner;cpu.regs[4]=n%5;cpu.regs[8]=n%3;cpu.regs[10]=n%15;cpu.regs[11]=n%12;put(&mut mem,sp+0x3c,owner);put(&mut mem,owner+0x10,chunk);put(&mut mem,chunk,base);put(&mut mem,base+8,0x12345);mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%6)as u8);}
                61=>{cpu.regs[8]=(n%6)*12;cpu.regs[10]=n%6;}
                62..=65 => {
                    for i in 0..360 {
                        let sample = ((i as i32 % 23 - 11) * (n as i32 % 13 - 6)) as f32 / 7.0;
                        put(&mut mem, owner + i*4, sample.to_bits());
                    }
                    match which {
                        62 => {cpu.regs[0]=owner; cpu.regs[8]=n%19; cpu.regs[2]=n%21;
                               cpu.regs[4]=cpu.regs[8]+1+n%4; cpu.regs[5]=cpu.regs[2]+n%5;}
                        63 => {cpu.regs[0]=n%12; cpu.regs[2]=n%4; cpu.regs[3]=owner;
                               cpu.regs[8]=4; cpu.regs[9]=n%4; cpu.regs[5]=cpu.regs[0]+n%5;}
                        64 => {cpu.regs[0]=n%12; cpu.regs[3]=n%4; cpu.regs[9]=owner;
                               cpu.regs[8]=4; cpu.regs[5]=n%4; cpu.regs[6]=cpu.regs[0]+n%5; cpu.regs[12]=n%16;}
                        _ => {cpu.regs[0]=owner+8; cpu.regs[1]=n%5; cpu.regs[14]=base+(n%4);}
                    }
                }
                66..=76 => {
                    let x=n%32; let z=n.rotate_left(3)%32; let y=n%145;
                    cpu.regs[7]=sp+512;
                    put(&mut mem,sp+520,z); put(&mut mem,sp+524,n%8); put(&mut mem,sp+528,n%5);
                    put(&mut mem,owner+0x10,base); put(&mut mem,owner+0x28,chunk);
                    put(&mut mem,chunk+0x1c,if n%7==0{33}else{x>>4}); put(&mut mem,chunk+0x20,z>>4);
                    put(&mut mem,base,base+256);put(&mut mem,base+264,0x12345);
                    let pos=base+512;
                    put(&mut mem,pos,x);put(&mut mem,pos+4,y);put(&mut mem,pos+8,z);
                    let block_id=1+n%3;
                    mem.write(Ptr::<u8,true>::from_bits(chunk+0x4c+((x&15)<<11)+((z&15)<<7)+y),block_id as u8);
                    let material=base+768; let block=base+1024;
                    put(&mut mem,block+0x6c,material);
                    for id in 0..4 {put(&mut mem,0x28e260+4*id,if id==0{0}else{block});}
                    for off in [0x1c,0x2c] {put(&mut mem,sp+off,owner);}
                    put(&mut mem,sp+0x20,block+0x6c);put(&mut mem,sp+0x24,x);
                    put(&mut mem,sp+0x28,y);put(&mut mem,sp+0x3c,x);
                    put(&mut mem,sp+0x40,y);put(&mut mem,sp+0x44,z);
                    match which {
                        66 => {cpu.regs[0]=owner;cpu.regs[1]=block;cpu.regs[3]=n;put(&mut mem,block,base+256);put(&mut mem,base+256+0x18,0x12345);}
                        67 => {cpu.regs[0]=n%9;cpu.regs[4]=y;cpu.regs[6]=base+1536;put(&mut mem,base+1540,owner);cpu.regs[8]=x;cpu.regs[9]=z;}
                        68 => {cpu.regs[0]=block;cpu.regs[1]=owner;cpu.regs[2]=x;cpu.regs[3]=y;}
                        69 => {cpu.regs[0]=if n%5==0{0}else{chunk};cpu.regs[8]=x;cpu.regs[5]=y;cpu.regs[4]=z;cpu.regs[6]=0x28b600;cpu.regs[10]=owner;
                               put(&mut mem,sp+0x10,block);put(&mut mem,sp+0xc,block+0x6c);cpu.regs[11]=n%2;}
                        70 => {cpu.regs[1]=owner;cpu.regs[2]=x;cpu.regs[3]=y;}
                        71 => {mem.write(Ptr::<u8,true>::from_bits(sp+0x18),block_id as u8);}
                        72 => {cpu.regs[0]=owner;cpu.regs[1]=pos;}
                        73 => {cpu.regs[0]=if n%7==0{0}else{chunk};cpu.regs[4]=pos;cpu.regs[5]=owner;cpu.regs[6]=n%2;}
                        74..=75 => {cpu.regs[8]=n%4;cpu.regs[4]=x;cpu.regs[5]=z;cpu.regs[6]=z;}
                        _ => {cpu.regs[0]=if n%7==0{0}else{chunk};cpu.regs[4]=n%2;}
                    }
                }
                77 => {
                    cpu.regs[0]=owner;cpu.regs[9]=owner+16*(1+n%12);
                    cpu.regs[3]=n.wrapping_mul(0x789abcde);
                    cpu.regs[10]=0x213028;cpu.regs[12]=0x213428;
                    cpu.regs[14]=0x213828;cpu.regs[8]=0x213c28;
                    for i in 0..48 {put(&mut mem,owner+i*4,(n+i).wrapping_mul(0x1234567));}
                }
                78..=79 => {
                    let pos=base+0x300;let q=chunk;let table=base+0x200;
                    cpu.regs[0]=owner;cpu.regs[1]=pos;cpu.regs[2]=sp+128;
                    cpu.regs[14]=0x18000+(n%4);
                    let vals=[(n%16) as u8,((n>>3)%16) as u8,((n>>5)%16) as u8];
                    let mut xyz=vals;if n%5<3 {xyz[(n%5) as usize]=if n%2==0 {16}else{255};}
                    let mask=if n%3==0{0x80}else{0x42};
                    mem.get_bytes_fallible_mut(Ptr::from_bits(pos),3).unwrap().copy_from_slice(&xyz);
                    mem.write(Ptr::<u8,true>::from_bits(sp+128),mask);
                    mem.write(Ptr::<u8,true>::from_bits(owner+0x3c+((vals[0] as u32)<<8)+((vals[1] as u32)<<4)+vals[2] as u32),(n%2) as u8);
                    put(&mut mem,owner+0x1040,table);put(&mut mem,owner+0x1044,table+4);
                    put(&mut mem,owner+0x104c,n%7);put(&mut mem,owner+0x1050,if n%11==0 {0x554-n%7} else {n%19});
                    put(&mut mem,table,q);put(&mut mem,sp+24,0x18000+n%4);
                    if which==79 {cpu.regs[10]=owner+0x1044;cpu.regs[6]=owner+0x1040;cpu.regs[5]=owner+0x104c;cpu.regs[8]=owner+0x1050;cpu.regs[4]=pos;}
                }
                80..=81 => {
                    let pos=base+0x300;let x=n.wrapping_sub(512);let y=n%139;let z=n%23;
                    cpu.regs[0]=sp+128;cpu.regs[1]=owner;cpu.regs[2]=pos;cpu.regs[14]=0x18000+n%4;
                    put(&mut mem,pos,x);put(&mut mem,pos+4,y);put(&mut mem,pos+8,z);
                    put(&mut mem,chunk+0x1c,if n%7==0{33}else{((x as i32)>>4) as u32});put(&mut mem,chunk+0x20,z>>4);
                    let idx=((x&15)<<11)|((z&15)<<7)|y;mem.write(Ptr::<u8,true>::from_bits(chunk+0x4c+idx),n as u8);mem.write(Ptr::<u8,true>::from_bits(chunk+0x804c+(idx>>1)),(n>>1) as u8);
                    if which==81 {cpu.regs[0]=if n%7==0{0}else{chunk};cpu.regs[4]=n%2;cpu.regs[5]=pos;cpu.regs[6]=owner;cpu.regs[8]=sp+128;mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),(n%5) as u8);}
                }
                82..=87 => {
                    // The caller uses a large stack frame and six static offset vectors.
                    cpu.regs[13]=base;cpu.regs[0]=owner;cpu.regs[3]=n%20;cpu.regs[4]=n%16;
                    cpu.regs[5]=owner;cpu.regs[6]=chunk;cpu.regs[8]=base+128;cpu.regs[10]=base+0x3c114;cpu.regs[11]=n%16;cpu.regs[14]=n%16;
                    for off in [0x44,0x48,0x58]{put(&mut mem,base+off,n%16);}put(&mut mem,base+0x4c,owner);put(&mut mem,base+0x3c,chunk);
                    if which==86 || which==87 {cpu.regs[4]=base+128;}
                }
                88 => {put(&mut mem,sp+0x34,owner);put(&mut mem,sp+0x18,n%8);put(&mut mem,sp+0x38,n%13);put(&mut mem,sp+0x30,n%7);cpu.regs[8]=n%16;}
                90 => {
                    cpu.regs[4]=owner;
                    cpu.regs[1]=if n%5==0 {owner+60} else {chunk};
                    cpu.regs[2]=if n&1!=0 {chunk+16} else {0};
                    cpu.regs[3]=if n&2!=0 {chunk+24} else {0};
                    cpu.regs[9]=if n&4!=0 {chunk+32} else {0};
                    for i in 0..13 {put(&mut mem,owner+60+i*4,((n+i) as f32/7.).to_bits());}
                }
                91 => {
                    put(&mut mem,sp,n%32);
                    put(&mut mem,sp+4,n%7);
                }
                92 => {
                    cpu.regs[0]=owner;cpu.regs[1]=base+0x300;cpu.regs[4]=n&1;
                    put(&mut mem,base+0x530,if n%3==0{owner}else{0});
                    put(&mut mem,base+0x628,if n%5==0{owner}else{0});
                    mem.write(Ptr::<u8,true>::from_bits(owner+0x44),n as u8);
                    mem.write(Ptr::<u8,true>::from_bits(0x28d620+0xd8+(n&255)),(n%2) as u8);
                }
                94 => {
                    // Both table-result branches, including early singleton matches.
                    cpu.regs[0]=chunk;
                    cpu.regs[5]=n%3;
                    cpu.regs[11]=owner;
                    for (i,off) in [0x230,0x78,0x328,0x1f0].into_iter().enumerate() {
                        put(&mut mem,owner+off,if n%8==i as u32 {chunk} else {0});
                    }
                    mem.write(Ptr::<u8,true>::from_bits(chunk+0x44),n as u8);
                    let table=0xadb7e+4+0x1dfa9e+0xd8;
                    mem.write(Ptr::<u8,true>::from_bits(table+(n&255)),(n&1) as u8);
                    put(&mut mem,sp+68,0x2101+n%4);
                }
                95 => {
                    // Cover a populated queue, growth fallback, empty tail and
                    // rejection before append. Guard bytes surround the record.
                    let globals=0xfbbb2+4+0x18fafa;
                    let tail=if n%7==0 {0} else {base+0xe00};
                    put(&mut mem,globals+0x828,tail);
                    put(&mut mem,globals+0x82c,if n%5==0 {tail} else {tail+16});
                    cpu.regs[2]=chunk;cpu.regs[3]=n%16;cpu.regs[9]=owner;cpu.regs[10]=n%8;
                    mem.write(Ptr::<u8,true>::from_bits(chunk+0xec),(n%2) as u8);
                    put(&mut mem,owner+12,if n%4==0 {2} else {32});
                    let direction=0xfbbcc+4+0x1163a8;
                    mem.write(Ptr::<u8,true>::from_bits(direction+n%8),(n*17) as u8);
                }
                93 => {
                    cpu.regs[4]=n&1;put(&mut mem,sp+52,0x2011+n%4);
                }
                89 => {cpu.regs[0]=owner;cpu.regs[1]=chunk;mem.write(Ptr::<u8,true>::from_bits(0x28b600+0x2f4),(n%2) as u8);}
                96|97 => {
                    // Tessellator: grow, equal, shrink and reserve requests,
                    // missing attributes, incremental or recomputed pointers.
                    let stride=n%40;let count=n%50;let size=(n*7)%300;let format=owner+0x200;
                    put(&mut mem,owner+0x18,chunk);put(&mut mem,owner+0x1c,chunk+size);put(&mut mem,owner+0x20,chunk+size+n%64);
                    put(&mut mem,owner+0x38,if n%5==0 {n%100} else {0});put(&mut mem,owner+0xb8,count);put(&mut mem,owner+0xbc,n);
                    put(&mut mem,owner+4,if n%3==0 {0} else {chunk+n%24});put(&mut mem,owner+0x14,format);mem.write(Ptr::<u8,true>::from_bits(format+4),(n%32) as u8);
                    for (k,off) in [8,0xc,0x10].into_iter().enumerate() {put(&mut mem,owner+off,if (n>>k)&1!=0 {chunk+off} else {0});}
                    for k in 0..4 {mem.write(Ptr::<u8,true>::from_bits(owner+0x30+k),if (n>>(k+1))&1!=0 {0xff} else {((n*3+k)%24) as u8});}
                    mem.write(Ptr::<u8,true>::from_bits(owner+0x34),stride as u8);mem.write(Ptr::<u8,true>::from_bits(owner+0x70),u8::from(n%7==0));
                    cpu.regs[0]=owner;
                    if which==97 {cpu.regs[4]=owner;cpu.regs[6]=owner+0x18;cpu.regs[10]=owner+0x30;cpu.regs[5]=if n%4==0 {chunk+16} else {chunk};}
                }
                98|101 => {
                    for i in 0..24 {put(&mut mem,sp+i*4,n.wrapping_mul(0x9e3779b9).rotate_left(i));}
                    // Return address slot: above R7 for the vertex exit, sp+28 for the append exit.
                    let frame=sp+0x30+(n%4)*8;cpu.regs[7]=frame;
                    put(&mut mem,if which==98 {frame+4} else {sp+28},0x30000|(n%4));
                }
                99 => {
                    // In place, length error, maximum capacity and doubling.
                    let vector=owner+0x18;cpu.regs[0]=vector;cpu.regs[1]=if n%9==0 {0x80000000} else {n%64};
                    put(&mut mem,vector,if n%11==0 {0} else {chunk});put(&mut mem,vector+4,if n%11==0 {0x40000000} else if n%13==0 {0} else {chunk+n%100});
                    put(&mut mem,vector+8,if n%11==0 {0x40000000} else {chunk+n%100+(n*5)%80});
                }
                100 => {}
                _=>unreachable!()
            }
            let before=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let mut m=Machine{c:before,a:Predict::new(&mem),retired:0};
            if loading::execute(which,&mut m).is_none(){continue;}
            let touches=m.a.touches;let nt=m.a.nt;drop(m);
            let Some((state,used))=cpu.minecraft_loading(&mut mem,loading::BLOCKS[which].2) else {continue;};
            assert!(used<=loading::BLOCKS[which].2);assert!(matches!(state,CpuState::Normal));assert!(!cpu.minecraft_worldgen.disabled[36+which],"island {which} case {n}");
            let expected=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let memory:Vec<_>=touches[..nt].iter().map(|t|mem.get_bytes_fallible(Ptr::from_bits(t.a),t.n as u32).unwrap().to_vec()).collect();
            for t in &touches[..nt]{mem.get_bytes_fallible_mut(Ptr::from_bits(t.a),t.n as u32).unwrap().copy_from_slice(&t.before.to_le_bytes()[..t.n as usize]);}
            cpu.regs=before.regs;cpu.extregs=before.extregs;cpu.cpsr=before.cpsr;cpu.fpscr=before.fpscr;
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected.regs,"island {which} case {n}");assert_eq!(cpu.extregs,expected.extregs,"FP island {which} case {n}");
            assert_eq!(cpu.cpsr,expected.cpsr,"flags island {which} case {n}");assert_eq!(cpu.fpscr,expected.fpscr);
            for(t,b)in touches[..nt].iter().zip(memory){assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(t.a),t.n as u32).unwrap(),b);}
            counts[which]+=1;
        }}
        for which in 0..loading::BLOCKS.len(){assert!(native_entry(loading::BLOCKS[which].0));assert!(counts[which]>=if matches!(which,94|95) {1024} else {128},"island {which} only {} cases",counts[which]);assert!(cpu.minecraft_worldgen.fast[36+which]>0);}
        cpu.regs[PC]=loading::BLOCKS[0].0;
        assert!(cpu.minecraft_loading(&mut mem,0).is_none());
        mem.write(Ptr::<u8,true>::from_bits(loading::BLOCKS[0].0),0);
        assert!(cpu.minecraft_loading(&mut mem,1000).is_none());
    }

    #[test]
    fn minecraft_probe_twist_differential() {
        let (mut mem, mut cpu, base, table) = fixture();
        let mut random = 0x12345678u32;
        for index in 0..2 {
            for n in 0..1024u32 {
                for v in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4) {
                    random ^= random << 13; random ^= random >> 17; random ^= random << 5;
                    v.copy_from_slice(&random.to_le_bytes());
                }
                for r in &mut cpu.regs { *r = random.wrapping_mul(0x9e3779b9); random = random.rotate_left(7).wrapping_add(3); }
                cpu.regs[0]=base; cpu.regs[3]=match n%4 {0=>0,1=>if index==0{226}else{395},2=>100,_=>1};
                cpu.regs[9]=if n%8==7 {base+4} else {table}; // aliasing lookup/store
                cpu.regs[PC]=ENTRIES[index]; cpu.cpsr=(n<<28)|0x30; cpu.fpscr=random;
                cpu.extregs.fill(random);
                let p=cpu.predict_minecraft_twist(&mem,index).unwrap();
                let mut expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                let at=(p.address-base) as usize; expected[at..at+4].copy_from_slice(&p.value.to_le_bytes());
                let fp=cpu.extregs; let fpscr=cpu.fpscr;
                for _ in 0..15 { assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal)); }
                assert_eq!(cpu.regs,p.regs,"block {index} case {n}"); assert_eq!(cpu.cpsr,p.cpsr);
                assert_eq!(cpu.extregs,fp); assert_eq!(cpu.fpscr,fpscr);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            }
        }
    }
    #[test]
    fn minecraft_probe_twist_guards_and_fail_closed() {
        let (mut mem, mut cpu, base, table)=fixture();
        cpu.regs[0]=base; cpu.regs[9]=table; cpu.regs[PC]=ENTRIES[0]; cpu.cpsr=0x30;
        assert!(cpu.minecraft_twist(&mut mem,14).is_none());
        cpu.excl_set(base); assert!(cpu.minecraft_twist(&mut mem,15).is_none()); cpu.excl_clear();
        cpu.set_itstate(0x18); assert!(cpu.minecraft_twist(&mut mem,15).is_none()); cpu.set_itstate(0);
        cpu.cpsr &= !0x20; assert!(cpu.minecraft_twist(&mut mem,15).is_none()); cpu.cpsr |= 0x20;
        cpu.regs[9]=0; assert!(cpu.minecraft_twist(&mut mem,15).is_none()); cpu.regs[9]=table;
        mem.write(Ptr::<u8, true>::from_bits(ENTRIES[0]),0xff); assert!(cpu.minecraft_twist(&mut mem,15).is_none());
        mem.write(Ptr::<u8, true>::from_bits(ENTRIES[0]),CODE[0][0]);
        let expected=cpu.predict_minecraft_twist(&mem,0).unwrap();
        cpu.minecraft_worldgen.fault=true;
        assert!(cpu.minecraft_twist(&mut mem,15).is_some());
        assert!(cpu.minecraft_worldgen.disabled[0]); assert_eq!(cpu.regs,expected.regs);
        assert!(cpu.minecraft_twist(&mut mem,15).is_none());
    }
    #[test]
    fn minecraft_probe_twist_native_budgets() {
        let (mut mem, mut cpu, base, table) = fixture();
        for index in 0..2 {
            for budget in [1, 14, 15, 16, 31, 1920, 2401, 3405] {
                let bytes = mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap();
                for (i,b) in bytes.iter_mut().enumerate() { *b=(i.wrapping_mul(137) >> 3) as u8; }
                let original = bytes.to_vec();
                cpu.regs=[0;16]; cpu.regs[0]=base; cpu.regs[9]=table;
                cpu.regs[12]=0x7ffffffe; cpu.regs[14]=0x87654321;
                cpu.regs[PC]=ENTRIES[index]; cpu.cpsr=0xb0000030;
                let before=cpu.regs;
                cpu.minecraft_worldgen=State::default();
                let mut ticks=budget;
                assert!(matches!(cpu.run_batch(&mut mem,&mut ticks),CpuState::Normal));
                assert_eq!(ticks,0);
                let expected_regs=cpu.regs; let expected_cpsr=cpu.cpsr;
                let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
                cpu.regs=before; cpu.cpsr=0xb0000030;
                cpu.minecraft_worldgen=State{enabled:true,..State::default()};
                ticks=budget;
                assert!(matches!(cpu.run_batch(&mut mem,&mut ticks),CpuState::Normal));
                assert_eq!(ticks,0); assert_eq!(cpu.regs,expected_regs); assert_eq!(cpu.cpsr,expected_cpsr);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
                if budget>1920 { assert!(cpu.minecraft_worldgen.fast[index]>0); }
            }
        }
    }
    #[cfg(all(target_arch="aarch64",target_endian="little"))]
    #[test]
    #[ignore]
    fn minecraft_twist_release_benchmark() {
        let (mut mem,mut cpu,base,table)=fixture();
        let mut expected=None;
        for vector in [false,true,true,false] {
            cpu.minecraft_worldgen.neighbors_enabled=vector;
            let state=mem.get_bytes_fallible_mut(Ptr::from_bits(base),0x9c4).unwrap();
            for (i,w) in state.chunks_exact_mut(4).enumerate(){w.copy_from_slice(&(i as u32).wrapping_mul(3266489917).to_le_bytes());}
            let start=std::time::Instant::now();
            for _ in 0..100000 {
                for (index,count) in [(0,227),(1,396)] {
                    cpu.regs[0]=base;cpu.regs[3]=0;cpu.regs[9]=table;cpu.regs[12]=0x7fffffff;
                    cpu.regs[if index==0{14}else{6}]=cpu.data_r_u32(&mem,base+if index==0{4}else{0x390}).unwrap();
                    assert_eq!(cpu.minecraft_twist_batch(&mut mem,index,count*15),Some(count));
                }
            }
            let seconds=start.elapsed().as_secs_f64();
            let bytes=mem.get_bytes_fallible(Ptr::from_bits(base),0x9c4).unwrap();
            if let Some(w)=&expected {assert_eq!(bytes,w);}else{expected=Some(bytes.to_vec());}
            eprintln!("twist vector={vector} words=62300000 seconds={seconds:.6}");
        }
    }

    #[test]
    fn minecraft_probe_twist_fused_differential() {
        let (mut mem, mut cpu, base, table) = fixture();
        let mut random = 0xabcdef01u32;
        for index in 0..2 {
            let limit = if index == 0 { 227 } else { 396 };
            for n in 0..128u32 {
                for v in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4) {
                    random ^= random << 13; random ^= random >> 17; random ^= random << 5;
                    v.copy_from_slice(&random.to_le_bytes());
                }
                mem.write(Ptr::<u32,true>::from_bits(table),random.rotate_left(n%32));
                mem.write(Ptr::<u32,true>::from_bits(table+4),random.wrapping_add(n));
                let before_bytes = mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                cpu.regs.fill(random); cpu.regs[0]=base; cpu.regs[9]=table;
                let pos=if n%3==0 {limit-1-n%12} else {n%limit};
                cpu.regs[3]=pos; cpu.regs[PC]=ENTRIES[index];
                cpu.cpsr=(n<<28)|0x30;
                let before_regs=cpu.regs; let before_cpsr=cpu.cpsr;
                let iterations=if n%2==0 {limit-pos} else {(1+n%100).min(limit-pos)};
                for _ in 0..iterations*15 { assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal)); }
                let expected_regs=cpu.regs; let expected_cpsr=cpu.cpsr;
                let expected_bytes=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                cpu.regs=before_regs; cpu.cpsr=before_cpsr;
                mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&before_bytes);
                assert_eq!(cpu.minecraft_twist_batch(&mut mem,index,u64::from(iterations)*15),Some(u64::from(iterations)));
                assert_eq!(cpu.regs,expected_regs); assert_eq!(cpu.cpsr,expected_cpsr);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected_bytes);
                cpu.regs=before_regs; cpu.regs[9]=base+4;
                assert_eq!(cpu.minecraft_twist_batch(&mut mem,index,6000),None,"overlapping table must use ordered single-iteration fallback");
            }
        }
    }
    #[test]
    #[cfg(target_arch="aarch64")]
    fn minecraft_probe_crc_prefix_matches_original_loop() {
        use super::super::{minecraft_crc as crc, minecraft_loading as loading};
        let (mut mem,mut cpu,base,_)=fixture();
        let index=loading::entry_index(crc::ENTRY).unwrap();
        let (_,_,cost,code)=loading::BLOCKS[index];
        mem.get_bytes_fallible_mut(Ptr::from_bits(crc::ENTRY),code.len() as u32).unwrap().copy_from_slice(code);
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x213028),4096).unwrap().copy_from_slice(&crc::TABLES);
        for n in 0..130u32 {
            let bytes=mem.get_bytes_fallible_mut(Ptr::from_bits(base),1100).unwrap();
            for (i,b) in bytes.iter_mut().enumerate(){*b=(i as u32).wrapping_mul(n+7).rotate_left(n%17) as u8;}
            cpu.regs.fill(n);cpu.regs[0]=base+n%4;cpu.regs[9]=cpu.regs[0]+16*(1+n%65)+n%16;
            cpu.regs[PC]=crc::ENTRY;cpu.regs[3]=n.wrapping_mul(0x9e3779b9);
            cpu.regs[10]=0x213028;cpu.regs[12]=0x213428;cpu.regs[14]=0x213828;cpu.regs[8]=0x213c28;
            cpu.cpsr=(n<<28)|0x30;
            let before=crate::cpu::CpuContext{regs:cpu.regs,extregs:cpu.extregs,cpsr:cpu.cpsr,fpscr:cpu.fpscr};
            let budget=cost*(1+u64::from(n%64))+u64::from(n%13);
            let limit=(budget/cost).min(u64::from((cpu.regs[9]-cpu.regs[0])/16));
            let mut check=cpu.regs;
            let expected=if limit>=4 && std::arch::is_aarch64_feature_detected!("crc"){limit-1}else{0};
            assert_eq!(crc::prefix(&mem,&mut check,limit),expected);
            let (result,used,visits)=loading::fast_batch(&mut mem,before,index,budget,64);
            assert_eq!(visits,limit);assert_eq!(used,cost*limit);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(result.regs,cpu.regs);assert_eq!(result.cpsr,cpu.cpsr);
        }
        let mut check=cpu.regs;check[0]=base;check[9]=base+128;
        mem.write(Ptr::<u8,true>::from_bits(0x213028),1);
        let before=check;assert_eq!(crc::prefix(&mem,&mut check,8),0);assert_eq!(check,before);
    }
    #[test]
    fn minecraft_probe_seed_differential() {
        let (mut mem, mut cpu, base, _) = fixture();
        let mut random=0x5a5a1234u32;
        mem.get_bytes_fallible_mut(Ptr::from_bits(FRONTIER_SEED),FRONTIER_CODE.len() as u32).unwrap().copy_from_slice(FRONTIER_CODE);
        for (index,pc,previous,next,factor) in [(2,ENTRIES[2],3,2,1),(3,ENTRIES[3],1,0,10),(FRONTIER_INDEX,FRONTIER_SEED,2,1,0)] {
            for n in 0..256u32 {
                for v in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4) {
                    random^=random<<13;random^=random>>17;random^=random<<5;
                    v.copy_from_slice(&random.to_le_bytes());
                }
                let start=1+(n*17)%623;
                let iterations=if n%2==0 {624-start} else {(624-start).min(1+n%67)};
                mem.write(Ptr::<u32,true>::from_bits(base+0x9c4),start);
                let before_bytes=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                cpu.regs.fill(random);cpu.regs[4]=base;cpu.regs[previous]=start-1;cpu.regs[next]=start;
                if index==FRONTIER_INDEX {cpu.regs[3]=base;cpu.regs[6]=base-12;}
                cpu.regs[factor]=random;cpu.regs[PC]=pc;cpu.cpsr=(n<<28)|0x30;
                if index==FRONTIER_INDEX && n==0 {
                    // Identical instructions with an independently addressed counter
                    // cannot use the contiguous MT-state update.
                    cpu.regs[6]=base;
                    assert!(cpu.minecraft_seed(&mut mem,11).is_none());
                    assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),before_bytes);
                    cpu.regs[6]=base-12;
                }
                let before_regs=cpu.regs;let before_cpsr=cpu.cpsr;
                for _ in 0..iterations*11 {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
                let expected_regs=cpu.regs;let expected_cpsr=cpu.cpsr;
                let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&before_bytes);
                cpu.regs=before_regs;cpu.cpsr=before_cpsr;
                let (state,used)=cpu.minecraft_seed(&mut mem,u64::from(iterations)*11).unwrap();
                assert!(matches!(state,CpuState::Normal));assert_eq!(used,u64::from(iterations)*11);
                assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_cpsr);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
                assert!(!cpu.minecraft_worldgen.disabled[index]);
            }
            assert!(cpu.minecraft_worldgen.fast[index]>0);
        }
    }
    #[test]
    fn minecraft_probe_empty_rect_differential() {
        let (mut mem,mut cpu,base,_)=fixture();
        for n in 0..512u32 {
            cpu.regs.fill(n.wrapping_mul(0x9e3779b9));
            cpu.regs[3]=base+32;cpu.regs[13]=base+512;
            cpu.regs[4]=if n%16==15 {0x7fffffff} else {1+n%100};
            cpu.regs[10]=n%3;cpu.regs[6]=1+n%15;cpu.regs[PC]=ENTRIES[4];cpu.cpsr=(n<<28)|0x30;
            let area=match n%4 {0=>0,1=>10,2=>0x80000000,_=>0x7fffffff};
            mem.write(Ptr::<u32,true>::from_bits(base+512+0x44),area);
            mem.write(Ptr::<u8,true>::from_bits(base+32),0);
            let before_regs=cpu.regs;let before_cpsr=cpu.cpsr;
            let before=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let (state,used)=cpu.minecraft_empty_rect(&mut mem,15+u64::from(n%32)*15).unwrap();
            assert!(matches!(state,CpuState::Normal));
            let expected_regs=cpu.regs;let expected_cpsr=cpu.cpsr;
            cpu.regs=before_regs;cpu.cpsr=before_cpsr;
            for _ in 0..used {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_cpsr);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),before);
            assert!(!cpu.minecraft_worldgen.disabled[4]);
        }
        assert!(cpu.minecraft_worldgen.fast[4]>0);
        cpu.regs[PC]=ENTRIES[4];mem.write(Ptr::<u8,true>::from_bits(base+32),1);
        assert!(cpu.minecraft_empty_rect(&mut mem,1000).is_none());
    }

    #[test]
    fn minecraft_probe_light_differential() {
        let (mut mem,mut cpu,base,_)=fixture();
        let chunk=mem.alloc(0x15000).to_bits();let pos=base+64;let sp=base+1024;
        for n in 0..2048u32 {
            let y=n%128;let layer=(n/128)%2;
            let x=n.wrapping_mul(0x9e3779b9);let z=n.wrapping_mul(0xabcdef01);
            cpu.regs.fill(n.wrapping_mul(0x12345679));cpu.regs[0]=base;
            cpu.regs[1]=if layer==0 {0x28bd1c} else {0x28bd24};cpu.regs[2]=pos;
            cpu.regs[13]=sp;cpu.regs[14]=0x4000+(n&3);cpu.regs[PC]=ENTRIES[5];
            cpu.cpsr=(n<<28)|0x30;cpu.extregs.fill(n);cpu.fpscr=n;
            for (a,v) in [(base+0x28,chunk),(pos,x),(pos+4,y),(pos+8,z),
                (chunk+0x1c,((x as i32)>>4) as u32),(chunk+0x20,((z as i32)>>4) as u32)] {
                mem.write(Ptr::<u32,true>::from_bits(a),v);
            }
            let light=chunk+(((y&0xfe)|((x&15)<<11)|((z&15)<<7))>>1)+if layer==0{0xc04c}else{0x1004c};
            mem.write(Ptr::<u8,true>::from_bits(light),(n/4) as u8);
            let regs=cpu.regs;let flags=cpu.cpsr;
            let original=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let (state,used)=cpu.minecraft_light_lookup(&mut mem,55).unwrap();
            assert!(matches!(state,CpuState::Normal));assert_eq!(used,if layer==0{51}else{55});
            let expected_regs=cpu.regs;let expected_flags=cpu.cpsr;
            let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            cpu.regs=regs;cpu.cpsr=flags;
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
            for _ in 0..used {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_regs,"case {n}");assert_eq!(cpu.cpsr,expected_flags,"case {n}");
            assert_eq!(cpu.extregs,[n;64]);assert_eq!(cpu.fpscr,n);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            assert!(cpu.minecraft_worldgen.disabled.iter().all(|v| !v),"case {n}");
            cpu.regs=regs;cpu.cpsr=flags;
            assert!(cpu.minecraft_light_lookup(&mut mem,used-1).is_none());
        }
        for i in 5..9 {assert!(cpu.minecraft_worldgen.fast[i]>0);}
        cpu.regs[2]=sp-16;assert!(cpu.minecraft_light_lookup(&mut mem,55).is_none());
        cpu.regs[2]=pos;mem.write(Ptr::<u32,true>::from_bits(pos+4),128);
        assert!(cpu.minecraft_light_lookup(&mut mem,55).is_none());
        mem.write(Ptr::<u32,true>::from_bits(pos+4),0xffffffff);
        assert!(cpu.minecraft_light_lookup(&mut mem,55).is_none());
        mem.write(Ptr::<u32,true>::from_bits(pos+4),4);
        mem.write(Ptr::<u32,true>::from_bits(chunk+0x20),0);
        assert!(cpu.minecraft_light_lookup(&mut mem,55).is_none());
    }

    #[test]
    fn minecraft_probe_fill_differential() {
        let (mut mem,mut cpu,base,_)=fixture();
        for block in 6..8 {
            let cost=if block==6{9}else{7};let ptr=if block==6{1}else{6};
            for n in 0..512u32 {
                cpu.regs.fill(n.wrapping_mul(0x10203041));cpu.regs[4]=base;cpu.regs[ptr]=base+512;
                cpu.regs[5]=match n%4 {0=>1,1=>63,2=>0x80000000,_=>0xffffffff};
                cpu.regs[PC]=ENTRIES[block];cpu.cpsr=(n<<28)|0x30;
                mem.write(Ptr::<u32,true>::from_bits(base+4),base+512);
                mem.get_bytes_fallible_mut(Ptr::from_bits(base+512),512).unwrap().fill(0x55);
                let before=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                let regs=cpu.regs;let flags=cpu.cpsr;let fp=cpu.extregs;let fpscr=cpu.fpscr;
                let (state,used)=cpu.minecraft_fill(&mut mem,cost*(1+u64::from(n%200))).unwrap();
                assert!(matches!(state,CpuState::Normal));let expected_regs=cpu.regs;let expected_flags=cpu.cpsr;
                let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
                cpu.regs=regs;cpu.cpsr=flags;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&before);
                for _ in 0..used {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
                assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_flags);assert_eq!(cpu.extregs,fp);assert_eq!(cpu.fpscr,fpscr);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
                assert!(!cpu.minecraft_worldgen.disabled[block+3],"block {block}, case {n}");
            }
            assert!(cpu.minecraft_worldgen.fast[block+3]>0);
        }
    }
    #[test]
    fn minecraft_probe_fill_large_first_use_and_fast_path() {
        let (mut mem,mut cpu,header,_)=fixture();let buffer=mem.alloc(0x10000).to_bits();
        for block in 6..8 {
            let index=block+3;let cost=if block==6{9}else{7};let ptr=if block==6{1}else{6};
            for visit in 0..130 {
                cpu.regs.fill(0);cpu.regs[0]=0xa5;cpu.regs[4]=header;cpu.regs[ptr]=buffer;
                cpu.regs[5]=16384;cpu.regs[PC]=ENTRIES[block];cpu.cpsr=0x30;
                mem.write(Ptr::<u32,true>::from_bits(header+4),buffer);
                mem.get_bytes_fallible_mut(Ptr::from_bits(buffer),0x10000).unwrap().fill(0x77);
                let regs=cpu.regs;let flags=cpu.cpsr;
                let (_,used)=cpu.minecraft_fill(&mut mem,16384*cost).unwrap();
                assert_eq!(used,if visit<128{64*cost}else{16384*cost});
                let expected=cpu.regs;let expected_flags=cpu.cpsr;
                let bytes=mem.get_bytes_fallible(Ptr::from_bits(buffer),0x10000).unwrap().to_vec();
                let end=mem.read(Ptr::<u32,true>::from_bits(header+4));
                cpu.regs=regs;cpu.cpsr=flags;
                mem.write(Ptr::<u32,true>::from_bits(header+4),buffer);
                mem.get_bytes_fallible_mut(Ptr::from_bits(buffer),0x10000).unwrap().fill(0x77);
                for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
                assert_eq!(cpu.regs,expected);assert_eq!(cpu.cpsr,expected_flags);
                assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(buffer),0x10000).unwrap(),bytes);
                assert_eq!(mem.read(Ptr::<u32,true>::from_bits(header+4)),end);
                assert!(!cpu.minecraft_worldgen.disabled[index]);
            }
            assert_eq!(cpu.minecraft_worldgen.checked[index],128);
            assert!(cpu.minecraft_worldgen.fast[index]>0);
        }
    }
    #[test]
    fn minecraft_probe_random_differential() {
        let (mut mem,mut cpu,base,_)=fixture();let stack=mem.alloc(4096).to_bits();
        for n in 0..2048u32 {
            let next=n%624;let sp=stack+1024;
            for (i,v) in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4).enumerate(){v.copy_from_slice(&(n.wrapping_mul(0x9e3779b9).wrapping_add(i as u32)).to_le_bytes());}
            mem.write(Ptr::<u32,true>::from_bits(base+0x9c4),next);
            cpu.regs.fill(n);cpu.regs[0]=base;cpu.regs[13]=sp;cpu.regs[14]=0x4000+(n%4);cpu.regs[PC]=ENTRIES[8];cpu.cpsr=(n<<28)|0x30;
            let original=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let original_stack=mem.get_bytes_fallible(Ptr::from_bits(stack),4096).unwrap().to_vec();
            let regs=cpu.regs;let flags=cpu.cpsr;
            let (state,used)=cpu.minecraft_random(&mut mem,20).unwrap();assert!(matches!(state,CpuState::Normal));assert_eq!(used,20);
            let expected_regs=cpu.regs;let expected_flags=cpu.cpsr;
            let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let expected_stack=mem.get_bytes_fallible(Ptr::from_bits(stack),4096).unwrap().to_vec();
            cpu.regs=regs;cpu.cpsr=flags;
            mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
            mem.get_bytes_fallible_mut(Ptr::from_bits(stack),4096).unwrap().copy_from_slice(&original_stack);
            for _ in 0..20 {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_flags);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(stack),4096).unwrap(),expected_stack);
            assert!(!cpu.minecraft_worldgen.disabled[11],"case {n}");
        }
        assert!(cpu.minecraft_worldgen.fast[11]>0);
    }

    #[test]
    fn minecraft_probe_light_tail_differential() {
        let (mut mem,mut cpu,base,_)=fixture();let chunk=mem.alloc(0x15000).to_bits();let pos=base+64;let sp=base+1024;
        for n in 0..2048u32 {
            let y=n%128;let layer=(n/128)%2;let x=n.wrapping_mul(0x9e3779b9);let z=n.wrapping_mul(0xabcdef01);
            cpu.regs.fill(n);cpu.regs[0]=chunk;cpu.regs[5]=pos;cpu.regs[8]=if layer==0{0x28bd1c}else{0x28bd24};cpu.regs[13]=sp;cpu.regs[PC]=0xbf310;cpu.cpsr=(n<<28)|0x30;
            for (i,v) in [x,y,z].iter().enumerate(){mem.write(Ptr::<u32,true>::from_bits(pos+i as u32*4),*v);}
            for i in 0..8 {mem.write(Ptr::<u32,true>::from_bits(sp+i*4),0x4000+n+i);}
            let light=chunk+(((y&0xfe)|((x&15)<<11)|((z&15)<<7))>>1)+if layer==0{0xc04c}else{0x1004c};
            mem.write(Ptr::<u8,true>::from_bits(light),(n/4) as u8);
            let regs=cpu.regs;let flags=cpu.cpsr;let fp=cpu.extregs;let fpscr=cpu.fpscr;
            let bytes=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let (state,used)=cpu.minecraft_light_tail(&mut mem,29).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_regs=cpu.regs;let expected_flags=cpu.cpsr;cpu.regs=regs;cpu.cpsr=flags;
            for _ in 0..used {assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_flags);assert_eq!(cpu.extregs,fp);assert_eq!(cpu.fpscr,fpscr);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),bytes);
            assert!(cpu.minecraft_worldgen.disabled.iter().all(|v|!*v),"case {n}");
        }
        for i in 12..16{assert!(cpu.minecraft_worldgen.fast[i]>0);}
    }

    #[test]
    fn minecraft_probe_light_prefix_differential() {
        let(mut mem,mut cpu,base,_)=fixture();let chunk=mem.alloc(128).to_bits();let pos=base+64;let sp=base+1024;let object=base+128;let vtable=base+160;
        for n in 0..2048u32 {
            let x=n.wrapping_mul(0x9e3779b9);let z=n.wrapping_mul(0xabcdef01);
            let cx=((x as i32)>>4) as u32;let cz=((z as i32)>>4) as u32;
            for(a,v)in[(pos,x),(pos+4,n%128),(pos+8,z),(base+40,if n%4==0{0}else{chunk}),(base+16,object),(object,vtable),(vtable+8,0x4000+(n%4)),(chunk+28,if n%2==0{cx}else{cx^0x80000000}),(chunk+32,cz^0x80000000)]{mem.write(Ptr::<u32,true>::from_bits(a),v);}
            mem.write(Ptr::<u8,true>::from_bits(base+8),n as u8);
            cpu.regs.fill(n.wrapping_mul(0x12345679));cpu.regs[0]=base;cpu.regs[2]=pos;cpu.regs[13]=sp;cpu.regs[PC]=ENTRIES[5];cpu.cpsr=(n<<28)|0x30;
            let regs=cpu.regs;let flags=cpu.cpsr;let original=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_light_prefix(&mut mem,30).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_regs=cpu.regs;let expected_flags=cpu.cpsr;let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            cpu.regs=regs;cpu.cpsr=flags;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_regs);assert_eq!(cpu.cpsr,expected_flags);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            assert!(cpu.minecraft_worldgen.disabled.iter().all(|v|!*v),"case {n}");
        }
        assert!(cpu.minecraft_worldgen.fast[16]>0);assert!(cpu.minecraft_worldgen.fast[17]>0);
    }

    #[test]
    fn minecraft_probe_noise_differential() {
        use super::super::minecraft_noise as noise;
        let(mut mem,mut cpu,base,_)=fixture();let stack=mem.alloc(4096).to_bits();let mut random=0xabcdef01u32;
        mem.get_bytes_fallible_mut(Ptr::from_bits(noise::ENTRY),noise::CODE.len() as u32).unwrap().copy_from_slice(noise::CODE);
        for n in 0..4096u32 {
            for v in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4){random^=random<<13;random^=random>>17;random^=random<<5;v.copy_from_slice(&(random%256).to_le_bytes());}
            for f in &mut cpu.extregs{random^=random<<13;random^=random>>17;random^=random<<5;*f=if n%8==0{random}else{((random as i32) as f32/2147483648.0).to_bits()};}
            cpu.regs.fill(n.wrapping_mul(0xabcdef01));cpu.regs[0]=base;cpu.regs[9]=n%256;cpu.regs[11]=(n*13)%256;cpu.regs[13]=stack;cpu.regs[PC]=noise::ENTRY;cpu.cpsr=(n<<28)|0x30;cpu.fpscr=n;
            mem.write(Ptr::<u32,true>::from_bits(stack+24),base);mem.write(Ptr::<u32,true>::from_bits(stack+28),base+4);
            let before=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();let before_stack=mem.get_bytes_fallible(Ptr::from_bits(stack),4096).unwrap().to_vec();
            let p=noise::predict(&cpu,&mem).unwrap();let regs=cpu.regs;let fp=cpu.extregs;let flags=cpu.cpsr;
            for _ in 0..p.used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,p.regs,"registers case {n}");assert_eq!(cpu.extregs,p.fp,"FP case {n}");assert_eq!(cpu.cpsr,p.cpsr,"flags case {n}");assert_eq!(cpu.fpscr,n);
            assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),before);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(stack),4096).unwrap(),before_stack);
            cpu.regs=regs;cpu.extregs=fp;cpu.cpsr=flags;
            let(state,used)=cpu.minecraft_noise(&mut mem,noise::MAX_INSTRUCTIONS).unwrap();assert!(matches!(state,CpuState::Normal));assert_eq!(used,p.used);
            assert_eq!(cpu.regs,p.regs);assert_eq!(cpu.extregs,p.fp);assert_eq!(cpu.cpsr,p.cpsr);assert!(!cpu.minecraft_worldgen.disabled[18]);
        }
        assert!(cpu.minecraft_worldgen.fast[18]>0);
    }

    #[test]
    fn minecraft_probe_ore_differential() {
        let(mut mem,mut cpu,_,_)=fixture();
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x1947b6),ORE_CODE.len() as u32).unwrap().copy_from_slice(ORE_CODE);
        // The back edge includes MOV r6,r0 immediately before the guarded entry.
        mem.write(Ptr::<u16,true>::from_bits(0x1947b6),0x4606);
        for n in 0..2048u32 {
            cpu.regs.fill(n.wrapping_mul(0xabcdef01));cpu.regs[6]=if n%16==0{0x80000000}else{n%50};cpu.regs[11]=if n%16==0{0x80000004}else{n%50+20};cpu.regs[PC]=0x1947b8;
            cpu.extregs.fill(n);cpu.set_s_f32(19,0.5);cpu.set_s_f32(31,(n%60) as f32);cpu.set_s_f32(23,0.25+(n%10) as f32);cpu.set_s_f32(30,(n%8) as f32/8.0);cpu.set_s_f32(29,if n%2==0{1.0}else{0.0});cpu.cpsr=(n<<28)|0x30;cpu.fpscr=n;
            let r=cpu.regs;let f=cpu.extregs;let c=cpu.cpsr;let fpscr=cpu.fpscr;
            let(state,used)=cpu.minecraft_ore(&mut mem,10+u64::from(n%60)*15).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_r=cpu.regs;let expected_f=cpu.extregs;let expected_c=cpu.cpsr;let expected_fpscr=cpu.fpscr;
            cpu.regs=r;cpu.extregs=f;cpu.cpsr=c;cpu.fpscr=fpscr;
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r);assert_eq!(cpu.extregs,expected_f);assert_eq!(cpu.cpsr,expected_c);assert_eq!(cpu.fpscr,expected_fpscr);
            assert!(!cpu.minecraft_worldgen.disabled[19],"case {n}");
        }
        assert!(cpu.minecraft_worldgen.fast[19]>0);
        cpu.regs[PC]=0x1947b8;cpu.set_s_f32(23,0.0);assert!(cpu.minecraft_ore(&mut mem,100).is_none());
        cpu.set_s_f32(23,1.0);cpu.set_s_f32(31,f32::NAN);assert!(cpu.minecraft_ore(&mut mem,100).is_none());
    }

    #[test]
    fn minecraft_probe_chunk_grid_differential() {
        let(mut mem,mut cpu,base,_)=fixture();let table=mem.alloc(4096).to_bits();let pos=base+256;let sp=base+1024;
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x12d924),RESOLVER_CODE.len() as u32).unwrap().copy_from_slice(RESOLVER_CODE);
        for n in 0..2048u32 {
            let min_x=(n as i32%40-20) as u32;let min_z=(n as i32%60-30) as u32;let x=min_x.wrapping_add(n%8);let z=min_z.wrapping_add(n%7);let min_y=(-(n as i32%3)) as u32;
            let at=(8*(z.wrapping_sub(min_z))+x.wrapping_sub(min_x)).wrapping_sub(64u32.wrapping_mul(min_y));
            for(a,v)in[(pos,x),(pos+4,z),(base+0x78,min_x),(base+0x84,min_x.wrapping_add(7)),(base+0x7c,min_y),(base+0x88,3),(base+0x80,min_z),(base+0x8c,min_z.wrapping_add(6)),(base+0x90,8),(base+0x9c,64),(base+0xa8,table),(table+at*4,n.wrapping_mul(0xabcdef01))]{mem.write(Ptr::<u32,true>::from_bits(a),v);}
            cpu.regs.fill(n);cpu.regs[0]=base;cpu.regs[1]=pos;cpu.regs[13]=sp;cpu.regs[14]=0x4000+n%4;cpu.regs[PC]=0x12d924;cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let original=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_chunk_grid(&mut mem,31).unwrap();assert!(matches!(state,CpuState::Normal));assert_eq!(used,31);
            let expected_r=cpu.regs;let expected_c=cpu.cpsr;let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            cpu.regs=r;cpu.cpsr=c;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r);assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            assert!(!cpu.minecraft_worldgen.disabled[20],"case {n}");
        }
        assert!(cpu.minecraft_worldgen.fast[20]>0);
    }

    #[test]
    fn minecraft_probe_cached_cell_differential() {
        let(mut mem,mut cpu,base,_)=fixture();let chunk=mem.alloc(0x15000).to_bits();let sp=base+512;let owner=base+256;
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x48214),CELL_CODE.len() as u32).unwrap().copy_from_slice(CELL_CODE);
        for height in [false,true] {for n in 0..2048u32 {
            let x=n.wrapping_mul(0x9e3779b9);let z=n.wrapping_mul(0xabcdef01);let y=if height{n%256}else{n%128};
            for(a,v)in[(owner+28,base),(base+40,chunk),(sp+0xa8,x),(sp+0xac,y),(sp+0xb0,z),(chunk+28,((x as i32)>>4) as u32),(chunk+32,((z as i32)>>4) as u32)]{mem.write(Ptr::<u32,true>::from_bits(a),v);}
            let address=chunk+if height{0x1444c+(x&15)+((z&15)<<4)}else{0x4c+((x&15)<<11)+((z&15)<<7)+y};mem.write(Ptr::<u8,true>::from_bits(address),(n*13) as u8);
            cpu.regs.fill(n);cpu.regs[8]=owner;cpu.regs[13]=sp;cpu.regs[PC]=if height{0x48214}else{0x482b2};cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let original=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_cached_cell(&mut mem,32).unwrap();assert!(matches!(state,CpuState::Normal));let expected_r=cpu.regs;let expected_c=cpu.cpsr;
            let expected=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();cpu.regs=r;cpu.cpsr=c;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&original);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r);assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),expected);
            assert!(!cpu.minecraft_worldgen.disabled[if height{22}else{21}],"height {height}, case {n}");
        }}
        assert!(cpu.minecraft_worldgen.fast[21]>0);assert!(cpu.minecraft_worldgen.fast[22]>0);
    }

    #[test]
    fn minecraft_probe_byte_scan_differential() {
        let(mut mem,mut cpu,base,_)=fixture();
        for n in 0..2048u32 {
            let len=1+n%1000;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().fill((1+n%255) as u8);
            if n%2==0 {mem.write(Ptr::<u8,true>::from_bits(base+(n*13)%len),0);}
            cpu.regs.fill(n);cpu.regs[4]=n%5+len;cpu.regs[5]=n%5;cpu.regs[9]=base;cpu.regs[PC]=0x25b32;cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let bytes=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_byte_scan(&mut mem,7*(1+u64::from(n%1200))).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_r=cpu.regs;let expected_c=cpu.cpsr;cpu.regs=r;cpu.cpsr=c;
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r);assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),bytes);
            assert!(!cpu.minecraft_worldgen.disabled[23],"case {n}");
        }
        assert!(cpu.minecraft_worldgen.fast[23]>0);
    }
    #[test]
    fn minecraft_probe_column_differential() {
        let(mut mem,mut cpu,_,_)=fixture();let chunk=mem.alloc(0x15000).to_bits();let table=mem.alloc(2048).to_bits();
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x47b08),COLUMN_CODE.len() as u32).unwrap().copy_from_slice(COLUMN_CODE);
        for sky in [false,true]{for n in 0..2048u32 {
            for (i,b)in mem.get_bytes_fallible_mut(Ptr::from_bits(chunk),0x15000).unwrap().iter_mut().enumerate(){*b=(i as u32*31+n) as u8;}
            for i in 0..256u32{mem.write(Ptr::<u32,true>::from_bits(table+0x400+i*4),if (i+n)%29==0{1+n%20}else{0});}
            let col=(n%256)*128;
            cpu.regs.fill(n);cpu.regs[0]=chunk;cpu.regs[1]=if sky{15}else{1+n%127};cpu.regs[2]=table;cpu.regs[3]=col.wrapping_sub(1);cpu.regs[4]=col;cpu.regs[5]=col;cpu.regs[9]=1+n%127;cpu.regs[10]=0xff0;cpu.regs[14]=0xc04c;cpu.regs[PC]=if sky{0x47b44}else{0x47b08};cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let before=mem.get_bytes_fallible(Ptr::from_bits(chunk),0x15000).unwrap().to_vec();
            let(state,used)=cpu.minecraft_column(&mut mem,23*(1+u64::from(n%130))).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_r=cpu.regs;let expected_c=cpu.cpsr;let after=mem.get_bytes_fallible(Ptr::from_bits(chunk),0x15000).unwrap().to_vec();
            cpu.regs=r;cpu.cpsr=c;mem.get_bytes_fallible_mut(Ptr::from_bits(chunk),0x15000).unwrap().copy_from_slice(&before);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r,"sky {sky}, case {n}");assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(chunk),0x15000).unwrap(),after);
            assert!(!cpu.minecraft_worldgen.disabled[if sky{25}else{24}],"sky {sky}, case {n}");
        }}
        assert!(cpu.minecraft_worldgen.fast[24]>0);assert!(cpu.minecraft_worldgen.fast[25]>0);
    }

    #[test]
    fn minecraft_probe_neighbors_differential() {
        let(mut mem,mut cpu,base,_)=fixture();let sp=base+512;let owner=base+256;
        mem.get_bytes_fallible_mut(Ptr::from_bits(0x483aa),NEIGHBOR_CODE.len() as u32).unwrap().copy_from_slice(NEIGHBOR_CODE);
        for(which,pc)in [0x483aa,0x483c6,0x483e2,0x483fe,0x48418,0x48430].into_iter().enumerate(){for n in 0..1024u32 {
            let x=n.wrapping_mul(0x9e3779b9);let y=n.wrapping_mul(0xabcdef01);let z=n.wrapping_mul(0xdeadbeef);
            for(a,v)in [(owner,n),(owner+28,base),(sp+0xa8,x),(sp+0xac,y),(sp+0xb0,z)]{mem.write(Ptr::<u32,true>::from_bits(a),v);}
            cpu.regs.fill(n);cpu.regs[0]=x;cpu.regs[5]=z;cpu.regs[6]=owner;cpu.regs[8]=owner;cpu.regs[10]=y;cpu.regs[11]=n;cpu.regs[13]=sp;cpu.regs[PC]=pc;cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let before=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_neighbors(&mut mem,23).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_r=cpu.regs;let expected_c=cpu.cpsr;let after=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            cpu.regs=r;cpu.cpsr=c;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&before);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r,"which {which}, case {n}");assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),after);
            assert!(!cpu.minecraft_worldgen.disabled[26+which],"which {which}, case {n}");
        }assert!(cpu.minecraft_worldgen.fast[26+which]>0);}
    }

    #[test]
    fn minecraft_probe_surface_differential() {
        let(mut mem,mut cpu,base,_)=fixture();let chunk=mem.alloc(0x15000).to_bits();let object=base+256;let material=base+1024;let stone=base+1536;let sp=base+2048;
        for(a,code)in[(0x4e55a,SURFACE_CODE),(0x4e5c2,SURFACE_RESET),(0x4e750,SURFACE_TAIL),(0x4e552,SURFACE_HEAD)]{mem.get_bytes_fallible_mut(Ptr::from_bits(a),code.len() as u32).unwrap().copy_from_slice(code);}
        for which in 0..4usize {for n in 0..1024u32{
            let column=(n%256)*128;let y=5+n%123;let block=if which==0{0}else{1+n%254};
            for(a,v)in [(0x28e260+block*4,material),(object+200,10),(material+0x6c,if which==1{10}else{20}),(object+0x80,stone),(sp+28,column)]{mem.write(Ptr::<u32,true>::from_bits(a),v);}
            mem.write(Ptr::<u8,true>::from_bits(chunk+column+y+0x4c),block as u8);mem.write(Ptr::<u8,true>::from_bits(stone+0x44),if which==3{block as u8}else{255});
            cpu.regs.fill(n);cpu.regs[0]=n.wrapping_mul(0x9e3779b9);cpu.regs[4]=object;cpu.regs[5]=column;cpu.regs[6]=0xcccccccd;cpu.regs[8]=y;cpu.regs[10]=chunk;cpu.regs[11]=base;cpu.regs[13]=sp;cpu.regs[PC]=0x4e55a;cpu.cpsr=(n<<28)|0x30;
            let r=cpu.regs;let c=cpu.cpsr;let before=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            let(state,used)=cpu.minecraft_surface(&mut mem,40).unwrap();assert!(matches!(state,CpuState::Normal));
            let expected_r=cpu.regs;let expected_c=cpu.cpsr;let after=mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap().to_vec();
            cpu.regs=r;cpu.cpsr=c;mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().copy_from_slice(&before);
            for _ in 0..used{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));}
            assert_eq!(cpu.regs,expected_r,"which {which}, case {n}");assert_eq!(cpu.cpsr,expected_c);assert_eq!(mem.get_bytes_fallible(Ptr::from_bits(base),4096).unwrap(),after);
            assert!(!cpu.minecraft_worldgen.disabled[32+which],"which {which}, case {n}");
        }assert!(cpu.minecraft_worldgen.fast[32+which]>0);}
    }

}
