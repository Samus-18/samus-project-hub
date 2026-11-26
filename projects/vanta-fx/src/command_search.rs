//! Local navigation only. Search never starts a game or applies a preset.
use crate::{Page,game_library::Entry};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Hit{Route(usize),Look(usize),Game(usize)}
pub struct Route{pub en:&'static str,pub cz:&'static str,pub words:&'static str,pub page:Page,pub sub:usize}
macro_rules! route{($en:expr,$cz:expr,$words:expr,$page:ident,$sub:expr)=>{Route{en:$en,cz:$cz,words:$words,page:Page::$page,sub:$sub}}}
pub static ROUTES:&[Route]=&[
route!("Home","Přehled","overview start domov",Home,0),
route!("Game library","Knihovna her","select game running hry vybrat",Library,0),
route!("Overlay / HUD","Herní překryv","overlay hud statistik",Overlay,0),
route!("Native looks","Grafické filtry","shader look preset vzhled",Filters,0),
route!("Light studio","Osvětlení","cinema radiance shadow stíny light světlo",Filters,1),
route!("Screen color","Barvy obrazovky","monitor display gamma",Filters,2),
route!("Stack builder","Vrstvy efektů","shader stack layers vrstva",Filters,3),
route!("Color Lab / LUT","Barevná laboratoř","photo lut reference fotografie",Filters,4),
route!("Upscaling","Zvětšení obrazu","fsr nis integer scale rozlišení",Scaling,0),
route!("Frame generation","Generování snímků","fg motion interpolace 2x 3x",FrameGen,0),
route!("Performance metrics","Měření výkonu","fps frametime statistiky",Performance,0),
route!("GPU diagnostics","Diagnostika GPU","vram adapter grafika paměť",Performance,1),
route!("Benchmark analysis","Analýza benchmarku","test měření percentily",Performance,2),
route!("Stability","Stabilita","stutter záseky lag",Performance,3),
route!("CPU affinity","Afinita CPU","processor procesor jádra",Performance,4),
route!("Crosshair","Zaměřovač","aim crosshair",Extras,0),
route!("Timers","Časovače","timer stopky odpočet",Extras,1),
route!("Quests","Úkoly","quest checklist",Extras,2),
route!("Training","Trénink","practice reaction reakce",Extras,3),
route!("Quick actions","Rychlé akce","actions nástroje",Extras,4),
route!("Macros","Makra","macro",Extras,5),
route!("View tools","Nástroje zobrazení","view monitor",Extras,6),
route!("Reminders","Připomínky","reminder sound upozornění",Extras,7),
route!("Controller lab","Test ovladače","gamepad drift deadzone p95",Extras,8),
route!("Session lab","Herní relace","session playtime historie",Extras,9),
route!("Replay and capture","Záznam a replay","record screenshot video nahrávání",Capture,0),
route!("Settings / profiles","Nastavení a profily","settings language jazyk profil",Settings,0),
route!("Lab","Lab","staging unsorted vsechny funkce pohromade docasne nerozlozene",Unsorted,0)];
pub fn fold(s:&str)->String{
 let mut out=String::new();
 for c in s.chars().take(512).flat_map(char::to_lowercase){
  let c=match c{'á'=>'a','č'=>'c','ď'=>'d','é'|'ě'=>'e','í'=>'i','ň'=>'n','ó'=>'o','ř'=>'r','š'=>'s','ť'=>'t','ú'|'ů'=>'u','ý'=>'y','ž'=>'z',x=>x};
  if c.is_alphanumeric(){out.push(c)}else if !out.ends_with(' '){out.push(' ')}
 }
 out.trim().to_owned()
}
fn score(q:&str,label:&str,words:&str)->Option<u32>{
 let l=fold(label);let w=fold(words);
 if q.is_empty(){return Some(0)}
 if l==q{return Some(1000)}
 let mut n=if l.starts_with(q){100}else{0};
 for t in q.split_whitespace(){
  n+=if l.split_whitespace().any(|x|x==t){40}
  else if l.split_whitespace().any(|x|x.starts_with(t)){30}
  else if l.contains(t){20}
  else if w.split_whitespace().any(|x|x==t){15}
  else if w.contains(t){5}else{return None};
 }
 Some(n)
}
pub fn matches(q:&str,label:&str)->bool{score(&fold(q),label,"").is_some()}
pub fn search(query:&str,games:&[Entry])->Vec<Hit>{
 let q=fold(query);let mut hits=Vec::new();
 for (i,r) in ROUTES.iter().enumerate(){
  let alias=format!("{} {}",r.cz,r.words);
  let a=score(&q,r.en,&alias);let b=score(&q,r.cz,&format!("{} {}",r.en,r.words));
  if let Some(v)=a.into_iter().chain(b).max(){hits.push((v,Hit::Route(i)))}
 }
 if !q.is_empty(){
  for(i,p)in crate::native_presets::PRESETS.iter().enumerate(){if let Some(s)=score(&q,p.label,"look preset shader filtr vzhled"){hits.push((s,Hit::Look(i)))}}
  for(i,g)in games.iter().enumerate(){if let Some(s)=score(&q,&g.title,"game hra knihovna library"){hits.push((s,Hit::Game(i)))}}
 }
 hits.sort_by(|a,b|b.0.cmp(&a.0));hits.truncate(48);hits.into_iter().map(|x|x.1).collect()
}
pub fn proof()->Result<(),String>{
 if search("",&[]).len()!=ROUTES.len(){return Err("Route coverage".into())}
 for (i,r) in ROUTES.iter().enumerate(){if !search(r.cz,&[]).contains(&Hit::Route(i)){return Err(format!("Missing CZ {}",r.cz))}}
 if search("test ovladace",&[]).first()!=Some(&Hit::Route(23)){return Err("Czech ranking".into())}
 if !matches("cesky hra","Český hra"){return Err("Accent fold".into())}
 if !search("zzzxnonexistent",&[]).is_empty(){return Err("False result".into())}
 let g=Entry{title:"Český příběh".into(),executable:None,source:"Manual".into(),banner:None};
 if search("cesky pribeh",&[g]).first()!=Some(&Hit::Game(0)){return Err("Game search".into())}
 let p=&crate::native_presets::PRESETS[0];if !search(p.label,&[]).contains(&Hit::Look(0)){return Err("Look search".into())}
 Ok(())
}
#[cfg(test)]mod tests{#[test]fn all_routes_and_rankings(){super::proof().unwrap();}}
