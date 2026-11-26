//! Single producer/single consumer bounded Copy-only ring, no mutex or allocation per push.
use std::{cell::{UnsafeCell,Cell},marker::PhantomData,mem::MaybeUninit,sync::{Arc,atomic::{AtomicUsize,AtomicBool,Ordering}}};
#[repr(align(64))]struct Cursor(AtomicUsize);
struct Shared<T:Copy,const N:usize>{slots:[UnsafeCell<MaybeUninit<T>>;N],write:Cursor,read:Cursor,consumer:AtomicBool}
 // Safety: endpoints are unique and !Sync. Producer writes only to unoccupied slots;
 // consumer reads only published slots; acquire/release cursor pairs transfer ownership.
unsafe impl<T:Copy+Send,const N:usize> Sync for Shared<T,N>{}
pub struct Tx<T:Copy,const N:usize>{inner:Arc<Shared<T,N>>,_unique:PhantomData<Cell<()>>}
pub struct Rx<T:Copy,const N:usize>{inner:Arc<Shared<T,N>>,_unique:PhantomData<Cell<()>>}
#[derive(Debug,PartialEq,Eq)]pub enum PushError{Full,Closed}
pub fn channel<T:Copy+Send,const N:usize>()->(Tx<T,N>,Rx<T,N>){
 assert!(N.is_power_of_two()&&N>=2&&N<=65536);
 let a=Arc::new(Shared{slots:std::array::from_fn(|_|UnsafeCell::new(MaybeUninit::uninit())),write:Cursor(AtomicUsize::new(0)),read:Cursor(AtomicUsize::new(0)),consumer:AtomicBool::new(true)});
 (Tx{inner:a.clone(),_unique:PhantomData},Rx{inner:a,_unique:PhantomData})
}
impl<T:Copy,const N:usize> Tx<T,N>{
 pub fn push(&self,item:T)->Result<(),PushError>{
  let a=&self.inner;if !a.consumer.load(Ordering::Acquire){return Err(PushError::Closed);}
  let w=a.write.0.load(Ordering::Relaxed);let r=a.read.0.load(Ordering::Acquire);
  if w.wrapping_sub(r)>=N{return Err(PushError::Full);}
  unsafe{(*a.slots[w&(N-1)].get()).write(item);}
  a.write.0.store(w.wrapping_add(1),Ordering::Release);Ok(())
 }
}
impl<T:Copy,const N:usize> Rx<T,N>{
 pub fn pop(&self)->Option<T>{
  let a=&self.inner;let r=a.read.0.load(Ordering::Relaxed);let w=a.write.0.load(Ordering::Acquire);
  if r==w{return None;}
  let v=unsafe{(*a.slots[r&(N-1)].get()).assume_init_read()};
  a.read.0.store(r.wrapping_add(1),Ordering::Release);Some(v)
 }
}
impl<T:Copy,const N:usize> Drop for Rx<T,N>{fn drop(&mut self){self.inner.consumer.store(false,Ordering::Release);}}
#[cfg(test)]mod tests{use super::*;
 #[test]fn bounds_wrap_and_closed(){let(t,r)=channel::<u64,4>();for i in 0..4{t.push(i).unwrap();}assert_eq!(t.push(4),Err(PushError::Full));assert_eq!(r.pop(),Some(0));t.push(4).unwrap();for i in 1..5{assert_eq!(r.pop(),Some(i));}assert_eq!(r.pop(),None);drop(r);assert_eq!(t.push(5),Err(PushError::Closed));}
 #[test]fn concurrent_sequence_never_corrupts(){let(t,r)=channel::<(u64,u64),512>();let j=std::thread::spawn(move||{for i in 0..100000{loop{match t.push((i,!i)){Ok(())=>break,Err(PushError::Full)=>std::thread::yield_now(),Err(PushError::Closed)=>panic!("consumer closed")}}}});for i in 0..100000{loop{if let Some(v)=r.pop(){assert_eq!(v,(i,!i));break;}std::thread::yield_now();}}j.join().unwrap();}
}
