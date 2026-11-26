use std::{path::{Path,PathBuf},process::Command,time::Duration};
fn powershell()->Result<PathBuf,String>{let p=PathBuf::from(std::env::var_os("SystemRoot").ok_or("Windows directory missing")?).join("System32/WindowsPowerShell/v1.0/powershell.exe");if p.is_file(){Ok(p)}else{Err("Windows PowerShell unavailable".into())}}
fn command(script:&str)->Result<Command,String>{let mut c=Command::new(powershell()?);c.args(["-NoProfile","-NonInteractive","-Command",script]);Ok(c)}
const OCR:&str=r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null=[Windows.Storage.StorageFile,Windows.Storage,ContentType=WindowsRuntime]
$null=[Windows.Storage.Streams.IRandomAccessStream,Windows.Storage.Streams,ContentType=WindowsRuntime]
$null=[Windows.Graphics.Imaging.BitmapDecoder,Windows.Graphics.Imaging,ContentType=WindowsRuntime]
$null=[Windows.Graphics.Imaging.SoftwareBitmap,Windows.Graphics.Imaging,ContentType=WindowsRuntime]
$null=[Windows.Media.Ocr.OcrEngine,Windows.Foundation,ContentType=WindowsRuntime]
$null=[Windows.Media.Ocr.OcrResult,Windows.Foundation,ContentType=WindowsRuntime]
$m=[System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {$_.Name -eq 'AsTask' -and $_.IsGenericMethodDefinition -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name.StartsWith('IAsyncOperation')} | Select-Object -First 1
function Await($op,$type){$task=$m.MakeGenericMethod($type).Invoke($null,@($op));$task.Wait();$task.Result}
$f=Await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($env:HB_LOCAL_OCR_FILE)) ([Windows.Storage.StorageFile])
$s=Await ($f.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
try{
$d=Await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($s)) ([Windows.Graphics.Imaging.BitmapDecoder])
$b=Await ($d.GetSoftwareBitmapAsync([Windows.Graphics.Imaging.BitmapPixelFormat]::Bgra8,[Windows.Graphics.Imaging.BitmapAlphaMode]::Premultiplied)) ([Windows.Graphics.Imaging.SoftwareBitmap])
try{$e=[Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages();if($null -eq $e){throw 'No installed OCR language'};$v=Await ($e.RecognizeAsync($b)) ([Windows.Media.Ocr.OcrResult]);[Console]::WriteLine($v.Text)}finally{$b.Dispose()}
}finally{$s.Dispose()}
"#;
pub fn ocr(path:&Path)->Result<String,String>{
 let im=crate::suite_finish::load_reference(path)?;let p=std::env::temp_dir().join(format!("hb-ocr-{}.png",crate::media_tools::stamp()));
 im.save(&p).map_err(|e|e.to_string())?;
 let result=(||{let mut c=command(OCR)?;c.env("HB_LOCAL_OCR_FILE",&p);crate::media_tools::bounded(&mut c,Duration::from_secs(20)).map(|s|s.trim().chars().take(16000).collect())})();
 let _=std::fs::remove_file(p);result
}
const VOICE:&str=r#"
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Speech
$r=[System.Speech.Recognition.SpeechRecognitionEngine]::new()
try{
$r.SetInputToDefaultAudioDevice()
$g=[System.Speech.Recognition.GrammarBuilder]::new()
$g.Culture=$r.RecognizerInfo.Culture
$g.Append('save clip')
$r.LoadGrammar([System.Speech.Recognition.Grammar]::new($g))
$r.InitialSilenceTimeout=[TimeSpan]::FromSeconds(4)
$v=$r.Recognize([TimeSpan]::FromSeconds(6))
if($null -ne $v -and $v.Text -eq 'save clip' -and $v.Confidence -ge 0.80){[Console]::WriteLine('SAVE_CLIP')}else{[Console]::WriteLine('NO_MATCH')}
}finally{$r.Dispose()}
"#;
pub fn listen_once()->Result<bool,String>{let mut c=command(VOICE)?;crate::media_tools::bounded(&mut c,Duration::from_secs(12)).map(|s|s.trim()=="SAVE_CLIP")}
#[cfg(test)]mod tests{use super::*;#[test]fn local_assist_fixed_scripts(){assert!(powershell().is_ok());assert!(OCR.contains("RecognizeAsync"));assert!(VOICE.contains("SetInputToDefaultAudioDevice"));assert!(!OCR.contains("Invoke-WebRequest"));}}
