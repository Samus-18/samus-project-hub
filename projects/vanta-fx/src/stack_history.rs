#[derive(Clone,Default)]
pub struct History{past:Vec<Vec<usize>>,future:Vec<Vec<usize>>}
impl History{
 pub fn record(&mut self,old:Vec<usize>){if self.past.len()>=32{self.past.remove(0);}self.past.push(old);self.future.clear();}
 pub fn undo(&mut self,current:Vec<usize>)->Option<Vec<usize>>{let prev=self.past.pop()?;self.future.push(current);Some(prev)}
 pub fn redo(&mut self,current:Vec<usize>)->Option<Vec<usize>>{let next=self.future.pop()?;self.past.push(current);Some(next)}
 pub fn can_undo(&self)->bool{!self.past.is_empty()}
 pub fn can_redo(&self)->bool{!self.future.is_empty()}
}
#[cfg(test)]mod tests{
 #[test]fn ordered_undo_redo_and_branch(){
  let mut h=super::History::default();h.record(vec![1,2]);
  assert_eq!(h.undo(vec![2,1]),Some(vec![1,2]));assert_eq!(h.redo(vec![1,2]),Some(vec![2,1]));
  h.undo(vec![2,1]);h.record(vec![1,2]);assert!(!h.can_redo());
  for _ in 0..100{h.record(vec![1]);}assert_eq!(h.past.len(),32);
 }
}
