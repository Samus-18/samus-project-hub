//! Bounded lock-free SPSC GPU texture transfer queue; zero heap allocation on
//! push/pop. One producer + one consumer, no clones and no hidden frame pileup.
//! This is a real GPU resource queue foundation; WGC render handoff integration
//! still requires device context synchronization / per-frame fences.
use std::{sync::{Arc,atomic::{AtomicU64,Ordering}},cell::UnsafeCell,mem::MaybeUninit};
const CAP:u64=3;
const CYCLE:u64=CAP*2;
fn distance(head:u64,tail:u64)->u64{(head+CYCLE-tail)%CYCLE}
struct Slot<T>(UnsafeCell<MaybeUninit<T>>);
unsafe impl<T:Send> Sync for Slot<T>{}
struct Inner<T>{slots:[Slot<T>;CAP as usize],written:AtomicU64,read:AtomicU64}
unsafe impl<T:Send> Sync for Inner<T>{}
impl<T> Drop for Inner<T>{
 fn drop(&mut self){
  let start=self.read.load(Ordering::Relaxed);
  let end=self.written.load(Ordering::Relaxed);
  for i in 0..distance(end,start){let sequence=(start+i)%CYCLE;
   unsafe{(*self.slots[(sequence%CAP)as usize].0.get()).assume_init_drop();}
  }
 }
}
pub struct Producer<T>{inner:Arc<Inner<T>>}
pub struct Consumer<T>{inner:Arc<Inner<T>>}
pub fn channel<T:Send>()->(Producer<T>,Consumer<T>){
 let inner=Arc::new(Inner{
  slots:std::array::from_fn(|_|Slot(UnsafeCell::new(MaybeUninit::uninit()))),
  written:AtomicU64::new(0),read:AtomicU64::new(0),
 });
 (Producer{inner:inner.clone()},Consumer{inner})
}
impl<T:Send> Producer<T>{
 pub fn pending(&self)->usize{distance(self.inner.written.load(Ordering::Acquire),self.inner.read.load(Ordering::Acquire)).min(CAP) as usize}
 pub fn capacity(&self)->usize{CAP as usize}
 pub fn push(&mut self,item:T)->Result<(),T>{
  let head=self.inner.written.load(Ordering::Relaxed);
  let tail=self.inner.read.load(Ordering::Acquire);
  if distance(head,tail)>=CAP{return Err(item);}
  unsafe{(*self.inner.slots[(head%CAP)as usize].0.get()).write(item);}
  self.inner.written.store((head+1)%CYCLE,Ordering::Release);
  Ok(())
 }
}
impl<T:Send> Consumer<T>{
 pub fn pending(&self)->usize{distance(self.inner.written.load(Ordering::Acquire),self.inner.read.load(Ordering::Acquire)).min(CAP) as usize}
 pub fn pop(&mut self)->Option<T>{
  let tail=self.inner.read.load(Ordering::Relaxed);
  if tail==self.inner.written.load(Ordering::Acquire){return None;}
  let item=unsafe{(*self.inner.slots[(tail%CAP)as usize].0.get()).assume_init_read()};
  self.inner.read.store((tail+1)%CYCLE,Ordering::Release);
  Some(item)
 }
}
pub struct GpuFrame{
 pub texture:windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
 pub capture_timestamp:i64,
 pub gpu_fence_value:u64,
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn bounded_exact_order_and_full_rejection(){
  let(mut tx,mut rx)=channel::<u32>();
  assert!(rx.pop().is_none());
  for i in 0..3{tx.push(i).unwrap();}
  assert_eq!(tx.push(3),Err(3));
  for i in 0..3{assert_eq!(rx.pop(),Some(i));}
  assert_eq!(rx.pop(),None);
 }
 #[test]fn concurrent_spsc_no_loss_or_reordering(){
  const N:u64=100_000;
  let(mut tx,mut rx)=channel::<u64>();
  let p=std::thread::spawn(move||{
   for i in 0..N{
    let mut item=i;
    loop{match tx.push(item){
     Ok(())=>break,Err(v)=>{item=v;std::thread::yield_now();}
    }}
   }
  });
  for i in 0..N{
   loop{
    if let Some(x)=rx.pop(){assert_eq!(x,i);break;}
    std::thread::yield_now();
   }
  }
  p.join().unwrap();
 }
 #[test]fn actual_gpu_com_textures_hold_and_release_through_lockfree_queue(){
  let gpu=crate::photo_lab::Gpu::new().unwrap();
  let original=image::RgbaImage::from_fn(13,11,|x,y|
    image::Rgba([(x*13)as u8,(y*21)as u8,30,255]));
  let tex=gpu.upload(&original).unwrap();
  let(mut tx,mut rx)=channel::<GpuFrame>();
  for n in 0..3{tx.push(GpuFrame{texture:tex.clone(),capture_timestamp:n,gpu_fence_value:n as u64}).map_err(|_|()).unwrap();}
  assert!(tx.push(GpuFrame{texture:tex.clone(),capture_timestamp:4,gpu_fence_value:4}).is_err());
  for n in 0..3{
   let frame=rx.pop().unwrap();assert_eq!(frame.capture_timestamp,n);
   let result=gpu.read(&frame.texture).unwrap();assert_eq!(result,original);
  }
 }
}

#[cfg(test)]mod v101_pending_tests{
 use super::*;
 #[test]fn ring_has_exact_capacity_without_heap_or_hidden_backlog(){
  let(mut tx,mut rx)=channel::<u8>();assert_eq!(tx.capacity(),3);assert_eq!(tx.pending(),0);
  for i in 1..=3{tx.push(i).unwrap();assert_eq!(tx.pending(),i as usize);}
  assert!(tx.push(4).is_err());assert_eq!(rx.pending(),3);
  assert_eq!(rx.pop(),Some(1));assert_eq!(rx.pending(),2);
  assert_eq!(tx.push(4),Ok(()));assert_eq!(rx.pop(),Some(2));
  assert_eq!(rx.pop(),Some(3));assert_eq!(rx.pop(),Some(4));assert_eq!(rx.pop(),None);
 }
}
