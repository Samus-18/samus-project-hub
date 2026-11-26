//! WGC SystemRelativeTime is a QPC timestamp expressed in 100ns units.
#[link(name="kernel32")]unsafe extern "system"{fn QueryPerformanceCounter(v:*mut i64)->i32;fn QueryPerformanceFrequency(v:*mut i64)->i32;}
fn convert(ticks:i64,freq:i64)->Option<i64>{if ticks<0||freq<=0{return None;}i64::try_from((ticks as i128)*10_000_000/freq as i128).ok()}
pub fn qpc_100ns()->Option<i64>{static F:std::sync::OnceLock<Option<i64>>=std::sync::OnceLock::new();let f=(*F.get_or_init(||{let mut f=0;(unsafe{QueryPerformanceFrequency(&mut f)}!=0&&f>0).then_some(f)}))?;let mut a=0;if unsafe{QueryPerformanceCounter(&mut a)}==0{return None;}convert(a,f)}
pub fn age_ms(stamp:i64,now:i64)->Option<f64>{if stamp<=0||now<stamp{return None;}let v=(now.checked_sub(stamp)?)as f64/10_000.;(v<=10_000.).then_some(v)}
#[cfg(test)]mod tests{use super::*;#[test]fn clocks_use_wide_math_and_reject_wrong_epoch(){assert_eq!(convert(100_000_000_000,10_000_000),Some(100_000_000_000));assert_eq!(age_ms(100_000,125_000),Some(2.5));assert!(age_ms(10,5).is_none());assert!(age_ms(0,5).is_none());assert!(qpc_100ns().is_some());}}

pub fn interval_ms(previous:Option<i64>,current:Option<i64>)->Option<f32>{
 let (a,b)=(previous?,current?);
 if a<=0||b<=a{return None;}
 let ms=b.checked_sub(a)? as f64/10_000.;
 (ms>=0.4&&ms<=350.).then_some(ms as f32)
}
#[cfg(test)]mod interval_tests{use super::*;#[test]fn rejects_duplicate_wrong_epoch_and_missing(){assert_eq!(interval_ms(Some(100_000),Some(266_667)),Some(16.6667));assert_eq!(interval_ms(Some(1),Some(1)),None);assert_eq!(interval_ms(None,Some(100)),None);assert_eq!(interval_ms(Some(200),Some(100)),None);}}
