/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Audited AddVertex native leaf: private prediction, exact integer/VFP state.
//! No guest callbacks, GL calls, approximate FP, persistent cache or JIT.
use crate::cpu::CpuContext;
pub(crate)const GLOBAL:u32=0x4194c4;pub(crate)const VERTICES:u32=0x47c51c;pub(crate)const COLORS:u32=0x48c21c;pub(crate)const COUNTS:u32=0x49409c;pub(crate)const CAPACITY:u32=8096;
#[derive(Clone)]pub(crate)struct Region{pub(crate)base:u32,pub(crate)data:Vec<u8>,pub(crate)dirty:Option<(usize,usize)>}
pub(crate)struct Inputs{pub(crate)context:CpuContext,pub(crate)configuration:[u8;8],pub(crate)counters:[u8;8],pub(crate)vertex:[u8;8],pub(crate)colors:Vec<u8>,pub(crate)stack:[u8;24],pub(crate)fifth:u32}
pub(crate)struct Outcome{pub(crate)context:CpuContext,pub(crate)stack:[u8;24],pub(crate)regions:Vec<Region>,pub(crate)class:usize,pub(crate)blocks:u32}
pub(crate)fn color_span(c:CpuContext,configuration:&[u8;8],counters:&[u8;8])->Option<(u32,u32,usize)>{
 let v=u32::from_le_bytes(counters[..4].try_into().unwrap());let colors=u32::from_le_bytes(counters[4..].try_into().unwrap());let old=u32::from_le_bytes(configuration[..4].try_into().unwrap());if v>=CAPACITY||colors>=CAPACITY||colors!=0&&colors!=v{return None;}
 let(path,base,len)=if colors!=0{(0,COLORS+colors*4,4)}else if old==c.regs[2]{(1,COLORS,0)}else if v==0{(2,COLORS,0)}else{(3,COLORS,(v+1)*4)};Some((base,len,path*2+usize::from(configuration[4]&1!=0)))
}
fn disjoint(a:u32,n:u32,b:u32,m:u32)->bool{match(a.checked_add(n),b.checked_add(m)){(Some(ae),Some(be))=>ae<=b||be<=a,_=>false}}
struct Machine{c:CpuContext,frame:u32,entry:u32,fifth:u32,stack:[u8;24],regions:Vec<Region>}
impl Machine{
 #[inline(always)]fn nz(&mut self,v:u32){self.c.cpsr=(self.c.cpsr&0x3fffffff)|(v&0x80000000)|if v==0{0x40000000}else{0};}
 #[inline(always)]fn add(&mut self,a:u32,b:u32)->u32{let r=a.wrapping_add(b);let carry=u64::from(a)+u64::from(b)>u64::from(u32::MAX);let overflow=(!(a^b)&(a^r))&0x80000000!=0;self.c.cpsr=(self.c.cpsr&0x0fffffff)|(r&0x80000000)|if r==0{0x40000000}else{0}|if carry{0x20000000}else{0}|if overflow{0x10000000}else{0};r}
 #[inline(always)]fn sub(&mut self,a:u32,b:u32)->u32{let r=a.wrapping_sub(b);let overflow=((a^b)&(a^r))&0x80000000!=0;self.c.cpsr=(self.c.cpsr&0x0fffffff)|(r&0x80000000)|if r==0{0x40000000}else{0}|if a>=b{0x20000000}else{0}|if overflow{0x10000000}else{0};r}
 #[inline(always)]fn lsl(&mut self,a:u32,n:u32)->u32{if n==0{return a;}let out=a.wrapping_shl(n);self.c.cpsr=(self.c.cpsr&!0x20000000)|if a>>(32-n)&1!=0{0x20000000}else{0};self.nz(out);out}
 #[inline(always)]fn load8(&self,a:u32)->Option<u8>{if let Some(o)=a.checked_sub(self.frame){if let Some(v)=self.stack.get(o as usize){return Some(*v);}}if let Some(o)=a.checked_sub(self.entry){if o<4{return Some(self.fifth.to_le_bytes()[o as usize]);}}for r in &self.regions{if let Some(o)=a.checked_sub(r.base){if let Some(v)=r.data.get(o as usize){return Some(*v);}}}None}
 #[inline(always)]fn load32(&self,a:u32)->Option<u32>{Some(u32::from_le_bytes([self.load8(a)?,self.load8(a.checked_add(1)?)?,self.load8(a.checked_add(2)?)?,self.load8(a.checked_add(3)?)?]))}
 #[inline(always)]fn store8(&mut self,a:u32,v:u8)->Option<()>{if let Some(o)=a.checked_sub(self.frame){if let Some(p)=self.stack.get_mut(o as usize){*p=v;return Some(());}}for r in &mut self.regions{if let Some(o)=a.checked_sub(r.base){if let Some(p)=r.data.get_mut(o as usize){*p=v;let o=o as usize;r.dirty=Some(match r.dirty{Some((b,e))=>(b.min(o),e.max(o+1)),None=>(o,o+1)});return Some(());}}}None}
 #[inline(always)]fn store32(&mut self,a:u32,v:u32)->Option<()>{for(i,b)in v.to_le_bytes().into_iter().enumerate(){self.store8(a.checked_add(i as u32)?,b)?;}Some(())}
 #[inline(always)]fn store16(&mut self,a:u32,v:u16)->Option<()>{for(i,b)in v.to_le_bytes().into_iter().enumerate(){self.store8(a.checked_add(i as u32)?,b)?;}Some(())}
}
pub(crate)fn calculate(input:Inputs)->Option<Outcome>{
 let c=input.context;if c.regs[13]<24||c.regs[13]&3!=0||c.cpsr&0x0600fc00!=0{return None;}let(color_base,color_len,class)=color_span(c,&input.configuration,&input.counters)?;if input.colors.len()!=color_len as usize{return None;}let count=u32::from_le_bytes(input.counters[..4].try_into().unwrap());let mut regions=vec![Region{base:GLOBAL,data:input.configuration.to_vec(),dirty:None},Region{base:COUNTS,data:input.counters.to_vec(),dirty:None},Region{base:VERTICES+count*8,data:input.vertex.to_vec(),dirty:None}];if color_len!=0{regions.push(Region{base:color_base,data:input.colors,dirty:None});}
 for(i,r)in regions.iter().enumerate(){if !disjoint(r.base,r.data.len()as u32,c.regs[13]-24,28){return None;}for s in &regions[i+1..]{if !disjoint(r.base,r.data.len()as u32,s.base,s.data.len()as u32){return None;}}}
 let mut m=Machine{c,frame:c.regs[13]-24,entry:c.regs[13],fifth:input.fifth,stack:input.stack,regions};let mut pc=0x20c83c;let limit=128+count*8;
 for blocks in 0..limit{match pc{
 0x20c83c=>{
  // 20c83c sub sp, #0x18
  m.c.regs[13]=m.c.regs[13].wrapping_sub(24u32);
  // 20c83e vmov d0, r3, r3
  m.c.extregs[0]=m.c.regs[3];m.c.extregs[1]=m.c.regs[3];
  // 20c842 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20c846 vmov d0, r1, r1
  m.c.extregs[0]=m.c.regs[1];m.c.extregs[1]=m.c.regs[1];
  // 20c84a vmov.f32 s8, s0
  m.c.extregs[8]=m.c.extregs[0];
  // 20c84e vmov d0, r0, r0
  m.c.extregs[0]=m.c.regs[0];m.c.extregs[1]=m.c.regs[0];
  // 20c852 vmov.f32 s12, s0
  m.c.extregs[12]=m.c.extregs[0];
  // 20c856 vldr s2, [sp, #24]
  m.c.extregs[2]=m.load32(m.c.regs[13].wrapping_add(24u32))?;
  // 20c85a movw r0, #0x783a
  m.c.regs[0]=30778u32;
  // 20c85e movt r0, #0x28
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(40u32<<16);
  // 20c862 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c866u32);
  // 20c864 vstr s12, [sp, #20]
  m.store32(m.c.regs[13].wrapping_add(20u32),m.c.extregs[12])?;
  // 20c868 vstr s8, [sp, #16]
  m.store32(m.c.regs[13].wrapping_add(16u32),m.c.extregs[8])?;
  // 20c86c str r2, [sp, #0xc]
  m.store32(m.c.regs[13].wrapping_add(12u32),m.c.regs[2])?;
  // 20c86e vstr s4, [sp, #8]
  m.store32(m.c.regs[13].wrapping_add(8u32),m.c.extregs[4])?;
  // 20c872 vstr s2, [sp, #4]
  m.store32(m.c.regs[13].wrapping_add(4u32),m.c.extregs[2])?;
  // 20c876 ldr r0, [r0]
  m.c.regs[0]=m.load32(m.c.regs[0].wrapping_add(0u32))?;
  // 20c878 cmp r0, #0x0
  m.sub(m.c.regs[0],0u32);
  // 20c87a ble 0x20c8a4 <__ZN4Sexy9AddVertexEffjff+0x68>
  if m.c.cpsr&0x40000000!=0||((m.c.cpsr>>31)^((m.c.cpsr>>28)&1))!=0{pc=0x20c8a4;continue;}
  pc=0x20c87c;continue;
 },
 0x20c87c=>{
  // 20c87c movw r0, #0xf994
  m.c.regs[0]=63892u32;
  // 20c880 movt r0, #0x27
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(39u32<<16);
  // 20c884 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c888u32);
  // 20c886 movw r1, #0x780e
  m.c.regs[1]=30734u32;
  // 20c88a movt r1, #0x28
  m.c.regs[1]=(m.c.regs[1]&0xffff)|(40u32<<16);
  // 20c88e add r1, pc
  m.c.regs[1]=m.c.regs[1].wrapping_add(0x20c892u32);
  // 20c890 ldr r2, [sp, #0xc]
  m.c.regs[2]=m.load32(m.c.regs[13].wrapping_add(12u32))?;
  // 20c892 ldr r3, [r1]
  m.c.regs[3]=m.load32(m.c.regs[1].wrapping_add(0u32))?;
  // 20c894 add.w r9, r3, #0x1
  m.c.regs[9]=m.c.regs[3].wrapping_add(1u32);
  // 20c898 str.w r9, [r1]
  m.store32(m.c.regs[1].wrapping_add(0u32),m.c.regs[9])?;
  // 20c89c lsls r1, r3, #0x2
  m.c.regs[1]=m.lsl(m.c.regs[3],2u32);
  // 20c89e add r0, r1
  m.c.regs[0]=m.c.regs[0].wrapping_add(m.c.regs[1]);
  // 20c8a0 str r2, [r0]
  m.store32(m.c.regs[0].wrapping_add(0u32),m.c.regs[2])?;
  // 20c8a2 b 0x20c94c <__ZN4Sexy9AddVertexEffjff+0x110>
  pc=0x20c94c;continue;
 },
 0x20c8a4=>{
  // 20c8a4 movw r0, #0xcc14
  m.c.regs[0]=52244u32;
  // 20c8a8 movt r0, #0x20
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(32u32<<16);
  // 20c8ac add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c8b0u32);
  // 20c8ae ldr r1, [sp, #0xc]
  m.c.regs[1]=m.load32(m.c.regs[13].wrapping_add(12u32))?;
  // 20c8b0 ldr r0, [r0]
  m.c.regs[0]=m.load32(m.c.regs[0].wrapping_add(0u32))?;
  // 20c8b2 cmp r1, r0
  m.sub(m.c.regs[1],m.c.regs[0]);
  // 20c8b4 beq 0x20c94a <__ZN4Sexy9AddVertexEffjff+0x10e>
  if m.c.cpsr&0x40000000!=0{pc=0x20c94a;continue;}
  pc=0x20c8b6;continue;
 },
 0x20c8b6=>{
  // 20c8b6 movw r0, #0x77da
  m.c.regs[0]=30682u32;
  // 20c8ba movt r0, #0x28
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(40u32<<16);
  // 20c8be add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c8c2u32);
  // 20c8c0 ldr r0, [r0]
  m.c.regs[0]=m.load32(m.c.regs[0].wrapping_add(0u32))?;
  // 20c8c2 cmp r0, #0x0
  m.sub(m.c.regs[0],0u32);
  // 20c8c4 ble 0x20c93a <__ZN4Sexy9AddVertexEffjff+0xfe>
  if m.c.cpsr&0x40000000!=0||((m.c.cpsr>>31)^((m.c.cpsr>>28)&1))!=0{pc=0x20c93a;continue;}
  pc=0x20c8c6;continue;
 },
 0x20c8c6=>{
  // 20c8c6 movs r0, #0x0
  m.c.regs[0]=0u32;
  m.nz(m.c.regs[0]);
  // 20c8c8 movt r0, #0x0
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(0u32<<16);
  // 20c8cc str r0, [sp]
  m.store32(m.c.regs[13].wrapping_add(0u32),m.c.regs[0])?;
  pc=0x20c8ce;continue;
 },
 0x20c8ce=>{
  // 20c8ce movw r0, #0x77c2
  m.c.regs[0]=30658u32;
  // 20c8d2 movt r0, #0x28
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(40u32<<16);
  // 20c8d6 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c8dau32);
  // 20c8d8 ldr r1, [sp]
  m.c.regs[1]=m.load32(m.c.regs[13].wrapping_add(0u32))?;
  // 20c8da ldr r0, [r0]
  m.c.regs[0]=m.load32(m.c.regs[0].wrapping_add(0u32))?;
  // 20c8dc cmp r1, r0
  m.sub(m.c.regs[1],m.c.regs[0]);
  // 20c8de bge 0x20c906 <__ZN4Sexy9AddVertexEffjff+0xca>
  if ((m.c.cpsr>>31)^((m.c.cpsr>>28)&1))==0{pc=0x20c906;continue;}
  pc=0x20c8e0;continue;
 },
 0x20c8e0=>{
  // 20c8e0 movw r0, #0xf930
  m.c.regs[0]=63792u32;
  // 20c8e4 movt r0, #0x27
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(39u32<<16);
  // 20c8e8 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c8ecu32);
  // 20c8ea movw r1, #0xcbce
  m.c.regs[1]=52174u32;
  // 20c8ee movt r1, #0x20
  m.c.regs[1]=(m.c.regs[1]&0xffff)|(32u32<<16);
  // 20c8f2 add r1, pc
  m.c.regs[1]=m.c.regs[1].wrapping_add(0x20c8f6u32);
  // 20c8f4 ldr r1, [r1]
  m.c.regs[1]=m.load32(m.c.regs[1].wrapping_add(0u32))?;
  // 20c8f6 ldr r2, [sp]
  m.c.regs[2]=m.load32(m.c.regs[13].wrapping_add(0u32))?;
  // 20c8f8 lsls r2, r2, #0x2
  m.c.regs[2]=m.lsl(m.c.regs[2],2u32);
  // 20c8fa add r0, r2
  m.c.regs[0]=m.c.regs[0].wrapping_add(m.c.regs[2]);
  // 20c8fc str r1, [r0]
  m.store32(m.c.regs[0].wrapping_add(0u32),m.c.regs[1])?;
  // 20c8fe ldr r0, [sp]
  m.c.regs[0]=m.load32(m.c.regs[13].wrapping_add(0u32))?;
  // 20c900 adds r0, #0x1
  m.c.regs[0]=m.add(m.c.regs[0],1u32);
  // 20c902 str r0, [sp]
  m.store32(m.c.regs[13].wrapping_add(0u32),m.c.regs[0])?;
  // 20c904 b 0x20c8ce <__ZN4Sexy9AddVertexEffjff+0x92>
  pc=0x20c8ce;continue;
 },
 0x20c906=>{
  // 20c906 movw r0, #0x778e
  m.c.regs[0]=30606u32;
  // 20c90a movt r0, #0x28
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(40u32<<16);
  // 20c90e add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c912u32);
  // 20c910 movw r1, #0x7780
  m.c.regs[1]=30592u32;
  // 20c914 movt r1, #0x28
  m.c.regs[1]=(m.c.regs[1]&0xffff)|(40u32<<16);
  // 20c918 add r1, pc
  m.c.regs[1]=m.c.regs[1].wrapping_add(0x20c91cu32);
  // 20c91a movw r2, #0xf8f6
  m.c.regs[2]=63734u32;
  // 20c91e movt r2, #0x27
  m.c.regs[2]=(m.c.regs[2]&0xffff)|(39u32<<16);
  // 20c922 add r2, pc
  m.c.regs[2]=m.c.regs[2].wrapping_add(0x20c926u32);
  // 20c924 ldr r3, [sp, #0xc]
  m.c.regs[3]=m.load32(m.c.regs[13].wrapping_add(12u32))?;
  // 20c926 ldr.w r9, [r1]
  m.c.regs[9]=m.load32(m.c.regs[1].wrapping_add(0u32))?;
  // 20c92a lsl.w r9, r9, #0x2
  m.c.regs[9]=m.c.regs[9].wrapping_shl(2u32);
  // 20c92e add r2, r9
  m.c.regs[2]=m.c.regs[2].wrapping_add(m.c.regs[9]);
  // 20c930 str r3, [r2]
  m.store32(m.c.regs[2].wrapping_add(0u32),m.c.regs[3])?;
  // 20c932 ldr r1, [r1]
  m.c.regs[1]=m.load32(m.c.regs[1].wrapping_add(0u32))?;
  // 20c934 adds r1, #0x1
  m.c.regs[1]=m.add(m.c.regs[1],1u32);
  // 20c936 str r1, [r0]
  m.store32(m.c.regs[0].wrapping_add(0u32),m.c.regs[1])?;
  // 20c938 b 0x20c948 <__ZN4Sexy9AddVertexEffjff+0x10c>
  pc=0x20c948;continue;
 },
 0x20c93a=>{
  // 20c93a movw r0, #0xcb7e
  m.c.regs[0]=52094u32;
  // 20c93e movt r0, #0x20
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(32u32<<16);
  // 20c942 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c946u32);
  // 20c944 ldr r1, [sp, #0xc]
  m.c.regs[1]=m.load32(m.c.regs[13].wrapping_add(12u32))?;
  // 20c946 str r1, [r0]
  m.store32(m.c.regs[0].wrapping_add(0u32),m.c.regs[1])?;
  pc=0x20c948;continue;
 },
 0x20c948=>{
  // 20c948 b 0x20c94a <__ZN4Sexy9AddVertexEffjff+0x10e>
  pc=0x20c94a;continue;
 },
 0x20c94a=>{
  // 20c94a b 0x20c94c <__ZN4Sexy9AddVertexEffjff+0x110>
  pc=0x20c94c;continue;
 },
 0x20c94c=>{
  // 20c94c movw r0, #0xcb70
  m.c.regs[0]=52080u32;
  // 20c950 movt r0, #0x20
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(32u32<<16);
  // 20c954 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20c958u32);
  // 20c956 movw r1, #0xfbba
  m.c.regs[1]=64442u32;
  // 20c95a movt r1, #0x26
  m.c.regs[1]=(m.c.regs[1]&0xffff)|(38u32<<16);
  // 20c95e add r1, pc
  m.c.regs[1]=m.c.regs[1].wrapping_add(0x20c962u32);
  // 20c960 movw r2, #0x7730
  m.c.regs[2]=30512u32;
  // 20c964 movt r2, #0x28
  m.c.regs[2]=(m.c.regs[2]&0xffff)|(40u32<<16);
  // 20c968 add r2, pc
  m.c.regs[2]=m.c.regs[2].wrapping_add(0x20c96cu32);
  // 20c96a vldr s0, [sp, #20]
  m.c.extregs[0]=m.load32(m.c.regs[13].wrapping_add(20u32))?;
  // 20c96e vmov.f64 d1, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[2]=pair[0];m.c.extregs[3]=pair[1];
  // 20c972 vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c976 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20c97a vmov.f32 d16, #8.000000e+00
  m.c.extregs[32]=0x41000000;m.c.extregs[33]=0x41000000;
  // 20c97e vmov.f64 d3, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[6]=pair[0];m.c.extregs[7]=pair[1];
  // 20c982 vmov.f32 s0, s6
  m.c.extregs[0]=m.c.extregs[6];
  // 20c986 vmov.f64 d3, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[6]=pair[0];m.c.extregs[7]=pair[1];
  // 20c98a vmov.f32 s6, s0
  m.c.extregs[6]=m.c.extregs[0];
  // 20c98e vmul.f32 d16, d2, d3
  let pair=[crate::neon_scalar_ops::mul_f32(m.c.extregs[4],m.c.extregs[6]),crate::neon_scalar_ops::mul_f32(m.c.extregs[5],m.c.extregs[7])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20c992 vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c996 vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20c99a vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c99e vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20c9a2 vcvt.s32.f32 d16, d2
  let pair=[crate::neon_scalar_ops::signed_from_f32(m.c.extregs[4]),crate::neon_scalar_ops::signed_from_f32(m.c.extregs[5])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20c9a6 vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c9aa vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20c9ae vmov r3, s0
  m.c.regs[3]=m.c.extregs[0];
  // 20c9b2 movw r9, #0x76de
  m.c.regs[9]=30430u32;
  // 20c9b6 movt r9, #0x28
  m.c.regs[9]=(m.c.regs[9]&0xffff)|(40u32<<16);
  // 20c9ba add r9, pc
  m.c.regs[9]=m.c.regs[9].wrapping_add(0x20c9beu32);
  // 20c9bc ldr.w r9, [r9]
  m.c.regs[9]=m.load32(m.c.regs[9].wrapping_add(0u32))?;
  // 20c9c0 movw r12, #0xfb50
  m.c.regs[12]=64336u32;
  // 20c9c4 movt r12, #0x26
  m.c.regs[12]=(m.c.regs[12]&0xffff)|(38u32<<16);
  // 20c9c8 add r12, pc
  m.c.regs[12]=m.c.regs[12].wrapping_add(0x20c9ccu32);
  // 20c9ca strh.w r3, [r12, r9, lsl #3]
  m.store16(m.c.regs[12].wrapping_add(m.c.regs[9].wrapping_shl(3)),m.c.regs[3] as u16)?;
  // 20c9ce vldr s0, [sp, #16]
  m.c.extregs[0]=m.load32(m.c.regs[13].wrapping_add(16u32))?;
  // 20c9d2 vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c9d6 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20c9da vmul.f32 d16, d2, d3
  let pair=[crate::neon_scalar_ops::mul_f32(m.c.extregs[4],m.c.extregs[6]),crate::neon_scalar_ops::mul_f32(m.c.extregs[5],m.c.extregs[7])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20c9de vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20c9e2 vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20c9e6 vmov.f32 s2, s0
  m.c.extregs[2]=m.c.extregs[0];
  // 20c9ea vcvt.s32.f32 d16, d1
  let pair=[crate::neon_scalar_ops::signed_from_f32(m.c.extregs[2]),crate::neon_scalar_ops::signed_from_f32(m.c.extregs[3])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20c9ee vmov.f64 d1, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[2]=pair[0];m.c.extregs[3]=pair[1];
  // 20c9f2 vmov.f32 s0, s2
  m.c.extregs[0]=m.c.extregs[2];
  // 20c9f6 vmov r3, s0
  m.c.regs[3]=m.c.extregs[0];
  // 20c9fa ldr r2, [r2]
  m.c.regs[2]=m.load32(m.c.regs[2].wrapping_add(0u32))?;
  // 20c9fc lsls r2, r2, #0x3
  m.c.regs[2]=m.lsl(m.c.regs[2],3u32);
  // 20c9fe add r1, r2
  m.c.regs[1]=m.c.regs[1].wrapping_add(m.c.regs[2]);
  // 20ca00 strh r3, [r1, #0x2]
  m.store16(m.c.regs[1].wrapping_add(2u32),m.c.regs[3] as u16)?;
  // 20ca02 ldrb r0, [r0]
  m.c.regs[0]=u32::from(m.load8(m.c.regs[0].wrapping_add(0u32))?);
  // 20ca04 tst.w r0, #0x1
  m.nz(m.c.regs[0]&1u32);
  // 20ca08 bne 0x20caf2 <__ZN4Sexy9AddVertexEffjff+0x2b6>
  if m.c.cpsr&0x40000000==0{pc=0x20caf2;continue;}
  pc=0x20ca0a;continue;
 },
 0x20ca0a=>{
  // 20ca0a movw r0, #0xfb06
  m.c.regs[0]=64262u32;
  // 20ca0e movt r0, #0x26
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(38u32<<16);
  // 20ca12 add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20ca16u32);
  // 20ca14 movw r1, #0x767c
  m.c.regs[1]=30332u32;
  // 20ca18 movt r1, #0x28
  m.c.regs[1]=(m.c.regs[1]&0xffff)|(40u32<<16);
  // 20ca1c add r1, pc
  m.c.regs[1]=m.c.regs[1].wrapping_add(0x20ca20u32);
  // 20ca1e vldr s0, [sp, #8]
  m.c.extregs[0]=m.load32(m.c.regs[13].wrapping_add(8u32))?;
  // 20ca22 vmov.f64 d1, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[2]=pair[0];m.c.extregs[3]=pair[1];
  // 20ca26 vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca2a vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20ca2e vmov.i32 d16, #0x45000000
  m.c.extregs[32]=0x45000000;m.c.extregs[33]=0x45000000;
  // 20ca32 vmov.f64 d3, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[6]=pair[0];m.c.extregs[7]=pair[1];
  // 20ca36 vmov.f32 s0, s6
  m.c.extregs[0]=m.c.extregs[6];
  // 20ca3a vmov.f64 d3, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[6]=pair[0];m.c.extregs[7]=pair[1];
  // 20ca3e vmov.f32 s6, s0
  m.c.extregs[6]=m.c.extregs[0];
  // 20ca42 vmul.f32 d16, d2, d3
  let pair=[crate::neon_scalar_ops::mul_f32(m.c.extregs[4],m.c.extregs[6]),crate::neon_scalar_ops::mul_f32(m.c.extregs[5],m.c.extregs[7])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20ca46 vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca4a vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20ca4e vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca52 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20ca56 vmov.i32 d16, #0x47000000
  m.c.extregs[32]=0x47000000;m.c.extregs[33]=0x47000000;
  // 20ca5a vmov.f64 d4, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[8]=pair[0];m.c.extregs[9]=pair[1];
  // 20ca5e vmov.f32 s0, s8
  m.c.extregs[0]=m.c.extregs[8];
  // 20ca62 vmov.f64 d4, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[8]=pair[0];m.c.extregs[9]=pair[1];
  // 20ca66 vmov.f32 s8, s0
  m.c.extregs[8]=m.c.extregs[0];
  // 20ca6a vsub.f32 d16, d2, d4
  let pair=[crate::neon_scalar_ops::sub_f32(m.c.extregs[4],m.c.extregs[8]),crate::neon_scalar_ops::sub_f32(m.c.extregs[5],m.c.extregs[9])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20ca6e vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca72 vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20ca76 vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca7a vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20ca7e vcvt.s32.f32 d16, d2
  let pair=[crate::neon_scalar_ops::signed_from_f32(m.c.extregs[4]),crate::neon_scalar_ops::signed_from_f32(m.c.extregs[5])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20ca82 vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20ca86 vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20ca8a vmov r2, s0
  m.c.regs[2]=m.c.extregs[0];
  // 20ca8e movw r3, #0x7602
  m.c.regs[3]=30210u32;
  // 20ca92 movt r3, #0x28
  m.c.regs[3]=(m.c.regs[3]&0xffff)|(40u32<<16);
  // 20ca96 add r3, pc
  m.c.regs[3]=m.c.regs[3].wrapping_add(0x20ca9au32);
  // 20ca98 ldr r3, [r3]
  m.c.regs[3]=m.load32(m.c.regs[3].wrapping_add(0u32))?;
  // 20ca9a movw r9, #0xfa76
  m.c.regs[9]=64118u32;
  // 20ca9e movt r9, #0x26
  m.c.regs[9]=(m.c.regs[9]&0xffff)|(38u32<<16);
  // 20caa2 add r9, pc
  m.c.regs[9]=m.c.regs[9].wrapping_add(0x20caa6u32);
  // 20caa4 add.w r3, r9, r3, lsl #3
  m.c.regs[3]=m.c.regs[9].wrapping_add(m.c.regs[3].wrapping_shl(3));
  // 20caa8 strh r2, [r3, #0x4]
  m.store16(m.c.regs[3].wrapping_add(4u32),m.c.regs[2] as u16)?;
  // 20caaa vldr s0, [sp, #4]
  m.c.extregs[0]=m.load32(m.c.regs[13].wrapping_add(4u32))?;
  // 20caae vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20cab2 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20cab6 vmul.f32 d16, d2, d3
  let pair=[crate::neon_scalar_ops::mul_f32(m.c.extregs[4],m.c.extregs[6]),crate::neon_scalar_ops::mul_f32(m.c.extregs[5],m.c.extregs[7])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20caba vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20cabe vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20cac2 vmov.f64 d2, d1
  let pair=[m.c.extregs[2],m.c.extregs[3]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20cac6 vmov.f32 s4, s0
  m.c.extregs[4]=m.c.extregs[0];
  // 20caca vsub.f32 d16, d2, d4
  let pair=[crate::neon_scalar_ops::sub_f32(m.c.extregs[4],m.c.extregs[8]),crate::neon_scalar_ops::sub_f32(m.c.extregs[5],m.c.extregs[9])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20cace vmov.f64 d2, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[4]=pair[0];m.c.extregs[5]=pair[1];
  // 20cad2 vmov.f32 s0, s4
  m.c.extregs[0]=m.c.extregs[4];
  // 20cad6 vmov.f32 s2, s0
  m.c.extregs[2]=m.c.extregs[0];
  // 20cada vcvt.s32.f32 d16, d1
  let pair=[crate::neon_scalar_ops::signed_from_f32(m.c.extregs[2]),crate::neon_scalar_ops::signed_from_f32(m.c.extregs[3])];m.c.extregs[32]=pair[0];m.c.extregs[33]=pair[1];
  // 20cade vmov.f64 d1, d16
  let pair=[m.c.extregs[32],m.c.extregs[33]];m.c.extregs[2]=pair[0];m.c.extregs[3]=pair[1];
  // 20cae2 vmov.f32 s0, s2
  m.c.extregs[0]=m.c.extregs[2];
  // 20cae6 vmov r2, s0
  m.c.regs[2]=m.c.extregs[0];
  // 20caea ldr r1, [r1]
  m.c.regs[1]=m.load32(m.c.regs[1].wrapping_add(0u32))?;
  // 20caec lsls r1, r1, #0x3
  m.c.regs[1]=m.lsl(m.c.regs[1],3u32);
  // 20caee add r0, r1
  m.c.regs[0]=m.c.regs[0].wrapping_add(m.c.regs[1]);
  // 20caf0 strh r2, [r0, #0x6]
  m.store16(m.c.regs[0].wrapping_add(6u32),m.c.regs[2] as u16)?;
  pc=0x20caf2;continue;
 },
 0x20caf2=>{
  // 20caf2 movw r0, #0x759e
  m.c.regs[0]=30110u32;
  // 20caf6 movt r0, #0x28
  m.c.regs[0]=(m.c.regs[0]&0xffff)|(40u32<<16);
  // 20cafa add r0, pc
  m.c.regs[0]=m.c.regs[0].wrapping_add(0x20cafeu32);
  // 20cafc ldr r1, [r0]
  m.c.regs[1]=m.load32(m.c.regs[0].wrapping_add(0u32))?;
  // 20cafe adds r1, #0x1
  m.c.regs[1]=m.add(m.c.regs[1],1u32);
  // 20cb00 str r1, [r0]
  m.store32(m.c.regs[0].wrapping_add(0u32),m.c.regs[1])?;
  // 20cb02 add sp, #0x18
  m.c.regs[13]=m.c.regs[13].wrapping_add(24u32);
  // 20cb04 bx lr
  return Some(Outcome{context:m.c,stack:m.stack,regions:m.regions,class,blocks});
 },
 _=>return None,
 }}None
}
