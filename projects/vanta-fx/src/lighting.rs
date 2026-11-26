//! Native, external capture-only SDR color-light studio. Not ray tracing.
//! Never needs Roblox game depth/geometry, process access, hook or file writes.
use serde::{Deserialize,Serialize};
pub const GPU_LIGHT_STAGE:usize=36; // appended after the original 28 programs
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct LightTuning{
    pub enabled:bool,
    pub shadow:f32,
    pub glow:f32,
    pub bounce:f32,
    pub detail:f32,
    pub film:f32,
    pub warmth:f32,
    pub mix:f32,
    pub grade:crate::color_grade::Grade,
    pub layers:crate::layer_mix::Layers,
    pub motion:crate::motion_options::Options,
    pub lut:crate::live_lut::Options,
    pub exposure:crate::exposure_options::Options,
    pub reconstruction:crate::reconstruction::Options,
    pub precision:crate::precision_pass::Options,
    pub recipe:crate::color_recipe::Recipe,
    pub cinema:crate::cinema_light::Options,
    pub bypass_look:bool,
}
impl Default for LightTuning{
    fn default()->Self{
        Self{enabled:false,shadow:0.36,glow:0.31,bounce:0.25,
            detail:0.24,film:0.26,warmth:0.12,mix:0.80,grade:crate::color_grade::Grade::default(),layers:Default::default(),motion:Default::default(),lut:Default::default(),exposure:Default::default(),reconstruction:Default::default(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),bypass_look:false}
    }
}
impl LightTuning{
    pub fn clamp(&mut self){
        self.grade.clamp();self.layers.clamp();self.motion.clamp();self.lut.clamp();self.exposure.clamp();self.reconstruction.clamp();self.precision.clamp();self.recipe.sanitize();self.cinema.clamp();
        for v in [&mut self.shadow,&mut self.glow,&mut self.bounce,
            &mut self.detail,&mut self.film,&mut self.warmth,&mut self.mix]{
            *v=if v.is_finite(){v.clamp(0.0,1.0)}else{0.0};
        }
    }
    pub fn gpu(&self)->GpuLightParams{
        GpuLightParams{
            a:[self.shadow,self.glow,self.bounce,self.detail],
            b:[self.film,self.warmth,self.mix,0.0]
        }
    }
    pub fn zero()->Self{
        Self{enabled:true,shadow:0.0,glow:0.0,bounce:0.0,
            detail:0.0,film:0.0,warmth:0.0,mix:0.0,grade:crate::color_grade::Grade::default(),layers:Default::default(),motion:Default::default(),lut:Default::default(),exposure:Default::default(),reconstruction:Default::default(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),bypass_look:false}
    }
}
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct GpuLightParams{
    pub a:[f32;4],
    pub b:[f32;4],
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn cbuffer_size_and_default_off(){
        assert_eq!(std::mem::size_of::<GpuLightParams>(),32);
        assert!(!LightTuning::default().enabled);
        assert!(LightTuning::default().glow>0.0);
    }
    #[test]fn light_settings_clamped_and_roundtrip(){
        let mut settings=LightTuning{enabled:true,shadow:f32::NAN,
            glow:5.0,bounce:-4.0,detail:1.0,film:0.4,
            warmth:0.3,mix:0.5,grade:crate::color_grade::Grade::default(),layers:Default::default(),motion:Default::default(),lut:Default::default(),exposure:Default::default(),reconstruction:Default::default(),precision:Default::default(),recipe:Default::default(),cinema:Default::default(),bypass_look:false};
        settings.clamp();
        assert_eq!(settings.shadow,0.0);assert_eq!(settings.glow,1.0);
        assert_eq!(settings.bounce,0.0);
        let again:LightTuning=serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(again,settings);
    }
    #[test]fn zero_gives_valid_gpu_neutral_state(){
        let l=LightTuning::zero();
        assert_eq!(l.gpu().a,[0.0;4]);assert_eq!(l.gpu().b,[0.0;4]);
    }
}
