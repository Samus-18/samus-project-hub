//! Explicit own-output budgets. Never changes game settings or OS drivers.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Profile{pub processing:u8,pub algorithm:u8,pub quality:u8,pub cap:u32,pub guard:bool}
pub const PROFILES:[Profile;3]=[
 Profile{processing:67,algorithm:11,quality:2,cap:30,guard:true},
 Profile{processing:75,algorithm:11,quality:3,cap:60,guard:true},
 Profile{processing:100,algorithm:0,quality:0,cap:60,guard:false},
];
#[cfg(test)]mod tests{use super::*;#[test]fn presets_are_bounded(){for p in PROFILES{assert!(crate::render_budget::PERCENTAGES.contains(&p.processing));assert!((p.algorithm as usize)<crate::spatial::MODES.len());assert!(crate::preview_pacer::LIMITS.contains(&p.cap));assert!(p.quality<=3);}assert!(PROFILES[0].processing<PROFILES[1].processing&&PROFILES[1].processing<PROFILES[2].processing);}}
