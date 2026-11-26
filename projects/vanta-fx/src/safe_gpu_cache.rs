//! Transactional optional stage creation; no failed retry every frame.
pub struct Cache<K,V>{good:Option<(K,V)>,failed:Option<K>}
impl<K,V> Default for Cache<K,V>{fn default()->Self{Self{good:None,failed:None}}}
impl<K:PartialEq+Clone,V> Cache<K,V>{
 pub fn clear(&mut self){self.good=None;self.failed=None;}
 pub fn current(&self)->Option<&V>{self.good.as_ref().map(|(_,v)|v)}
 pub fn ensure(&mut self,k:&K,allow_old:bool,build:impl FnOnce()->Result<V,String>)->(Option<&V>,bool){
  if self.good.as_ref().is_some_and(|(key,_)|key==k){self.failed=None;return (self.current(),false);}
  if self.failed.as_ref()!=Some(k){
   match build(){Ok(v)=>{self.good=Some((k.clone(),v));self.failed=None;return (self.current(),false);},Err(_)=>self.failed=Some(k.clone())}
  }
  (if allow_old{self.current()}else{None},true)
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn keeps_previous_and_quarantines_once(){let mut c=Cache::<u8,u8>::default();assert_eq!(c.ensure(&1,true,||Ok(4)),(Some(&4),false));assert_eq!(c.ensure(&2,true,||Err("bad".into())),(Some(&4),true));assert_eq!(c.ensure(&2,true,||panic!("retried")),(Some(&4),true));assert_eq!(c.ensure(&2,false,||panic!("retried")),(None,true));assert_eq!(c.ensure(&3,true,||Ok(8)),(Some(&8),false));c.clear();assert_eq!(c.ensure(&2,true,||Err("bad".into())),(None,true));}
}
