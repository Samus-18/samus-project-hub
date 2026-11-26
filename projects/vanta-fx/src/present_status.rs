//! DXGI submission outcome, not physical display scanout.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Outcome{Submitted,Occluded,OtherStatus,Busy,Failure}
pub fn classify(hr:i32)->Outcome{match hr{0=>Outcome::Submitted,0x087a0001=>Outcome::Occluded,n if n==0x887a000au32 as i32=>Outcome::Busy,n if n<0=>Outcome::Failure,_=>Outcome::OtherStatus}}
#[cfg(test)]mod tests{use super::*;#[test]fn only_s_ok_counts(){assert_eq!(classify(0),Outcome::Submitted);assert_eq!(classify(0x087a0001),Outcome::Occluded);assert_eq!(classify(1),Outcome::OtherStatus);assert_eq!(classify(0x887a000au32 as i32),Outcome::Busy);assert_eq!(classify(0x887a0005u32 as i32),Outcome::Failure);}}
