/* This Source Code Form is subject to the Mozilla Public
 * License, v. 2.0. https://mozilla.org/MPL/2.0/ */
//! Direct RGBA render target and internal-context recovery operations.
//! No guest semantics are virtualized here; the framework layer guards routing.
use super::{gles11_raw as gl,GLES};
use gl::types::*;
use std::sync::atomic::{AtomicBool,Ordering};
static ENABLED:AtomicBool=AtomicBool::new(false);
pub fn configure(verified_hd: bool, minecraft_gpu: bool) {
 let verified = verified_hd || minecraft_gpu;
 let requested = (verified_hd && std::env::var("PEGGLE_HD_DIRECT_TEXTURE").as_deref() == Ok("1"))
     || (minecraft_gpu && std::env::var("MINECRAFT_DIRECT_TEXTURE").as_deref() != Ok("0"));
 let enabled=cfg!(target_os="ios")&&verified&&super::gpu_transfer::enabled()&&requested;
 ENABLED.store(enabled,Ordering::Relaxed);
 crate::diagnostics::event(&format!("{{\"event\":\"direct_texture_configuration\",\"requested\":{requested},\"verified_image\":{verified},\"enabled\":{enabled}}}"));
}
pub fn enabled()->bool{ENABLED.load(Ordering::Relaxed)}

unsafe fn int(g:&mut dyn GLES,p:GLenum)->GLint {let mut v=0;g.GetIntegerv(p,&mut v);v}
unsafe fn ints<const N:usize>(g:&mut dyn GLES,p:GLenum)->[GLint;N] {let mut v=[0;N];g.GetIntegerv(p,v.as_mut_ptr());v}
unsafe fn floats<const N:usize>(g:&mut dyn GLES,p:GLenum)->[GLfloat;N] {let mut v=[0.0;N];g.GetFloatv(p,v.as_mut_ptr());v}
unsafe fn pointer(g:&mut dyn GLES,p:GLenum)->*const GLvoid {let mut v=std::ptr::null();g.GetPointerv(p,&mut v);v}
pub unsafe fn drain_errors(g:&mut dyn GLES)->Vec<GLenum> {let mut v=Vec::new();for _ in 0..16 {let e=g.GetError();if e==gl::NO_ERROR{break;}v.push(e);}v}

const ENV_PARAMETERS:[GLenum;9]=[
 gl::TEXTURE_ENV_MODE,gl::COMBINE_RGB,gl::COMBINE_ALPHA,gl::SRC0_RGB,gl::SRC1_RGB,
 gl::OPERAND0_RGB,gl::OPERAND1_RGB,gl::SRC0_ALPHA,gl::OPERAND0_ALPHA,
];
// The scales are float parameters; GLES1OnGL2 rejects them through the int query.
const ENV_SCALES:[GLenum;2]=[gl::RGB_SCALE,gl::ALPHA_SCALE];
pub type OpaqueEnv=Option<([GLint;9],[GLfloat;2])>;
/// RGB *= primary color; alpha comes only from primary color. This matches
/// opaque CALayer's CPU alpha=255 followed by opacity modulation, including
/// parent opacity/blending. Does not modify the shared texture object.
pub unsafe fn opaque_begin(g:&mut dyn GLES,enabled:bool)->OpaqueEnv {
 if !enabled{return None;}
 let old=ENV_PARAMETERS.map(|p|{let mut v=0;g.GetTexEnviv(gl::TEXTURE_ENV,p,&mut v);v});
 let scales=ENV_SCALES.map(|p|{let mut v=0.0;g.GetTexEnvfv(gl::TEXTURE_ENV,p,&mut v);v});
 let values=[gl::COMBINE,gl::MODULATE,gl::REPLACE,gl::TEXTURE,gl::PRIMARY_COLOR,gl::SRC_COLOR,gl::SRC_COLOR,gl::PRIMARY_COLOR,gl::SRC_ALPHA];
 for (p,v) in ENV_PARAMETERS.into_iter().zip(values) {g.TexEnvi(gl::TEXTURE_ENV,p,v as GLint);}
 for p in ENV_SCALES {g.TexEnvf(gl::TEXTURE_ENV,p,1.0);}
 Some((old,scales))
}
pub unsafe fn opaque_end(g:&mut dyn GLES,old:OpaqueEnv) {
 if let Some((old,scales))=old {
  for(p,v)in ENV_PARAMETERS.into_iter().zip(old){g.TexEnvi(gl::TEXTURE_ENV,p,v);}
  for(p,v)in ENV_SCALES.into_iter().zip(scales){g.TexEnvf(gl::TEXTURE_ENV,p,v);}
 }
}

const CAPS:[GLenum;21]=[
 gl::ALPHA_TEST,gl::BLEND,gl::COLOR_LOGIC_OP,gl::CULL_FACE,gl::DEPTH_TEST,gl::DITHER,
 gl::FOG,gl::LIGHTING,gl::SCISSOR_TEST,gl::STENCIL_TEST,gl::POLYGON_OFFSET_FILL,
 gl::SAMPLE_ALPHA_TO_COVERAGE,gl::SAMPLE_ALPHA_TO_ONE,gl::SAMPLE_COVERAGE,gl::MULTISAMPLE,
 gl::CLIP_PLANE0,gl::CLIP_PLANE1,gl::CLIP_PLANE2,gl::CLIP_PLANE3,gl::CLIP_PLANE4,gl::CLIP_PLANE5,
];
struct Array {size:GLint,type_:GLenum,stride:GLsizei,pointer:*const GLvoid,buffer:GLuint}
impl Array {
 unsafe fn vertex(g:&mut dyn GLES)->Self {Self{size:int(g,gl::VERTEX_ARRAY_SIZE),type_:int(g,gl::VERTEX_ARRAY_TYPE)as _,stride:int(g,gl::VERTEX_ARRAY_STRIDE),pointer:pointer(g,gl::VERTEX_ARRAY_POINTER),buffer:int(g,gl::VERTEX_ARRAY_BUFFER_BINDING)as _}}
 unsafe fn coords(g:&mut dyn GLES)->Self {Self{size:int(g,gl::TEXTURE_COORD_ARRAY_SIZE),type_:int(g,gl::TEXTURE_COORD_ARRAY_TYPE)as _,stride:int(g,gl::TEXTURE_COORD_ARRAY_STRIDE),pointer:pointer(g,gl::TEXTURE_COORD_ARRAY_POINTER),buffer:int(g,gl::TEXTURE_COORD_ARRAY_BUFFER_BINDING)as _}}
}
/// For the retained INTERNAL compositor context, whose palette shader is
/// unused. Saves every state item modified below, including texture units and
/// all client-array bindings; matrices are copied without pushing small stacks.
/// Source is an owned direct texture with LINEAR filters and CLAMP wrapping.
pub unsafe fn blit_owned_into_bound(g:&mut dyn GLES,texture:GLuint,width:GLsizei,height:GLsizei,force_opaque:bool)->Result<(),&'static str> {
 if super::gpu_transfer::rgba_size(width,height,int(g,gl::MAX_TEXTURE_SIZE)).is_none(){return Err("blit_size");}
 let active=int(g,gl::ACTIVE_TEXTURE)as GLenum;let client=int(g,gl::CLIENT_ACTIVE_TEXTURE)as GLenum;
 let units=int(g,gl::MAX_TEXTURE_UNITS).clamp(1,32)as u32;
 let mut secondary=Vec::new();
 for unit in 1..units {
  g.ActiveTexture(gl::TEXTURE0+unit);g.ClientActiveTexture(gl::TEXTURE0+unit);
  secondary.push((g.IsEnabled(gl::TEXTURE_2D),g.IsEnabledClientStateCompat(gl::TEXTURE_COORD_ARRAY)));
  g.Disable(gl::TEXTURE_2D);g.DisableClientState(gl::TEXTURE_COORD_ARRAY);
 }
 g.ActiveTexture(gl::TEXTURE0);g.ClientActiveTexture(gl::TEXTURE0);
 let old_texture=int(g,gl::TEXTURE_BINDING_2D)as GLuint;let texture_enabled=g.IsEnabled(gl::TEXTURE_2D);
 let caps=CAPS.map(|c|g.IsEnabled(c));for c in CAPS {g.Disable(c);}
 let array_caps=[gl::VERTEX_ARRAY,gl::TEXTURE_COORD_ARRAY,gl::COLOR_ARRAY,gl::NORMAL_ARRAY];
 let arrays=array_caps.map(|p|{let mut v=0;g.GetBooleanv(p,&mut v);g.DisableClientState(p);v});
 let color_mask={let mut v=[0;4];g.GetBooleanv(gl::COLOR_WRITEMASK,v.as_mut_ptr());v};
 let viewport=ints::<4>(g,gl::VIEWPORT);let color=floats::<4>(g,gl::CURRENT_COLOR);let matrix_mode=int(g,gl::MATRIX_MODE)as GLenum;
 let matrices=[(gl::MODELVIEW,gl::MODELVIEW_MATRIX),(gl::PROJECTION,gl::PROJECTION_MATRIX),(gl::TEXTURE,gl::TEXTURE_MATRIX)].map(|(mode,p)|{let m=floats::<16>(g,p);g.MatrixMode(mode);g.LoadIdentity();(mode,m)});
 let array_buffer=int(g,gl::ARRAY_BUFFER_BINDING)as GLuint;let vertex=Array::vertex(g);let coords=Array::coords(g);
 let old_env={let mut v=0;g.GetTexEnviv(gl::TEXTURE_ENV,gl::TEXTURE_ENV_MODE,&mut v);v};
 g.BindTexture(gl::TEXTURE_2D,texture);g.Enable(gl::TEXTURE_2D);
 for p in [gl::TEXTURE_MIN_FILTER,gl::TEXTURE_MAG_FILTER]{g.TexParameteri(gl::TEXTURE_2D,p,gl::NEAREST as GLint);}
 let opaque=opaque_begin(g,force_opaque);if !force_opaque{g.TexEnvi(gl::TEXTURE_ENV,gl::TEXTURE_ENV_MODE,gl::REPLACE as GLint);}
 g.ColorMask(gl::TRUE,gl::TRUE,gl::TRUE,gl::TRUE);g.Color4f(1.0,1.0,1.0,1.0);g.Viewport(0,0,width,height);
 g.BindBuffer(gl::ARRAY_BUFFER,0);
 let vertices:[f32;8]=[-1.0,-1.0,1.0,-1.0,-1.0,1.0,1.0,1.0];let uv:[f32;8]=[0.0,0.0,1.0,0.0,0.0,1.0,1.0,1.0];
 g.VertexPointer(2,gl::FLOAT,0,vertices.as_ptr().cast());g.TexCoordPointer(2,gl::FLOAT,0,uv.as_ptr().cast());
 g.EnableClientState(gl::VERTEX_ARRAY);g.EnableClientState(gl::TEXTURE_COORD_ARRAY);g.DrawArrays(gl::TRIANGLE_STRIP,0,4);g.Flush();
 opaque_end(g,opaque);g.TexEnvi(gl::TEXTURE_ENV,gl::TEXTURE_ENV_MODE,old_env);
 for p in [gl::TEXTURE_MIN_FILTER,gl::TEXTURE_MAG_FILTER]{g.TexParameteri(gl::TEXTURE_2D,p,gl::LINEAR as GLint);}
 g.BindBuffer(gl::ARRAY_BUFFER,vertex.buffer);g.VertexPointer(vertex.size,vertex.type_,vertex.stride,vertex.pointer);
 g.BindBuffer(gl::ARRAY_BUFFER,coords.buffer);g.TexCoordPointer(coords.size,coords.type_,coords.stride,coords.pointer);g.BindBuffer(gl::ARRAY_BUFFER,array_buffer);
 for (mode,m)in matrices{g.MatrixMode(mode);g.LoadMatrixf(m.as_ptr());}
 g.MatrixMode(matrix_mode);g.Color4f(color[0],color[1],color[2],color[3]);g.Viewport(viewport[0],viewport[1],viewport[2],viewport[3]);g.ColorMask(color_mask[0],color_mask[1],color_mask[2],color_mask[3]);
 for(c,on)in CAPS.into_iter().zip(caps){if on!=0{g.Enable(c);}else{g.Disable(c);}}
 for(c,on)in array_caps.into_iter().zip(arrays){if on!=0{g.EnableClientState(c);}else{g.DisableClientState(c);}}
 g.BindTexture(gl::TEXTURE_2D,old_texture);if texture_enabled!=0{g.Enable(gl::TEXTURE_2D);}else{g.Disable(gl::TEXTURE_2D);}
 for(unit,(tex,coords))in(1..units).zip(secondary){g.ActiveTexture(gl::TEXTURE0+unit);g.ClientActiveTexture(gl::TEXTURE0+unit);if tex!=0{g.Enable(gl::TEXTURE_2D);}else{g.Disable(gl::TEXTURE_2D);}if coords!=0{g.EnableClientState(gl::TEXTURE_COORD_ARRAY);}else{g.DisableClientState(gl::TEXTURE_COORD_ARRAY);}}
 g.ActiveTexture(active);g.ClientActiveTexture(client);
 if drain_errors(g).is_empty(){Ok(())}else{Err("blit_gl_error")}
}
/// Restore a direct image into its original renderbuffer before unsupported
/// guest access/mutation. No window target or swap is used.
pub unsafe fn restore_renderbuffer(g:&mut dyn GLES,texture:GLuint,renderbuffer:GLuint,width:GLsizei,height:GLsizei)->Result<(),&'static str> {
 if !drain_errors(g).is_empty(){return Err("restore_host_prior_error");}
 let old=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;let mut fbo=0;g.GenFramebuffersOES(1,&mut fbo);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,fbo);
 g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,renderbuffer);
 let result=if g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)==gl::FRAMEBUFFER_COMPLETE_OES {blit_owned_into_bound(g,texture,width,height,false)}else{Err("restore_fbo_incomplete")};
 g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old);g.DeleteFramebuffersOES(1,&fbo);result
}
// GetBooleanv accepts client enables; IsEnabled is not specified for these.
trait ClientEnable {unsafe fn IsEnabledClientStateCompat(&mut self,p:GLenum)->GLboolean;}
impl ClientEnable for dyn GLES+'_ {unsafe fn IsEnabledClientStateCompat(&mut self,p:GLenum)->GLboolean {let mut v=0;self.GetBooleanv(p,&mut v);v}}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Mode {Shadow,Active}
pub struct Surface {
 pub renderbuffer:GLuint,pub framebuffer:GLuint,pub texture:GLuint,pub width:GLsizei,pub height:GLsizei,
 shadow_fbo: GLuint, shadow_depth: GLuint,
 pub bound_framebuffer:GLuint,pub needs_rebind:bool,pub mode:Mode,pub verified:u8,pub attempts:u32,pub mirrored_ops:u64,pub frames:u64,
}
impl Surface {
 /// The caller has verified the image/sharegroup and made the owner current.
 /// Requires a complete single-sampled RGBA8 guest FBO. Minecraft may also have depth.
 pub unsafe fn create(g:&mut dyn GLES,renderbuffer:GLuint,allow_depth:bool)->(Result<Self,&'static str>,Vec<GLenum>){
  let prior=drain_errors(g);if prior.len()==16{return(Err("prior_errors_saturated"),prior);}
  let framebuffer=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;
  if framebuffer==0||renderbuffer==0{return(Err("no_source_object"),prior);}
  let mut width=0;let mut height=0;let mut format=0;
  g.GetRenderbufferParameterivOES(gl::RENDERBUFFER_OES,gl::RENDERBUFFER_WIDTH_OES,&mut width);g.GetRenderbufferParameterivOES(gl::RENDERBUFFER_OES,gl::RENDERBUFFER_HEIGHT_OES,&mut height);g.GetRenderbufferParameterivOES(gl::RENDERBUFFER_OES,gl::RENDERBUFFER_INTERNAL_FORMAT_OES,&mut format);
  if framebuffer==0||renderbuffer==0||format!=gl::RGBA8_OES as GLint||super::gpu_transfer::rgba_size(width,height,int(g,gl::MAX_TEXTURE_SIZE)).is_none(){let _=drain_errors(g);return(Err("ineligible_source"),prior);}
  let attachment=|g:&mut dyn GLES,where_:GLenum,p:GLenum|{let mut v=0;g.GetFramebufferAttachmentParameterivOES(gl::FRAMEBUFFER_OES,where_,p,&mut v);v};
  let depth_type = attachment(g,gl::DEPTH_ATTACHMENT_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE_OES);
  let color_type = attachment(g,gl::COLOR_ATTACHMENT0_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE_OES);
  let color_name = attachment(g,gl::COLOR_ATTACHMENT0_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME_OES);
  let stencil_type = attachment(g,gl::STENCIL_ATTACHMENT_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE_OES);
  let packed_depth_stencil = allow_depth && stencil_type==gl::RENDERBUFFER_OES as GLint
      && depth_type==gl::RENDERBUFFER_OES as GLint
      && attachment(g,gl::DEPTH_ATTACHMENT_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME_OES)
          == attachment(g,gl::STENCIL_ATTACHMENT_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME_OES);
  if color_type!=gl::RENDERBUFFER_OES as GLint || color_name!=renderbuffer as GLint
      || (depth_type!=0 && !(allow_depth && depth_type==gl::RENDERBUFFER_OES as GLint))
      || (stencil_type!=0 && !packed_depth_stencil) || g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)!=gl::FRAMEBUFFER_COMPLETE_OES {
   crate::diagnostics::event(&format!("{{\"event\":\"direct_source_layout\",\"fbo\":{framebuffer},\"rbo\":{renderbuffer},\"color_type\":{color_type},\"color_name\":{color_name},\"depth_type\":{depth_type},\"stencil_type\":{stencil_type}}}"));
   let _=drain_errors(g);return(Err("ineligible_attachments"),prior);
  }
  if int(g,gl::SAMPLE_BUFFERS)!=0||int(g,gl::SAMPLES)!=0{let _=drain_errors(g);return(Err("multisampled_source"),prior);}
  let precision=[gl::RED_BITS,gl::GREEN_BITS,gl::BLUE_BITS,gl::ALPHA_BITS].map(|p|int(g,p));
  if precision!=[8;4]{let _=drain_errors(g);return(Err("source_precision"),prior);}
  // Give the reference and mirror independent depth during the pixel check.
  // The original depth attachment stays on the guest FBO after activation.
  let depth_format = if depth_type != 0 {
   let depth = attachment(g,gl::DEPTH_ATTACHMENT_OES,gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME_OES);
   let previous = int(g,gl::RENDERBUFFER_BINDING_OES) as GLuint;
   g.BindRenderbufferOES(gl::RENDERBUFFER_OES,depth as GLuint);
   let mut format = 0;
   g.GetRenderbufferParameterivOES(gl::RENDERBUFFER_OES,gl::RENDERBUFFER_INTERNAL_FORMAT_OES,&mut format);
   g.BindRenderbufferOES(gl::RENDERBUFFER_OES,previous);
   const DEPTH_COMPONENT24_OES: GLint = 0x81A6;
   const DEPTH24_STENCIL8_OES: GLint = 0x88F0;
   let supported = if packed_depth_stencil { format == DEPTH24_STENCIL8_OES }
       else { format == gl::DEPTH_COMPONENT16_OES as GLint || format == DEPTH_COMPONENT24_OES };
   if !supported {
    let _=drain_errors(g);return (Err("unsupported_depth_format"),prior);
   }
   Some(format as GLenum)
  } else { None };
  let active=int(g,gl::ACTIVE_TEXTURE)as GLenum;g.ActiveTexture(gl::TEXTURE0);let old_texture=int(g,gl::TEXTURE_BINDING_2D)as GLuint;
  if let Err(e)=super::full_device_budget::texture(g,width,height){g.ActiveTexture(active);return(Err(e),prior);}
  let mut texture=0;g.GenTextures(1,&mut texture);g.BindTexture(gl::TEXTURE_2D,texture);
  // One seed copy preserves retained/partially redrawn pixels during the
  // initial mirror gate. There are no copy commands in active presentation.
  g.CopyTexImage2D(gl::TEXTURE_2D,0,gl::RGBA,0,0,width,height,0);
  for p in [gl::TEXTURE_MIN_FILTER,gl::TEXTURE_MAG_FILTER]{g.TexParameteri(gl::TEXTURE_2D,p,gl::LINEAR as GLint);}
  for p in [gl::TEXTURE_WRAP_S,gl::TEXTURE_WRAP_T]{g.TexParameteri(gl::TEXTURE_2D,p,gl::CLAMP_TO_EDGE as GLint);}
  g.FramebufferTexture2DOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::TEXTURE_2D,texture,0);
  let complete=g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)==gl::FRAMEBUFFER_COMPLETE_OES;
  let target_precision=[gl::RED_BITS,gl::GREEN_BITS,gl::BLUE_BITS,gl::ALPHA_BITS].map(|p|int(g,p));
  g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,renderbuffer);
  g.BindTexture(gl::TEXTURE_2D,old_texture);g.ActiveTexture(active);
  let own=drain_errors(g);
  if !complete||target_precision!=precision||!own.is_empty(){if texture!=0{g.DeleteTextures(1,&texture);}return(Err("texture_setup_or_precision"),prior);}
  let (mut shadow_fbo, mut shadow_depth) = (0, 0);
  if let Some(format) = depth_format {
   g.GenRenderbuffersOES(1,&mut shadow_depth);
   g.BindRenderbufferOES(gl::RENDERBUFFER_OES,shadow_depth);
   g.RenderbufferStorageOES(gl::RENDERBUFFER_OES,format,width,height);
   g.BindRenderbufferOES(gl::RENDERBUFFER_OES,renderbuffer);
   g.GenFramebuffersOES(1,&mut shadow_fbo);
   g.BindFramebufferOES(gl::FRAMEBUFFER_OES,shadow_fbo);
   g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::DEPTH_ATTACHMENT_OES,gl::RENDERBUFFER_OES,shadow_depth);
   if packed_depth_stencil {
    g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::STENCIL_ATTACHMENT_OES,gl::RENDERBUFFER_OES,shadow_depth);
   }
   g.FramebufferTexture2DOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::TEXTURE_2D,texture,0);
   let complete = g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)==gl::FRAMEBUFFER_COMPLETE_OES;
   g.BindFramebufferOES(gl::FRAMEBUFFER_OES,framebuffer);
   let errors = drain_errors(g);
   if !complete || !errors.is_empty() {
    g.DeleteFramebuffersOES(1,&shadow_fbo);g.DeleteRenderbuffersOES(1,&shadow_depth);g.DeleteTextures(1,&texture);
    return (Err("shadow_depth_setup"),prior);
   }
  }
  (Ok(Self{renderbuffer,framebuffer,texture,width,height,shadow_fbo,shadow_depth,bound_framebuffer:framebuffer,needs_rebind:false,mode:Mode::Shadow,verified:0,attempts:0,mirrored_ops:0,frames:0}),prior)
 }
 /// Run the original reference draw first, then the exact same command/state
 /// against the RGBA texture while validating with independent depth if used.
 /// Active mode runs the command just once against the texture attachment.
 pub unsafe fn draw(&mut self,g:&mut dyn GLES,mut command:impl FnMut(&mut dyn GLES)){
  if self.mode==Mode::Active&&self.needs_rebind&&self.bound_framebuffer==self.framebuffer {g.BindFramebufferOES(gl::FRAMEBUFFER_OES,self.framebuffer);self.needs_rebind=false;}
  command(g);
  if self.mode==Mode::Shadow&&self.bound_framebuffer==self.framebuffer {
   if self.shadow_fbo != 0 {
    g.BindFramebufferOES(gl::FRAMEBUFFER_OES,self.shadow_fbo);
    command(g);
    g.BindFramebufferOES(gl::FRAMEBUFFER_OES,self.framebuffer);
   } else {
    g.FramebufferTexture2DOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::TEXTURE_2D,self.texture,0);
    command(g);
    g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,self.renderbuffer);
   }
   self.mirrored_ops+=1;
  }
 }
 unsafe fn release_shadow(&mut self,g:&mut dyn GLES) {
  if self.shadow_fbo != 0 {
   g.DeleteFramebuffersOES(1,&self.shadow_fbo);
   g.DeleteRenderbuffersOES(1,&self.shadow_depth);
   self.shadow_fbo=0;self.shadow_depth=0;
  }
 }
 pub unsafe fn activate(&mut self,g:&mut dyn GLES)->Result<(),&'static str>{
  if self.verified<4{return Err("mirror_gate_pending");}
  let old=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;g.BindFramebufferOES(gl::FRAMEBUFFER_OES,self.framebuffer);
  g.FramebufferTexture2DOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::TEXTURE_2D,self.texture,0);
  let complete=g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)==gl::FRAMEBUFFER_COMPLETE_OES;let own=drain_errors(g);
  if !complete||!own.is_empty(){g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,self.renderbuffer);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old);return Err("activate_incomplete");}
  g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old);self.release_shadow(g);self.mode=Mode::Active;Ok(())
 }
 /// Call only after active pixels have been restored into the renderbuffer.
 pub unsafe fn restore_attachment(&mut self,g:&mut dyn GLES)->Result<(),&'static str>{
  self.release_shadow(g);
  let old=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;let old_rb=int(g,gl::RENDERBUFFER_BINDING_OES)as GLuint;
  g.BindFramebufferOES(gl::FRAMEBUFFER_OES,self.framebuffer);g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,self.renderbuffer);
  g.BindRenderbufferOES(gl::RENDERBUFFER_OES,self.renderbuffer);g.BindRenderbufferOES(gl::RENDERBUFFER_OES,old_rb);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old);
  if drain_errors(g).is_empty(){self.mode=Mode::Shadow;Ok(())}else{Err("restore_attachment_error")}
 }
}

pub unsafe fn read_renderbuffer(g:&mut dyn GLES,renderbuffer:GLuint,width:GLsizei,height:GLsizei,mut pixels:Vec<u8>)->Result<Vec<u8>,&'static str>{
 let size=super::gpu_transfer::rgba_size(width,height,int(g,gl::MAX_TEXTURE_SIZE)).ok_or("reference_size")?;
 let old=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;let pack=int(g,gl::PACK_ALIGNMENT);let mut fbo=0;
 g.GenFramebuffersOES(1,&mut fbo);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,fbo);g.FramebufferRenderbufferOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::RENDERBUFFER_OES,renderbuffer);
 let complete=g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)==gl::FRAMEBUFFER_COMPLETE_OES;
 if complete{pixels.resize(size,0);g.PixelStorei(gl::PACK_ALIGNMENT,1);g.ReadPixels(0,0,width,height,gl::RGBA,gl::UNSIGNED_BYTE,pixels.as_mut_ptr().cast());}
 g.PixelStorei(gl::PACK_ALIGNMENT,pack);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old);g.DeleteFramebuffersOES(1,&fbo);
 if complete&&drain_errors(g).is_empty(){Ok(pixels)}else{Err("reference_read_error")}
}
/// One bounded preactivation oracle for the actual opaque texture combiner.
/// Internal context only; source image and guest state remain unchanged.
pub unsafe fn opaque_oracle(g:&mut dyn GLES,source:GLuint,width:GLsizei,height:GLsizei)->Result<Vec<u8>,&'static str>{
 let size=super::gpu_transfer::rgba_size(width,height,int(g,gl::MAX_TEXTURE_SIZE)).ok_or("opaque_oracle_size")?;
 let _budget=super::full_device_budget::temporary(size)?;
 let old_fbo=int(g,gl::FRAMEBUFFER_BINDING_OES)as GLuint;let old_active=int(g,gl::ACTIVE_TEXTURE)as GLenum;g.ActiveTexture(gl::TEXTURE0);let old_tex=int(g,gl::TEXTURE_BINDING_2D)as GLuint;let pack=int(g,gl::PACK_ALIGNMENT);
 let(mut tex,mut fbo)=(0,0);g.GenTextures(1,&mut tex);g.BindTexture(gl::TEXTURE_2D,tex);g.TexImage2D(gl::TEXTURE_2D,0,gl::RGBA as GLint,width,height,0,gl::RGBA,gl::UNSIGNED_BYTE,std::ptr::null());
 for p in [gl::TEXTURE_MIN_FILTER,gl::TEXTURE_MAG_FILTER]{g.TexParameteri(gl::TEXTURE_2D,p,gl::LINEAR as GLint);}for p in [gl::TEXTURE_WRAP_S,gl::TEXTURE_WRAP_T]{g.TexParameteri(gl::TEXTURE_2D,p,gl::CLAMP_TO_EDGE as GLint);}
 g.GenFramebuffersOES(1,&mut fbo);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,fbo);g.FramebufferTexture2DOES(gl::FRAMEBUFFER_OES,gl::COLOR_ATTACHMENT0_OES,gl::TEXTURE_2D,tex,0);
 let result=(||{
  if g.CheckFramebufferStatusOES(gl::FRAMEBUFFER_OES)!=gl::FRAMEBUFFER_COMPLETE_OES{return Err("opaque_oracle_fbo");}
  blit_owned_into_bound(g,source,width,height,true)?;let mut pixels=vec![0;size];g.PixelStorei(gl::PACK_ALIGNMENT,1);g.ReadPixels(0,0,width,height,gl::RGBA,gl::UNSIGNED_BYTE,pixels.as_mut_ptr().cast());Ok(pixels)
 })();
 g.PixelStorei(gl::PACK_ALIGNMENT,pack);g.BindFramebufferOES(gl::FRAMEBUFFER_OES,old_fbo);g.DeleteFramebuffersOES(1,&fbo);g.BindTexture(gl::TEXTURE_2D,old_tex);g.DeleteTextures(1,&tex);g.ActiveTexture(old_active);
 if drain_errors(g).is_empty(){result}else{Err("opaque_oracle_error")}
}

/// The normal CALayer quad keeps its geometry, UVs and blend equation. Only
/// opaque source-alpha policy differs for an RGBA shared producer texture.
pub unsafe fn draw_elements_opaque(g:&mut dyn GLES,mode:GLenum,count:GLsizei,type_:GLenum,indices:*const GLvoid,opaque:bool){
 let saved=opaque_begin(g,opaque);g.DrawElements(mode,count,type_,indices);opaque_end(g,saved);
}

#[cfg(test)]
pub fn fixture_enable(value:bool){ENABLED.store(value,Ordering::Relaxed);}
