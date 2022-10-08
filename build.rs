use std::fs;
use std::path::Path;
use std::env;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let res = winres::WindowsResource::new();
        res.compile().unwrap();
    }

    copy_config_to_build_folder();
}

fn copy_config_to_build_folder() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_folder_root = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();

    if build_folder_root.exists() {
        let _ = fs::copy("./config/sidid.cfg", build_folder_root.join("sidid.cfg").to_str().unwrap());
        let _ = fs::copy("./config/tedid.cfg", build_folder_root.join("tedid.cfg").to_str().unwrap());
        let _ = fs::copy("./config/sidid.nfo", build_folder_root.join("sidid.nfo").to_str().unwrap());
    }
}
