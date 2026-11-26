use serde::{Serialize,Deserialize};
pub const STAGE:usize=51;
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Grade{
 pub enabled:bool,pub linear:bool,pub exposure:f32,pub lift:f32,pub gamma:f32,pub gain:f32,
 pub temperature:f32,pub tint:f32,pub saturation:f32,pub contrast:f32,
 pub red:f32,pub green:f32,pub blue:f32,pub luma:f32,
 pub hue:f32,pub hue_width:f32,pub hue_shift:f32,pub selective_sat:f32,pub mix:f32,
 pub curves:[[f32;5];4],
 pub gradient:[[f32;3];3],pub gradient_mix:f32,pub gradient_mid:f32,
}
impl Default for Grade{fn default()->Self{Self{enabled:false,linear:true,exposure:0.,lift:0.,gamma:1.,gain:1.,
 temperature:0.,tint:0.,saturation:1.,contrast:1.,red:0.,green:0.,blue:0.,luma:0.,
 hue:0.,hue_width:0.12,hue_shift:0.,selective_sat:1.,mix:1.,curves:[[0.,0.25,0.5,0.75,1.];4],gradient:[[0.,0.,0.],[0.5,0.5,0.5],[1.,1.,1.]],gradient_mix:0.,gradient_mid:0.5}}}
impl Grade{
 pub fn clamp(&mut self){
  let f=|v:&mut f32,a:f32,b:f32,d:f32|*v=if v.is_finite(){v.clamp(a,b)}else{d};
  f(&mut self.exposure,-2.,2.,0.);f(&mut self.lift,-0.15,0.15,0.);f(&mut self.gamma,0.5,2.,1.);f(&mut self.gain,0.5,1.5,1.);
  f(&mut self.temperature,-1.,1.,0.);f(&mut self.tint,-1.,1.,0.);f(&mut self.saturation,0.,2.,1.);f(&mut self.contrast,0.5,1.5,1.);
  for v in [&mut self.red,&mut self.green,&mut self.blue,&mut self.luma]{f(v,-0.25,0.25,0.);}
  for curve in &mut self.curves{for (i,v) in curve.iter_mut().enumerate(){f(v,0.,1.,i as f32/4.);}}
  for (i,c) in self.gradient.iter_mut().enumerate(){for v in c{f(v,0.,1.,i as f32*0.5);}}
  f(&mut self.gradient_mix,0.,1.,0.);f(&mut self.gradient_mid,0.05,0.95,0.5);
  f(&mut self.hue,0.,1.,0.);f(&mut self.hue_width,0.03,0.5,0.12);f(&mut self.hue_shift,-0.5,0.5,0.);f(&mut self.selective_sat,0.,2.,1.);f(&mut self.mix,0.,1.,1.);
 }
 pub fn curve_preset(&mut self,kind:u8){
  self.curves[3]=match kind{1=>[0.,0.18,0.5,0.82,1.],2=>[0.06,0.29,0.5,0.71,0.94],3=>[0.,0.34,0.60,0.82,1.],_=>[0.,0.25,0.5,0.75,1.]};
 }
 pub fn gpu(&self)->Params{
  Params{a:[self.exposure,self.lift,self.gamma,self.gain],b:[self.temperature,self.tint,self.saturation,self.contrast],
  c:[self.red,self.green,self.blue,self.luma],d:[self.hue,self.hue_width,self.hue_shift,self.selective_sat],e:[self.mix,if self.linear{1.}else{0.},self.gradient_mix,self.gradient_mid],curves:std::array::from_fn(|i|std::array::from_fn(|channel|self.curves[channel][i])),gradient:self.gradient.map(|c|[c[0],c[1],c[2],0.])}
 }
}
#[repr(C)]#[derive(Clone,Copy)]pub struct Params{a:[f32;4],b:[f32;4],c:[f32;4],d:[f32;4],e:[f32;4],curves:[[f32;4];5],gradient:[[f32;4];3]}
#[cfg(test)]mod tests{
 #[test]fn old_settings_neutral_and_bounded(){
  let g:super::Grade=serde_json::from_str("{}").unwrap();assert!(!g.enabled);assert_eq!(g.gamma,1.);
  assert_eq!(std::mem::size_of::<super::Params>(),208);
  let mut g=g;g.gamma=f32::NAN;g.exposure=99.;g.clamp();assert_eq!(g.gamma,1.);assert_eq!(g.exposure,2.);
 }
}

#[cfg(test)]mod preset_tests{#[test]fn only_luma_changes(){let mut g=super::Grade::default();let rgb=g.curves[0..3].to_vec();g.curve_preset(1);assert_eq!(g.curves[3],[0.,0.18,0.5,0.82,1.]);assert_eq!(&g.curves[0..3],&rgb);g.curve_preset(0);assert_eq!(g.curves,super::Grade::default().curves);}}

#[cfg(test)]mod gradient_tests{
 #[test]fn old_grade_off_and_clamp(){let mut g:super::Grade=serde_json::from_str("{}").unwrap();assert_eq!(g.gradient_mix,0.);g.gradient_mix=9.;g.gradient_mid=0.;g.gradient[0][1]=f32::NAN;g.clamp();assert_eq!(g.gradient_mix,1.);assert_eq!(g.gradient_mid,0.05);assert_eq!(g.gradient[0][1],0.);}
}
