//! Opt-in local HLSL CS_5_0 shader hot-reload for Horizon's own D3D11 output.
//! No arbitrary execution, no shader injection and no file reads outside an
//! explicitly controlled settings folder. Live runtime is rebuilt after reload.
use std::{fs,path::PathBuf,sync::{OnceLock,RwLock,atomic::{AtomicU64,Ordering}}};
const LIMIT:usize=8192;
pub const DEFAULT:&str=r#"Texture2D<float4> Input:register(t0);
RWTexture2D<float4> Output:register(u0);
[numthreads(8,8,1)]
void CSMain(uint3 id:SV_DispatchThreadID){
 uint w,h;Input.GetDimensions(w,h);if(id.x>=w||id.y>=h)return;
 Output[id.xy]=Input.Load(int3(id.xy,0));
}"#;
static ACTIVE:OnceLock<RwLock<String>>=OnceLock::new();
static REVISION:AtomicU64=AtomicU64::new(0);
fn buffer()->&'static RwLock<String>{ACTIVE.get_or_init(||RwLock::new(DEFAULT.to_owned()))}
pub fn source()->String{buffer().read().map(|s|s.clone()).unwrap_or_else(|_|DEFAULT.to_owned())}
pub fn revision()->u64{REVISION.load(Ordering::Acquire)}
pub fn folder()->PathBuf{crate::settings::folder().join("custom-shaders")}
pub fn name_valid(name:&str)->bool{
 !name.is_empty()&&name.len()<=48&&(name.ends_with(".hlsl")||name.ends_with(".wgsl")||name.ends_with(".spv"))
 &&name.chars().all(|c|c.is_ascii_alphanumeric()||matches!(c,'_'|'-'|'.'))
 &&!name.starts_with('.')&&!name.contains("..")
}
pub fn validate_source(source:&str)->Result<(),String>{
 if source.is_empty()||source.len()>LIMIT||source.bytes().any(|b|b==0){
  return Err("HLSL must be UTF-8, 1-8192 bytes, no NUL".into());
 }
 let compact=source.bytes().filter(|b|!b.is_ascii_whitespace()).map(|b|b.to_ascii_lowercase()).collect::<Vec<_>>();
 let text=String::from_utf8(compact).map_err(|e|e.to_string())?;
 if text.contains("#include")||text.contains("#pragma")||text.contains("groupshared")||text.contains("while(")
 ||text.contains("for(")||text.contains("doforever")||text.contains("interlocked"){
  return Err("Restricted custom HLSL: no includes, loops, atomics or shared memory".into());
 }
 for needed in ["register(t0)","register(u0)","voidcsmain(","numthreads(8,8,1)"]{
  if !text.contains(needed){return Err(format!("HLSL must declare {needed}"));}
 }
 crate::shadercheck::compile_native_hlsl(source).map(|_|())
}
pub fn load_source(code:&str)->Result<u64,String>{
 validate_source(code)?;
 let mut guard=buffer().write().map_err(|_|"Shader state unavailable".to_string())?;
 if guard.as_str()==code{return Ok(revision());}
 *guard=code.to_owned();
 Ok(REVISION.fetch_add(1,Ordering::AcqRel)+1)
}
pub fn load_named(name:&str)->Result<u64,String>{
 if !name_valid(name){return Err("Shader name: ASCII letters, digits, _ or -, .hlsl; no paths".into());}
 let path=folder().join(name);
 let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
 let max=if name.ends_with(".spv"){16384}else{LIMIT};
 if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>max as u64{
  return Err("Custom shader must be a regular file within its strict format size limit".into());
 }
 let bytes=fs::read(path).map_err(|e|e.to_string())?;
 let hlsl=if name.ends_with(".spv"){crate::shader_translate::spv_to_hlsl(&bytes)?}
    else{let source=String::from_utf8(bytes).map_err(|_|"Shader text must be UTF-8".to_string())?;
       if name.ends_with(".wgsl"){crate::shader_translate::wgsl_to_hlsl(&source)?}else{source}};
 load_source(&hlsl)
}
pub fn make_example()->Result<PathBuf,String>{
 let dir=folder();
 fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
 let name=dir.join("example.hlsl");
 if !name.exists(){fs::write(&name,DEFAULT).map_err(|e|e.to_string())?;}
 Ok(name)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn default_shader_real_d3d_compiles_and_security_rejects_bad_io(){
  validate_source(DEFAULT).unwrap();
  assert!(!name_valid("..\\secrets.hlsl"));assert!(!name_valid("../../secret.hlsl"));
  assert!(!name_valid("x.fx"));assert!(name_valid("pixel_test.hlsl"));assert!(name_valid("pixel_test.wgsl"));assert!(name_valid("pixel_test.spv"));
  assert!(validate_source("#include \"a\"\nvoid CSMain(){}").is_err());
  assert!(validate_source(&" ".repeat(LIMIT+1)).is_err());
 }
 #[test]fn reload_does_not_promote_invalid_shader_or_change_revision(){
  let initial=source();let stamp=revision();
  let invalid=DEFAULT.replace("register(u0)","register(u2)");
  assert!(load_source(&invalid).is_err());assert_eq!(revision(),stamp);assert_eq!(source(),initial);
  assert_eq!(load_source(&initial).unwrap(),stamp);
 }
 #[test]fn real_gpu_live_dynamic_shader_copies_input(){
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  let input=image::RgbaImage::from_fn(49,33,|x,y|image::Rgba([(x*5)as u8,(y*7)as u8,177,31]));
  let texture=gpu.upload(&input).unwrap();
  let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,49,33).unwrap();
  let output=gpu.read(&pipe.dispatch(&gpu.device,&gpu.ctx,&texture,&[crate::native_filter::CUSTOM_STAGE]).unwrap()).unwrap();
  assert_eq!(output,input);
 }
}
