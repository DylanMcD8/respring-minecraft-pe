/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Interpreter hot-path benchmark, plus fault-semantics tests for the scalar
//! memory/fetch accessors and the in-interpreter division leaf calls.
use super::{InterpreterCpu, PC};
use crate::cpu::{CpuError, CpuState};
use crate::mem::{Mem, Ptr};

// Thumb-2 loop assembled with clang -target armv7-none-eabi -mthumb: PUSH/POP
// (16 and 32-bit), VPUSH/VPOP, LDR/STR/LDRB/LDRH/STRH, shifted-register ALU,
// ITE/IT with VFP, STM/LDM!, VLDR/VSTR/VLDM/VSTM, BL to a leaf, CBZ, LDRB
// (register), TBB, SUBS/BNE, then BKPT.
const LOOP_CODE:&[u8]=&[0xf0,0xb5,0x03,0xaf,0x2d,0xe9,0x00,0x0d,0x2d,0xed,0x04,0x8b,0x0a,0x68,0x4b,0x68,0xd2,0x18,0x8a,0x60,0x0c,0x7b,0xcd,0x89,0x04,0xeb,0x85,0x06,0x0e,0x82,0xd1,0xf8,0x14,0x80,0x88,0xea,0xf2,0x18,0xc1,0xf8,0x14,0x80,0xac,0x42,0xb4,0xbf,0x2c,0x46,0x25,0x46,0x01,0xf1,0x40,0x0a,0x8a,0xe8,0x7c,0x00,0xba,0xe8,0x0c,0x00,0x91,0xed,0x06,0x8a,0xd1,0xed,0x07,0x8a,0x28,0xee,0x28,0x9a,0x39,0xee,0x28,0x8a,0xb5,0xee,0xc0,0x8a,0xf1,0xee,0x10,0xfa,0xc8,0xbf,0x81,0xed,0x06,0x8a,0x91,0xec,0x04,0xaa,0x8a,0xec,0x04,0xaa,0x00,0xf0,0x1b,0xf8,0xd3,0x07,0x0b,0xb1,0x0b,0xf1,0x01,0x0b,0x11,0xf8,0x04,0x30,0x03,0xf0,0x07,0x03,0xdf,0xe8,0x03,0xf0,0x04,0x04,0x05,0x05,0x06,0x06,0x07,0x07,0x00,0xbf,0x00,0xbf,0x00,0xbf,0x00,0xbf,0xbd,0xec,0x04,0x8b,0xbd,0xe8,0x00,0x0d,0xbd,0xe8,0xf0,0x40,0x01,0x38,0xb1,0xd1,0x00,0xbe,0x80,0xb5,0x6f,0x46,0x0a,0x6a,0x02,0x44,0x0a,0x62,0xc2,0xf3,0xc4,0x0c,0x0c,0xfb,0x02,0xfc,0x80,0xbd];

fn fixture()->(Mem,Box<InterpreterCpu>){let mut mem=Mem::new();mem.set_null_segment_size(0x4000);let cpu=InterpreterCpu::new(4);(mem,cpu)}
fn fold(h:u64,w:u32)->u64{(h^u64::from(w)).wrapping_mul(0x100000001b3)}
fn digest(cpu:&InterpreterCpu,mem:&Mem,data:u32,len:u32)->u64{
    let mut h=0xcbf29ce484222325;for &w in cpu.regs.iter().chain(&cpu.extregs).chain([cpu.cpsr,cpu.fpscr].iter()){h=fold(h,w);}
    for b in mem.get_bytes_fallible(Ptr::from_bits(data),len).unwrap(){h=fold(h,u32::from(*b));}h
}

// Real guest code (the preserved noise gradient block) plus the call-heavy loop
// above, both through the shipping step_one/run_batch path.
fn workloads(scale:u32)->[(u64,f64,u64);2]{
    use super::minecraft_noise as noise;
    let(mut mem,mut cpu)=fixture();
    let code=mem.alloc(0x400).to_bits();let base=mem.alloc(4096).to_bits();let stack=mem.alloc(4096).to_bits();
    mem.get_bytes_fallible_mut(Ptr::from_bits(code),noise::CODE.len() as u32).unwrap().copy_from_slice(noise::CODE);
    let end=code+noise::CODE.len() as u32;
    for (i,v) in mem.get_bytes_fallible_mut(Ptr::from_bits(base),4096).unwrap().chunks_exact_mut(4).enumerate(){v.copy_from_slice(&((i as u32).wrapping_mul(2654435761)>>24).to_le_bytes());}
    mem.write(Ptr::<u32,true>::from_bits(stack+24),base);mem.write(Ptr::<u32,true>::from_bits(stack+28),base+4);
    let start=std::time::Instant::now();let mut n=0u64;let mut h=0u64;
    for run in 0..1300*scale {
        cpu.regs.fill(run.wrapping_mul(0xabcdef01));cpu.regs[0]=base;cpu.regs[9]=run%256;cpu.regs[11]=(run*13)%256;cpu.regs[13]=stack;cpu.regs[PC]=code;cpu.cpsr=(run<<28)|0x30;
        for (i,f) in cpu.extregs.iter_mut().enumerate(){*f=(((i as u32*7+run)%61) as f32/16.0-1.5).to_bits();}
        while cpu.regs[PC]!=end{assert!(matches!(cpu.step_one(&mut mem),CpuState::Normal));n+=1;}
        if run%64==0{h=fold(h,digest(&cpu,&mem,stack,64) as u32);}
    }
    let noise=(n,start.elapsed().as_secs_f64(),h);
    let(mut mem,mut cpu)=fixture();
    let code=mem.alloc(0x400).to_bits();let data=mem.alloc(256).to_bits();let stack=mem.alloc(4096).to_bits()+4096;
    mem.get_bytes_fallible_mut(Ptr::from_bits(code),LOOP_CODE.len() as u32).unwrap().copy_from_slice(LOOP_CODE);
    for (i,b) in mem.get_bytes_fallible_mut(Ptr::from_bits(data),256).unwrap().iter_mut().enumerate(){*b=(i as u8).wrapping_mul(29)&0x3f;}
    mem.write(Ptr::<f32,true>::from_bits(data+24),3.0);mem.write(Ptr::<f32,true>::from_bits(data+28),0.5);
    cpu.regs[0]=4500*scale;cpu.regs[1]=data;cpu.regs[13]=stack;cpu.regs[PC]=code;cpu.cpsr=0x30;
    let start=std::time::Instant::now();let mut budget=u64::MAX;
    let state=cpu.run_or_step(&mut mem,Some(&mut budget));
    assert!(matches!(state,CpuState::Error(CpuError::Breakpoint)),"{state:?}");assert_eq!((cpu.regs[0],cpu.regs[13],cpu.regs[PC]),(0,stack,code+0x9e));
    [noise,(u64::MAX-budget,start.elapsed().as_secs_f64(),digest(&cpu,&mem,data,256))]
}

// MCPE's division shape: Thumb code blx-ing 4-byte ARM stubs that dyld rewrote
// to `svc idx|RET_FLAG` (___divsi3 and ___umodsi3 here). With leaf calls off,
// the loop below stands in for handle_cpu_state, so its cost is a lower bound
// on the real host round trip.
const DIV_CODE:&[u16]=&[0x4620,0x2107,0x47b0,0x4405,0x4620,0x210d,0x47b8,0x4405,0x3c01,0xd1f5,0xbe00];
fn division_workload(iterations:u32,leaf:bool)->(u64,f64,u32){
    use crate::dyld::Dyld;
    let(mut mem,mut cpu)=fixture();let code=mem.alloc(0x100).to_bits();let stubs=code+0x80;
    for (i,h) in DIV_CODE.iter().enumerate(){mem.write(Ptr::<u16,true>::from_bits(code+2*i as u32),*h);}
    for i in 0..2{mem.write(Ptr::<u32,true>::from_bits(stubs+4*i),0xef00_0000|(Dyld::SVC_LINKED_FUNCTIONS_BASE+i)|Dyld::SVC_LAZY_LINK_RET_FLAG);}
    cpu.register_leaf_host_function(0,"___divsi3");cpu.register_leaf_host_function(1,"___umodsi3");cpu.set_leaf_host_calls(leaf);
    cpu.regs[4]=iterations;cpu.regs[5]=0;cpu.regs[6]=stubs;cpu.regs[7]=stubs+4;cpu.regs[PC]=code;cpu.cpsr=0x30;
    let start=std::time::Instant::now();let mut budget=u64::MAX;
    loop {
        match cpu.run_or_step(&mut mem,Some(&mut budget)) {
            CpuState::Svc(svc)=>{
                let(a,b)=(cpu.regs[0],cpu.regs[1]);let index=(svc&!Dyld::SVC_LAZY_LINK_RET_FLAG)-Dyld::SVC_LINKED_FUNCTIONS_BASE;
                cpu.regs[0]=if index==0{(a as i32).wrapping_div(b as i32) as u32}else{a%b};
                cpu.bx_write_pc(cpu.regs[14]);
            }
            CpuState::Error(CpuError::Breakpoint)=>break,
            state=>panic!("{state:?}"),
        }
    }
    (u64::MAX-budget,start.elapsed().as_secs_f64(),cpu.regs[5])
}
fn division_reference(iterations:u32)->u32{(1..=iterations).fold(0u32,|h,i|h.wrapping_add((i as i32/7) as u32).wrapping_add(i%13))}

#[test]
fn interpreter_workloads_keep_reference_state() {
    let [noise,call]=workloads(1);
    assert_eq!((noise.0,noise.2),(243080,0xcb9fb48c4ef520a2));assert_eq!((call.0,call.2),(249_751,0x20142789ec8f5415));
    // Same result and retire count whether the stubs finish in the interpreter or not.
    for leaf in [false,true]{let(n,_,r)=division_workload(1000,leaf);assert_eq!((n,r),(12*1000+1,division_reference(1000)),"leaf={leaf}");}
}

#[test]
#[ignore] // release timing; run explicitly
fn interpreter_release_benchmark() {
    for pass in 0..3 {
        let [noise,call]=workloads(200);
        let total=(noise.0+call.0) as f64;
        let(div_off,div_on)=(division_workload(2_000_000,false),division_workload(2_000_000,true));
        eprintln!("interp pass={pass} noise={} {:.3}ns/insn call={} {:.3}ns/insn mixed={:.3}ns/insn digests={:#x}/{:#x} div host={:.3}ns leaf={:.3}ns",
            noise.0,noise.1*1e9/noise.0 as f64,call.0,call.1*1e9/call.0 as f64,(noise.1+call.1)*1e9/total,noise.2,call.2,div_off.1*1e9/4e6,div_on.1*1e9/4e6);
        assert_eq!((noise.2,call.2),(0xc2f0a4e7b1ae5d93,0xdfd9ffd5bd212cfb));
        assert_eq!((div_off.0,div_off.2),(div_on.0,div_on.2));assert_eq!(div_on.2,division_reference(2_000_000));
    }
}

const TOP:u32=0xffff_fff8;
fn exec(cpu:&mut InterpreterCpu,mem:&mut Mem,pc:u32,thumb:bool,code:&[u8])->CpuState{
    mem.get_bytes_fallible_mut(Ptr::from_bits(pc),code.len() as u32).unwrap().copy_from_slice(code);
    cpu.regs[PC]=pc;cpu.cpsr=if thumb{0x30}else{0x10};cpu.step_one(mem)
}
fn fault(s:CpuState)->bool{matches!(s,CpuState::Error(CpuError::MemoryError))}
fn normal(s:CpuState)->bool{matches!(s,CpuState::Normal)}

#[test]
fn scalar_accessors_match_byte_slices() {
    let(mut mem,_)=fixture();
    for a in [0x4000,TOP]{mem.get_bytes_fallible_mut(Ptr::from_bits(a),8).unwrap().copy_from_slice(&[1,2,3,4,5,6,7,8]);}
    for a in [0,0x3ffc,0x3fff,0x4000,0x4001,0x4004,TOP,0xffff_fffb,0xffff_fffc,0xffff_fffd,0xffff_fffe,0xffff_ffff] {
        let p=Ptr::from_bits(a);
        assert_eq!(mem.read_fallible::<u32>(a),mem.get_bytes_fallible(p,4).map(|b|u32::from_le_bytes(b.try_into().unwrap())),"{a:#x}");
        assert_eq!(mem.read_fallible::<u16>(a),mem.get_bytes_fallible(p,2).map(|b|u16::from_le_bytes(b.try_into().unwrap())),"{a:#x}");
        assert_eq!(mem.read_fallible::<u8>(a),mem.get_bytes_fallible(p,1).map(|b|b[0]),"{a:#x}");
        for n in [1,2,4] {
            let ok=mem.get_bytes_fallible(p,n).is_some();let v=0x8765_4321u32;
            let wrote=match n{1=>mem.write_fallible(a,v as u8),2=>mem.write_fallible(a,v as u16),_=>mem.write_fallible(a,v)};
            assert_eq!(wrote,ok,"{a:#x}/{n}");if ok{assert_eq!(mem.get_bytes_fallible(p,n).unwrap(),&v.to_le_bytes()[..n as usize]);}
        }
    }
}

#[test]
fn fetch_faults_at_null_segment_and_top_of_memory() {
    let mut mem=Mem::new();mem.set_null_segment_size(0x4000);let mut cpu=InterpreterCpu::new(5);
    // The interpreter's own larger null segment still faults a fetch Mem allows, PC untouched.
    for (pc,thumb,code) in [(0x4ffe,true,&[0x01,0x20,0x02,0x20][..]),(0x4ffc,false,&0xe3a00004u32.to_le_bytes()[..])] {
        cpu.regs[0]=0;assert!(fault(exec(&mut cpu,&mut mem,pc,thumb,code)));assert_eq!((cpu.regs[0],cpu.regs[PC]),(0,pc));
    }
    assert!(normal(exec(&mut cpu,&mut mem,0x5000,true,&[0x03,0x20])));assert_eq!((cpu.regs[0],cpu.regs[PC]),(3,0x5002));
    assert!(normal(exec(&mut cpu,&mut mem,0x5000,false,&0xe3a00004u32.to_le_bytes())));assert_eq!((cpu.regs[0],cpu.regs[PC]),(4,0x5004));
    // The last halfword (16-bit) and the last word (32-bit MOV.W r0,#2) still execute.
    assert!(normal(exec(&mut cpu,&mut mem,0xffff_fffe,true,&[0x01,0x20])));assert_eq!((cpu.regs[0],cpu.regs[PC]),(1,0));
    assert!(normal(exec(&mut cpu,&mut mem,0xffff_fffc,true,&[0x4f,0xf0,0x02,0x00])));assert_eq!((cpu.regs[0],cpu.regs[PC]),(2,0));
}

#[test]
fn fetch_faults_in_mem_only_null_segment() {
    // No interpreter null segment, so Mem's own check alone faults these.
    let mut mem=Mem::new();mem.set_null_segment_size(0x4000);let mut cpu=InterpreterCpu::new(0);
    mem.get_bytes_fallible_mut(Ptr::from_bits(0x4000),4).unwrap().copy_from_slice(&[0x01,0x20,0x02,0x20]);
    for (pc,cpsr) in [(0x3ffe,0x30),(0x3ffc,0x10)] {
        cpu.regs[0]=0;cpu.regs[PC]=pc;cpu.cpsr=cpsr;assert!(fault(cpu.step_one(&mut mem)));assert_eq!((cpu.regs[0],cpu.regs[PC]),(0,pc));
    }
    cpu.regs[PC]=0x4000;cpu.cpsr=0x30;assert!(normal(cpu.step_one(&mut mem)));assert_eq!((cpu.regs[0],cpu.regs[PC]),(1,0x4002));
}

#[test]
fn it_block_mask_matches_itstate() {
    let mut cpu=InterpreterCpu::new(0);
    for other in [0,0xf9ff_03ff] {
        for it in 0..=255u8 {cpu.cpsr=other;cpu.set_itstate(it);assert_eq!(cpu.itstate(),it);assert_eq!(cpu.in_it_block(),it&0x0f!=0,"{it:#x}");}
    }
}

#[test]
fn leaf_division_symbols_resolve_to_compiler_rt() {
    // The interpreter keys its division leaves on these names, so compiler-rt's
    // must be the only host exports do_lazy_link can find under them.
    for s in ["___udivsi3","___umodsi3","___divsi3","___modsi3"] {
        let all=crate::dyld::DYLIB_LIST.iter().flat_map(|d|d.function_exports.iter()).flat_map(|l|l.iter()).filter(|e|e.0==s).count();
        assert_eq!((all,crate::libc::compiler_rt::FUNCTIONS.iter().filter(|e|e.0==s).count()),(1,1),"{s}");
    }
}

#[test]
fn leaf_division_calls_match_the_host_round_trip() {
    use crate::dyld::Dyld;
    let(mut mem,mut cpu)=fixture();let code=mem.alloc(64).to_bits();let base=Dyld::SVC_LINKED_FUNCTIONS_BASE;
    for (i,s) in ["___udivsi3","___umodsi3","___divsi3","___modsi3","_floorf"].iter().enumerate(){cpu.register_leaf_host_function(i as u32,s);}
    // (a, b, [udiv, umod, div, mod]); a zero divisor is left to the host.
    let cases=[(7,2,Some([3,1,3,1])),(u32::MAX,3,Some([0x5555_5555,0,0,u32::MAX])),(0x8000_0000,u32::MAX,Some([0,0x8000_0000,0x8000_0000,0])),
        ((-7i32) as u32,2,Some([0x7fff_fffc,1,(-3i32) as u32,u32::MAX])),(5,0,None)];
    for on in [false,true] {
        cpu.set_leaf_host_calls(on);
        for (a,b,expected) in cases { for kind in 0..5u32 { for ret in [0,Dyld::SVC_LAZY_LINK_RET_FLAG] {
            let svc=(base+kind)|ret;
            cpu.regs[0]=a;cpu.regs[1]=b;cpu.regs[14]=0x5001;
            let state=exec(&mut cpu,&mut mem,code,false,&(0xef00_0000|svc).to_le_bytes());
            match expected.filter(|_|on&&kind<4) {
                Some(r)=>{
                    assert!(normal(state));assert_eq!(cpu.regs[0],r[kind as usize],"{a:#x}/{b:#x} kind {kind}");
                    assert_eq!((cpu.regs[1],cpu.regs[PC],cpu.cpsr&0x20),if ret==0{(b,code+4,0)}else{(b,0x5000,0x20)});
                }
                None=>{assert!(matches!(state,CpuState::Svc(s) if s==svc));assert_eq!((cpu.regs[0],cpu.regs[PC],cpu.cpsr&0x20),(a,code+4,0));}
            }
        }}}
    }
    // Thumb SVC is never a dyld stub, so the same index still traps.
    cpu.regs[0]=7;cpu.regs[1]=2;
    assert!(matches!(exec(&mut cpu,&mut mem,code,true,&[0x03,0xdf]),CpuState::Svc(3)));assert_eq!((cpu.regs[0],cpu.regs[PC]),(7,code+2));
}
