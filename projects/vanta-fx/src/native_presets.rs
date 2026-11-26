//! Original HORIZON BOOST native D3D11 83-preset GPU look library.
//! Each sequence invokes REAL cs_5_0 shader programs on GPU, not Magpie.
//! The photo previews are baked from actual RX 580 shader output; this module
//! does not claim the filtered frame appears over any game/exclusive fullscreen.
#[derive(Clone,Copy,Debug)]
pub struct NativePreset{
    pub label:&'static str,
    pub desc:&'static str,
    pub stages:&'static [usize],
}
pub const PRESETS:&[NativePreset]=&[
    NativePreset{label:"Polaris Natural",desc:"Balanced outdoor texture and color",stages:&[0,2]},
    NativePreset{label:"Filmic Cinema",desc:"Film highlight curve with fine detail",stages:&[1,2]},
    NativePreset{label:"Clear Sight",desc:"Low-cost clarity for competitive scenes",stages:&[2]},
    NativePreset{label:"Night Ops",desc:"Strong green-tinted night optics",stages:&[3,2]},
    NativePreset{label:"Shadow Scout",desc:"Dark-region boost without full-screen brightness",stages:&[4,2]},
    NativePreset{label:"Amber Golden Hour",desc:"Warm sunlight film grade",stages:&[5,1,0]},
    NativePreset{label:"Blue Moon",desc:"Cool moonlit detail",stages:&[6,2]},
    NativePreset{label:"Noir Detective",desc:"Contrast-rich monochrome",stages:&[7,2]},
    NativePreset{label:"Sepia Adventure",desc:"Old world sepia texture",stages:&[8,1]},
    NativePreset{label:"Color Vision Assist",desc:"Red-green separation control",stages:&[9,2]},
    NativePreset{label:"Retro Pixel",desc:"Flat-color stylization and sharpening",stages:&[10,2]},
    NativePreset{label:"Crystal Clean",desc:"Cool balanced microcontrast",stages:&[11,2]},
    NativePreset{label:"Dungeon Visibility",desc:"Shadow boost and color assist",stages:&[4,9,2]},
    NativePreset{label:"Fantasy Warmth",desc:"Golden adventure colors and film",stages:&[5,0,1]},
    NativePreset{label:"Cinematic Noir",desc:"Monochrome film and detail",stages:&[7,1,2]},
    NativePreset{label:"Morning Mountains",desc:"Cool clean with a pop of color",stages:&[11,0,2]},
    NativePreset{label:"Neon Action",desc:"Bright synthetic color edge separation",stages:&[0,11,2]},
    NativePreset{label:"Tactical Minimum",desc:"Minimal detail + color separation",stages:&[2,9]},
    NativePreset{label:"Historical RPG",desc:"Sepia and retro film response",stages:&[8,10,1]},
    NativePreset{label:"Moonlight Explorer",desc:"Cold ambience with shadow recovery",stages:&[6,4,2]},
    NativePreset{label:"Vivid RPG",desc:"Vibrant warmth and clean blues",stages:&[5,11,0]},
    NativePreset{label:"Deep Dusk",desc:"Moody cold highlights and film",stages:&[6,1,0]},
    NativePreset{label:"Soft Portrait",desc:"Natural film with color correction",stages:&[1,11,9]},
    NativePreset{label:"Polar Night",desc:"Night-mode green-blue cross style",stages:&[6,3,4]},
    NativePreset{label:"Prism Halo",desc:"Bloom with preserved color and cinematic highlights",stages:&[12, 1]},
    NativePreset{label:"Dawn Lightshafts",desc:"Artistic directional lightshafts and sunrise warmth",stages:&[13, 5, 1]},
    NativePreset{label:"Glimmer Cavern",desc:"Faux color-bounce lighting in dungeon shadows",stages:&[14, 4, 2]},
    NativePreset{label:"Contact Drama",desc:"Edge-derived contact darkening, not real SSAO",stages:&[15, 1, 2]},
    NativePreset{label:"Neon Boulevard",desc:"Colored edge illumination over night color",stages:&[16, 6, 12]},
    NativePreset{label:"Manga Lines",desc:"Ink outlines and cel shading",stages:&[17, 2]},
    NativePreset{label:"Comic RPG",desc:"Graphic-novel toon stylization with color grading",stages:&[17, 5, 2]},
    NativePreset{label:"Painter Valley",desc:"Watercolor diffusion and warmth",stages:&[18, 5]},
    NativePreset{label:"Retro Arcade",desc:"CRT phosphor masking and posterized detail",stages:&[19, 10, 2]},
    NativePreset{label:"VHS Dream",desc:"Analog chroma noise and color bleed",stages:&[20, 8]},
    NativePreset{label:"Cinematic Widescreen",desc:"Anamorphic streaks with filmic highlights",stages:&[21, 1]},
    NativePreset{label:"Winter Frost",desc:"Frosted refractive lens and cool clean tones",stages:&[22, 11]},
    NativePreset{label:"Chromatic Drift",desc:"Lens color fringing with subtle detail",stages:&[23, 2]},
    NativePreset{label:"Desert Heat",desc:"Heat haze and sunset color in lower screen",stages:&[24, 5]},
    NativePreset{label:"Hologram HUD",desc:"Cool hologram grid and local clarity",stages:&[25, 2]},
    NativePreset{label:"Oil Canvas",desc:"Oil-paint brush sampling and aged palette",stages:&[26, 8]},
    NativePreset{label:"Portrait Lens",desc:"Tilt-focus faux depth-of-field and filmic grades",stages:&[27, 1]},
    NativePreset{label:"Aurora Bloom",desc:"Blue night ambience, glow and color bounce",stages:&[6, 12, 14]},
    NativePreset{label:"Spectral Noir",desc:"Monochrome with lens dispersion and contours",stages:&[7, 23, 15]},
    NativePreset{label:"Raylit Fantasy",desc:"2D pseudo-bounce and screen-space lightshafts",stages:&[14, 13, 5]},
    NativePreset{label:"Gothic Bloom",desc:"High-contrast darkening with glow",stages:&[15, 7, 12]},
    NativePreset{label:"Ultraviolet City",desc:"Neon edges, chromatic lens, bloom",stages:&[16, 23, 12]},
    NativePreset{label:"Frost Walker",desc:"Frost refraction, shadow boost, clarity",stages:&[22, 4, 2]},
    NativePreset{label:"Graphic Outline",desc:"Cel steps, darkened edges, fine contrast",stages:&[17, 15, 2]},
    NativePreset{label:"Faded Memory",desc:"Film + VHS + sepia treatment",stages:&[1, 20, 8]},
    NativePreset{label:"Old Monitor",desc:"Phosphor CRT with VHS color drift",stages:&[19, 20, 10]},
    NativePreset{label:"Lightforge",desc:"Cinematic bloom, shafts and diffusion",stages:&[12, 13, 14]},
    NativePreset{label:"Magic Echo",desc:"Artistic colored bounces and neon rim",stages:&[14, 16, 12]},
    NativePreset{label:"Soft Dreamscape",desc:"Strong bloom and tilted soft focus",stages:&[12, 27, 1]},
    NativePreset{label:"Sunset 35mm",desc:"Anamorphic streak and warm film",stages:&[21, 5, 1]},
    NativePreset{label:"Street Neon",desc:"Retro grid + edged neon flare",stages:&[25, 16, 23]},
    NativePreset{label:"Night Arcade",desc:"Vivid RGB and analog CRT under neon",stages:&[19, 16, 0]},
    NativePreset{label:"Ink Garden",desc:"Watercolor wash and cel lines",stages:&[18, 17, 11]},
    NativePreset{label:"Vivid Painting",desc:"Oil brush appearance with adaptive color",stages:&[26, 0, 2]},
    NativePreset{label:"Thermal Desert",desc:"Heat shimmer, shadow recovery and warm sunset",stages:&[24, 4, 5]},
    NativePreset{label:"Arctic Scan",desc:"Cold holographic detail and contrast",stages:&[25, 6, 2]},
    NativePreset{label:"Mirage Refraction",desc:"Glass distortion with heat shimmer",stages:&[22, 24, 11]},
    NativePreset{label:"Silver Noir",desc:"Monochrome, anamorphic light and darkened contact lines",stages:&[7, 21, 15]},
    NativePreset{label:"Star Glare",desc:"Wide highlight streaks and prism bloom",stages:&[21, 12, 1]},
    NativePreset{label:"Adventure Max",desc:"Warmth, color bounce, local detail and bloom",stages:&[5, 14, 2, 12]},
    NativePreset{label:"Daylight Film",desc:"Natural outdoor contrast and controlled highlights",stages:&[31, 28, 32]},
    NativePreset{label:"Real Shadow Pop",desc:"2D local shade, film latitude and conservative edge detail",stages:&[29, 31, 32]},
    NativePreset{label:"Soft Indirect",desc:"Subtle bright-color spill and local balanced tone",stages:&[30, 31]},
    NativePreset{label:"Cave Studio",desc:"2D shadow pockets, indirect light and dungeon detail",stages:&[29, 30, 4, 32]},
    NativePreset{label:"Sunburst Film",desc:"Warm bloom and restrained film halation",stages:&[28, 34, 5]},
    NativePreset{label:"Natural Bloom",desc:"RealBloom highlight response and low-halation clarity",stages:&[28, 31, 35]},
    NativePreset{label:"Soft Cinematic",desc:"Wide SDR tone, halation and softened high contrast edges",stages:&[31, 34, 35]},
    NativePreset{label:"Morning Contrast",desc:"Localized contact contrast and balanced glow",stages:&[29, 28, 32]},
    NativePreset{label:"Rainy Boulevard",desc:"Screen-floor wet sheen, contact accents and halation",stages:&[33, 29, 34]},
    NativePreset{label:"Fantasy Ambience",desc:"2D diffuse bounce plus cinematic ambient glow",stages:&[30, 28, 34]},
    NativePreset{label:"Clear Motion",desc:"Conservative detail and edge smoothing",stages:&[32, 35]},
    NativePreset{label:"Warm Radiance",desc:"Indirect color bleed with warm lighting and tone",stages:&[30, 5, 31]},
    NativePreset{label:"Noir Rimlight",desc:"Deep 2D contrast, mono and restrained highlight scatter",stages:&[29, 7, 34]},
    NativePreset{label:"Balanced View",desc:"Natural tone with adaptive vibrance and fine details",stages:&[31, 0, 32]},
    NativePreset{label:"Night Luma",desc:"Shadow recovery and subdued indirect light",stages:&[4, 30, 32]},
    NativePreset{label:"Clean Bloom Plus",desc:"Natural glow with adaptive sharpen and edge smoothing",stages:&[28, 32, 35]},
    NativePreset{label:"Quiet Sharp",desc:"Actual bilateral edge denoise with restrained local detail",stages:&[37, 32, 31]},
    NativePreset{label:"Clean Film SDR",desc:"GPU edge-preserving denoise, film tone and optional 8-bit dithering",stages:&[37, 31, 38]},
    NativePreset{label:"Gradient Guard",desc:"Natural SDR tone plus subtle ordered 8-bit debanding dither",stages:&[31, 38]},
];
pub fn validate()->Result<(),String>{
    let mut names=std::collections::BTreeSet::new();
    let mut sequences=std::collections::BTreeSet::new();
    for p in PRESETS{
        if !names.insert(p.label) {return Err(format!("Duplicate native preset name: {}",p.label));}
        if p.stages.is_empty()||p.stages.len()>8{return Err("Invalid native GPU pass count".into());}
        if !sequences.insert(p.stages.to_vec()){return Err(format!("Duplicate native GPU pass chain: {}",p.label));}
        if p.stages.iter().any(|i|*i>=crate::native_filter::PROGRAMS.len()){
            return Err(format!("Native shader ID not present for {}",p.label));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn native_presets_use_80_distinct_real_gpu_chains(){
        assert_eq!(PRESETS.len(),83);
        assert!(PRESETS.iter().any(|p|p.stages.contains(&13)));
        assert!(PRESETS.iter().any(|p|p.stages.contains(&14)));
        assert!(PRESETS.iter().any(|p|p.stages.contains(&17)));
        assert!(PRESETS.iter().any(|p|p.stages.contains(&27)));
        validate().unwrap();
    }
}
