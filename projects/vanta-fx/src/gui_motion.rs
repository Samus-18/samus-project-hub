// Shared, bounded UI motion. No GPU/game state changes, no input injection.
fn motion_value(ui:&Ui,id:egui::Id,on:bool,seconds:f32)->f32{
 if e_motion(ui){ui.ctx().animate_bool_with_time(id,on,seconds)}
 else{ui.ctx().animate_bool_with_time(id,on,0.);if on{1.}else{0.}}
}
fn motion_ease(t:f32)->f32{
 let x=if t.is_finite(){t.clamp(0.,1.)}else{0.};1.-(1.-x).powi(3)
}
fn motion_icon_scale(hover:f32,pressed:f32,base:f32)->f32{
 base*(1.+0.10*motion_ease(hover)-0.12*pressed.clamp(0.,1.))
}
fn motion_page_opacity(elapsed:f64,enabled:bool)->f32{
 if !enabled{return 1.;}
 0.80+0.20*motion_ease((elapsed/0.18)as f32)
}
fn motion_page(ui:&mut Ui,page:Page,area:SuiteArea){
 let id=egui::Id::new("hb-motion-page");
 let now=ui.input(|i|i.time);
 let (previous,start)=ui.data(|d|d.get_temp::<((Page,SuiteArea),f64)>(id)).unwrap_or(((page,area),now-1.));
 let start=if previous!=(page,area){now}else{start};
 ui.data_mut(|d|d.insert_temp(id,((page,area),start)));
 let enabled=e_motion(ui);
 let opacity=motion_page_opacity(now-start,enabled);
 ui.multiply_opacity(opacity);
 if enabled&&now-start<0.18{ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));}
}
fn motion_button(ui:&Ui,response:&egui::Response){
 if !response.enabled(){return;}
 let focus=response.has_focus();
 if focus{
  ui.painter().rect_stroke(response.rect.shrink(1.),6.,
   Stroke::new(1.,Color32::from_rgb(255,145,165)),StrokeKind::Inside);
 }
 let t=motion_value(ui,response.id.with("motion-button-hover"),response.hovered(),0.15);
 let press=motion_value(ui,response.id.with("motion-button-down"),response.is_pointer_button_down_on(),0.09);
 if t>0.001||press>0.001{
  let rect=response.rect.shrink(2.);
  if rect.width()>8.&&rect.height()>5.{
   let half=rect.width()*0.43*motion_ease(t);
   let y=rect.bottom()-0.8;
   ui.painter().line_segment([egui::pos2(rect.center().x-half,y),egui::pos2(rect.center().x+half,y)],
    Stroke::new(1.+press,Color32::from_rgba_unmultiplied(255,73,105,(120.*t+80.*press)as u8)));
  }
 }
}
#[cfg(test)]mod v105_motion_tests{
 use super::*;
 #[test]fn reduced_motion_is_instant_and_page_fade_is_bounded(){
  assert_eq!(motion_page_opacity(0.,false),1.);
  assert_eq!(motion_page_opacity(-1.,true),0.8);
  assert_eq!(motion_page_opacity(0.2,true),1.);
  assert!(motion_page_opacity(0.05,true)>0.8);
 }
 #[test]fn icons_have_subtle_bounded_press_and_hover(){
  assert_eq!(motion_icon_scale(0.,0.,1.),1.);
  assert!((motion_icon_scale(1.,0.,1.)-1.1).abs()<0.001);
  assert!((motion_icon_scale(0.,1.,1.)-0.88).abs()<0.001);
  assert!(motion_icon_scale(1.,1.,0.84)>0.8);
 }
}

#[cfg(test)]mod v106_motion_tests{
 use super::*;
 #[test]fn curve_is_monotone_and_never_overshoots(){
  let mut last=0.;
  for i in 0..=100{let x=motion_ease(i as f32/100.);assert!((0.0..=1.0).contains(&x));assert!(x>=last);last=x;}
  assert_eq!(motion_ease(f32::NAN),0.);
  assert_eq!(motion_ease(-0.5),0.);
  assert_eq!(motion_ease(2.),1.);
 }
 #[test]fn page_transition_respects_disabled_motion_at_every_time(){
  for t in [0.,0.016,0.09,0.18,1.]{assert_eq!(motion_page_opacity(t,false),1.);}
  assert!(motion_page_opacity(0.09,true)>0.9);
 }
}
