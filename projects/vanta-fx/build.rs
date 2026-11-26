use std::{env,path::PathBuf,process::Command};
fn main() {
    if !env::var("TARGET").unwrap_or_default().contains("windows") {return;}
    let out=PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let obj=out.join("horizon-icon.o");
    let bundled=PathBuf::from("toolchain/ucrt64/bin/windres.exe");
    let windres=if bundled.is_file(){bundled}else{PathBuf::from("windres")};
    // A bare Windows developer environment may have GNU windres but no GCC.
    // The RC contains only one ICON directive, so the included transparent
    // passthrough can parse it without a full C preprocessor.
    let run = |preprocessor:Option<&str>| {
        let mut command=Command::new(&windres);
        command.args(["--input","horizon.rc","--output-format","coff"]);
        if let Some(pp)=preprocessor{command.arg(format!("--preprocessor={pp}"));}
        command.arg("--output").arg(&obj);
        command.status().map(|s|s.success()).unwrap_or(false)
    };
    let ok=run(Some("python assets/rc_passthrough.py"))||run(None);
    if ok{
        println!("cargo:rustc-link-arg={}",obj.display());
    }else{
        println!("cargo:warning=Native icon resource not embedded; runtime icon still available");
    }
    println!("cargo:rerun-if-changed=horizon.rc");
    println!("cargo:rerun-if-changed=assets/vanta.ico");
    println!("cargo:rerun-if-changed=assets/rc_passthrough.py");
}
