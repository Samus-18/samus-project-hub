#[derive(Clone,Debug)]pub struct Cube{pub size:u32,pub values:Vec<[f32;3]>}
impl Cube{
 pub fn parse(raw:&str)->Result<Self,String>{
  if raw.len()>4*1024*1024{return Err("Cube exceeds 4 MiB".into());}
  let mut size=None;let mut values=Vec::new();let mut min_seen=false;let mut max_seen=false;
  for line in raw.lines(){
   let line=line.trim().trim_start_matches('\u{feff}').split('#').next().unwrap_or("").trim();
   if line.is_empty(){continue;}
   let parts=line.split_whitespace().collect::<Vec<_>>();
   match parts[0]{
    "TITLE"=>{if !values.is_empty(){return Err("TITLE after data".into());}},
    "LUT_3D_SIZE"=>{
     if size.is_some()||parts.len()!=2{return Err("Invalid/duplicate 3D size".into());}
     let n=parts[1].parse::<u32>().map_err(|_|"Invalid cube size")?;
     if !(2..=33).contains(&n){return Err("Supported cube sizes: 2 to 33".into());}size=Some(n);
    },
    "DOMAIN_MIN"|"DOMAIN_MAX"=>{
     if !values.is_empty(){return Err("Domain after data".into());}
     let is_min=parts[0]=="DOMAIN_MIN";let target=if is_min{0.}else{1.};
     if (is_min&&min_seen)||(!is_min&&max_seen)||parts.len()!=4||parts[1..].iter().any(|v|v.parse::<f32>().ok()!=Some(target)){
      return Err("Only one standard 0..1 input domain supported".into());
     }
     if is_min{min_seen=true;}else{max_seen=true;}
    },
    _=>{
     let n=size.ok_or("LUT_3D_SIZE must precede RGB data")?;
     if parts.len()!=3||values.len()>=n.pow(3) as usize{return Err("Invalid cube RGB row or excess data".into());}
     let mut v=[0f32;3];
     for i in 0..3{v[i]=parts[i].parse().map_err(|_|"Invalid cube RGB number")?;if !v[i].is_finite()||!(0.0..=1.0).contains(&v[i]){return Err("Only finite SDR 0..1 RGB entries supported".into());}}
     values.push(v);
    }
   }
  }
  let n=size.ok_or("Missing 3D cube size")?;
  if values.len()!=n.pow(3) as usize{return Err("Cube row count must equal size cubed".into());}
  Ok(Self{size:n,values})
 }
 pub fn packed(&self)->Vec<f32>{
  let n=self.size as usize;let mut data=vec![0.;n*n*n*4];
  for b in 0..n{for g in 0..n{for r in 0..n{
   let v=self.values[(b*n+g)*n+r];let pos=(g*n*n+b*n+r)*4;
   data[pos..pos+4].copy_from_slice(&[v[0],v[1],v[2],1.]);
  }}}data
 }
}
pub fn fixture(invert:bool)->Cube{
 let mut s="LUT_3D_SIZE 2\n".to_owned();
 for b in 0..2{for g in 0..2{for r in 0..2{
  let a=|v|if invert{1-v}else{v};s.push_str(&format!("{} {} {}\n",a(r),a(g),a(b)));
 }}}Cube::parse(&s).unwrap()
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn cube_row_order_and_strict_limits(){
  let c=fixture(false);assert_eq!(&c.packed()[4..8],&[1.,0.,0.,1.]);assert_eq!(&c.packed()[8..12],&[0.,0.,1.,1.]);
  assert!(Cube::parse("LUT_3D_SIZE 65").is_err());assert!(Cube::parse("LUT_3D_SIZE 2\n0 0 NaN").is_err());
  assert!(Cube::parse("LUT_1D_SIZE 2\n0 0 0").is_err());
 }
}
