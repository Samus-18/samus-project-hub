//! Compile a tightly constrained compute WGSL / SPIR-V input to D3D11 CS_5_0 HLSL
//! via Naga. Never accesses games, and only uses explicit local files.
use naga::{
    back::hlsl::{BindTarget,Options as HlslOptions,PipelineOptions,ShaderModel,Writer},
    valid::{Capabilities,ValidationFlags,Validator},
    ResourceBinding,ShaderStage,
};
fn translate(mut module:naga::Module)->Result<String,String>{
    if module.entry_points.len()!=1||module.entry_points[0].stage!=ShaderStage::Compute{
        return Err("Exactly one compute entry point is required".into());
    }
    if module.entry_points[0].workgroup_size!=[8,8,1]{
        return Err("Only @workgroup_size(8,8,1) is allowed".into());
    }
    let mut map=naga::back::hlsl::BindingMap::default();
    let mut seen=[false;2];let mut resources=0;
    for (_,v) in module.global_variables.iter(){
        if let Some(binding)=v.binding.as_ref(){
            if binding.group!=0||binding.binding>1||seen[binding.binding as usize]{
                return Err("Only unique @group(0) @binding(0=input texture,1=output storage image) are supported".into());
            }
            let allowed=match(binding.binding,&v.space){
                (0,naga::AddressSpace::Handle)=>true,
                (1,naga::AddressSpace::Handle)=>true,
                _=>false
            };
            if !allowed{return Err("Compute shader may only bind images (no other GPU resources)".into());}
            seen[binding.binding as usize]=true;resources+=1;
            map.insert(ResourceBinding{group:binding.group,binding:binding.binding},BindTarget{
                space:0,register:0,binding_array_size:None,dynamic_storage_buffer_offsets_index:None,restrict_indexing:false,
            });
        }
    }
    if resources!=2||seen!=[true,true]{return Err("Input and output texture bindings required".into());}
    // Keep output entry name identical to Horizon's existing CS_5_0 stage ABI.
    module.entry_points[0].name="CSMain".into();
    let info=Validator::new(ValidationFlags::all(),Capabilities::empty())
        .validate(&module).map_err(|e|format!("Shader Naga validation: {e}"))?;
    let options=HlslOptions{
        shader_model:ShaderModel::V5_0,binding_map:map,
        fake_missing_bindings:false,..HlslOptions::default()
    };
    let pipeline=PipelineOptions{entry_point:Some((ShaderStage::Compute,"CSMain".into()))};
    let mut hlsl=String::new();
    Writer::new(&mut hlsl,&options,&pipeline).write(&module,&info,None)
        .map_err(|e|format!("Naga HLSL writer: {e}"))?;
    if !hlsl.contains("CSMain")||!hlsl.contains("register(t0)")||!hlsl.contains("register(u0)") {
        return Err("Translation violated Horizon's shader ABI".into());
    }
    crate::custom_hlsl::validate_source(&hlsl)?;
    Ok(hlsl)
}
pub fn wgsl_to_hlsl(source:&str)->Result<String,String>{
    if source.is_empty()||source.len()>8192{return Err("WGSL must be 1–8192 UTF-8 bytes".into());}
    let module=naga::front::wgsl::parse_str(source).map_err(|e|format!("WGSL parse failed: {e}"))?;
    translate(module)
}
pub fn spv_to_hlsl(bytes:&[u8])->Result<String,String>{
    if bytes.len()<20||bytes.len()>16384||bytes.len()%4!=0{
        return Err("SPIR-V must contain 20–16384 bytes of complete u32 instructions".into());
    }
    if bytes[0..4]!=[3,2,35,7]{return Err("Missing SPIR-V 1.x little-endian magic".into());}
    let module=naga::front::spv::parse_u8_slice(bytes,&naga::front::spv::Options::default())
        .map_err(|e|format!("SPIR-V parsing failed: {e}"))?;
    translate(module)
}
#[cfg(test)]mod tests{
    use super::*;
    const COPY:&str=r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var destination: texture_storage_2d<rgba8unorm,write>;
@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) gid:vec3<u32>){
 let dimensions=textureDimensions(source);
 if(gid.x>=dimensions.x||gid.y>=dimensions.y){return;}
 let px=textureLoad(source,vec2<i32>(gid.xy),0);
 textureStore(destination,vec2<i32>(gid.xy),px);
}"#;
    #[test]fn wgsl_to_real_dx11_shader_preserves_pixels(){
        let hlsl=wgsl_to_hlsl(COPY).unwrap();
        assert!(hlsl.contains("register(t0)")&&hlsl.contains("register(u0)"));
        let gpu=crate::photo_lab::Gpu::new().unwrap();
        let before=image::RgbaImage::from_fn(49,33,|x,y|image::Rgba([(x*5)as u8,(y*7)as u8,173,255]));
        let source=gpu.upload(&before).unwrap();
        let bytes=crate::shadercheck::compile_native_hlsl(&hlsl).unwrap();
        let _=bytes;
        // Existing dispatch path runs this exact dynamic HLSL via custom stage.
        let version=crate::custom_hlsl::load_source(&hlsl).unwrap();
        assert!(version>=crate::custom_hlsl::revision());
        let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,49,33).unwrap();
        let filtered=pipe.dispatch(&gpu.device,&gpu.ctx,&source,&[crate::native_filter::CUSTOM_STAGE]).unwrap();
        let result=gpu.read(&filtered).unwrap();
        assert_eq!(before,result);
        crate::custom_hlsl::load_source(crate::custom_hlsl::DEFAULT).unwrap();
    }

    #[test]fn spirv_real_roundtrip_to_native_d3d11_bytecode(){
        let module=naga::front::wgsl::parse_str(COPY).unwrap();
        let info=naga::valid::Validator::new(ValidationFlags::all(),Capabilities::empty()).validate(&module).unwrap();
        let words=naga::back::spv::write_vec(&module,&info,&naga::back::spv::Options::default(),None).unwrap();
        let bytes=words.iter().flat_map(|w|w.to_le_bytes()).collect::<Vec<u8>>();
        let hlsl=spv_to_hlsl(&bytes).unwrap();
        let bytecode=crate::shadercheck::compile_native_hlsl(&hlsl).unwrap();
        assert!(bytecode.len()>256);
        let gpu=crate::photo_lab::Gpu::new().unwrap();
        let original=image::RgbaImage::from_fn(23,19,|x,y|image::Rgba([(x*9)as u8,(y*11)as u8,124,255]));
        let tex=gpu.upload(&original).unwrap();
        crate::custom_hlsl::load_source(&hlsl).unwrap();
        let pipe=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,23,19).unwrap();
        let result=gpu.read(&pipe.dispatch(&gpu.device,&gpu.ctx,&tex,&[crate::native_filter::CUSTOM_STAGE]).unwrap()).unwrap();
        assert_eq!(result,original);
        crate::custom_hlsl::load_source(crate::custom_hlsl::DEFAULT).unwrap();
    }

    #[test]fn rejects_shader_with_extra_resources_and_untrusted_spv(){
        assert!(spv_to_hlsl(&[0u8;32]).is_err());
        assert!(wgsl_to_hlsl("@compute @workgroup_size(8) fn main(){}").is_err());
        assert!(wgsl_to_hlsl(&COPY.replace("@binding(1)","@binding(3)")).is_err());
    }
}
