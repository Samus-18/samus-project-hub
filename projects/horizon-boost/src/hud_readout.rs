//! HUD presentation strings and monitoring policy. Never infer FPS from requested FG factor.
use crate::output_rates::Rates;
pub fn reuse_game_meter(target:Option<u32>,game:Option<u32>,enabled:bool)->bool{enabled&&target.is_some()&&target==game}
pub fn shortcut_allowed(enabled:bool,rising:bool,modifiers:bool,macro_active:bool)->bool{enabled&&rising&&!modifiers&&!macro_active}
pub fn rate(v:Option<f64>)->String{v.filter(|n|n.is_finite()&&*n>=0.).map(|n|format!("{n:.0}")).unwrap_or("--".into())}
#[derive(Debug,PartialEq)]
pub struct Readout{pub game:String,pub output:String,pub state:String,pub detail:String}
pub fn readout(game:Option<f64>,rates:Option<Rates>,requested:Option<u8>,effective:Option<u8>)->Readout{
 let game=rate(game);
 let output=rate(rates.map(|r|r.total));
 let state=match requested{
  Some(n) if n>1&&rates.is_some_and(|r|r.generated<=0.)&&effective.is_some_and(|e|e>1)=>format!("FG x{n} / no synth"),
  Some(n) if n>1=>match effective{Some(e) if e>0&&e!=n=>format!("FG x{n} > x{e}"),Some(e) if e>1=>format!("FG x{e}"),_=>format!("FG x{n} / waiting")},
  Some(_)=>"GPU x1".into(),None=>"FG off".into()
 };
 let detail=rates.map(|r|format!("{} real + {} generated",rate(Some(r.real)),rate(Some(r.generated)))).unwrap_or("Waiting for output samples".into());
 Readout{game,output,state,detail}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn measured_not_multiplied(){let r=readout(Some(50.),Some(Rates{real:37.,generated:31.,total:68.}),Some(2),Some(2));assert_eq!(r.game,"50");assert_eq!(r.output,"68");assert_eq!(r.state,"FG x2");assert_eq!(r.detail,"37 real + 31 generated");}
 #[test]fn missing_and_fallback_do_not_invent_fps(){let r=readout(Some(50.),None,Some(2),None);assert_eq!(r.output,"--");assert_eq!(r.state,"FG x2 / waiting");let r=readout(None,Some(Rates{real:35.,generated:0.,total:35.}),Some(3),Some(1));assert_eq!(r.game,"--");assert_eq!(r.output,"35");assert_eq!(r.state,"FG x3 > x1");}
 #[test]fn monitor_sharing_and_f12_guard(){assert!(reuse_game_meter(Some(10),Some(10),true));assert!(!reuse_game_meter(Some(10),Some(11),true));assert!(!reuse_game_meter(None,None,true));assert!(!reuse_game_meter(Some(10),Some(10),false));assert!(shortcut_allowed(true,true,false,false));for args in [(false,true,false,false),(true,false,false,false),(true,true,true,false),(true,true,false,true)]{assert!(!shortcut_allowed(args.0,args.1,args.2,args.3));}}
 #[test]fn old_settings_defaults_and_roundtrip(){let mut s:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert!(s.hud_follow_foreground&&s.hud_show_fg&&!s.hud_f12&&!s.hud_start_open);s.hud_f12=true;s.hud_start_open=true;let r:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert!(r.hud_f12&&r.hud_start_open);}
}
