use std::{io::{Read,Write},net::{TcpListener,TcpStream,SocketAddr},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},thread,time::Duration};
#[derive(Clone,Default,serde::Serialize)]
pub struct Metrics{pub fps:Option<f32>,pub frametime_ms:Option<f32>,pub low_1_percent:Option<f32>,pub output_active:bool}
const HTML:&str=r#"<!doctype html><html><meta name="viewport" content="width=device-width"><title>Horizon Boost local</title><style>body{background:#090a10;color:#eee;font:16px system-ui;padding:24px}h1{color:#ff5271}section{max-width:560px;background:#181a24;border:1px solid #414354;border-radius:14px;padding:20px}strong{font-size:32px}p{color:#aab3c3}</style><h1>HORIZON BOOST</h1><section><p>READ-ONLY LOCAL TELEMETRY</p><strong id="fps">-- FPS</strong><p id="info">Waiting for measured data</p><p id="gpu"></p></section><script>async function update(){try{let r=await fetch('/metrics',{cache:'no-store'});if(!r.ok)throw 0;let m=await r.json();document.getElementById('fps').textContent=m.fps==null?'-- FPS':m.fps.toFixed(1)+' FPS';document.getElementById('info').textContent=(m.frametime_ms==null?'--':m.frametime_ms.toFixed(2))+' ms / 1% low '+(m.low_1_percent==null?'--':m.low_1_percent.toFixed(1));document.getElementById('gpu').textContent='Own GPU output: '+(m.output_active?'active':'off')}catch(e){document.getElementById('fps').textContent='Disconnected'}}update();setInterval(update,1500)</script></html>"#;
fn serve(mut stream:TcpStream,m:&Arc<Mutex<Metrics>>){
 let _=stream.set_read_timeout(Some(Duration::from_millis(300)));
 let _=stream.set_write_timeout(Some(Duration::from_millis(300)));
 let mut buf=[0u8;2048];let Ok(n)=stream.read(&mut buf)else{return;};
 let Ok(text)=std::str::from_utf8(&buf[..n])else{return;};
 let first=text.lines().next().unwrap_or("");
 // No CORS, no writes, no file serving and no query parameters.
 let (status,kind,body)=match first{
  "GET / HTTP/1.1"|"GET / HTTP/1.0"=>("200 OK","text/html; charset=utf-8",HTML.to_owned()),
  "GET /metrics HTTP/1.1"|"GET /metrics HTTP/1.0"=>("200 OK","application/json",m.lock().ok().and_then(|v|serde_json::to_string(&*v).ok()).unwrap_or_else(||"{}".into())),
  _=>("404 Not Found","text/plain","Not found".into()),
 };
 let header=format!("HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",body.len());
 let _=stream.write_all(header.as_bytes());let _=stream.write_all(body.as_bytes());
}
#[derive(Default)]
pub struct Companion{run:Option<Arc<AtomicBool>>,data:Arc<Mutex<Metrics>>,worker:Option<thread::JoinHandle<()>>,pub addr:Option<SocketAddr>}
impl Companion{
 pub fn start(&mut self)->Result<(),String>{
  if self.addr.is_some(){return Ok(());}
  let listener=TcpListener::bind("127.0.0.1:0").map_err(|e|e.to_string())?;
  listener.set_nonblocking(true).map_err(|e|e.to_string())?;
  let addr=listener.local_addr().map_err(|e|e.to_string())?;
  let run=Arc::new(AtomicBool::new(true));let active=run.clone();let data=self.data.clone();
  let t=thread::Builder::new().name("Horizon local dashboard".into()).spawn(move||{
   while active.load(Ordering::Acquire){
    match listener.accept(){
     Ok((stream,peer)) if peer.ip().is_loopback()=>serve(stream,&data),
     Ok(_)=>{},
     Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(80)),
     Err(_)=>break,
    }
   }
  }).map_err(|e|e.to_string())?;
  self.addr=Some(addr);self.run=Some(run);self.worker=Some(t);Ok(())
 }
 pub fn update(&self,m:Metrics){if self.addr.is_some(){if let Ok(mut d)=self.data.lock(){*d=m;}}}
 pub fn stop(&mut self){if let Some(r)=self.run.take(){r.store(false,Ordering::Release);}if let Some(t)=self.worker.take(){let _=t.join();}self.addr=None;}
}
impl Drop for Companion{fn drop(&mut self){self.stop();}}
pub fn proof()->Result<String,String>{
 let mut c=Companion::default();c.start()?;
 let addr=c.addr.ok_or("No loopback address")?;
 if !addr.ip().is_loopback(){return Err("Not loopback".into());}
 c.update(Metrics{fps:Some(60.0),..Default::default()});
 let mut s=TcpStream::connect(addr).map_err(|e|e.to_string())?;
 s.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
 s.write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").map_err(|e|e.to_string())?;
 let mut response=String::new();s.read_to_string(&mut response).map_err(|e|e.to_string())?;
 c.stop();
 if !response.contains("200 OK")||!response.contains("\"fps\":60.0"){return Err("Invalid metrics response".into());}
 Ok("LOCAL_COMPANION=PASS LOOPBACK_ONLY=true READ_ONLY=true SYNTHETIC_METRIC=true".into())
}
#[cfg(test)]mod tests{#[test]fn real_loopback_http(){assert!(super::proof().is_ok());}}
