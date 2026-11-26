pub fn sum(v:&[f32])->f32{
 #[cfg(target_arch="x86_64")]
 if std::is_x86_feature_detected!("avx2"){
  return unsafe{avx(v)};
 }
 v.iter().copied().sum()
}
#[cfg(target_arch="x86_64")]
#[target_feature(enable="avx2")]
unsafe fn avx(v:&[f32])->f32{
 use std::arch::x86_64::*;
 let(mut acc,mut i)=(_mm256_setzero_ps(),0usize);
 while i+8<=v.len(){
  acc=_mm256_add_ps(acc,unsafe{_mm256_loadu_ps(v.as_ptr().add(i))});i+=8;
 }
 let mut values=[0f32;8];unsafe{_mm256_storeu_ps(values.as_mut_ptr(),acc)};
 values.iter().copied().sum::<f32>()+v[i..].iter().copied().sum::<f32>()
}
pub fn implementation()->&'static str{
 #[cfg(target_arch="x86_64")]
 if std::is_x86_feature_detected!("avx2"){return "AVX2";}
 "scalar"
}
#[cfg(test)]mod tests{#[test]fn simd_agrees_with_scalar_and_handles_tail(){let v=(0..193).map(|i|0.3+i as f32/13.).collect::<Vec<_>>();assert!((super::sum(&v)-v.iter().sum::<f32>()).abs()<0.01);assert_eq!(super::sum(&[]),0.);}}
