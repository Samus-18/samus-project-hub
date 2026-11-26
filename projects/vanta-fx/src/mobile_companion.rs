//! A small, explicitly opt-in LAN server with a random per-run read-only token.
//! No writes, no cross-origin permissions, no user files and no remote controls.
//! HTTP is unencrypted: ONLY use on a trusted local network; off by default.
use std::{net::{TcpListener,TcpStream,SocketAddr},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},time::Duration,thread,io::{Read,Write}};
use crate::local_companion::Metrics;
fn serve_one(mut stream:TcpStream,metrics:&Arc<Mutex<Metrics>>,token:&str){
 let _=stream.set_read_timeout(Some(Duration::from_millis(300)));
 let _=stream.set_write_timeout(Some(Duration::from_millis(300)));
 let mut data=[0u8;2048];let Ok(n)=stream.read(&mut data)else{return;};
 let Ok(line)=std::str::from_utf8(&data[..n])else{return;};
 let first=line.split("\r\n").next().unwrap_or("");
 let token_home=format!("GET /?key={token} HTTP/1.1");
 let token_metrics=format!("GET /metrics?key={token} HTTP/1.1");
 let token_home10=format!("GET /?key={token} HTTP/1.0");
 let token_metrics10=format!("GET /metrics?key={token} HTTP/1.0");
 let (status,kind,body)=if first==token_home||first==token_home10{
   let html=format!(r#"<!doctype html><html><meta name="viewport" content="width=device-width"><title>Horizon Boost</title>
   <body style="background:#111;color:#eee;font:16px system-ui;padding:24px">
   <h2>Horizon Boost - read-only private LAN telemetry</h2>
   <div id="out">Waiting for measured game FPS...</div>
   <script>async function update(){{try{{let r=await fetch('/metrics?key={token}',{{cache:'no-store'}});if(!r.ok)throw Error('offline');
   let s=await r.json();document.getElementById('out').textContent=(s.fps??'--')+' game FPS / '+(s.frametime_ms??'--')+' ms; own preview '+(s.output_active?'on':'off');}}
   catch(_e){{document.getElementById('out').textContent='Unavailable';}}}}update();setInterval(update,1500)</script>
   <small>Local network only. Do not share this address or use untrusted Wi-Fi.</small></body></html>"#);
   ("200 OK","text/html; charset=utf-8",html)
 }else if first==token_metrics||first==token_metrics10{
   let data=metrics.lock().ok().and_then(|m|serde_json::to_string(&*m).ok()).unwrap_or_else(||"{}".into());
   ("200 OK","application/json",data)
 }else{
   ("403 Forbidden","text/plain","Read-only dashboard authorization required".into())
 };
 let headers=format!("HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self' 'unsafe-inline'\r\n\r\n",body.len());
 let _=stream.write_all(headers.as_bytes());let _=stream.write_all(body.as_bytes());
}
#[derive(Default)]
pub struct Server{worker:Option<thread::JoinHandle<()>>,run:Option<Arc<AtomicBool>>,data:Arc<Mutex<Metrics>>,pub addr:Option<SocketAddr>,pub token:Option<String>,lan_ip:Option<std::net::IpAddr>}
impl Server{
 pub fn start(&mut self)->Result<(),String>{
  if self.addr.is_some(){return Ok(());}
  let token=crate::secure_random::token()?;
  let l=TcpListener::bind("0.0.0.0:0").map_err(|e|format!("Mobile LAN listener: {e}"))?;
  l.set_nonblocking(true).map_err(|e|e.to_string())?;
  let addr=l.local_addr().map_err(|e|e.to_string())?;
  let run=Arc::new(AtomicBool::new(true));let active=run.clone();
  let m=self.data.clone();let auth=token.clone();
  let worker=thread::Builder::new().name("Horizon mobile LAN read-only".into()).spawn(move||{
   while active.load(Ordering::Acquire){
    match l.accept(){
     Ok((stream,_peer))=>serve_one(stream,&m,&auth),
     Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(80)),
     Err(_)=>break,
    }
   }
  }).map_err(|e|e.to_string())?;
  self.lan_ip=std::net::UdpSocket::bind("0.0.0.0:0").ok().and_then(|s|{s.connect("192.0.2.1:9").ok()?;s.local_addr().ok().map(|a|a.ip())});
  self.token=Some(token);self.addr=Some(addr);self.run=Some(run);self.worker=Some(worker);Ok(())
 }
 pub fn update(&self,m:Metrics){if self.addr.is_some(){if let Ok(mut v)=self.data.lock(){*v=m;}}}
 pub fn stop(&mut self){
  if let Some(r)=self.run.take(){r.store(false,Ordering::Release);}
  if let Some(t)=self.worker.take(){let _=t.join();}
  self.addr=None;self.token=None;self.lan_ip=None;
 }
 pub fn connection_hint(&self)->Option<String>{
  let a=self.addr?;let t=self.token.as_deref()?;
  let ip=self.lan_ip.map_or_else(||"YOUR_PC_LAN_IP".into(),|v|v.to_string());
  Some(format!("http://{ip}:{}/?key={t}",a.port()))
 }
}
impl Drop for Server{fn drop(&mut self){self.stop()}}
#[cfg(test)]mod tests{
 #[test]fn defaults_disabled_no_http_listener(){let s=super::Server::default();assert!(s.addr.is_none());assert!(s.token.is_none());assert!(s.connection_hint().is_none());}
}


#[cfg(test)]mod http_auth_tests{
 use super::*;
 #[test]fn localhost_http_requires_exact_token_and_is_read_only(){
  fn read(route:&str)->String{
   let listener=TcpListener::bind("127.0.0.1:0").unwrap();
   let addr=listener.local_addr().unwrap();
   let metrics=Arc::new(Mutex::new(Metrics{fps:Some(60.),..Default::default()}));
   let worker=std::thread::spawn(move||{let (s,_)=listener.accept().unwrap();serve_one(s,&metrics,"testtoken");});
   let mut socket=TcpStream::connect(addr).unwrap();
   let request=format!("GET {route} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
   socket.write_all(request.as_bytes()).unwrap();
   let mut response=String::new();socket.read_to_string(&mut response).unwrap();worker.join().unwrap();response
  }
  assert!(read("/metrics?key=wrong").contains("403 Forbidden"));
  assert!(read("/metrics").contains("403 Forbidden"));
  let ok=read("/metrics?key=testtoken");
  assert!(ok.contains("200 OK")&&ok.contains("\"fps\":60.0"));
  assert!(!ok.contains("Access-Control-Allow-Origin"));
 }
}
