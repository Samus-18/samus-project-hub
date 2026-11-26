use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]pub struct Options{pub enabled:bool,pub temporal:bool,pub seconds:f32,pub reset_delta:f32}
impl Default for Options{fn default()->Self{Self{enabled:false,temporal:false,seconds:0.5,reset_delta:0.45}}}
impl Options{pub fn clamp(&mut self){self.seconds=if self.seconds.is_finite(){self.seconds.clamp(0.05,3.)}else{0.5};self.reset_delta=if self.reset_delta.is_finite(){self.reset_delta.clamp(0.1,0.75)}else{0.45};}}
#[cfg(test)]mod tests{
 #[test]fn safe_defaults(){let mut o:super::Options=serde_json::from_str("{}").unwrap();assert!(!o.enabled&&!o.temporal);o.seconds=f32::NAN;o.reset_delta=99.;o.clamp();assert_eq!(o.seconds,0.5);assert_eq!(o.reset_delta,0.75);}
}
