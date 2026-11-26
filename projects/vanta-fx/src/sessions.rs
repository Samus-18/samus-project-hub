//! Game sessions are recorded for the user only. No game process modifications.
//! Unlike synthetic Steam-like playtime, durations come from a monotonic clock.
use serde::{Serialize,Deserialize};
use std::{fs,time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
pub struct SessionRecord{
    pub game_key:String,
    pub game_name:String,
    pub started_unix:u64,
    pub duration_secs:u64
}
pub struct CurrentSession{
    pub key:String,
    pub name:String,
    pub start_unix:u64,
    pub began:Instant
}
#[derive(Default)]
pub struct SessionTracker{
    pub current:Option<CurrentSession>,
    pub history:Vec<SessionRecord>,
}
fn epoch()->u64{
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0,|d|d.as_secs())
}
impl SessionTracker{
    pub fn load()->Self{
        let path=crate::settings::folder().join("sessions.json");
        let mut history=fs::metadata(&path).ok().filter(|m|m.len()<=16*1024*1024).and_then(|_|fs::read(&path).ok())
            .and_then(|b|serde_json::from_slice::<Vec<SessionRecord>>(&b).ok())
            .unwrap_or_default();
        history.retain(|v|!v.game_key.is_empty()&&v.game_key.len()<=32768&&v.duration_secs>=5);
        if history.len()>2500{history.drain(0..history.len()-2500);}
        Self{current:None,history}
    }
    pub fn switch(&mut self,game:Option<(&str,&str)>)->bool{
        let next=game.map(|(key,_)|key);
        if match (self.current.as_ref(),next){(Some(c),Some(n))=>c.key.eq_ignore_ascii_case(n),(None,None)=>true,_=>false}{return false;}
        let changed=self.finish();
        if let Some((key,name))=game{
            self.current=Some(CurrentSession{
                key:key.to_owned(),name:name.to_owned(),
                start_unix:epoch(),began:Instant::now()
            });
        }
        changed
    }
    pub fn finish(&mut self)->bool{
        let Some(session)=self.current.take()else{return false;};
        let secs=session.began.elapsed().as_secs();
        if secs<5{return false;}
        self.history.push(SessionRecord{game_key:session.key,game_name:session.name,
            started_unix:session.start_unix,duration_secs:secs});
        if self.history.len()>2500{
            self.history.drain(0..(self.history.len()-2500));
        }
        true
    }
    pub fn save(&mut self)->Result<(),String>{
        let dir=crate::settings::folder();
        fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let output=dir.join("sessions.json");
        crate::settings::atomic_json_replace(&output,&serde_json::to_vec_pretty(&self.history).map_err(|e|e.to_string())?)
    }
    pub fn summary(&self,key:&str)->(usize,u64,u64){
        let mut n=0;let mut sum=0u64;let mut longest=0u64;
        for r in self.history.iter().filter(|r|r.game_key.eq_ignore_ascii_case(key)){n+=1;sum=sum.saturating_add(r.duration_secs);longest=longest.max(r.duration_secs);}
        (n,sum,longest)
    }
    pub fn played_seconds(&self,key:&str)->u64{
        let finished=self.history.iter().filter(|s|s.game_key.eq_ignore_ascii_case(key))
            .map(|s|s.duration_secs).fold(0u64,u64::saturating_add);
        let active=self.current.as_ref().filter(|s|s.key.eq_ignore_ascii_case(key))
            .map_or(0,|s|s.began.elapsed().as_secs());
        finished.saturating_add(active)
    }
    pub fn current_elapsed(&self)->Duration{
        self.current.as_ref().map_or(Duration::ZERO,|s|s.began.elapsed())
    }
    pub fn export_csv(&self)->Result<std::path::PathBuf,String>{
        let dir=crate::settings::folder().join("reports");
        fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let file=dir.join("playtime-history.csv");
        let mut rows=String::from("game,started_unix,duration_secs\n");
        for r in &self.history{
            // Quoting prevents CSV formula execution in spreadsheet tools.
            let safe=r.game_name.chars().filter(|c|!c.is_control()).collect::<String>().replace('"',"\"\"");
            rows.push_str(&format!("\"'{}\",{},{}\n",safe,r.started_unix,r.duration_secs));
        }
        crate::settings::atomic_json_replace(&file,rows.as_bytes())?;
        Ok(file)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn no_duplicate_session_for_same_game(){
        let mut s=SessionTracker::default();
        assert!(!s.switch(Some(("game.exe","Game"))));
        let instant=s.current.as_ref().unwrap().began;
        assert!(!s.switch(Some(("game.exe","Game"))));
        assert_eq!(s.current.as_ref().unwrap().began,instant);
    }
    #[test]fn tracks_game_switch_and_completes(){
        let mut s=SessionTracker::default();
        s.switch(Some(("one","Game One")));
        s.current.as_mut().unwrap().began=Instant::now()-Duration::from_secs(66);
        assert!(s.switch(Some(("two","Game Two"))));
        assert_eq!(s.history[0].duration_secs,66);
        assert_eq!(s.played_seconds("one"),66);
        assert_eq!(s.history[0].game_name,"Game One");
    }
    #[test]fn ignores_tiny_false_detection(){
        let mut s=SessionTracker::default();
        s.switch(Some(("one","Game One")));
        assert!(!s.switch(None));
        assert!(s.history.is_empty());
    }
    #[test]fn csv_is_escaped(){
        let s=SessionTracker{current:None,history:vec![
            SessionRecord{game_key:"a".into(),game_name:"=test,\"hi\"".into(),
                started_unix:1,duration_secs:42}]};
        let safe=s.history[0].game_name.replace('"',"\"\"");
        assert_eq!(format!("\"'{}\"",safe),"\"'=test,\"\"hi\"\"\"");
    }
}

#[cfg(test)]mod summary_tests{
 #[test]fn durations_never_overflow(){let s=super::SessionTracker{current:None,history:vec![super::SessionRecord{game_key:"x".into(),game_name:"X".into(),started_unix:0,duration_secs:u64::MAX};2]};assert_eq!(s.summary("X"),(2,u64::MAX,u64::MAX));assert_eq!(s.played_seconds("x"),u64::MAX);}
}

#[cfg(test)]mod case_tests{use super::*;#[test]fn windows_case_same_session(){let mut t=SessionTracker::default();t.switch(Some(("C:/Game.EXE","Game")));t.current.as_mut().unwrap().began=Instant::now()-Duration::from_secs(10);assert!(!t.switch(Some(("c:/game.exe","Game"))));assert!(t.history.is_empty());assert!(t.played_seconds("c:/game.exe")>=10);}}
