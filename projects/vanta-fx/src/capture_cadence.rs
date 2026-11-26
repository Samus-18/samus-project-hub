//! Optional OS capture cadence, with the app's existing pacing as fallback.
#[derive(Default)]pub struct Control{last:Option<u32>,disabled:bool}
pub fn interval_ticks(cap:u32)->i64{if cap==0{0}else{{let fps=i64::from(crate::preview_pacer::valid_limit(cap));(10_000_000+fps-1)/fps}}}
impl Control{
 pub fn change(&mut self,cap:u32)->Option<i64>{let cap=crate::preview_pacer::valid_limit(cap);if self.disabled||self.last==Some(cap){return None;}self.last=Some(cap);Some(interval_ticks(cap))}
 pub fn disable(&mut self){self.disabled=true;}
}
pub fn duplicate(previous:Option<i64>,current:Option<i64>)->bool{matches!((previous,current),(Some(a),Some(b)) if a>0&&a==b)}
#[cfg(test)]mod tests{use super::*;
 #[test]fn interval_changes_only_once_and_never_limits_game(){let mut c=Control::default();assert_eq!(c.change(30),Some(333334));assert_eq!(c.change(30),None);assert_eq!(c.change(60),Some(166667));assert_eq!(c.change(0),Some(0));c.disable();assert_eq!(c.change(45),None);}
 #[test]fn unknown_or_reversed_clocks_do_not_discard_a_frame(){assert!(duplicate(Some(25),Some(25)));assert!(!duplicate(Some(25),Some(26)));assert!(!duplicate(Some(25),Some(24)));assert!(!duplicate(None,None));assert!(!duplicate(Some(0),Some(0)));}
}
