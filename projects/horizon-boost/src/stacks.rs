//! Multistage original shader presets. Each stage is an ACTUAL MagpieFX shader
//! pass in a sequential effect chain, not a color-gamma-only preset.
#[derive(Clone,Copy,Debug)]
pub struct Stage{pub effect:usize,pub weight:f32}
#[derive(Clone,Copy,Debug)]
pub struct VisualStack{
    pub label:&'static str,
    pub desc:&'static str,
    pub cost:&'static str,
    pub stages:&'static [Stage],
}
const COMPETITIVE:&[Stage]=&[
    Stage{effect:1,weight:0.60},Stage{effect:2,weight:0.34},Stage{effect:3,weight:0.34},
];
const CINEMATIC:&[Stage]=&[
    Stage{effect:4,weight:0.75},Stage{effect:5,weight:0.38},
    Stage{effect:11,weight:0.52},Stage{effect:6,weight:0.21},
];
const NIGHT:&[Stage]=&[
    Stage{effect:10,weight:0.42},Stage{effect:2,weight:0.38},
    Stage{effect:3,weight:0.28},
];
const ANIME:&[Stage]=&[
    Stage{effect:8,weight:0.62},Stage{effect:1,weight:0.55},
    Stage{effect:3,weight:0.36},
];
const NATURAL:&[Stage]=&[
    Stage{effect:4,weight:0.26},Stage{effect:2,weight:0.32},
    Stage{effect:12,weight:0.22},
];
const RETRO:&[Stage]=&[
    Stage{effect:4,weight:0.44},Stage{effect:9,weight:0.31},
    Stage{effect:6,weight:0.36},
];
const COLOR_POP:&[Stage]=&[
    Stage{effect:1,weight:0.78},Stage{effect:11,weight:0.24},
    Stage{effect:2,weight:0.28},
];
const SOFT_CLEAN:&[Stage]=&[
    Stage{effect:12,weight:0.42},Stage{effect:3,weight:0.22},
    Stage{effect:5,weight:0.14},
];
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(default)]
pub struct CustomStage { pub effect:usize,pub weight:f32 }
impl Default for CustomStage {fn default()->Self{Self{effect:2,weight:0.45}}}
pub fn default_custom()->Vec<CustomStage>{
    vec![CustomStage{effect:4,weight:0.40},CustomStage{effect:2,weight:0.32}]
}
// Move real render passes; order directly determines the external HLSL chain.
pub fn move_custom_stage(stages:&mut [CustomStage],from:usize,delta:i32)->bool{
    let dest=match delta{
        -1=>from.checked_sub(1),
         1=>from.checked_add(1),
         _=>None
    };
    if from>=stages.len(){return false;}
    if let Some(to)=dest.filter(|&to|to<stages.len()){
        stages.swap(from,to);
        true
    }else{false}
}
pub const CUSTOM_INDEX:usize=22;
const FANTASY:&[Stage]=&[Stage{effect:4,weight:0.40},Stage{effect:5,weight:0.40},
    Stage{effect:1,weight:0.38},Stage{effect:2,weight:0.30}];
const CRYSTAL:&[Stage]=&[Stage{effect:2,weight:0.55},Stage{effect:3,weight:0.32},
    Stage{effect:12,weight:0.35}];
const DUSK:&[Stage]=&[Stage{effect:4,weight:0.53},Stage{effect:11,weight:0.53},
    Stage{effect:6,weight:0.22},Stage{effect:5,weight:0.30}];
const DUNGEON:&[Stage]=&[Stage{effect:10,weight:0.20},Stage{effect:2,weight:0.45},
    Stage{effect:4,weight:0.32}];
const CEL:&[Stage]=&[Stage{effect:8,weight:0.52},Stage{effect:1,weight:0.43},
    Stage{effect:12,weight:0.25}];
const COZY:&[Stage]=&[Stage{effect:4,weight:0.36},Stage{effect:1,weight:0.28},
    Stage{effect:5,weight:0.31}];
const CYBER:&[Stage]=&[Stage{effect:1,weight:0.59},Stage{effect:5,weight:0.42},
    Stage{effect:7,weight:0.25},Stage{effect:11,weight:0.40}];
const OCEAN:&[Stage]=&[Stage{effect:11,weight:0.53},Stage{effect:1,weight:0.38},
    Stage{effect:2,weight:0.29}];
const DESERT:&[Stage]=&[Stage{effect:4,weight:0.47},Stage{effect:6,weight:0.23},
    Stage{effect:9,weight:0.15}];
const FOG:&[Stage]=&[Stage{effect:4,weight:0.23},Stage{effect:5,weight:0.44},
    Stage{effect:12,weight:0.41}];
const SUNSET:&[Stage]=&[Stage{effect:4,weight:0.33},Stage{effect:11,weight:0.57},
    Stage{effect:5,weight:0.23}];
const DETAIL:&[Stage]=&[Stage{effect:2,weight:0.57},Stage{effect:3,weight:0.35},
    Stage{effect:16,weight:0.34}];
const V16_SCENE_01:&[Stage]=&[Stage{effect:2,weight:0.56},Stage{effect:3,weight:0.23}];
const V16_SCENE_02:&[Stage]=&[Stage{effect:4,weight:0.34},Stage{effect:11,weight:0.60},Stage{effect:2,weight:0.29}];
const V16_SCENE_03:&[Stage]=&[Stage{effect:4,weight:0.64},Stage{effect:9,weight:0.18},Stage{effect:6,weight:0.22}];
const V16_SCENE_04:&[Stage]=&[Stage{effect:1,weight:0.36},Stage{effect:4,weight:0.20},Stage{effect:12,weight:0.34}];
const V16_SCENE_05:&[Stage]=&[Stage{effect:10,weight:0.53},Stage{effect:12,weight:0.30},Stage{effect:3,weight:0.22}];
const V16_SCENE_06:&[Stage]=&[Stage{effect:1,weight:0.51},Stage{effect:5,weight:0.69},Stage{effect:11,weight:0.24}];
const V16_SCENE_07:&[Stage]=&[Stage{effect:10,weight:0.31},Stage{effect:2,weight:0.48},Stage{effect:16,weight:0.25}];
const V16_SCENE_08:&[Stage]=&[Stage{effect:4,weight:0.49},Stage{effect:6,weight:0.32},Stage{effect:9,weight:0.30}];
const V16_SCENE_09:&[Stage]=&[Stage{effect:1,weight:0.85},Stage{effect:5,weight:0.64},Stage{effect:7,weight:0.17}];
const V16_SCENE_10:&[Stage]=&[Stage{effect:11,weight:0.48},Stage{effect:12,weight:0.60},Stage{effect:3,weight:0.46}];
const V16_SCENE_11:&[Stage]=&[Stage{effect:4,weight:0.27},Stage{effect:1,weight:0.18},Stage{effect:5,weight:0.25}];
const V16_SCENE_12:&[Stage]=&[Stage{effect:18,weight:0.52},Stage{effect:3,weight:0.30},Stage{effect:12,weight:0.40}];
const V16_SCENE_13:&[Stage]=&[Stage{effect:1,weight:0.91},Stage{effect:2,weight:0.38},Stage{effect:17,weight:0.24}];
const V16_SCENE_14:&[Stage]=&[Stage{effect:4,weight:0.70},Stage{effect:6,weight:0.65},Stage{effect:9,weight:0.42}];
const V16_SCENE_15:&[Stage]=&[Stage{effect:5,weight:0.62},Stage{effect:11,weight:0.44},Stage{effect:12,weight:0.22}];
const V16_SCENE_16:&[Stage]=&[Stage{effect:10,weight:0.68},Stage{effect:2,weight:0.44},Stage{effect:12,weight:0.20}];
const V16_SCENE_17:&[Stage]=&[Stage{effect:4,weight:0.37},Stage{effect:3,weight:0.40},Stage{effect:1,weight:0.28}];
const V16_SCENE_18:&[Stage]=&[Stage{effect:11,weight:0.64},Stage{effect:2,weight:0.40},Stage{effect:5,weight:0.14}];
const V16_SCENE_19:&[Stage]=&[Stage{effect:4,weight:0.51},Stage{effect:9,weight:0.43},Stage{effect:6,weight:0.49},Stage{effect:7,weight:0.06}];
const V16_SCENE_20:&[Stage]=&[Stage{effect:5,weight:0.52},Stage{effect:12,weight:0.64},Stage{effect:4,weight:0.23}];
const V16_SCENE_21:&[Stage]=&[Stage{effect:8,weight:0.70},Stage{effect:1,weight:0.55},Stage{effect:12,weight:0.45}];
const V16_SCENE_22:&[Stage]=&[Stage{effect:3,weight:0.35},Stage{effect:12,weight:0.52},Stage{effect:4,weight:0.22}];
const V16_SCENE_23:&[Stage]=&[Stage{effect:1,weight:0.29},Stage{effect:2,weight:0.36},Stage{effect:4,weight:0.22}];
const V16_SCENE_24:&[Stage]=&[Stage{effect:10,weight:0.46},Stage{effect:2,weight:0.52},Stage{effect:13,weight:0.32}];
pub const STACKS:&[VisualStack]=&[
    VisualStack{label:"Single Filter",desc:"One of the 20 standalone effects",cost:"Variable",stages:&[]},
    VisualStack{label:"Competitive Clarity",desc:"Better contrast and visibility, controlled detail",cost:"3 passes",stages:COMPETITIVE},
    VisualStack{label:"Cinematic Atmosphere",desc:"Filmic highlights, soft bloom, teal-orange grade and vignette",cost:"4 passes",stages:CINEMATIC},
    VisualStack{label:"Night Explorer",desc:"Lift dark scenes with night optics + texture clarity",cost:"3 passes",stages:NIGHT},
    VisualStack{label:"Anime Worlds",desc:"Stylized edge ink, vibrant shading and sharpened details",cost:"3 passes",stages:ANIME},
    VisualStack{label:"Natural Detail",desc:"Subtle filmic tones, microcontrast and smooth gradients",cost:"3 passes",stages:NATURAL},
    VisualStack{label:"Retro Film",desc:"Classic filmic contrast, fine grain and soft vignette",cost:"3 passes",stages:RETRO},
    VisualStack{label:"Color Pop",desc:"Adaptive color separation with shadow/highlight styling",cost:"3 passes",stages:COLOR_POP},
    VisualStack{label:"Soft Clean",desc:"Gentle deband and sharpness with soft glow",cost:"3 passes",stages:SOFT_CLEAN},
    VisualStack{label:"FPS Priority",desc:"No additional filter passes. Preserve GPU time.",cost:"0 extra passes",stages:&[]},
    VisualStack{label:"Fantasy Bloom",desc:"Subtle RPG glow, warm colors and textural contrast",cost:"4 passes",stages:FANTASY},
    VisualStack{label:"Crystal Clear",desc:"Detail and gradient smoothing for adventure scenes",cost:"3 passes",stages:CRYSTAL},
    VisualStack{label:"Dusk Cinematic",desc:"Evening film grades, softness and lens vignette",cost:"4 passes",stages:DUSK},
    VisualStack{label:"Dungeon Lantern",desc:"Dim-environment luminance lift with local detail",cost:"3 passes",stages:DUNGEON},
    VisualStack{label:"Cel-Shaded Adventure",desc:"Cartoon edges, vibrant detail and debanding",cost:"3 passes",stages:CEL},
    VisualStack{label:"Cozy RPG",desc:"Warm film tone, subtle saturation and soft highlights",cost:"3 passes",stages:COZY},
    VisualStack{label:"Neon Cyberpunk",desc:"Stronger neon, bloom and edge chromatic lens",cost:"4 passes",stages:CYBER},
    VisualStack{label:"Ocean Color",desc:"Cyan/orange separation and natural texture contrast",cost:"3 passes",stages:OCEAN},
    VisualStack{label:"Desert Film",desc:"Dusty warm film curve with grain and vignette",cost:"3 passes",stages:DESERT},
    VisualStack{label:"Soft Atmosphere",desc:"Soft highlight glow, gently balanced contrast and debanding",cost:"3 passes",stages:FOG},
    VisualStack{label:"Golden Sunset",desc:"Sunset highlights and warm film-grade luminance",cost:"3 passes",stages:SUNSET},
    VisualStack{label:"High Detail",desc:"Microcontrast + two controlled sharpen stages",cost:"3 passes",stages:DETAIL},
    VisualStack{label:"My Custom Stack",desc:"Edit your own 1 to 4 shader stages and strengths",cost:"Editable",stages:&[]},
    VisualStack{label:"Tactical Minimal",desc:"Clean competitive visibility, controlled detail",cost:"2 passes",stages:V16_SCENE_01},
    VisualStack{label:"Moonlit Tundra",desc:"Cool twilight color and more readable edges",cost:"3 passes",stages:V16_SCENE_02},
    VisualStack{label:"Crimson Ash",desc:"Aggressive film grain and fiery contrast",cost:"3 passes",stages:V16_SCENE_03},
    VisualStack{label:"Moss & Forest",desc:"Natural greens, softer highlights, clear texture",cost:"3 passes",stages:V16_SCENE_04},
    VisualStack{label:"Midnight Stealth",desc:"Luminance lift and low-glare edges",cost:"3 passes",stages:V16_SCENE_05},
    VisualStack{label:"Aurora Dream",desc:"Dreamy chroma and soft glow",cost:"3 passes",stages:V16_SCENE_06},
    VisualStack{label:"Hardcore Survival",desc:"Sharper survival detail and dark-area visibility",cost:"3 passes",stages:V16_SCENE_07},
    VisualStack{label:"Desaturated War",desc:"Muted cinematic color and grain",cost:"3 passes",stages:V16_SCENE_08},
    VisualStack{label:"Neon Pulse",desc:"Vibrant night lights and controlled glow",cost:"3 passes",stages:V16_SCENE_09},
    VisualStack{label:"Winter Crystal",desc:"Cold clean whites and edge detail",cost:"3 passes",stages:V16_SCENE_10},
    VisualStack{label:"Golden Portrait",desc:"Warm but natural skin tone and subtle film",cost:"3 passes",stages:V16_SCENE_11},
    VisualStack{label:"Sci-Fi HUD Clean",desc:"Minimal edge antialiasing and crisp details",cost:"3 passes",stages:V16_SCENE_12},
    VisualStack{label:"Arcade Max Color",desc:"Strong color, anti-aliasing and microcontrast",cost:"3 passes",stages:V16_SCENE_13},
    VisualStack{label:"Noir Detective",desc:"Moody filmic tones, vignette and grain",cost:"3 passes",stages:V16_SCENE_14},
    VisualStack{label:"Lavender Bloom",desc:"Soft-violet dreamlike image with highlight bloom",cost:"3 passes",stages:V16_SCENE_15},
    VisualStack{label:"Dungeon Scout",desc:"Night readability with gentle detail",cost:"3 passes",stages:V16_SCENE_16},
    VisualStack{label:"Bright Daylight",desc:"Strong but controlled highlight and texture details",cost:"3 passes",stages:V16_SCENE_17},
    VisualStack{label:"Ocean Depths",desc:"Cyan-rich water mood and clear fine detail",cost:"3 passes",stages:V16_SCENE_18},
    VisualStack{label:"Old Camera",desc:"Vintage contrast, soft edge shade and fine grain",cost:"4 passes",stages:V16_SCENE_19},
    VisualStack{label:"Pastel Clouds",desc:"Gentle glow with reduced harshness",cost:"3 passes",stages:V16_SCENE_20},
    VisualStack{label:"Cel Adventure Plus",desc:"Dense animated-world ink lines and smooth gradients",cost:"3 passes",stages:V16_SCENE_21},
    VisualStack{label:"Sharp Retro Pixels",desc:"Pixel art friendly contrast and anti-gradient noise",cost:"3 passes",stages:V16_SCENE_22},
    VisualStack{label:"Balanced RPG",desc:"Lightweight natural color & microcontrast",cost:"3 passes",stages:V16_SCENE_23},
    VisualStack{label:"Accessibility High Contrast",desc:"Distinct edges and visibility in dark areas",cost:"3 passes",stages:V16_SCENE_24},
];
pub fn active_stages(index:usize,strength:f32)->Vec<(usize,f32)>{
    if index==0||index>=STACKS.len(){return vec![];}
    STACKS[index].stages.iter().map(|s|
        (s.effect,(s.weight*strength.clamp(0.0,1.0)).min(1.0))
    ).collect()
}
/// Compact browsing pages; originals keep their persistent numeric indices.
pub fn preset_category_indices(category:usize)->Vec<usize>{
    match category {
        0=>vec![0,1,2,3,4,5,7,9,15,22,23,25,29,38],
        1=>vec![3,4,10,13,14,15,17,23,24,26,27,32,36,38,40,43,45],
        2=>vec![2,6,8,12,16,18,19,20,25,28,31,33,34,35,37,41,42],
        3=>vec![0,1,3,9,11,21,23,27,29,30,33,34,39,44,46],
        _=>(0..STACKS.len()).collect()
    }
}
pub fn valid(index:usize)->usize{index.min(STACKS.len()-1)}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn custom_shader_order_really_changes(){
        let mut s=default_custom();
        let old_first=s[0].effect;
        let old_second=s[1].effect;
        assert!(move_custom_stage(&mut s,1,-1));
        assert_eq!(s[0].effect,old_second);
        assert_eq!(s[1].effect,old_first);
        assert!(!move_custom_stage(&mut s,0,-1));
        assert!(!move_custom_stage(&mut s,1,1));
        assert!(!move_custom_stage(&mut s,100,-1));
        assert_eq!(s.len(),2);
    }
    #[test]fn all_stacks_map_to_real_gpu_effects(){
        assert!(STACKS.len()>=9);
        for stack in STACKS {
            assert!(stack.stages.len()<=4);
            for stage in stack.stages {
                assert!(stage.effect>0 && stage.effect<crate::engine::EFFECTS.len());
                assert!(stage.weight>0.0 && stage.weight<=1.0);
            }
        }
    }
    #[test]fn zero_strength_disables_every_added_stage(){
        for idx in 1..STACKS.len(){
            assert!(active_stages(idx,0.0).iter().all(|v|v.1==0.0));
        }
    }
    #[test]fn all_preset_ids_stable_for_existing_users(){
        assert_eq!(CUSTOM_INDEX,22);
        assert_eq!(STACKS.len(),47);
        assert_eq!(STACKS[9].label,"FPS Priority");
        assert_eq!(STACKS[CUSTOM_INDEX].label,"My Custom Stack");
    }
    #[test]fn new_scene_presets_have_distinct_shaders_and_preserve_old_indices(){
        assert_eq!(CUSTOM_INDEX,22);
        assert_eq!(STACKS[0].label,"Single Filter");
        assert_eq!(STACKS[22].label,"My Custom Stack");
        assert_eq!(STACKS.len(),47);
        let unique=STACKS[23..].iter().map(|preset|{
            preset.stages.iter().map(|pass|(pass.effect,pass.weight.to_bits()))
                .collect::<Vec<_>>()
        }).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(),24);
    }
    #[test]fn presets_actually_include_multistage_effects(){
        assert_eq!(active_stages(2,1.0).len(),4);
        assert_eq!(active_stages(9,1.0).len(),0);
        assert_eq!(active_stages(0,1.0).len(),0);
    }
}
