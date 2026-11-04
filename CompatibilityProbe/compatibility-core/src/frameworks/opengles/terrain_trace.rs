//! Read-only measurements for the verified PE 0.9.0 renderer. A submitted
//! draw can be occluded; this does not measure illuminated display pixels.
use crate::{mem::ConstVoidPtr, Environment};
use std::{cell::RefCell, collections::{HashMap, HashSet}, sync::atomic::{AtomicBool, Ordering}, time::{SystemTime,UNIX_EPOCH}};
static ENABLED: AtomicBool = AtomicBool::new(false);
#[derive(Default)]
struct Trace {
    upload: HashMap<u32,u128>,
    seen: HashSet<(i32,i32,i32)>,
    pending: Vec<(i32,i32,i32,u128,Option<u128>)>,
    camera: [f32;3],
    last_camera: u128,
}
thread_local! {static TRACE: RefCell<Trace> = RefCell::default();}
pub(crate) fn configure(verified: bool) {
    ENABLED.store(verified && std::env::var("MINECRAFT_DRAW_TRACE").as_deref()==Ok("1"),Ordering::Relaxed);
    TRACE.with(|t| *t.borrow_mut()=Trace::default());
}
fn now()->u128 {SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()}
fn word(env:&Environment,p:u32,off:u32)->Option<u32>{
    Some(u32::from_le_bytes(env.mem.get_bytes_fallible(ConstVoidPtr::from_bits(p.checked_add(off)?),4)?.try_into().ok()?))
}
pub(super) fn uploaded(env:&Environment){
    if !ENABLED.load(Ordering::Relaxed) || env.cpu.regs()[14]!=0x31c8b {return;}
    if let Some(id)=word(env,env.cpu.regs()[13],0x28) {
        TRACE.with(|t|{let mut t=t.borrow_mut(); if t.upload.len()>=4096 {t.upload.clear();} t.upload.insert(id,now());});
    }
}
pub(super) fn translated(env:&Environment,x:f32,y:f32,z:f32){
    if ENABLED.load(Ordering::Relaxed) && env.cpu.regs()[14]==0x994cd {TRACE.with(|t|t.borrow_mut().camera=[-x,-y,-z]);}
}
pub(super) fn drew(env:&Environment,count:i32){
    if !ENABLED.load(Ordering::Relaxed) || count<=0 || !matches!(env.cpu.regs()[14],0x9951f|0x9957f){return;}
    // Both terrain passes restore r6 (section) before tail-calling GLES.
    let ptr=env.cpu.regs()[6];
    let (Some(x),Some(y),Some(z))=(word(env,ptr,4),word(env,ptr,8),word(env,ptr,12)) else{return};
    let key=(x as i32,y as i32,z as i32);
    if x%16!=0 || y%16!=0 || z%16!=0 || y>=128 {return;}
    TRACE.with(|t|{
        let mut t=t.borrow_mut();
        if t.seen.len()>=16384 || !t.seen.insert(key){return;}
        let up=word(env,ptr,0x6c+0x18).and_then(|id|t.upload.get(&id).copied());
        t.pending.push((key.0,key.1,key.2,now(),up));
    });
}
pub(super) fn presented(){
    if !ENABLED.load(Ordering::Relaxed){return;}
    TRACE.with(|t|{
        let mut t=t.borrow_mut(); let stamp=now(); let [cx,cy,cz]=t.camera;
        if t.camera!=[0.;3] && stamp.saturating_sub(t.last_camera)>=250 {
            t.last_camera=stamp;
            crate::diagnostics::event(&format!(r#"{{"event":"terrain_camera","pos":[{cx},{cy},{cz}],"unix_time_ms":{stamp}}}"#));
        }
        for (x,y,z,d,u) in t.pending.drain(..) {
            let up=u.map_or_else(||"null".to_owned(),|n|n.to_string());
            crate::diagnostics::event(&format!(r#"{{"event":"terrain_first_frame","origin":[{x},{y},{z}],"upload_ms":{up},"draw_ms":{d},"unix_time_ms":{stamp}}}"#));
        }
    });
}
