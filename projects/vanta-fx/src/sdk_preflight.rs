//! Offline, fail-closed prerequisite discovery for OPTIONAL vendor SDK backends.
//! Finding a header or static library is NOT having a working GPU renderer.
//! Never downloads SDKs, loads arbitrary binaries, changes game processes,
//! or represents source-only scaffolding as official AMD/Windows integration.
use std::path::{Path,PathBuf};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SdkPresence {
 pub source_header:bool,
 pub opticalflow_library:bool,
 pub source_name:String,
 pub note:&'static str,
}
pub fn fidelityfx_root()->PathBuf{
 std::env::var_os("HORIZON_FIDELITYFX_SDK")
  .map(PathBuf::from)
  .unwrap_or_else(||PathBuf::from("engines/FidelityFX-SDK"))
}
pub fn check_fidelityfx(root:&Path)->SdkPresence{
 let headers=[
  "Kits/FidelityFX/framegeneration/fsr3/include/ffx_opticalflow.h",
  "sdk/include/FidelityFX/host/ffx_opticalflow.h",
 ];
 let libraries=[
  "sdk/bin/ffx_sdk/ffx_opticalflow_x64.lib",
  "lib/ffx_opticalflow_x64.lib",
 ];
 let header=headers.iter().any(|p|root.join(p).is_file());
 let lib=libraries.iter().any(|p|root.join(p).is_file());
 SdkPresence{source_header:header,opticalflow_library:lib,
  source_name:"AMD FidelityFX Optical Flow".into(),
  note:"SDK prerequisite discovery only. This is NOT a working official Optical Flow dispatch; native DX12 backend and WGC bridge remain required."
 }
}
pub fn report()->String{
 let root=fidelityfx_root();
 let amd=check_fidelityfx(&root);
 format!("AMD official OF SDK: header={} static library={} root={} (native integration NOT active)\nXbox Game Bar: UWP/XAML widget requires separate SDK/MSIX package, Microsoft Store association and secure per-package IPC; not enabled/built. No Win32 overlay can be called an official Game Bar widget.\nSoftware backend: WGC -> D3D11 native own preview remains the only live native GPU renderer. D3D12/Vulkan compute are separate GPU diagnostic proofs; NOT selectable real-time processing backends.",
 amd.source_header,amd.opticalflow_library,root.display())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn missing_sdk_must_not_report_a_working_vendor_backend(){
  let temp=std::env::temp_dir().join(format!("hb-not-installed-{}",std::process::id()));
  let p=check_fidelityfx(&temp);assert!(!p.source_header&&!p.opticalflow_library);
  assert!(p.note.contains("NOT"));
 }
 #[test]fn file_presence_is_only_a_prerequisite_not_execution(){
  let dir=std::env::temp_dir().join(format!("hb-v102-sdk-{}",std::process::id()));
  let h=dir.join("Kits/FidelityFX/framegeneration/fsr3/include/ffx_opticalflow.h");
  let lib=dir.join("sdk/bin/ffx_sdk/ffx_opticalflow_x64.lib");
  std::fs::create_dir_all(h.parent().unwrap()).unwrap();std::fs::write(&h,b"offline test").unwrap();
  std::fs::create_dir_all(lib.parent().unwrap()).unwrap();std::fs::write(&lib,b"fixture not a real binary").unwrap();
  let p=check_fidelityfx(&dir);assert!(p.source_header&&p.opticalflow_library);assert!(p.note.contains("NOT a working"));
  std::fs::remove_dir_all(dir).unwrap();
 }
}