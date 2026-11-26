//! Bounded ordered native D3D11 effects. Lighting Studio owns its own cbuffer.
pub const MAX_STAGES:usize=8;
pub fn valid_stage(id:usize)->bool{
    id<crate::native_filter::PROGRAMS.len()&&id!=crate::lighting::GPU_LIGHT_STAGE&&id!=crate::color_grade::STAGE
}
pub fn validate(stages:&[usize])->Result<(),String>{
    if stages.is_empty()||stages.len()>MAX_STAGES{
        return Err("Choose 1 to 8 GPU effects".into());
    }
    if stages.iter().any(|&id|!valid_stage(id)){
        return Err("Invalid stage; Light and Color Studio are configured separately".into());
    }
    Ok(())
}
pub fn move_stage(stages:&mut [usize],from:usize,to:usize)->bool{
    if from>=stages.len()||to>=stages.len(){return false;}
    stages.swap(from,to);true
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn validate_bounds_and_reserved_constant_stage(){
        assert!(validate(&[37,2,38]).is_ok());
        assert!(validate(&[]).is_err());
        assert!(validate(&[0;9]).is_err());
        assert!(validate(&[999]).is_err());
        assert!(validate(&[crate::lighting::GPU_LIGHT_STAGE]).is_err());
    }
    #[test]fn ordered_stack_profile_survives_settings_json(){
        let mut settings=crate::settings::Settings::default();
        settings.native_stack_profiles.insert("test.exe".into(),vec![37,31,38,31]);
        let restored:crate::settings::Settings=serde_json::from_str(
            &serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.native_stack_profiles["test.exe"],vec![37,31,38,31]);
    }
    #[test]fn ordering_is_not_sorted_or_deduplicated(){
        let mut x=[2,37,2];
        assert!(move_stage(&mut x,1,0));
        assert_eq!(x,[37,2,2]);
        assert!(!move_stage(&mut x,1,9));
        assert!(validate(&x).is_ok());
    }
}
