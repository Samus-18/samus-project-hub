use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Options{
 pub mask:bool,pub left:f32,pub top:f32,pub right:f32,pub bottom:f32,
 pub edge_protection:f32,pub consistency:f32,pub max_motion:f32,pub disocclusion_guard:f32,pub smooth_flow:bool,pub pyramid_flow:bool,pub adaptive_fg:bool,pub max_fg_age_ms:f32,pub temporal_mode:u8,pub temporal_strength:f32
}
impl Default for Options{fn default()->Self{Self{mask:false,left:0.,top:0.8,right:1.,bottom:1.,
 edge_protection:0.98,consistency:1.,max_motion:12.,disocclusion_guard:0.8,smooth_flow:false,pyramid_flow:false,adaptive_fg:false,max_fg_age_ms:85.,temporal_mode:0,temporal_strength:0.35}}}
impl Options{
 pub fn clamp(&mut self){
  let c=|x:f32,min:f32,max:f32,default:f32|if x.is_finite(){x.clamp(min,max)}else{default};
  self.left=c(self.left,0.,0.95,0.);self.top=c(self.top,0.,0.95,0.8);
  self.right=c(self.right,self.left+0.05,1.,1.);self.bottom=c(self.bottom,self.top+0.05,1.,1.);
  self.edge_protection=c(self.edge_protection,0.,1.,0.98);self.consistency=c(self.consistency,0.5,2.,1.);
  self.max_motion=c(self.max_motion,1.,12.,12.);self.disocclusion_guard=c(self.disocclusion_guard,0.,1.,0.8);
  self.max_fg_age_ms=c(self.max_fg_age_ms,25.,250.,85.);
  self.temporal_mode=self.temporal_mode.min(2);self.temporal_strength=c(self.temporal_strength,0.,0.65,0.35);
 }
 pub fn temporal_gpu(&self)->Params{let mut v=self.clone();v.clamp();let mut p=v.gpu(0);p.extras[2]=v.temporal_strength;p.extras[3]=v.temporal_mode as f32;p}
 pub fn gpu(&self,diagnostic:u8)->Params{
  let mut v=self.clone();v.clamp();
  Params{rect:[v.left,v.top,v.right,v.bottom],tuning:[v.edge_protection,v.consistency,v.max_motion,if v.mask{1.}else{0.}],
   extras:[if v.smooth_flow{1.}else{0.},diagnostic.min(2) as f32,v.disocclusion_guard,0.]}
 }
}
#[repr(C)]#[derive(Clone,Copy)]
pub struct Params{rect:[f32;4],tuning:[f32;4],extras:[f32;4]}
#[cfg(test)]mod tests{
 #[test]fn motion_options_old_defaults_and_bounds(){
  let mut o:super::Options=serde_json::from_str("{}").unwrap();assert_eq!(o,super::Options::default());
  o.left=2.;o.right=-1.;o.edge_protection=f32::NAN;o.max_motion=100.;
  o.clamp();assert_eq!(o.left,0.95);assert_eq!(o.right,1.);assert_eq!(o.edge_protection,0.98);assert_eq!(o.max_motion,12.);
  assert_eq!(std::mem::size_of::<super::Params>(),48);
 }
}

#[cfg(test)]mod temporal_tests{
 #[test]fn temporal_off_by_default_and_bounds(){let mut o:super::Options=serde_json::from_str("{}").unwrap();assert_eq!(o.temporal_mode,0);o.temporal_mode=9;o.temporal_strength=99.;o.clamp();assert_eq!(o.temporal_gpu().extras[2..],[0.65,2.]);assert_eq!(o.gpu(0).extras[2..],[0.8,0.]);}
}
