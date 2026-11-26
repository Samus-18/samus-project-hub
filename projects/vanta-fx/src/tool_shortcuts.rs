//! User-selectable Ctrl+Shift chords. No keyboard hooks and no logging.
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]
pub struct Shortcuts{pub macro_key:u16,pub replay_key:u16}
impl Default for Shortcuts{fn default()->Self{Self{macro_key:0x55,replay_key:0x4c}}}
pub const KEYS:[(u16,&str);8]=[(0x55,"U"),(0x4c,"L"),(0x70,"F1"),(0x71,"F2"),(0x72,"F3"),(0x73,"F4"),(0x74,"F5"),(0x75,"F6")];
pub fn label(key:u16)->&'static str{KEYS.iter().find(|x|x.0==key).map(|x|x.1).unwrap_or("?")}
pub const RESERVED_CTRL_SHIFT:[u16;13]=[0x48,0x47,0x4A,0x4B,0x42,0x58,0x59,0x54,0x4D,0x50,0x52,0x4E,0x7B];
impl Shortcuts{
    pub fn valid(&self)->bool{self.macro_key!=self.replay_key&&label(self.macro_key)!="?"&&label(self.replay_key)!="?"&&!RESERVED_CTRL_SHIFT.contains(&self.macro_key)&&!RESERVED_CTRL_SHIFT.contains(&self.replay_key)}
    pub fn sanitize(&mut self){if !self.valid(){*self=Self::default();}}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn conflicts_and_unsupported_keys_rejected(){let mut s=Shortcuts{macro_key:0x55,replay_key:0x55};assert!(!s.valid());s.sanitize();assert!(s.valid());}
}

#[cfg(test)]mod reservation_tests{use super::*;#[test]fn all_fixed_ctrl_shift_bindings_and_mutable_options_are_disjoint(){let mut r=RESERVED_CTRL_SHIFT.to_vec();r.sort_unstable();r.dedup();assert_eq!(r.len(),RESERVED_CTRL_SHIFT.len());for (v,_)in KEYS{assert!(!r.contains(&v));}assert!(Shortcuts::default().valid());}}
