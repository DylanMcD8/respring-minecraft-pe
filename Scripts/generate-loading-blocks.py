#!/usr/bin/env python3
"""Ahead-of-time translations of profiled Minecraft loading islands.
Local preserved ARMv7 code only. Calls leave the island; no runtime compiler.
"""
from pathlib import Path
import sys
import re, struct
root=Path(__file__).resolve().parent.parent
ranges=[(0x4fb30,0x4fb92),(0x4fe0a,0x4fe3e),(0x194734,0x194774),(0x194774,0x1947b8),(0x1947de,0x194896),(0x48348,0x483aa),(0x48462,0x4849e),(0x10ceb0,0x10cf52),(0x6fc54,0x6fc92)]
ranges += [(0x163b84,0x163c58),(0x164ca2,0x164cf8),(0x1bd38,0x1bd96),(0xbf2e0,0xbf310),(0x163b0e,0x163b84),(0x164c70,0x164ca2)]
ranges += [(0x10cf52,0x10cfaa),(0x10cfaa,0x10cffc),(0x10d230,0x10d268),(0x4e752,0x4e76a),(0x25b04,0x25b32),(0x25b6a,0x25b8e)]
ranges += [(0x10d034,0x10d0d2),(0x10d07e,0x10d0d2),(0x10d0d2,0x10d152),(0x10d0e8,0x10d152),(0x10d152,0x10d200),(0x10d166,0x10d200),(0x10d200,0x10d26c),(0x10d26c,0x10d2b4)]
# Streaming pass: block update/cache prefixes, biome tint, lighting and ore tails.
ranges += [(0x6fba4,0x6fbec),(0x6fbec,0x6fc50),(0x6fc96,0x6fcc4),(0x6fce6,0x6fd74),
           (0xd6bfe,0xd6c36),(0xd6c36,0xd6c92),(0xd6bca,0xd6bfe),(0xd6c92,0xd6cc8),
           (0xd9c3e,0xd9cba),(0xd9cc2,0xd9cec),(0x19488c,0x1948ac),
           (0x4852a,0x48550),(0x48550,0x48576),(0x48576,0x485a8)]
ranges += [(0xbf3a2,0xbf3f2),(0xbf3f2,0xbf450),(0xbf36e,0xbf398),(0x99c6c,0x99c84),(0x99c84,0x99c9e)]
# Infinite frontier: cave material/bounds scan and camera frustum plane tests.
ranges += [(0x1c836c,0x1c838a),(0x1c838a,0x1c8412),(0x1c8412,0x1c842a),(0x1b7cb8,0x1b7cee),(0x1b7cfa,0x1b7dc0)]
# Mesh neighbor bounds, ready-neighbor iteration and cached render-cell lookup.
ranges += [(0xfb96a,0xfb97e),(0xfb97e,0xfba4e),(0xfba4e,0xfbaaa),
           (0x99634,0x99682),(0x99682,0x996a0),(0x91a80,0x91a94),
           (0x91a94,0x91aac),(0x91aac,0x91ad0),(0xfbd08,0xfbd18)]
# The three texture stencils and the camera plane loop can complete several
# rows/planes in one dispatch. Reserve one full straight-line path before
# following a back edge so a malformed counter cannot exceed the guest budget.
loop_budgets = {0x12d000: 256, 0x12d384: 256, 0x12d712: 256, 0x1b7cee: 480}
ranges += [(0x12d000, 0x12d030), (0x12d384, 0x12d3b8),
           (0x12d712, 0x12d746), (0x1b7cee, 0x1b7dc8)]
# Face emission and material queries repeatedly hit the same cached chunk.
ranges += [(0xad936,0xad99a), (0xad99a,0xada0a), (0x111276,0x111376),
           (0x1112c8,0x111376), (0xa6bda,0xa6bfe), (0xa6bfe,0xa6c34),
           (0x10288e,0x10296e), (0x1028c0,0x10296e), (0x1029c6,0x102a40),
           (0x102a40,0x102a76), (0x102a76,0x102aec)]
ranges += [(0x1eb864, 0x1eb908)]  # LevelDB checksum, 16 bytes per iteration
# Profiled six-neighbor visibility flood, including queue append and the six
# original call sites. Calls still leave for the original grow operation.
ranges += [(0x39884,0x3999a),(0x39946,0x3999a),
           (0x24884,0x2493e),(0x248c2,0x2493e),
           (0xe4764,0xe47be),(0xe47be,0xe47f8),(0xe47f8,0xe482e),
           (0xe482e,0xe4868),(0xe4868,0xe48a0),(0xe48a0,0xe48e6),
           (0x194896,0x1948c6),(0x995e4,0x99608)]
# Original tessellator: pack the position, texture coordinates, color and normal.
ranges += [(0x39b54, 0x39bba)]
# Block-face visibility: visit setup, material transparency lookup and return.
ranges += [(0xa6bd0, 0xa6bfe), (0xa6c6e, 0xa6ca2), (0xa6d06, 0xa6d10)]
# Face material classification and appending a terrain traversal record.
ranges += [(0xadb4a,0xadb9a),(0xfbb8c,0xfbbec)]
# The rest of Tessellator::vertex around that tail: entry and offset updates
# (also resumed after the vector append call), the untransformed exit, and
# the in-place vector<uint8_t> append around the existing zero-fill kernel.
ranges += [(0x3999c,0x39b1c),(0x39a68,0x39b1c),(0x39bba,0x39bcc),
           (0x1a37d0,0x1a3828),(0x1a383c,0x1a383e),(0x1a389c,0x1a38a2)]
lines=[]
for line in (Path(sys.argv[1])/'disassembly-armv7.txt').read_text().splitlines():
 m=re.match(r'^([0-9a-f]{8})\s+[0-9a-f]+\s+(\S+)\s*(.*)',line)
 if m:lines.append((int(m[1],16),m[2],m[3]))
reg={f'r{i}':i for i in range(16)}|{'sp':13,'lr':14,'pc':15}
def val(s,pc):
 s=s.strip()
 if s.startswith('#'):return f'{int(s[1:],0)&0xffffffff}u32'
 if s=='pc':return f'{pc+4}u32'
 if s.startswith('s') and s[1:].isdigit():return f'm.c.extregs[{int(s[1:])}]'
 return f'm.c.regs[{reg[s]}]'
def dest(s):return val(s,0)
def shifted(v,s):
 kind,n=s.split();n=int(n[1:])
 return f'(({v} as i32)>>{n}) as u32' if kind=='asr' else f'{v}.wrapping_'+('shl' if kind=='lsl' else 'shr')+f'({n})' 
def address(s,pc):
 a=[x.strip() for x in s.strip()[1:-1].split(',')];v=val(a[0],pc)
 if len(a)>1:
  offset=val(a[1],pc)
  if len(a)>2:offset=shifted(offset,a[2])
  v=f'{v}.wrapping_add({offset})'
 return v
def jump(op):return op.split('.')[0] in ['b','beq','bne','bge','bgt','bhi','bhs','blo','ble','bls','blt','bl','blx','cbz','cbnz']
def target(op,args):return int(args.split(',')[-1].strip(),0) if not(op=='blx') else None
conditions=['eq','ne','hi','lo','lt','ge','gt','le','hs','ls']
def translate(pc,op,args,nextpc,loop_guard=None):
 a=[x.strip() for x in args.split(',')];base=op.replace('.w','');condition=None
 if base.startswith('it'):return ''
 if base in ('bxge','bxle'):
  assert args=='lr' and pc in (0x1b7cf8,0x1b7dc4)  # final slot of ITT
  return (f'if m.cond("{base[2:]}") {{ let target=m.c.regs[14]; '
          'm.c.regs[15]=target & !1; m.c.cpsr=(m.c.cpsr & !0x0600fc20) | ((target&1)<<5); return Some(()); }')
 if jump(op):
  if base=='blx' and args.startswith('0x'):
   t=int(args.split()[0],0)
   return f'm.c.regs[14]={nextpc|1}; m.c.regs[15]={t&~3}; m.c.cpsr&=!32; return Some(());'
  if base=='blx':return f'let target={val(args,pc)}; if target&1==0 {{return None;}} m.c.regs[14]={nextpc|1}; m.c.regs[15]=target&!1; return Some(());'
  t=target(op,args)
  if base=='bl':return f'm.c.regs[14]={nextpc|1}; m.c.regs[15]={t}; return Some(());'
  # End at backward edges. One dispatch never executes an unbounded loop;
  # metadata's instruction count therefore bounds every path and guest budget.
  code=f'm.c.regs[15]={t}; '+('return Some(());' if t<=pc else 'continue;')
  if t<=pc and loop_guard and t>=loop_guard[0]:
   code=(f'm.c.regs[15]={t}; if m.retired-start_retired+{loop_guard[1]} <= {loop_guard[2]} {{ continue; }} return Some(());')
  if base=='b':return code
  c=f'{val(a[0],pc)}'+('==0' if base=='cbz' else '!=0') if base in ['cbz','cbnz'] else f'm.cond("{base[1:]}")'
  return f'if {c} {{{code}}}'
 for c in conditions:
  if base in [f'{o}{c}' for o in ['mov','movw','mvn','add','sub','cmp','ldr','ldrb','str','strh','vldr','asr','mul','strb','ldrh']] or base in (f'vmov{c}.f32',f'vadd{c}.f32'):
   condition=c;base=base.replace(c+'.f32','.f32') if base.endswith('.f32') else base[:-2];break
 d=dest(a[0]) if a[0] in reg or re.fullmatch('s[0-9]+',a[0]) else ''
 if base in ['add','adds','addw','sub','subs']:
  b,c=(a[0],a[1]) if len(a)==2 else (a[1],a[2]);v=val(c,pc)
  if len(a)==4:v=shifted(v,a[3])
  oper='add' if base.startswith('add') else 'sub'
  code=f'let a={val(b,pc)}; let b={v}; '
  if base.endswith('s'):code+=f'm.{oper}_flags(a,b); '
  code+=f'{d}=a.wrapping_{oper}(b);'
 elif base in ['mov','movs','movw','movt']:
  v=val(a[1],pc)
  code=f'{d}='+ (f'({d}&0xffff)|({v}<<16);' if base=='movt' else f'{v};')
  if base=='movs':code+=f'm.nz({d});'
 elif base in ['and','orr','orrs','eor','eors']:
  b,c=(a[0],a[1]) if len(a)==2 else (a[1],a[2]);v=val(c,pc)
  if len(a)==4:v=shifted(v,a[3])
  code=f'{d}={val(b,pc)}'+({'and':'&','eor':'^','eors':'^'}.get(base,'|'))+v+';'
  if base.endswith('s'):code+=f'm.nz({d});'
 elif base in ['adc','adcs']:
  b,c=(a[0],a[1]) if len(a)==2 else (a[1],a[2]);v=val(c,pc)
  if len(a)==4:v=shifted(v,a[3])
  code=f'let a={val(b,pc)};let b={v};let carry=(m.c.cpsr>>29)&1;let sum=u64::from(a)+u64::from(b)+u64::from(carry);let result=sum as u32;'
  if base=='adcs':code+='let signed=i64::from(a as i32)+i64::from(b as i32)+i64::from(carry);m.c.cpsr=(m.c.cpsr&0x0fffffff)|(result&0x80000000)|(u32::from(result==0)<<30)|(u32::from(sum>>32!=0)<<29)|(u32::from(signed>i32::MAX as i64||signed<i32::MIN as i64)<<28);'
  code+=f'{d}=result;'
 elif base=='umull':code=f'let product=u64::from({val(a[2],pc)})*u64::from({val(a[3],pc)});{d}=product as u32;{dest(a[1])}=(product>>32) as u32;'
 elif base=='mls':code=f'{d}={val(a[3],pc)}.wrapping_sub({val(a[1],pc)}.wrapping_mul({val(a[2],pc)}));'
 elif base=='mla':code=f'{d}={val(a[1],pc)}.wrapping_mul({val(a[2],pc)}).wrapping_add({val(a[3],pc)});'
 elif base=='mul':code=f'{d}={val(a[1],pc)}.wrapping_mul({val(a[2],pc)});'
 elif base=='muls':code=f'{d}={val(a[1],pc)}.wrapping_mul({val(a[2],pc)});m.nz({d});'
 elif base=='bic':
  v=val(a[2],pc)
  if len(a)==4:v=f'({shifted(v,a[3])})'
  code=f'{d}={val(a[1],pc)}&!{v};'
 elif base=='cmp':
  rhs=val(a[1],pc)
  if len(a)==3:rhs=shifted(rhs,a[2])
  code=f'm.cmp({val(a[0],pc)},{rhs});'
 elif base=='cmn':code=f'm.add_flags({val(a[0],pc)},{val(a[1],pc)});'
 elif base=='tst':code=f'm.nz({val(a[0],pc)}&{val(a[1],pc)});'
 elif base in ['asr','asrs','lsl','lsls','lsr','lsrs']:code=f'{d}=m.shift({val(a[1],pc)},{val(a[2],pc)},'+str({'asr':2,'asrs':2,'lsl':0,'lsls':0,'lsr':1,'lsrs':1}[base])+(','+'true' if base.endswith('s') else ',false')+');'
 elif base=='rsb':
  v=val(a[2],pc)
  if len(a)==4:v=shifted(v,a[3])
  code=f'{d}=({v}).wrapping_sub({val(a[1],pc)});'
 elif base=='sxtb':code=f'{d}=({val(a[1],pc)} as i8 as i32) as u32;'
 elif base=='mvn':code=f'{d}=!{val(a[1],pc)};'
 elif base=='ubfx':
  lsb,width=int(a[2][1:],0),int(a[3][1:],0);code=f'{d}=({val(a[1],pc)}>>{lsb})&{(1<<width)-1}u32;'
 elif base=='bfi':
  lsb,width=int(a[2][1:],0),int(a[3][1:],0);mask=((1<<width)-1)<<lsb
  code=f'{d}=({d}&!{mask}u32)|(({val(a[1],pc)}<<{lsb})&{mask}u32);'
 elif base=='uxth':code=f'{d}={val(a[1],pc)}&65535;'
 elif base=='uxtab':code=f'{d}={val(a[1],pc)}.wrapping_add({val(a[2],pc)}&255);'
 elif base=='uxtb':code=f'{d}={val(a[1],pc)}&255;'
 elif base in ('push','pop','ldm'):
  regs=[reg[x.strip()] for x in args.split('{')[1].rstrip('}').split(',')]
  assert sorted(regs)==regs and 13 not in regs
  if base=='push':
   code=f'let addr=m.c.regs[13].wrapping_sub({len(regs)*4}); m.a.push_words(addr,&['+','.join(f'm.c.regs[{r}]' for r in regs)+'])?; m.c.regs[13]=addr;'
  else:
   addr='m.c.regs[13]' if base=='pop' else val(a[0],pc)
   # Stage the complete load first. If a word faults, the interpreter replays
   # this one instruction and reproduces even its partial effects.
   code=f'let addr={addr};'+''.join(f'let w{i}=m.a.read(addr.wrapping_add({4*i}),4)?;' for i in range(len(regs)))
   code+=''.join(f'm.c.regs[{r}]=w{i};' for i,r in enumerate(regs) if r!=15)
   if base=='pop':code+=f'm.c.regs[13]=addr.wrapping_add({len(regs)*4});'
   if 15 in regs:code+=f'let dest=w{regs.index(15)};m.c.cpsr=(m.c.cpsr & !32)|((dest&1)<<5);m.c.regs[15]=dest&!1;return Some(());'
 elif base=='stm':
  regs=[reg[x.strip()] for x in args.split('{')[1].rstrip('}').split(',')]
  assert a[0] in reg and reg[a[0]] not in regs and 15 not in regs and sorted(regs)==regs  # no write-back
  code=f'm.a.push_words({val(a[0],pc)},&['+','.join(f'm.c.regs[{r}]' for r in regs)+'])?;'
 elif base in ('vst1.64','vld1.64'):
  ds=[int(x.strip()[1:]) for x in args.split('}')[0].lstrip('{').split(',')];rn=args.split('[')[1].split(':')[0].rstrip(']')
  assert args.endswith(']') and ds==list(range(ds[0],ds[0]+len(ds)))  # no write-back; alignment is unchecked, as in the interpreter
  words=[2*ds[0]+i for i in range(2*len(ds))]
  if base=='vst1.64':code=f'm.a.push_words({val(rn,pc)},&['+','.join(f'm.c.extregs[{w}]' for w in words)+'])?;'
  else:code=f'let addr={val(rn,pc)};'+''.join(f'let w{i}=m.a.read(addr.wrapping_add({4*i}),4)?;' for i in range(len(words)))+''.join(f'm.c.extregs[{w}]=w{i};' for i,w in enumerate(words))
 elif base=='ldr' and args.endswith('!'):
  f=re.fullmatch(r'(\w+), \[(\w+), (#-?\w+)\]!',args);assert f and f[1]!=f[2] and 'pc' not in (f[1],f[2])
  code=f'let addr={val(f[2],pc)}.wrapping_add({val(f[3],pc)});{dest(f[1])}=m.a.read(addr,4)?;{dest(f[2])}=addr;'
 elif args.endswith('[sp, #-4]!') and base=='str':
  code=f'let addr=m.c.regs[13].wrapping_sub(4);m.a.write(addr,4,{d})?;m.c.regs[13]=addr;'
 elif args.endswith('[sp], #4') and base=='ldr':
  code=f'let addr=m.c.regs[13];{d}=m.a.read(addr,4)?;m.c.regs[13]=addr.wrapping_add(4);'
 elif base in ['ldr','ldrb','ldrh','str','strb','strh','vldr','vstr']:
  n={'ldrb':1,'ldrh':2,'strb':1,'strh':2}.get(base,4);addr=address(args.split(',',1)[1],pc)
  if base=='vldr' and '[pc,' in args:addr=addr.replace(f'{pc+4}u32',f'{(pc+4)&~3}u32')
  code=f'{d}=m.a.read({addr},{n})?;' if base!='vstr' and not base.startswith('str') else f'm.a.write({addr},{n},{val(a[0],pc)})?;'
 elif base=='vmov.f64':
  dst,src=[2*int(x[1:]) for x in a]
  code=f'm.c.extregs[{dst}]=m.c.extregs[{src}]; m.c.extregs[{dst+1}]=m.c.extregs[{src+1}];'
 elif base=='vstmia':
  r=val(a[0].rstrip('!'),pc);f=val(a[1].strip('{} '),pc)
  code=f'm.a.write({r},4,{f})?; {r}={r}.wrapping_add(4);'
 elif base in ['vmov','vmov.f32']:
  v=str(struct.unpack('<I',struct.pack('<f',float(a[1][1:])))[0])+'u32' if a[1].startswith('#') and base=='vmov.f32' else val(a[1],pc)
  code=f'{d}={v};'
 elif base=='vcvt.f32.u32':code=f'{d}=({val(a[1],pc)} as f32).to_bits();'
 elif base=='vcvt.f32.s32':code=f'{d}=({val(a[1],pc)} as i32 as f32).to_bits();'
 elif base=='vcvt.s32.f32':code=f'{d}=f32::from_bits({val(a[1],pc)}) as i32 as u32;'
 elif base in ['vadd.f32','vsub.f32','vmul.f32','vdiv.f32','vmla.f32','vmls.f32']:
  if a[0].startswith('d'):
   assert base in ['vadd.f32','vsub.f32','vmul.f32']
   ds,bs,cs=[int(x[1:])*2 for x in a];oper={'vadd.f32':'+','vsub.f32':'-','vmul.f32':'*'}[base]
   code=''.join(f'let lane{i}=(f32::from_bits(m.c.extregs[{bs+i}]){oper}f32::from_bits(m.c.extregs[{cs+i}])).to_bits();' for i in range(2))+''.join(f'm.c.extregs[{ds+i}]=lane{i};' for i in range(2))
  else:
   b,c=[f'f32::from_bits({val(x,pc)})' for x in a[1:]]
   if base in ['vmla.f32','vmls.f32']:code=f'let product={b}*{c}; {d}=(f32::from_bits({d})'+('+' if base=='vmla.f32' else '-')+'product).to_bits();'
   else:code=f'{d}=({b}'+{'vadd.f32':'+','vsub.f32':'-','vmul.f32':'*','vdiv.f32':'/'}[base]+c+').to_bits();'
 elif base=='vorr':
  ds,bs,cs=[int(x[1:])*2 for x in a];code=''.join(f'let lane{i}=m.c.extregs[{bs+i}]|m.c.extregs[{cs+i}];' for i in range(2))+''.join(f'm.c.extregs[{ds+i}]=lane{i};' for i in range(2))
 elif base=='vcmpe.f32':
  rhs='0u32' if a[1]=='#0' else val(a[1],pc)
  code=f'let flags=match f32::from_bits({val(a[0],pc)}).partial_cmp(&f32::from_bits({rhs})) {{Some(std::cmp::Ordering::Less)=>0x80000000,Some(std::cmp::Ordering::Equal)=>0x60000000,Some(std::cmp::Ordering::Greater)=>0x20000000,None=>0x30000000}}; m.c.fpscr=(m.c.fpscr&0x0fffffff)|flags;'
 elif base=='vmrs':code='m.c.cpsr=(m.c.cpsr&0x0fffffff)|(m.c.fpscr&0xf0000000);'
 elif base=='dmb':code='// Guest interpreter also treats this barrier as a no-op.'
 else:raise ValueError((hex(pc),op,args))
 if condition:code=f'if m.cond("{condition}") {{ {code} }}'
 return code
binary=(Path(sys.argv[1])/'minecraftpe').read_bytes();out=['''/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Generated by Private/Tools/generate-loading-blocks.py. Exact original
//! instruction ordering, scalar FP rounding, flags, writes and guest call exits.
use crate::pipeline_native::{Access, Machine};
// A live multiword store must decline the whole instruction before writing
// anything if the span faults or overlaps the translated code.
pub(super) trait LoadingAccess: Access {
 fn push_words(&mut self,addr:u32,words:&[u32])->Option<()> {
  for (i,&w) in words.iter().enumerate(){self.write(addr.wrapping_add(4*i as u32),4,w)?;}
  Some(())
 }
}
impl LoadingAccess for crate::pipeline_native::Predict<'_> {}
''']
out.append('pub(super) const BLOCKS:&[(u32,u32,u64,&[u8])]=&[')
for lo,hi in ranges:
 ins=[l for l in lines if lo<=l[0]<hi]
 out.append(f'({lo},{hi},{loop_budgets.get(lo,len(ins))},&['+','.join(str(b) for b in binary[lo:hi])+']),')
out.append('];')
for index,(lo,hi) in enumerate(ranges):
 ins=[l for l in lines if lo<=l[0]<hi];leaders={lo}
 for i,(pc,op,args) in enumerate(ins):
  if jump(op):
   t=target(op,args)
   if t is not None and lo<=t<hi:leaders.add(t)
   if i+1<len(ins):leaders.add(ins[i+1][0])
 guard=(lo,len(ins),loop_budgets[lo]) if lo in loop_budgets else None
 start='let start_retired=m.retired;' if guard else ''
 out.append(f'fn block_{index}<A:LoadingAccess>(m:&mut Machine<A>)->Option<()> {{ {start} loop {{match m.c.regs[15] {{')
 it=0
 for i,(pc,op,args) in enumerate(ins):
  if pc in leaders:out.append(f'{pc}=>{{')
  nxt=ins[i+1][0] if i+1<len(ins) else hi
  before_it=it
  if op.startswith('it'):it=binary[pc]
  elif it:it=0 if it&7==0 else (it&0xe0)|((it<<1)&0x1f)
  out.append(f'// {pc:08x}: {op} {args}\nm.c.regs[15]={pc}; m.retired+=1;\n'+translate(pc,op,args,nxt,guard))
  if before_it or it:
   bits=((it>>2)<<10)|((it&3)<<25)
   out.append(f'm.c.cpsr=(m.c.cpsr&!0x0600fc00)|{bits};')
  if nxt in leaders or nxt==hi:
   if op not in ['b','bl','blx'] and not (op=='pop' and 'pc}' in args):out.append(f'm.c.regs[15]={nxt};')
   out.append('},')
 out.append('_=>return Some(()), } } }')
out.append('pub(super) fn entry_index(pc:u32)->Option<usize>{match pc{'+''.join(f'{lo}=>Some({i}),' for i,(lo,hi) in enumerate(ranges))+'_=>None}}')
out.append('pub(super) fn execute<A:LoadingAccess>(index:usize,m:&mut Machine<A>)->Option<()>{match index{'+''.join(f'{i}=>block_{i}(m),' for i in range(len(ranges)))+'_=>None}}')
out.append('''
// After shadow validation, execute directly against guest memory. A failing
// access leaves PC/ITSTATE at that instruction and retires only earlier work.
struct Live<'a>{mem:&'a mut super::Mem,lo:u32,hi:u32}
impl LoadingAccess for Live<'_>{
 fn push_words(&mut self,addr:u32,words:&[u32])->Option<()> {
  let n=(words.len()*4) as u32;let end=addr.checked_add(n)?;
  if addr<self.hi&&end>self.lo{return None;}
  let bytes=self.mem.get_bytes_fallible_mut(super::Ptr::from_bits(addr),n)?;
  for (w,b) in words.iter().zip(bytes.chunks_exact_mut(4)){b.copy_from_slice(&w.to_le_bytes());}Some(())
 }
}
impl Access for Live<'_>{
 fn read(&mut self,a:u32,n:u32)->Option<u32>{
  if a<self.mem.null_segment_size(){return None;}
  let b=self.mem.get_bytes_fallible(super::Ptr::from_bits(a),n)?;let mut v=[0;4];v[..n as usize].copy_from_slice(b);Some(u32::from_le_bytes(v))
 }
 fn write(&mut self,a:u32,n:u32,v:u32)->Option<()>{
  if a<self.mem.null_segment_size()||u64::from(a)<u64::from(self.hi)&&u64::from(self.lo)<u64::from(a)+u64::from(n){return None;}
  self.mem.get_bytes_fallible_mut(super::Ptr::from_bits(a),n)?.copy_from_slice(&v.to_le_bytes()[..n as usize]);Some(())
 }
}
pub(super) fn fast(mem:&mut super::Mem,c:crate::cpu::CpuContext,index:usize)->(crate::cpu::CpuContext,u64){
 let (c,used,_)=fast_batch(mem,c,index,BLOCKS[index].2,1);(c,used)
}
pub(super) fn fast_batch(mem:&mut super::Mem,c:crate::cpu::CpuContext,index:usize,budget:u64,max_visits:u64)->(crate::cpu::CpuContext,u64,u64){
 let b=BLOCKS[index];let mut m=Machine{c,a:Live{mem,lo:b.0,hi:b.1},retired:0};
 let mut visits=0;
 if b.0==super::minecraft_crc::ENTRY {
  visits=super::minecraft_crc::prefix(m.a.mem,&mut m.c.regs,max_visits.min(budget/b.2));
  m.retired=(visits*b.2) as u32;
 }
 while visits<max_visits && budget-u64::from(m.retired)>=b.2 {
  visits+=1;
  if execute(index,&mut m).is_none(){m.retired-=1;break;}
  if m.c.regs[15]!=b.0 {break;}
 }
 (m.c,u64::from(m.retired),visits)
}
''')
(root/'CompatibilityProbe/compatibility-core/src/cpu/interpreter/minecraft_loading.rs').write_text('\n'.join(out)+'\n')
print('Generated',len(ranges),'native loading islands')
