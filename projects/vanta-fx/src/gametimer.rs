//! Stopwatch / game splits, based on monotonic time. Never changes game timing.
use std::{fs,path::PathBuf,time::{Instant,Duration,SystemTime,UNIX_EPOCH}};
#[derive(Clone,Debug)]
pub struct Split {pub label:String,pub elapsed:Duration}
#[derive(Default)]
pub struct RunTimer {
    started:Option<Instant>,
    accrued:Duration,
    pub splits:Vec<Split>,
}
impl RunTimer {
    pub fn running(&self)->bool{self.started.is_some()}
    pub fn elapsed(&self)->Duration{
        self.accrued+self.started.map_or(Duration::ZERO,|start|start.elapsed())
    }
    pub fn toggle(&mut self)->bool{
        if let Some(start)=self.started.take(){
            self.accrued+=start.elapsed();
            false
        }else{
            self.started=Some(Instant::now());true
        }
    }
    pub fn reset(&mut self){self.started=None;self.accrued=Duration::ZERO;self.splits.clear();}
    pub fn split(&mut self,label:Option<&str>)->Result<usize,String>{
        if !self.running(){return Err("Start the timer before adding a split".into());}
        if self.splits.len()>=128{return Err("Reached the maximum of 128 splits".into());}
        let n=self.splits.len()+1;
        let name=label.filter(|s|!s.trim().is_empty())
            .map_or_else(||format!("Split {n}"),|s|s.trim().chars().filter(|c|!c.is_control()).take(60).collect::<String>());
        self.splits.push(Split{label:name,elapsed:self.elapsed()});
        Ok(n)
    }
    pub fn undo_split(&mut self)->bool{self.splits.pop().is_some()}
    pub fn best_split(&self)->Option<(usize,Duration)>{(0..self.splits.len()).filter_map(|i|self.split_delta(i).map(|d|(i,d))).min_by_key(|(_,d)|*d)}
    pub fn rename_split(&mut self,i:usize,label:&str)->bool{let Some(s)=self.splits.get_mut(i)else{return false;};let name=label.trim().chars().filter(|c|!c.is_control()).take(60).collect::<String>();if name.is_empty(){return false;}s.label=name;true}
    pub fn split_delta(&self,i:usize)->Option<Duration>{let v=self.splits.get(i)?.elapsed;Some(v.saturating_sub(i.checked_sub(1).and_then(|i|self.splits.get(i)).map_or(Duration::ZERO,|s|s.elapsed)))}
    pub fn fmt_elapsed(elapsed:Duration)->String{
        let ms=elapsed.as_millis() as u64;
        let secs=ms/1000;
        format!("{:02}:{:02}:{:02}.{:02}",
            secs/3600,(secs/60)%60,secs%60,(ms%1000)/10)
    }
    pub fn export(&self)->Result<PathBuf,String>{
        let dir=crate::settings::folder().join("reports");
        fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let stamp=SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e|e.to_string())?.as_micros();
        let path=dir.join(format!("game-timer-{stamp}.csv"));
        let mut data=String::from("split,label,total_ms,delta_ms\n");
        let mut prev=Duration::ZERO;
        for (i,s) in self.splits.iter().enumerate(){
            let safe=s.label.chars().filter(|c|!c.is_control()).collect::<String>().replace('"',"\"\"");
            data.push_str(&format!("{},\"'{}\",{},{}\n",
                i+1,safe,s.elapsed.as_millis(),s.elapsed.saturating_sub(prev).as_millis()));
            prev=s.elapsed;
        }
        crate::settings::atomic_json_replace(&path,data.as_bytes())?;
        Ok(path)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn pause_resume_never_loses_elapsed(){
        let mut timer=RunTimer::default();
        assert_eq!(timer.elapsed(),Duration::ZERO);
        assert!(timer.toggle());
        timer.started=Some(Instant::now()-Duration::from_secs(65));
        assert!(!timer.toggle());
        assert!(timer.elapsed()>=Duration::from_secs(65));
        let stopped=timer.elapsed();
        assert!(timer.toggle());
        assert!(timer.elapsed()>=stopped);
        timer.reset();
        assert_eq!(timer.elapsed(),Duration::ZERO);
    }
    #[test]fn splits_require_active_timer(){
        let mut timer=RunTimer::default();
        assert!(timer.split(None).is_err());
        timer.toggle();
        assert_eq!(timer.split(None).unwrap(),1);
        assert_eq!(timer.splits[0].label,"Split 1");
        timer.toggle();
        assert!(timer.split(None).is_err());
    }
    #[test]fn clock_format_and_split_label(){
        assert_eq!(RunTimer::fmt_elapsed(Duration::from_millis(3661970)),"01:01:01.97");
        let mut t=RunTimer::default();
        t.toggle();
        t.split(Some("Dungeon boss")).unwrap();
        assert_eq!(t.splits[0].label,"Dungeon boss");
    }
}

#[cfg(test)]mod split_editor_tests{use super::*;#[test]fn rename_and_best_segment(){let mut t=RunTimer::default();t.splits=vec![Split{label:"one".into(),elapsed:Duration::from_secs(5)},Split{label:"two".into(),elapsed:Duration::from_secs(8)}];assert_eq!(t.best_split(),Some((1,Duration::from_secs(3))));assert!(t.rename_split(1,"  boss  "));assert_eq!(t.splits[1].label,"boss");assert!(!t.rename_split(1," "));}}
