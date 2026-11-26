//! Conservative, opt-out protection around external game capture. This is a
//! denylist heuristic, not anti-cheat certification or bypass capability.
use std::path::Path;
pub struct Verdict{pub blocked:bool,pub reason:&'static str}
const PROTECTED:&[&str]=&[
 "valorant","fortnite","easyanticheat","beservice","battleye",
 "faceit","rainbowsix","r6s","destiny2","cod.exe","modernwarfare",
 "pubg","escape_from_tarkov","escapefromtarkov","league of legends",
 "apex legends","apex_legends","xdefiant","the finals",
 "eldenring","helldivers2","eaclauncher","eacservice","easy_anticheat",
 "vgc.exe","vgtray","riotclientservices","cs2.exe","destiny 2","escape from tarkov",
 "call of duty","blackops","r6siege","rainbow six","lostark","lost ark",
 "genshinimpact","genshin impact","arena breakout","deltaforce",
];
pub fn inspect(path:&Path,title:&str)->Verdict{
 let basename=path.file_name().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();
 let title=title.to_ascii_lowercase();
 let matched=PROTECTED.iter().any(|term|basename.contains(term)||title.contains(term));
 if matched{Verdict{blocked:true,reason:"Known anti-cheat-sensitive game title/process; external capture is disabled in Safe Mode. There is no game compatibility certification."}}
 else{Verdict{blocked:false,reason:"No known restricted title matched. This does NOT certify anti-cheat compatibility or permission to capture."}}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn conservative_game_denylist_never_claims_anticheat_certification(){
  for game in ["FortniteClient-Win64-Shipping.exe","VALORANT-Win64-Shipping.exe","EscapeFromTarkov.exe","BEService.exe"]{
   assert!(inspect(Path::new(game),"game").blocked);
  }
  let ordinary=inspect(Path::new("minecraft.exe"),"Minecraft");
  assert!(!ordinary.blocked);assert!(ordinary.reason.contains("NOT certify"));
 }
}

#[cfg(test)]mod v101_guard_tests {
 use super::*;
 #[test]fn blocks_known_aliases_case_insensitively(){
  for name in ["CS2.exe","Helldivers2.exe","RiotClientServices.exe","R6Siege.exe","EACLauncher.exe"]{
   assert!(inspect(Path::new(name),"Game").blocked, "{}", name);
  }
  assert!(inspect(Path::new("client.exe"),"The Finals").blocked);
  assert!(!inspect(Path::new("javaw.exe"),"Minecraft").blocked);
 }
 #[test]fn unknown_does_not_claim_certification(){
  let value=inspect(Path::new("unknown.exe"),"Some Custom Game");
  assert!(!value.blocked);assert!(value.reason.contains("NOT certify"));
 }
}
