use windows::{core::Interface,Win32::{Foundation::HMODULE,Graphics::{
 Direct3D::{D3D_DRIVER_TYPE_HARDWARE,D3D_FEATURE_LEVEL_11_0},
 Direct3D11::{D3D11CreateDevice,D3D11_SDK_VERSION,D3D11_CREATE_DEVICE_BGRA_SUPPORT,
 D3D11_TEXTURE2D_DESC,D3D11_SUBRESOURCE_DATA,D3D11_USAGE_DEFAULT,D3D11_USAGE_STAGING,
 D3D11_CPU_ACCESS_READ,D3D11_MAP_READ,D3D11_MAPPED_SUBRESOURCE,D3D11_BIND_SHADER_RESOURCE,
 ID3D11Device,ID3D11DeviceContext,ID3D11Texture2D,ID3D11Resource},
 Dxgi::{IDXGIAdapter,Common::{DXGI_FORMAT_R8G8B8A8_UNORM,DXGI_SAMPLE_DESC}}}}};
fn reference(input:&[u8],iw:usize,ih:usize,ow:usize,oh:usize,x:usize,y:usize)->f64{
 let (rx,ry)=(iw as f64/ow as f64,ih as f64/oh as f64);
 let (left,top)=(x as f64*rx,y as f64*ry);
 let (right,bottom)=(left+rx,top+ry);
 let mut sum=0.;
 for iy in 0..ih{for ix in 0..iw{
  let wx=(right.min(ix as f64+1.)-left.max(ix as f64)).max(0.);
  let wy=(bottom.min(iy as f64+1.)-top.max(iy as f64)).max(0.);
  sum+=input[(iy*iw+ix)*4] as f64*wx*wy;
 }}
 sum/(rx*ry)
}
pub fn run()->Result<String,String>{
 let(mut device,mut ctx)=(None,None);
 unsafe{D3D11CreateDevice(None::<&IDXGIAdapter>,D3D_DRIVER_TYPE_HARDWARE,HMODULE::default(),
 D3D11_CREATE_DEVICE_BGRA_SUPPORT,Some(&[D3D_FEATURE_LEVEL_11_0]),D3D11_SDK_VERSION,
 Some(&mut device),None,Some(&mut ctx))}.map_err(|e|e.to_string())?;
 let device:ID3D11Device=device.ok_or("No GPU")?;let ctx:ID3D11DeviceContext=ctx.ok_or("No context")?;
 let (iw,ih,ow,oh)=(5u32,3u32,3u32,2u32);
 let data=(0..iw*ih).flat_map(|i|{let v=if i%3==0{255}else{(i*13) as u8};[v,v,v,255]}).collect::<Vec<_>>();
 let desc=D3D11_TEXTURE2D_DESC{Width:iw,Height:ih,MipLevels:1,ArraySize:1,Format:DXGI_FORMAT_R8G8B8A8_UNORM,
 SampleDesc:DXGI_SAMPLE_DESC{Count:1,Quality:0},Usage:D3D11_USAGE_DEFAULT,BindFlags:D3D11_BIND_SHADER_RESOURCE.0 as u32,
 CPUAccessFlags:0,MiscFlags:0};
 let init=D3D11_SUBRESOURCE_DATA{pSysMem:data.as_ptr().cast(),SysMemPitch:iw*4,SysMemSlicePitch:iw*ih*4};
 let mut src=None;unsafe{device.CreateTexture2D(&desc,Some(&init),Some(&mut src))}.map_err(|e|e.to_string())?;
 let src:ID3D11Texture2D=src.ok_or("No source")?;
 let mut scale=crate::spatial::NativeSpatialScaler::new(&device,ow,oh)?;
 let result=scale.render_area(&device,&ctx,&src)?;
 let mut staging=None;unsafe{device.CreateTexture2D(&D3D11_TEXTURE2D_DESC{Width:ow,Height:oh,
 Usage:D3D11_USAGE_STAGING,BindFlags:0,CPUAccessFlags:D3D11_CPU_ACCESS_READ.0 as u32,..desc},None,Some(&mut staging))}.map_err(|e|e.to_string())?;
 let staging:ID3D11Texture2D=staging.ok_or("No staging")?;
 let a:ID3D11Resource=staging.cast().map_err(|e|e.to_string())?;
 let b:ID3D11Resource=result.cast().map_err(|e|e.to_string())?;
 unsafe{ctx.CopyResource(&a,&b);}
 crate::native_filter::verify_gpu_completion(&device,&ctx)?;
 let mut mapped=D3D11_MAPPED_SUBRESOURCE::default();
 unsafe{ctx.Map(&a,0,D3D11_MAP_READ,0,Some(&mut mapped))}.map_err(|e|e.to_string())?;
 let mut max_error=0f64;
 for y in 0..oh as usize{let row=unsafe{std::slice::from_raw_parts((mapped.pData as *const u8).add(y*mapped.RowPitch as usize),ow as usize*4)};
 for x in 0..ow as usize{
  let expected=reference(&data,iw as usize,ih as usize,ow as usize,oh as usize,x,y);
  max_error=max_error.max((row[x*4] as f64-expected).abs());
 }}
 unsafe{ctx.Unmap(&a,0);}
 if max_error>1.01{return Err(format!("GPU area precision error {max_error}"));}
 Ok(format!("GPU_AREA_RESAMPLE=PASS INPUT=5x3 OUTPUT=3x2 MAX_ERROR_8BIT={max_error:.4}"))
}
#[cfg(test)]mod tests{
 #[test]fn reference_fractional_weights(){assert!((super::reference(&[0,0,0,255,255,255,255,255,0,0,0,255],3,1,2,1,0,0)-85.).abs()<1e-6);}
}
