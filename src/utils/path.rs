use std::{
    env,
    path::{Path, PathBuf},
};

pub fn get_bkp_path() -> PathBuf {
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");

    let path = Path::new(&home_dir).join(".purrgres");

    path
}
