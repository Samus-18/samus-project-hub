//! HORIZON BOOST: non-AI image previews for the 20 GPU filter choices.
//!
//! The sample is an ORIGINAL PROCEDURALLY RENDERED test scene, not a generated AI image.
//! The 13 HorizonBoost effects (1-13) use CPU reference implementations of their
//! HLSL formulas. The Magpie built-ins (14-19) are APPROXIMATIONS, not exact GPU results.
//! Thumbnails are local/static; no game injection or screenshots are taken without consent.
use eframe::egui::{Color32,ColorImage,Context,TextureHandle,TextureOptions};
use std::path::Path;

const W:usize=512;
const H:usize=288;
const TW:usize=176;
const TH:usize=99;
const EFFECT_COUNT:usize=20;

#[derive(Default)]
pub struct PreviewBank {
    source:Option<ColorImage>,
    original:Option<TextureHandle>,
    selected:Option<TextureHandle>,
    selected_key:Option<(usize,u8,u64,usize,u64)>,
    thumbs:Vec<Option<TextureHandle>>,
    stack_thumbs:Vec<Option<TextureHandle>>,
    custom_thumb_sig:u64,
    single_thumb_effect:usize,
    generation:u64,
    pub source_label:String,
}
fn clamp(v:f32)->f32{v.clamp(0.0,1.0)}
fn smooth(a:f32,b:f32,v:f32)->f32{
    let t=clamp((v-a)/(b-a).max(0.00001));t*t*(3.0-2.0*t)
}
fn mix(a:f32,b:f32,t:f32)->f32{a*(1.0-t)+b*t}
fn noise(x:u32,y:u32)->f32{
    let n=x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    let m=(n^(n>>13)).wrapping_mul(1274126177);
    ((m^(m>>16))&65535) as f32/65535.0
}
fn rgb(c:[f32;3])->Color32{
    Color32::from_rgb((clamp(c[0])*255.0).round() as u8,
        (clamp(c[1])*255.0).round() as u8,(clamp(c[2])*255.0).round() as u8)
}
/// Real photographs bundled locally with documented public-domain origins.
pub const BUILTIN_PHOTOS:[(&str,&[u8]);2]=[
    ("SpaceX rocket photo · public domain",include_bytes!("../assets/preview_photo_cc0.jpg")),
    ("NASA astronaut portrait · public domain",include_bytes!("../assets/preview_portrait_publicdomain.png")),
];
pub fn real_photo(index:usize)->Result<ColorImage,String>{
    let (_,bytes)=BUILTIN_PHOTOS.get(index).ok_or("Unknown reference photograph")?;
    let img=image::load_from_memory(bytes)
        .map_err(|e|format!("Bundled real photograph decoding failed: {e}"))?
        .resize_to_fill(W as u32,H as u32,image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    Ok(ColorImage::from_rgba_unmultiplied([W,H],img.as_raw()))
}
fn scene()->ColorImage{
    // Original illustrative test scene with detail, gradients, highlight glows,
    // dark edges, warm/cool color separation, and clean silhouettes.
    let mut px=Vec::with_capacity(W*H);
    for yy in 0..H {
        for xx in 0..W{
            let u=xx as f32/W as f32;
            let v=yy as f32/H as f32;
            let g=noise(xx as u32,yy as u32);
            let mut c=[
                mix(0.045,0.55,smooth(0.05,0.73,v)),
                mix(0.083,0.25,smooth(0.05,0.82,v)),
                mix(0.23,0.31,smooth(0.2,0.95,v)),
            ];
            // Sunset pink sky.
            let glow=(1.0-(((u-0.75)*1.95).powi(2)+((v-0.42)*1.8).powi(2)).sqrt()).max(0.0);
            c[0]+=glow*0.3;c[1]+=glow*0.08;c[2]+=glow*0.07;
            let sx=u-0.76;let sy=(v-0.34)*1.65;
            let sun_distance=(sx*sx+sy*sy).sqrt();
            let sun=1.0-smooth(0.105,0.115,sun_distance);
            for j in 0..3{c[j]=mix(c[j],[1.0,0.66,0.43][j],sun);}
            // Wispy clouds and atmospheric gradients, even with no AI imagery.
            let cloud_y=0.22+0.029*(u*14.0).sin()+0.02*(u*38.0).sin();
            let cloud=1.0-smooth(0.009,0.034,(v-cloud_y).abs());
            let cloud_mask=(0.5+0.5*(u*31.0).sin())*cloud*0.11;
            c[0]+=cloud_mask;c[1]+=cloud_mask*0.6;c[2]+=cloud_mask*0.7;
            // Distant rock formations, midground ridge and reflective lake.
            let distant=0.54+0.075*(u*13.0).sin()+0.038*(u*29.0).sin();
            let middle=0.64+0.07*(u*21.0+0.4).sin()+0.035*(u*48.0).sin();
            if v>distant{
                c=[0.12+g*0.017,0.16+g*0.015,0.23+g*0.022];
            }
            if v>middle{
                c=[0.095+g*0.06,0.15+g*0.05,0.175+g*0.04];
            }
            let waterline=0.75+0.025*(u*7.0).sin();
            if v>waterline{
                let shimmer=0.045*(v*420.0).sin()+0.017*(u*70.0).sin();
                c=[0.06+shimmer+g*0.013,0.21+shimmer*0.6+g*0.03,
                    0.29+shimmer*0.6+g*0.035];
                let reflection=clamp(0.14-(u-0.76).abs())*(1.0-v)*1.8;
                c[0]+=reflection*0.8;c[1]+=reflection*0.4;
            }
            // Ancient brick ruin with thin mullions: ideal for clarity and AA comparisons.
            for (base,w,h) in [(0.11,0.12,0.29),(0.235,0.055,0.22),(0.34,0.10,0.17),(0.62,0.045,0.12)]{
                if (u-base).abs()<w*0.5 && v>middle-h && v<middle+0.015{
                    c=[0.10+g*0.055,0.115+g*0.055,0.165+g*0.07];
                    let mx=((xx as f32)/11.0).fract();
                    let my=((yy as f32)/9.0).fract();
                    if mx<0.11||my<0.10 {c=[0.055,0.068,0.11];}
                    if (u-(base+w*0.17)).abs()<0.009&&v>middle-h+0.025{
                        c=[0.53,0.23,0.25];
                    }
                }
            }
            // Foreground geometric pines and highlighted meadow.
            let foreground=0.85+0.02*(u*15.0).sin()+0.02*(u*31.0).cos();
            if v>foreground{
                c=[0.075+g*0.065,0.14+g*0.09,0.10+g*0.045];
                if noise(xx as u32/3,yy as u32/3)>0.94{
                    c=[0.48,0.38,0.20];
                }
            }
            for (tx,ty,size) in [(0.035,0.72,0.15),(0.55,0.76,0.12),(0.92,0.72,0.19)]{
                let dx=(u-tx)/size;let dv=(v-ty)/size;
                if dv> -1.0 && dv<0.5&&dx.abs()<(1.0+dv)*0.42 {
                    let b=(noise(xx as u32/4,yy as u32/4)-0.5)*0.04;
                    c=[0.025+b,0.08+b,0.075+b];
                }
                if (u-tx).abs()<0.004&&v>ty&&v<ty+size*0.7{
                    c=[0.07,0.065,0.07];
                }
            }
            // Small warm pixels intentionally exercise glow & grain.
            let lantern=(u-0.273).abs()<0.007 && (v-0.565).abs()<0.013;
            if lantern {c=[1.0,0.67,0.15];}
            let sparkle=noise(xx as u32/5,yy as u32/5);
            if v<0.47&&sparkle>0.9995&&xx%4==0{c=[0.8,0.82,0.93];}
            px.push(rgb(c));
        }
    }
    ColorImage::new([W,H],px)
}
fn channel(c:Color32)->[f32;3]{
    [c.r() as f32/255.0,c.g() as f32/255.0,c.b() as f32/255.0]
}
fn sample(src:&ColorImage,x:i32,y:i32)->[f32;3]{
    let x=x.clamp(0,src.size[0] as i32-1) as usize;
    let y=y.clamp(0,src.size[1] as i32-1) as usize;
    channel(src.pixels[y*src.size[0]+x])
}
fn sample_linear(src:&ColorImage,x:f32,y:f32)->[f32;3]{
    let x0=x.floor() as i32;
    let y0=y.floor() as i32;
    let tx=(x-x0 as f32).clamp(0.0,1.0);
    let ty=(y-y0 as f32).clamp(0.0,1.0);
    let a=sample(src,x0,y0);
    let b=sample(src,x0+1,y0);
    let c=sample(src,x0,y0+1);
    let d=sample(src,x0+1,y0+1);
    std::array::from_fn(|i|
        mix(mix(a[i],b[i],tx),mix(c[i],d[i],tx),ty)
    )
}
fn lum(c:[f32;3])->f32{c[0]*0.2126+c[1]*0.7152+c[2]*0.0722}
fn length(a:[f32;3])->f32{(a[0]*a[0]+a[1]*a[1]+a[2]*a[2]).sqrt()}
fn diff(a:[f32;3],b:[f32;3])->[f32;3]{
    [a[0]-b[0],a[1]-b[1],a[2]-b[2]]
}
fn blur4(src:&ColorImage,x:i32,y:i32,step:i32)->[f32;3]{
    let a=sample(src,x+step,y);let b=sample(src,x-step,y);
    let c=sample(src,x,y+step);let d=sample(src,x,y-step);
    [(a[0]+b[0]+c[0]+d[0])*0.25,
    (a[1]+b[1]+c[1]+d[1])*0.25,
    (a[2]+b[2]+c[2]+d[2])*0.25]
}
fn process_pixel(src:&ColorImage,x:i32,y:i32,index:usize,s:f32)->[f32;3]{
    let c=sample(src,x,y);
    let u=(x as f32+0.5)/src.size[0] as f32;
    let v=(y as f32+0.5)/src.size[1] as f32;
    let yl=lum(c);
    match index {
        1=>{
            let hi=c[0].max(c[1]).max(c[2]);
            let lo=c[0].min(c[1]).min(c[2]);
            let adaptive=clamp(1.0-(hi-lo)*1.2)*clamp(1.5-(yl-0.5).abs()*1.8);
            std::array::from_fn(|i|yl+(c[i]-yl)*(1.0+s*adaptive))
        }
        2=>{
            let b=blur4(src,x,y,1);let high=diff(c,b);
            let detail=lum(high).abs();
            let m=clamp(detail*14.0);
            std::array::from_fn(|i|c[i]+high[i].clamp(-0.15,0.15)*s*m)
        }
        3=>{
            let a=sample(src,x+1,y);let b=sample(src,x-1,y);
            let d=sample(src,x,y+1);let e=sample(src,x,y-1);
            let n=blur4(src,x,y,1);
            let lc=length(diff(c,a)).max(length(diff(c,b)))
                .max(length(diff(c,d))).max(length(diff(c,e)));
            let suppression=clamp(lc*4.0);
            std::array::from_fn(|i|c[i]+(c[i]-n[i]).clamp(-0.12,0.12)*s*(0.4+0.6*suppression)*2.0)
        }
        4=>std::array::from_fn(|i|{
            let k=c[i];
            let curve=clamp(k*(2.51*k+0.03)/(k*(2.43*k+0.59)+0.14));
            let warm=if i==0{1.01}else if i==2{0.988}else{1.0};
            let tone=mix(clamp(curve*warm),clamp(curve*warm)*0.94+0.015,
                clamp((0.45-yl)*2.0));
            mix(k,tone,s)
        }),
        5=>{
            let b=blur4(src,x,y,3);let l=lum(b);
            let mask=smooth(0.68,1.0,l);
            std::array::from_fn(|i|c[i]+b[i]*mask*s*0.35)
        }
        6=>{
            let dx=u-0.5;let dy=(v-0.5)*0.7;
            let d=(dx*dx+dy*dy).sqrt();
            let k=1.0-s*smooth(0.62*0.45,0.62,d);
            std::array::from_fn(|i|c[i]*k)
        }
        7=>{
            let dx=(u-0.5)*s*0.006*src.size[0] as f32;
            let dy=(v-0.5)*s*0.006*src.size[1] as f32;
            let a=sample_linear(src,x as f32+dx,y as f32+dy);
            let b=sample_linear(src,x as f32-dx,y as f32-dy);
            [a[0],c[1],b[2]]
        }
        8=>{
            let a=diff(sample(src,x+1,y),sample(src,x-1,y));
            let b=diff(sample(src,x,y+1),sample(src,x,y-1));
            let edge=clamp((length(a)+length(b))*2.2);
            std::array::from_fn(|i|mix(c[i],(c[i]*8.0+0.5).floor()/8.0,s*0.5)*
                (1.0-s*edge*0.7))
        }
        9=>{
            let n=((x as f32*12.9898+y as f32*78.233).sin()*43758.5453).fract();
            let n=if n<0.0{n+1.0}else{n};
            let amount=s*(0.5+0.5*clamp(1.0-yl))*0.16;
            std::array::from_fn(|i|c[i]+(n-0.5)*amount)
        }
        10=>{
            let lifted=clamp(yl).powf(0.62);
            let scan=1.0-0.03*(0.5+0.5*(v*src.size[1] as f32*1.6).sin());
            let rgb=[lifted*0.38*scan,lifted*1.03*scan,lifted*0.49*scan];
            std::array::from_fn(|i|mix(c[i],rgb[i],s))
        }
        11=>{
            let shadows=1.0-smooth(0.18,0.69,yl);
            let highlights=smooth(0.34,0.89,yl);
            let shift=[shadows*-0.055+highlights*0.07,
                shadows*0.03+highlights*0.024,
                shadows*0.07+highlights*-0.052];
            std::array::from_fn(|i|c[i]+shift[i]*s*2.0)
        }
        12=>{
            let a=sample(src,x+2,y+2);let b=sample(src,x-2,y-2);
            let avg: [f32;3]=std::array::from_fn(|i|(a[i]+b[i]+c[i])/3.0);
            let d=(0..3).map(|i|(c[i]-avg[i]).abs()).fold(0.0,f32::max);
            let mask=1.0-smooth(0.004,0.065,d);
            std::array::from_fn(|i|mix(c[i],avg[i],s*mask))
        }
        13=>{
            let rg=c[0]-c[1];
            let delta=[rg*0.055,rg*-0.025,rg*0.045];
            std::array::from_fn(|i|c[i]+delta[i]*s*4.0)
        }
        // Entries 14..19 are Magpie built-in ports. These CPU thumbnail
        // representations are only explanatory, NOT pixel-identical GPU output.
        14|15|16=>{
            let b=blur4(src,x,y,1);
            let factor=match index{14=>0.75,15=>0.95,_=>0.68};
            std::array::from_fn(|i|c[i]+(c[i]-b[i]).clamp(-0.08,0.08)*s*factor)
        }
        17|18=>{
            let b=blur4(src,x,y,1);let edge=length(diff(c,b));
            let weight=clamp(edge*4.0)*s*(if index==18{0.55}else{0.38});
            std::array::from_fn(|i|mix(c[i],b[i],weight))
        }
        19=>{
            let a=sample(src,x+2,y+2);let b=sample(src,x-2,y-2);
            std::array::from_fn(|i|mix(c[i],(a[i]+b[i]+c[i])/3.0,s*0.45))
        }
        _=>c,
    }
}
pub fn render_stack_reference(src:&ColorImage,stack:usize,strength:f32,
    custom:&[crate::stacks::CustomStage])->ColorImage{
    let mut result=src.clone();
    let steps=if stack==crate::stacks::CUSTOM_INDEX{
        custom.iter().take(4).map(|s|(s.effect,s.weight*strength.clamp(0.0,1.0)))
            .collect::<Vec<_>>()
    }else{crate::stacks::active_stages(stack,strength)};
    for (index,pass_strength) in steps{
        if index==0||index>=crate::engine::EFFECTS.len(){continue;}
        result=render_reference(&result,index,pass_strength);
    }
    result
}
pub fn render_reference(src:&ColorImage,index:usize,strength:f32)->ColorImage{
    let strength=strength.clamp(0.0,1.0);
    let mut out=src.clone();
    for y in 0..src.size[1]{
        for x in 0..src.size[0]{
            let c=process_pixel(src,x as i32,y as i32,index,strength);
            out.pixels[y*src.size[0]+x]=rgb(c);
        }
    }
    out
}
fn small(src:&ColorImage,w:usize,h:usize)->ColorImage{
    let mut out=Vec::with_capacity(w*h);
    for y in 0..h {
        for x in 0..w{
            let sx=(x*src.size[0]/w).min(src.size[0]-1);
            let sy=(y*src.size[1]/h).min(src.size[1]-1);
            out.push(src.pixels[sy*src.size[0]+sx]);
        }
    }
    ColorImage::new([w,h],out)
}
impl PreviewBank {
    pub fn new(ctx:&Context)->Self{
        let mut bank=Self{
            source:None,original:None,selected:None,selected_key:None,
            thumbs:(0..EFFECT_COUNT).map(|_|None).collect(),
            stack_thumbs:(0..crate::stacks::STACKS.len()).map(|_|None).collect(),
            custom_thumb_sig:0,
            single_thumb_effect:usize::MAX,
            generation:0,source_label:BUILTIN_PHOTOS[0].0.into()
        };
        bank.set_source(ctx,real_photo(0).unwrap_or_else(|_|scene()));
        bank
    }
    pub fn set_source(&mut self,ctx:&Context,image:ColorImage){
        self.generation=self.generation.wrapping_add(1);
        self.original=Some(ctx.load_texture(
            "hb-preview-original",image.clone(),TextureOptions::LINEAR));
        self.source=Some(image);
        self.selected=None;
        self.selected_key=None;
        self.thumbs=(0..EFFECT_COUNT).map(|_|None).collect();
        self.stack_thumbs=(0..crate::stacks::STACKS.len()).map(|_|None).collect();
        self.custom_thumb_sig=0;
        self.single_thumb_effect=usize::MAX;
    }
    pub fn restore(&mut self,ctx:&Context){self.use_photo(ctx,0);}
    pub fn use_photo(&mut self,ctx:&Context,index:usize){
        if let Ok(photo)=real_photo(index){
            self.source_label=BUILTIN_PHOTOS[index].0.into();
            self.set_source(ctx,photo);
        }
    }
    pub fn from_file(&mut self,ctx:&Context,path:&Path)->Result<(),String>{
        let metadata=std::fs::metadata(path).map_err(|e|e.to_string())?;
        if metadata.len()>25_000_000 {
            return Err("Preview image must be under 25 MB".into());
        }
        let image=image::open(path).map_err(|e|format!("Cannot open image: {e}"))?;
        if image.width()>8000||image.height()>8000||image.width()<64||image.height()<64{
            return Err("Supported preview sizes: 64–8000 pixels per side".into());
        }
        let image=image.resize_to_fill(W as u32,H as u32,image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        self.set_source(ctx,ColorImage::from_rgba_unmultiplied([W,H],image.as_raw()));
        self.source_label=path.file_name().map(|s|s.to_string_lossy().to_string())
            .unwrap_or_else(||"Local preview image".into());
        Ok(())
    }
    pub fn original(&self)->Option<&TextureHandle>{self.original.as_ref()}
    pub fn selected(&mut self,ctx:&Context,index:usize,strength:f32,stack:usize,
        custom:&[crate::stacks::CustomStage])->Option<&TextureHandle>{
        let signature=custom.iter().fold(1469598103934665603u64,|hash,s|{
            let h=hash^s.effect as u64;
            (h.wrapping_mul(1099511628211)^s.weight.to_bits() as u64).wrapping_mul(1099511628211)
        });
        let key=(index,(strength.clamp(0.0,1.0)*100.0).round() as u8,self.generation,stack,
            if stack==crate::stacks::CUSTOM_INDEX{signature}else{0});
        if self.selected_key!=Some(key){
            let src=self.source.as_ref()?;
            let output=if stack==0{render_reference(src,index,strength)}
                else{render_stack_reference(src,stack,strength,custom)};
            match self.selected.as_mut(){
                Some(t)=>t.set(output,TextureOptions::LINEAR),
                None=>self.selected=Some(ctx.load_texture(
                    "hb-preview-selected",output,TextureOptions::LINEAR))
            }
            self.selected_key=Some(key);
        }
        self.selected.as_ref()
    }
    pub fn thumbnail(&mut self,ctx:&Context,index:usize)->Option<&TextureHandle>{
        if index>=EFFECT_COUNT{return None;}
        if self.thumbs[index].is_none(){
            let src=small(self.source.as_ref()?,TW,TH);
            let processed=render_reference(&src,index,0.7);
            self.thumbs[index]=Some(ctx.load_texture(
                format!("hb-filter-thumb-{index}-{}",self.generation),processed,
                TextureOptions::LINEAR));
        }
        self.thumbs[index].as_ref()
    }
    pub fn stack_thumbnail(&mut self,ctx:&Context,stack:usize,
        single_effect:usize,custom:&[crate::stacks::CustomStage])->Option<&TextureHandle>{
        if stack>=crate::stacks::STACKS.len(){return None;}
        if stack==0&&self.single_thumb_effect!=single_effect{
            self.stack_thumbs[0]=None;
            self.single_thumb_effect=single_effect;
        }
        if stack==crate::stacks::CUSTOM_INDEX {
            let hash=custom.iter().fold(0xcbf29ce484222325_u64,|x,s|
                (x^s.effect as u64).wrapping_mul(0x100000001b3)^s.weight.to_bits() as u64
            );
            if self.custom_thumb_sig!=hash {
                self.stack_thumbs[stack]=None;
                self.custom_thumb_sig=hash;
            }
        }
        if self.stack_thumbs[stack].is_none(){
            let src=small(self.source.as_ref()?,TW,TH);
            let processed=if stack==0{render_reference(&src,single_effect,1.0)}
                else{render_stack_reference(&src,stack,1.0,custom)};
            let texture=ctx.load_texture(
                format!("hb-stack-preview-{stack}-{}-{}",self.generation,
                    self.custom_thumb_sig),processed,TextureOptions::LINEAR);
            self.stack_thumbs[stack]=Some(texture);
        }
        self.stack_thumbs[stack].as_ref()
    }
    pub fn exact_reference(index:usize)->bool{index<=13}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn actual_public_domain_photos_not_illustrations(){
        assert_eq!(BUILTIN_PHOTOS.len(),2);
        for i in 0..2{
            let photo=real_photo(i).expect("Every bundled real photo decodes");
            assert_eq!(photo.size,[W,H]);
            let different=photo.pixels.windows(2).filter(|v|v[0]!=v[1]).count();
            assert!(different>10_000,"Reference should contain photographic image detail");
        }
    }
    #[test]fn generates_all_twenty_thumbnails(){
        let image=scene();
        assert_eq!(image.size,[W,H]);
        let scaled=small(&image,TW,TH);
        for i in 0..20 {
            let p=render_reference(&scaled,i,0.7);
            assert_eq!(p.pixels.len(),TW*TH);
            assert!(p.pixels.iter().all(|c|c.a()==255));
        }
    }
    #[test]fn off_is_bit_exact(){
        let image=scene();
        assert_eq!(render_reference(&image,0,1.0).pixels,image.pixels);
    }
    #[test]fn intensity_zero_is_neutral_for_all(){
        let sample=small(&scene(),TW,TH);
        for i in 0..20{
            let output=render_reference(&sample,i,0.0);
            // Some built-ins are approximated, but all must become identity at 0.
            assert_eq!(output.pixels,sample.pixels,"filter index {i}");
        }
    }
    #[test]fn stack_preview_uses_real_sequential_cpu_passes(){
        let source=small(&scene(),TW,TH);
        let out=render_stack_reference(&source,2,0.85,&[]);
        assert_ne!(out.pixels,source.pixels);
        let direct=crate::stacks::active_stages(2,0.85).into_iter()
            .fold(source.clone(),|image,(effect,amount)|
                render_reference(&image,effect,amount));
        assert_eq!(out.pixels,direct.pixels);
        assert_eq!(render_stack_reference(&source,9,1.0,&[]).pixels,source.pixels);
        assert_eq!(render_stack_reference(&source,2,0.0,&[]).pixels,source.pixels);
    }
    #[test]fn custom_reference_matches_ordered_manual_passes(){
        let source=small(&scene(),TW,TH);
        let filters=[crate::stacks::CustomStage{effect:3,weight:0.7},
            crate::stacks::CustomStage{effect:11,weight:0.6}];
        let a=render_stack_reference(&source,crate::stacks::CUSTOM_INDEX,1.0,&filters);
        let b=render_reference(&render_reference(&source,3,0.7),11,0.6);
        assert_eq!(a.pixels,b.pixels);
    }
    #[test]fn filter_previews_are_visually_distinct(){
        let sample=small(&scene(),TW,TH);
        let count=(1..14).filter(|&i|render_reference(&sample,i,0.75).pixels!=sample.pixels).count();
        assert_eq!(count,13,"Every custom filter should alter the benchmark image");
    }
}
