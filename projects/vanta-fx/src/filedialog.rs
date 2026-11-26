//! Native Windows open-image dialog; never requests elevation or confirmation.
use std::{ffi::c_void,path::PathBuf};
#[repr(C)]
struct OpenFileName {
    l_struct_size:u32,hwnd_owner:*mut c_void,h_instance:*mut c_void,
    lpstr_filter:*const u16,lpstr_custom_filter:*mut u16,
    n_max_cust_filter:u32,n_filter_index:u32,
    lpstr_file:*mut u16,n_max_file:u32,lpstr_file_title:*mut u16,
    n_max_file_title:u32,lpstr_initial_dir:*const u16,
    lpstr_title:*const u16,flags:u32,n_file_offset:u16,
    n_file_extension:u16,lpstr_def_ext:*const u16,l_cust_data:isize,
    lpfn_hook:*mut c_void,lp_template_name:*const u16,
    pv_reserved:*mut c_void,dw_reserved:u32,flags_ex:u32
}
#[link(name="comdlg32")]
unsafe extern "system" {
    fn GetOpenFileNameW(file:*mut OpenFileName)->i32;
}
pub fn select_profile()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="Horizon Boost profiles\0*.json\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Import a local Horizon Boost JSON graphics profile\0"
        .encode_utf16().collect::<Vec<_>>();
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008;
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}
pub fn select_image()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="Image files\0*.png;*.jpg;*.jpeg\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Select a local image for shader before/after preview\0"
        .encode_utf16().collect::<Vec<_>>();
    // SAFETY: zero is the required default for all optional Win32 dialog fields.
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008; // Explorer, existing file, no ReadOnly
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    if len==0{return None;}
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}

pub fn select_cube()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="3D Cube LUT\0*.cube\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Select a local normalized SDR 3D Cube LUT\0"
        .encode_utf16().collect::<Vec<_>>();
    // SAFETY: zero is the required default for all optional Win32 dialog fields.
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008; // Explorer, existing file, no ReadOnly
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    if len==0{return None;}
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}

pub fn select_game_exe()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="Game executable\0*.exe\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Choose trusted game executable\0"
        .encode_utf16().collect::<Vec<_>>();
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008;
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}

pub fn select_launcher_manifest()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="Steam, Epic or GOG manifest\0*.acf;*.item;goggame-*.info\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Import one local launcher manifest\0"
        .encode_utf16().collect::<Vec<_>>();
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008;
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}

#[repr(C)]
struct BrowseInfo{owner:*mut c_void,root:*const c_void,display:*mut u16,title:*const u16,flags:u32,callback:*const c_void,param:isize,image:i32}
#[link(name="shell32")]unsafe extern "system"{
 fn SHBrowseForFolderW(info:*mut BrowseInfo)->*mut c_void;
 fn SHGetPathFromIDListW(p:*const c_void,path:*mut u16)->i32;
}
#[link(name="ole32")]unsafe extern "system"{fn CoTaskMemFree(p:*mut c_void);}
pub fn select_directory()->Option<PathBuf>{
 let mut display=[0u16;260];let mut path=[0u16;260];
 let title="Select a local directory\0".encode_utf16().collect::<Vec<_>>();
 let mut info=BrowseInfo{owner:std::ptr::null_mut(),root:std::ptr::null(),display:display.as_mut_ptr(),title:title.as_ptr(),flags:0x1,callback:std::ptr::null(),param:0,image:0};
 let p=unsafe{SHBrowseForFolderW(&mut info)};if p.is_null(){return None;}
 let ok=unsafe{SHGetPathFromIDListW(p,path.as_mut_ptr())};unsafe{CoTaskMemFree(p);}
 if ok==0{return None;}let len=path.iter().position(|x|*x==0)?;Some(PathBuf::from(String::from_utf16_lossy(&path[..len])))
}

pub fn select_reminder_wav()->Option<PathBuf>{
    let mut file=[0u16;32768];
    let filter="PCM WAV\0*.wav\0All files\0*.*\0\0"
        .encode_utf16().collect::<Vec<u16>>();
    let title="Select a local reminder WAV\0"
        .encode_utf16().collect::<Vec<_>>();
    // SAFETY: zero is the required default for all optional Win32 dialog fields.
    let mut data:OpenFileName=unsafe{std::mem::zeroed()};
    data.l_struct_size=std::mem::size_of::<OpenFileName>() as u32;
    data.lpstr_filter=filter.as_ptr();
    data.lpstr_file=file.as_mut_ptr();
    data.n_max_file=file.len() as u32;
    data.lpstr_title=title.as_ptr();
    data.flags=0x0008_0000|0x0000_1000|0x0000_0008; // Explorer, existing file, no ReadOnly
    if unsafe{GetOpenFileNameW(&mut data)}==0{return None;}
    let len=file.iter().position(|v|*v==0)?;
    if len==0{return None;}
    Some(PathBuf::from(String::from_utf16_lossy(&file[..len])))
}
