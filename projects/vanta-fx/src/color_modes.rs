//! In-place display-wide native Win32 MAGCOLOREFFECT presets.
//! Pure 5x5 linear RGBA color transforms via existing Magnification API.
//! They are NOT neural filters, HLSL post-processing or 2x/3x frame generation.
//! Every mode is opt-in; no game mode/resolution/driver changes.
pub const MODES:&[(&str,&str)]=&[
    ("Neutral","Identity matrix; does not change display colors"),
    ("Vibrance","Boosts color separation without touching geometry"),
    ("Golden Hour","Warm cinematic red/gold temperature and mild contrast"),
    ("Arctic Clean","Cold blue highlights with restrained contrast"),
    ("Night Scout","Lift dark areas with linear brightness adjustment"),
    ("Noir Mono","Black-and-white photo-like contrast"),
    ("Old Film","Warm sepia wash; genuinely different RGB coefficients"),
    ("Emerald Night · artistic","Green cinematic stylization, not visibility enhancement"),
    ("Dusk Purple","Cool violet/blue atmospheric grading"),
    ("Soft Comfort","Lower saturation and blue intensity for evening use"),
    ("Amber Contrast","Warm amber with stronger contrast and dimmer blues"),
    ("Vintage Fade","Desaturated warm film with lifted blacks"),
    ("Red-green distinguishability · experimental",
     "Mild red/green separation; aesthetic accessibility aid, not clinically calibrated"),
    ("Green-red distinguishability · experimental",
     "Mild green/red contrast separation; individually adjustable, not a medical device"),
    ("Blue-yellow distinguishability · experimental",
     "Mild blue/yellow separation; effectiveness varies by display and person"),
];
fn identity()->[f32;25]{
    let mut m=[0.0;25];for i in 0..5{m[i*5+i]=1.0;}m
}
fn grade(mode:usize)->[f32;25]{
    // Distinct conservative 3x3 RGB linear accessibility assists.
    // Row-major: source R/G/B each contributes to output R/G/B.
    // Every output column sums to one so neutral greys remain neutral.
    // NOT medical correction, individual calibration or proven color vision
    // restoration; these are deliberately opt-in adjustable display grades.
    if (12..=14).contains(&mode) {
        let rows:[[f32;3];3]=match mode{
            12=>[[1.14,0.03,-0.04],[-0.10,0.98,-0.08],[-0.04,-0.01,1.12]],
            13=>[[1.04,-0.06,0.02],[-0.08,1.14,0.01],[0.04,-0.08,0.97]],
            _=>[[1.02,0.0,-0.04],[-0.02,1.10,-0.08],[0.0,-0.10,1.12]]
        };
        let mut m=identity();
        for i in 0..3{
            for j in 0..3 {m[i*5+j]=rows[i][j];}
        }
        return m;
    }
    let mut m=identity();
    let (contrast,saturation,red,green,blue,offset)=match mode {
        1=>(1.04,1.42,1.00,1.00,1.00,0.00),
        2=>(1.11,1.10,1.09,1.01,0.87,0.004),
        3=>(1.08,1.06,0.91,1.02,1.12,0.008),
        4=>(0.94,1.03,1.07,1.08,1.08,0.077),
        5=>(1.20,0.00,1.00,1.00,1.00,-0.014),
        6=>(1.06,0.46,1.11,1.02,0.71,0.020),
        7=>(0.98,0.48,0.55,1.12,0.58,0.008),
        8=>(1.13,1.16,1.04,0.92,1.18,-0.016),
        9=>(0.96,0.79,1.02,0.99,0.77,0.010),
        10=>(1.22,1.17,1.09,1.01,0.76,-0.018),
        11=>(0.83,0.72,1.10,1.04,0.87,0.050),
        _=>(1.0,1.0,1.0,1.0,1.0,0.0)
    };
    // Luma coeff are for the RGB source components. f32 5x5 row-major:
    // m[src*5 + output], translation row 4, output column 0..2.
    let luma=[0.2126,0.7152,0.0722];
    for source in 0..3 {
        for output in 0..3 {
            let diag=if source==output{1.0}else{0.0};
            let color=diag*saturation+luma[source]*(1.0-saturation);
            let gain=[red,green,blue][output];
            m[source*5+output]=color*contrast*gain;
        }
    }
    // Contrast around mid-gray, then additional user-visible black lift.
    for i in 0..3 {
        let gain=[red,green,blue][i];
        m[20+i]=0.5*(1.0-contrast)*gain+offset;
    }
    m
}
pub fn matrix(mode:usize,strength:f32)->[f32;25]{
    let level=if strength.is_finite(){strength.clamp(0.0,1.0)}else{0.0};
    let mut result=identity();
    let full=grade(mode);
    for i in 0..25{result[i]+=level*(full[i]-result[i]);}
    result
}
pub fn valid_mode(mode:usize)->usize{mode.min(MODES.len()-1)}
pub fn mode_name(mode:usize)->&'static str{MODES[valid_mode(mode)].0}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn all_os_color_matrices_are_finite_and_distinct(){
        let mut all=std::collections::BTreeSet::new();
        assert_eq!(MODES.len(),15);
        for i in 0..MODES.len(){
            let m=matrix(i,1.0);
            assert!(m.iter().all(|v|v.is_finite()&&v.abs()<4.0));
            assert!(all.insert(m.iter().map(|v|v.to_bits()).collect::<Vec<_>>()));
            assert_eq!(m[18],1.0);assert_eq!(m[24],1.0);
            assert_eq!(m[3],0.0);assert_eq!(m[8],0.0);assert_eq!(m[13],0.0);
        }
    }
    #[test]fn emerald_stylization_does_not_turn_gray_into_neon_green(){
        let m=matrix(7,0.80);
        let gray=[0.5,0.5,0.5,1.0,1.0];
        let mut output=[0.0_f32;3];
        for o in 0..3 {
            output[o]=(0..5).map(|i|gray[i]*m[i*5+o]).sum();
        }
        assert!(output[1]<0.65,"No oversaturated bright green on ordinary gray");
        assert!(output[0]>0.25&&output[2]>0.25,"Preserve useful red/blue structure");
    }
    #[test]fn accessibility_assists_preserve_neutral_colors_and_default_off(){
        for mode in 12..15 {
            let m=matrix(mode,1.0);
            assert_ne!(m,identity());
            assert_eq!(matrix(mode,0.0),identity());
            for dest in 0..3{
                let gray=(0..3).map(|source|0.5*m[source*5+dest]).sum::<f32>();
                assert!((gray-0.5).abs()<0.0001,
                    "Neutral grayscale should remain neutral, mode {mode} channel {dest}: {gray}");
            }
        }
    }
    #[test]fn strength_zero_leaves_display_identity(){
        for mode in 0..MODES.len(){
            assert_eq!(matrix(mode,0.0),identity());
            assert_eq!(matrix(mode,f32::NAN),identity());
        }
    }
    #[test]fn basic_modes_have_expected_distinct_color_semantics(){
        let noir=matrix(5,1.0);
        assert!((noir[0]-noir[1]).abs()<0.0001);
        assert!((noir[5]-noir[6]).abs()<0.0001);
        let night=matrix(4,1.0);assert!(night[20]>0.07);
        let blue=matrix(3,1.0);assert!(blue[12]>blue[0]);
        let warm=matrix(2,1.0);assert!(warm[0]>warm[12]);
    }
}
