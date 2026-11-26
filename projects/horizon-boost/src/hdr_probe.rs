//! Read-only DXGI 1.6 Advanced Color monitor metadata.
//! A HDR-capable monitor does not imply own D3D11 swapchain outputs HDR.
//! No SetColorSpace1, SetHDRMetaData, display mode changes or capture.
use windows::{core::Interface,Win32::Graphics::Dxgi::{CreateDXGIFactory1,IDXGIFactory1,IDXGIOutput6,DXGI_OUTPUT_DESC1}};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020,DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709};
#[derive(Clone,Debug)]pub struct Display{
 pub name:String,pub bits:u32,pub hdr10_active:bool,pub peak_nits:Option<f32>,pub min_nits:Option<f32>,pub full_frame_nits:Option<f32>,
}
fn sane_nits(x:f32)->Option<f32>{(x.is_finite()&&x>0.&&x<=10000.).then_some(x)}
pub fn probe()->Result<Vec<Display>,String>{
 let f:IDXGIFactory1=unsafe{CreateDXGIFactory1()}.map_err(|e|format!("DXGI factory unavailable: {e}"))?;
 let mut outputs=Vec::new();
 for i in 0..16{
  let Ok(adapter)=(unsafe{f.EnumAdapters1(i)})else{break;};
  if let Ok(desc)=unsafe{adapter.GetDesc1()}{if desc.VendorId==0x1414{continue;}}
  for j in 0..16{
   let Ok(output)=(unsafe{adapter.EnumOutputs(j)})else{break;};
   let Ok(v6)=output.cast::<IDXGIOutput6>()else{continue;};
   let desc=match unsafe{v6.GetDesc1()}{Ok(d)=>d,Err(_)=>continue};
   let name=String::from_utf16_lossy(&desc.DeviceName).trim_end_matches('\0').to_owned();
   if name.is_empty(){continue;}
   outputs.push(Display{name,bits:desc.BitsPerColor,
    hdr10_active:desc.ColorSpace==DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020||desc.ColorSpace==DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709,
    peak_nits:sane_nits(desc.MaxLuminance),
    min_nits:(desc.MinLuminance.is_finite()&&desc.MinLuminance>=0.&&desc.MinLuminance<=10000.).then_some(desc.MinLuminance),
    full_frame_nits:sane_nits(desc.MaxFullFrameLuminance)});
  }
 }
 Ok(outputs)
}
pub fn report()->Result<String,String>{
 let ds=probe()?;
 if ds.is_empty(){return Ok("DXGI 1.6 Advanced Color metadata not available; do not assume HDR".into());}
 let mut text=String::new();
 for d in ds{
  text.push_str(&format!("{}: {} bits/channel | active HDR-compatible color space={} | max {:.0?} nits | full-screen {:.0?} nits. Own output color space must be checked separately; SDR fallback possible.\n",
   d.name,d.bits,d.hdr10_active,d.peak_nits,d.full_frame_nits));
 }
 Ok(text)
}
#[cfg(test)]mod tests{
 #[test]fn invalid_luminance_metadata_is_not_treated_as_real(){
  assert_eq!(super::sane_nits(f32::NAN),None);
  assert_eq!(super::sane_nits(f32::INFINITY),None);
  assert_eq!(super::sane_nits(-10.),None);
  assert_eq!(super::sane_nits(1000.),Some(1000.));
 }
}


/// A user-activated starting estimate from DXGI metadata. This is NOT
/// instrumented color calibration: measured nit meters / ICC are external.
pub fn suggested_white(displays:&[Display],selected:&str)->Option<f32>{
 let d=displays.iter().find(|v|v.name.eq_ignore_ascii_case(selected)&&v.hdr10_active&&v.bits>=10)?;
 let full=d.full_frame_nits.filter(|n|*n>=100.&&*n<=2000.)?;
 let peak=d.peak_nits.filter(|n|*n>=full&&*n<=10000.)?;
 // Keep SDR reference well below measured full-frame and peak brightness;
 // no assumption that source-game SDR can recover HDR detail.
 Some((full*0.5).min(peak*0.35).clamp(80.,300.))
}
#[cfg(test)]mod v102_metadata_suggestions{
 use super::*;
 #[test]fn only_named_active_monitor_allows_suggestion(){
  let displays=vec![Display{name:"\\.\\DISPLAY1".into(),bits:10,hdr10_active:true,peak_nits:Some(600.),min_nits:Some(0.02),full_frame_nits:Some(400.)}];
  assert_eq!(suggested_white(&displays,"\\.\\DISPLAY1"),Some(200.));
  assert_eq!(suggested_white(&displays,"DISPLAY2"),None);
  let mut invalid=displays.clone();invalid[0].hdr10_active=false;
  assert_eq!(suggested_white(&invalid,"\\.\\DISPLAY1"),None);
  invalid[0].hdr10_active=true;invalid[0].full_frame_nits=None;
  assert_eq!(suggested_white(&invalid,"\\.\\DISPLAY1"),None);
 }
 #[test]fn physical_metadata_limits_clamp_estimate(){
  let v=Display{name:"M".into(),bits:10,hdr10_active:true,peak_nits:Some(1500.),min_nits:None,full_frame_nits:Some(1200.)};
  assert_eq!(suggested_white(&[v],"M"),Some(300.));
 }
}

