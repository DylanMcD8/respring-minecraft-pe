/* This Source Code Form is subject to the Mozilla Public License, v. 2.0. */
//! Present the completed compositor image through native Core Animation.
//! The guest renderer, compositor overlays and frame deadlines are unchanged.
//! Unsupported orientations/cursors or native failures retain GL presentation.
use crate::matrix::Matrix;
use std::sync::atomic::{AtomicBool, Ordering};
static ENABLED: AtomicBool = AtomicBool::new(false);

pub(crate) fn configure(verified: bool) {
    ENABLED.store(cfg!(target_os = "ios") && verified
        && std::env::var("MINECRAFT_NATIVE_PRESENT").as_deref() != Ok("0"), Ordering::Relaxed);
}

// Use one physical refresh signal for both the guest display timer and the
// completed composite. Before UIKit starts delivering ticks, use guest time.
pub(crate) fn refresh(env: &crate::Environment) -> Option<(u64, std::time::Instant)> {
    #[cfg(all(target_os="ios", not(target_abi="sim")))]
    if env.libc_state.minecraft_background_terrain
        && (env.options.device_family == Some(crate::window::DeviceFamily::iPad)
            || env.options.wrapper_resolution == Some(crate::wrapper_resolution::Mode::Canvas))
        && env.options.fps_limit.is_some_and(|fps| fps >= 60.0) {
        extern "C" { fn minecraft_refresh_deadline(seconds: *mut f64) -> u64; }
        let mut seconds = 0.0;
        let sequence = unsafe { minecraft_refresh_deadline(&mut seconds) };
        if sequence != 0 {
            return Some((sequence, std::time::Instant::now() + std::time::Duration::from_secs_f64(seconds)));
        }
    }
    let _ = env;
    None
}

fn quarter_turn(matrix: Matrix<2>) -> Option<usize> {
    let rotations = [[[1.0,0.0],[0.0,1.0]], [[0.0,1.0],[-1.0,0.0]],
                     [[-1.0,0.0],[0.0,-1.0]], [[0.0,-1.0],[1.0,0.0]]];
    rotations.iter().position(|m| matrix.columns().iter().flatten().zip(m.iter().flatten())
        .all(|(a,b)| (a-b).abs()<0.00001))
}

/// Equivalent to present.rs's centered texture transform, with GL's bottom-up
/// rows converted to top-down CGImage rows. Quarter turns require no resampling.
fn orient(src: &[u8], width: usize, height: usize, turn: usize, dst: &mut Vec<u8>) -> Option<(usize,usize)> {
    let size = width.checked_mul(height)?.checked_mul(4)?;
    if width==0 || height==0 || turn>3 || src.len()!=size { return None; }
    let (w,h) = if turn%2==0 {(width,height)} else {(height,width)};
    dst.resize(size,0);
    let row = width*4;
    match turn {
        // Flips are whole-row copies, reversed pixel order for a half turn.
        0 => for (y,out) in dst.chunks_exact_mut(row).enumerate() {
            out.copy_from_slice(&src[(height-1-y)*row..][..row]);
        },
        2 => for (y,out) in dst.chunks_exact_mut(row).enumerate() {
            for (o,i) in out.chunks_exact_mut(4).zip(src[y*row..][..row].chunks_exact(4).rev()) { o.copy_from_slice(i); }
        },
        // Quarter turns transpose, so walk tiles that stay in cache.
        _ => {
            const TILE: usize = 32;
            for ty in (0..h).step_by(TILE) { for tx in (0..w).step_by(TILE) {
                for y in ty..(ty+TILE).min(h) {
                    let out=&mut dst[y*w*4..][..w*4];
                    for x in tx..(tx+TILE).min(w) {
                        let (sx,sy) = if turn==1 {(y,x)} else {(width-1-y,height-1-x)};
                        out[x*4..x*4+4].copy_from_slice(&src[(sy*width+sx)*4..][..4]);
                    }
                }
            }}
        }
    }
    Some((w,h))
}

#[cfg(target_os="ios")]
extern "C" {
    fn minecraft_native_present_rgba(pixels:*const u8,width:u32,height:u32,
        vx:u32,vy:u32,vw:u32,vh:u32)->bool;
    fn minecraft_native_present_hide();
}

#[cfg(target_os="ios")]
thread_local! {
    static BUFFERS: std::cell::RefCell<(Vec<u8>,Vec<u8>)> = const {std::cell::RefCell::new((Vec::new(),Vec::new()))};
}

pub(crate) fn try_present(gles:&mut dyn crate::gles::GLES, width:u32,height:u32,
    viewport:(u32,u32,u32,u32), rotation:Matrix<2>, cursor:Option<(f32,f32,bool)>)->bool {
    if !ENABLED.load(Ordering::Relaxed) {return false;}
    #[cfg(target_os="ios")]
    {
        use crate::gles::gles11_raw as gl;
        let Some(turn)=quarter_turn(rotation).filter(|_|cursor.is_none()&&width>0&&height>0&&width<=4096&&height<=4096) else {
            unsafe {minecraft_native_present_hide();} return false;
        };
        let result=BUFFERS.with(|buffers| {
            let mut buffers=buffers.borrow_mut(); let (raw,upright)=&mut *buffers;
            raw.resize(width as usize*height as usize*4,0);
            unsafe {
                let _read=crate::diagnostics::scope(crate::diagnostics::Phase::Readback);
                gles.ReadPixels(0,0,width as _,height as _,gl::RGBA,gl::UNSIGNED_BYTE,raw.as_mut_ptr().cast());
                if gles.GetError()!=gl::NO_ERROR{return false;}
            }
            let Some((w,h))=orient(raw,width as usize,height as usize,turn,upright) else {return false;};
            unsafe {minecraft_native_present_rgba(upright.as_ptr(),w as u32,h as u32,viewport.0,viewport.1,viewport.2,viewport.3)}
        });
        if result {
            crate::diagnostics::frame(true);
            static REPORTED:AtomicBool=AtomicBool::new(false);
            if !REPORTED.swap(true,Ordering::Relaxed) {
                crate::diagnostics::event("{\"event\":\"minecraft_native_presentation\",\"route\":\"core_animation\",\"guest_resolution_unchanged\":true}");
            }
        } else {unsafe {minecraft_native_present_hide();}}
        return result;
    }
    #[cfg(not(target_os="ios"))]
    {let _=(gles,width,height,viewport,rotation,cursor);false}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minecraft_probe_presentation_matches_texture_transform() {
        for width in [1,2,5,16] {for height in [1,3,7,12] {for q in 0..4 {
            let matrix=Matrix::z_rotation(q as f32*std::f32::consts::FRAC_PI_2);
            assert_eq!(quarter_turn(matrix),Some(q));
            let source:Vec<_>=(0..width*height*4).map(|i|(i*37) as u8).collect();let mut dest=Vec::new();
            let(w,h)=orient(&source,width,height,q,&mut dest).unwrap();
            for y in 0..h {for x in 0..w {
                let uv=matrix.transform([(x as f32+0.5)/w as f32-0.5,0.5-(y as f32+0.5)/h as f32]);
                let sx=((uv[0]+0.5)*width as f32).floor() as usize;
                let sy=((uv[1]+0.5)*height as f32).floor() as usize;
                assert_eq!(&dest[(y*w+x)*4..(y*w+x+1)*4],&source[(sy*width+sx)*4..(sy*width+sx+1)*4]);
            }}
        }}}
    }
    #[test]
    fn minecraft_probe_presentation_rejects_non_quarter_turns_and_bad_buffers() {
        assert_eq!(quarter_turn(Matrix::z_rotation(0.25)),None);
        assert_eq!(quarter_turn(Matrix::from_columns([[f32::NAN,0.0],[0.0,1.0]])),None);
        let mut dst=vec![7];assert!(orient(&[0;15],2,2,0,&mut dst).is_none());assert_eq!(dst,vec![7]);
        assert!(orient(&[],0,1,0,&mut dst).is_none());assert!(orient(&[0;4],1,1,4,&mut dst).is_none());
    }
}
