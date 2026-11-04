/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Guarded AddVertex leaf. Never runs synthetic input against live GL globals.
//! Each of eight color/texture paths requires actual original-call validation.
use crate::{Environment,abi::GuestFunction,cpu::CpuContext,mem::{ConstPtr,MutPtr}};
use crate::dyld::{export_c_func,FunctionExports};
use crate::vertex_pack_kernel::{self as kernel,Inputs,Outcome};
const GLOBAL_SPANS:&[(u32,u32)]=&[(kernel::GLOBAL,8),(kernel::VERTICES,kernel::COUNTS+8-kernel::VERTICES)];
#[derive(Default)]struct Entry{shadows:u64,fast:u64,calls:u64}
#[derive(Default)]pub(crate)struct State{original:Option<GuestFunction>,address:u32,prefix:[u8;8],disabled:bool,entries:[Entry;8],shadows:u64,fallback:u64}
struct Prediction{outcome:Outcome,fifth:u32}
fn report(class:usize,kind:&str,n:u64){crate::diagnostics::event(&format!("{{\"event\":\"vertex_pack_native\",\"class\":{class},\"kind\":\"{kind}\",\"count\":{n}}}"));}
fn equal(a:CpuContext,b:CpuContext)->bool{a.regs==b.regs&&a.extregs==b.extregs&&a.cpsr==b.cpsr&&a.fpscr==b.fpscr}
fn original(env:&mut Environment){env.libc_state.peggle_vertex_pack.original.unwrap().call_without_pushing_stack_frame(env);}
fn disable(env:&mut Environment,class:usize,why:&str){let s=&mut env.libc_state.peggle_vertex_pack;s.disabled=true;let(a,p)=(s.address,s.prefix);env.mem.bytes_at_mut(MutPtr::<u8>::from_bits(a),8).copy_from_slice(&p);env.cpu.invalidate_cache_range(a,8);report(class,why,0);}
fn inputs(env:&Environment,c:CpuContext)->Option<Prediction>{
 let sp=c.regs[13];if sp<24||sp&3!=0||c.cpsr&0x0600fc00!=0{return None;}
 let configuration:[u8;8]=env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(kernel::GLOBAL).cast(),8)?.try_into().unwrap();
 let counters:[u8;8]=env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(kernel::COUNTS).cast(),8)?.try_into().unwrap();
 let v=u32::from_le_bytes(counters[..4].try_into().unwrap());let(base,len,_)=kernel::color_span(c,&configuration,&counters)?;
 let vertex=env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(kernel::VERTICES+v*8).cast(),8)?.try_into().unwrap();
 let colors=if len==0{Vec::new()}else{env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(base).cast(),len)?.to_vec()};
 let stack=env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(sp-24).cast(),24)?.try_into().unwrap();
 let fifth=u32::from_le_bytes(env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(sp).cast(),4)?.try_into().unwrap());
 Some(Prediction{outcome:kernel::calculate(Inputs{context:c,configuration,counters,vertex,colors,stack,fifth})?,fifth})
}
fn compare(env:&mut Environment,before:CpuContext,p:&Prediction)->bool{
 let mut snapshots=Vec::new();
 for &(base,len) in GLOBAL_SPANS {
  let Some(bytes)=env.mem.get_bytes_fallible(ConstPtr::<u8>::from_bits(base).cast(),len) else {original(env);return false;};
  let mut expected=bytes.to_vec();
  for r in &p.outcome.regions {if let Some((lo,hi))=r.dirty {
   if let Some(offset)=r.base.checked_sub(base) {let start=offset as usize+lo;let end=offset as usize+hi;if end<=expected.len(){expected[start..end].copy_from_slice(&r.data[lo..hi]);}}
  }}
  snapshots.push((base,expected));
 }
 original(env);
 equal(env.cpu.visible_context(),p.outcome.context)
 &&snapshots.iter().all(|(base,bytes)|env.mem.bytes_at(ConstPtr::<u8>::from_bits(*base),bytes.len()as u32)==bytes)
 &&env.mem.bytes_at(ConstPtr::<u8>::from_bits(before.regs[13]-24),24)==p.outcome.stack
 &&env.mem.read(ConstPtr::<u32>::from_bits(before.regs[13]))==p.fifth
}
fn vertex_pack(env:&mut Environment){
 if env.libc_state.peggle_vertex_pack.disabled{return original(env);}
 let before=env.cpu.visible_context();let Some(p)=inputs(env,before)else{let s=&mut env.libc_state.peggle_vertex_pack;s.fallback+=1;let n=s.fallback;if n==1||n%65536==0{report(255,"unsupported_contract_original_forwarded",n);}return original(env);};let class=p.outcome.class;
 let s=&mut env.libc_state.peggle_vertex_pack;let e=&mut s.entries[class];e.calls+=1;let shadow=e.shadows<64||s.shadows<128||e.calls%4096==0;
 if shadow{
  if !compare(env,before,&p){disable(env,class,"disabled_after_real_state_or_memory_mismatch");return;}
  let s=&mut env.libc_state.peggle_vertex_pack;s.shadows+=1;let e=&mut s.entries[class];e.shadows+=1;
  if e.shadows==64{report(class,"real_branch_gate_passed",64);}if env.libc_state.peggle_vertex_pack.shadows==128{report(255,"real_global_gate_passed",128);}return;
 }
 for r in &p.outcome.regions{if let Some((lo,hi))=r.dirty{env.mem.bytes_at_mut(MutPtr::<u8>::from_bits(r.base+lo as u32),(hi-lo)as u32).copy_from_slice(&r.data[lo..hi]);}}
 env.mem.bytes_at_mut(MutPtr::<u8>::from_bits(before.regs[13]-24),24).copy_from_slice(&p.outcome.stack);env.cpu.restore_visible_context(p.outcome.context);
 let e=&mut env.libc_state.peggle_vertex_pack.entries[class];e.fast+=1;let n=e.fast;if n==1||n%65536==0{report(class,"fast_vertices",n);}
}
const FUNCTIONS:FunctionExports=&[export_c_func!(vertex_pack())];
fn trampoline_bytes(prefix:&[u8],address:u32)->[u8;28]{assert_eq!(prefix.len(),18);let mut b=[0u8;28];b[..18].copy_from_slice(prefix);b[18..20].copy_from_slice(&0xbf00u16.to_le_bytes());b[20..24].copy_from_slice(&0xf000f8dfu32.to_le_bytes());b[24..].copy_from_slice(&((address+18)|1).to_le_bytes());b}
pub(crate)fn install(verified:bool,bins:&[crate::mach_o::MachO],mem:&mut crate::mem::Mem,dyld:&mut crate::dyld::Dyld,libc:&mut crate::libc::State){
 use digest::Digest;if !verified||std::env::var("PEGGLE_HD_VERTEX_PACK_NATIVE").as_deref()!=Ok("1"){return;}
 let address=0x20c83c;let symbol="__ZN4Sexy9AddVertexEffjff";
 let globals=[("__ZN4Sexy11GlobalColorE",kernel::GLOBAL),("__ZN4SexyL14NoTextureBoundE",kernel::GLOBAL+4),("__ZN4Sexy8VerticesE",kernel::VERTICES),("__ZN4Sexy12VertexColorsE",kernel::COLORS),("__ZN4Sexy11VertexCountE",kernel::COUNTS),("__ZN4Sexy11ColorsCountE",kernel::COUNTS+4)];
 if bins[0].exported_symbols.get(symbol)!=Some(&(address|1))||globals.iter().any(|(name,a)|bins[0].exported_symbols.get(*name)!=Some(a))||format!("{:x}",sha1::Sha1::digest(mem.bytes_at(ConstPtr::<u8>::from_bits(address),0x2cc)))!="2986ce15c2613dc2800eb54a8b1d5eeda6072472"{report(255,"image_or_global_guard_rejected",0);return;}
 let prefix:[u8;8]=mem.bytes_at(ConstPtr::<u8>::from_bits(address),8).try_into().unwrap();let first18=mem.bytes_at(ConstPtr::<u8>::from_bits(address),18).to_vec();let original=mem.alloc(28).cast::<u8>();mem.bytes_at_mut(original,28).copy_from_slice(&trampoline_bytes(&first18,address));
 let host=dyld.create_guest_function(mem,symbol,FUNCTIONS[0].1);libc.peggle_vertex_pack=State{original:Some(GuestFunction::from_addr_with_thumb_bit(original.to_bits()|1)),address,prefix,..State::default()};let p=MutPtr::<u32>::from_bits(address);mem.write(p,0xf000_f8dfu32);mem.write(p+1,host.addr_with_thumb_bit());report(255,"guarded_hook_installed",0);
}
