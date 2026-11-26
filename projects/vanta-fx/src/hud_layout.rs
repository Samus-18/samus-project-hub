use serde::{Serialize,Deserialize};
pub const NAMES:[&str;22]=["Game FPS","Frame time","1% low","GPU time","Horizon RAM","Ping","Clock","Own output","0.1% low","Game RAM","Game CPU","Horizon CPU","ICMP no reply %","ICMP jitter","Capture source age","Generated output","Game note","Daily quests","Cooldown","Recording","Next quest","Timer name"];
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]pub struct Widget{pub metric:u8,pub scale:f32,pub opacity:f32,pub mono:bool}
impl Default for Widget{fn default()->Self{Self{metric:0,scale:1.,opacity:1.,mono:false}}}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]pub struct Layout{pub enabled:bool,pub widgets:Vec<Widget>,pub graph:bool,pub audio:bool,pub custom_position:Option<[f32;2]>,pub display:Option<String>,pub quest_list:bool}
impl Default for Layout{fn default()->Self{Self{enabled:false,widgets:vec![Widget::default(),Widget{metric:1,..Default::default()},Widget{metric:4,..Default::default()},Widget{metric:5,..Default::default()}],graph:true,audio:false,custom_position:None,display:None,quest_list:false}}}
impl Layout{
 pub fn sanitize(&mut self){
  self.display=self.display.take().filter(|s|!s.is_empty()&&s.len()<=64&&!s.chars().any(char::is_control));
  self.widgets.truncate(6);self.widgets.retain(|w|(w.metric as usize)<NAMES.len());if self.widgets.is_empty(){self.widgets.push(Widget::default());}
  for w in &mut self.widgets{w.scale=if w.scale.is_finite(){w.scale.clamp(0.8,1.25)}else{1.};w.opacity=if w.opacity.is_finite(){w.opacity.clamp(0.4,1.)}else{1.};}
  if let Some(p)=&mut self.custom_position{for v in p{*v=if v.is_finite(){v.clamp(0.,1.)}else{0.};}}
 }
 pub fn move_widget(&mut self,i:usize,delta:isize)->bool{if let Some(j)=i.checked_add_signed(delta){if i<self.widgets.len()&&j<self.widgets.len(){self.widgets.swap(i,j);return true;}}false}
 pub fn dimensions(&self,z:f32)->(f32,f32){let z=if z.is_finite(){z.clamp(0.6,1.5)}else{1.};(300.*z,(32.+self.widgets.len().div_ceil(2) as f32*36.+if self.graph{38.}else{0.}+if self.quest_list{80.}else{0.})*z)}
 pub fn position(&self,screen:(u32,u32),size:(f32,f32))->Option<(f32,f32)>{
  self.custom_position.map(|p|{let axis=|span:u32,extent:f32,a:f32|{let free=span as f32-if extent.is_finite(){extent.max(0.)}else{span as f32};if free>=16.{8.+(free-16.)*if a.is_finite(){a.clamp(0.,1.)}else{0.}}else{0.}};(axis(screen.0,size.0,p[0]),axis(screen.1,size.1,p[1]))})
 }
}
#[cfg(test)]mod tests{
 #[test]fn widget_grid_bounds_and_restore(){let mut l=super::Layout::default();l.widgets[0].scale=f32::NAN;l.custom_position=Some([2.,-1.]);l.sanitize();assert_eq!(l.widgets[0].scale,1.);let size=l.dimensions(1.);let p=l.position((1920,1080),size).unwrap();assert_eq!(p,(1612.,8.));assert!(size.0*size.1<60000.);let raw=serde_json::to_string(&l).unwrap();assert_eq!(serde_json::from_str::<super::Layout>(&raw).unwrap().widgets.len(),4);}
}

#[cfg(test)]mod v49_tests{
 #[test]fn new_widgets_old_profiles_and_reorder(){let mut l=super::Layout::default();l.widgets[0].metric=11;l.widgets[0].mono=true;l.sanitize();assert_eq!(l.widgets[0].metric,11);assert!(!l.move_widget(0,-1));assert!(l.move_widget(0,1));assert_eq!(l.widgets[1].metric,11);assert!(l.widgets[1].mono);assert!(!serde_json::from_str::<super::Widget>(r#"{"metric":8}"#).unwrap().mono);}
}

pub fn physical_anchor(d:&crate::display_probe::Target,size:(i32,i32),anchor:[f32;2])->(i32,i32){
 let ratio=|a:f32|if a.is_finite(){a.clamp(0.,1.)}else{0.};
 let axis=|origin:i32,span:u32,extent:i32,a:f32|{
   let free=i64::from(span)-i64::from(extent.max(0));
   let offset=if free>=16{8+((free-16)as f64*f64::from(ratio(a))).round()as i64}else{0};
   (i64::from(origin)+offset).clamp(i64::from(i32::MIN),i64::from(i32::MAX))as i32
 };
 (axis(d.x,d.width,size.0,anchor[0]),axis(d.y,d.height,size.1,anchor[1]))
}
#[cfg(test)]mod monitor_tests{use super::*;#[test]fn negative_monitor_coordinates_and_bounds(){let d=crate::display_probe::Target{name:"fixture".into(),x:-1920,y:-200,width:1920,height:1080,refresh:Some(144)};assert_eq!(physical_anchor(&d,(300,150),[1.,0.]),(-308,-192));assert_eq!(physical_anchor(&d,(300,150),[0.,1.]),(-1912,722));let h:Layout=serde_json::from_str("{}").unwrap();assert!(h.display.is_none());}}

#[cfg(test)]mod overflow_tests{use super::*;#[test]fn oversized_and_invalid_positions_are_bounded(){let d=crate::display_probe::Target{name:"fixture".into(),x:i32::MAX-1,y:-200,width:u32::MAX,height:10,refresh:None};assert_eq!(physical_anchor(&d,(i32::MAX,100),[1.,1.]),(i32::MAX,-200));assert_eq!(physical_anchor(&d,(0,100),[f32::NAN,1.]),(i32::MAX,-200));}}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn position_oversized_and_bad_anchor_safe(){let mut l=Layout::default();l.custom_position=Some([f32::NAN,1.]);assert_eq!(l.position((100,100),(300.,150.)),Some((0.,0.)));assert_eq!(l.position((100,100),(20.,20.)),Some((8.,72.)));}}

pub fn drag_anchor(pointer:[f32;2],viewport:[f32;2],marker:[f32;2])->[f32;2]{
 std::array::from_fn(|i|{let free=viewport[i]-marker[i];if !free.is_finite()||free<=0.||!pointer[i].is_finite(){0.}else{((pointer[i]-marker[i]*0.5)/free).clamp(0.,1.)}})
}
#[cfg(test)]mod drag_tests{use super::*;#[test]fn preview_drag_keeps_full_marker_inside(){assert_eq!(drag_anchor([0.,0.],[320.,135.],[64.,24.]),[0.,0.]);assert_eq!(drag_anchor([320.,135.],[320.,135.],[64.,24.]),[1.,1.]);assert_eq!(drag_anchor([160.,67.5],[320.,135.],[64.,24.]),[0.5,0.5]);assert_eq!(drag_anchor([f32::NAN,0.],[0.,0.],[64.,24.]),[0.,0.]);}}


pub fn anchor_from_outer(screen:(u32,u32),physical_window:(f32,f32),top_left:(f32,f32))->Option<[f32;2]>{
 if screen.0<160||screen.1<90||![physical_window.0,physical_window.1,top_left.0,top_left.1].iter().all(|x|x.is_finite()){return None;}
 let inverse=|span:u32,size:f32,position:f32|{
  let free=span as f32-size.max(0.)-16.;
  if free>=1.{((position-8.)/free).clamp(0.,1.)}else{0.}
 };
 Some([inverse(screen.0,physical_window.0,top_left.0),inverse(screen.1,physical_window.1,top_left.1)])
}
#[cfg(test)]mod native_editor_drag_tests{
 use super::*;
 #[test]fn real_absolute_screen_to_normalized_hud_roundtrip(){
  let mut l=Layout::default();
  let screen=(1920,1080);let size=(456.,410.);
  for p in [(8.,8.),(728.,337.),(1456.,662.)]{
   l.custom_position=anchor_from_outer(screen,size,p);
   let got=l.position(screen,size).unwrap();
   assert!((got.0-p.0).abs()<=1.);assert!((got.1-p.1).abs()<=1.);
  }
  assert!(anchor_from_outer(screen,size,(f32::NAN,0.)).is_none());
  assert_eq!(anchor_from_outer((400,300),(800.,600.),(-100.,-100.)),Some([0.,0.]));
 }
}
