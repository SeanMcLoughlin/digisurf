use directories::ProjectDirs;
use std::path::PathBuf;
pub mod keys;
pub mod ui;

pub fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "digisurf").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        config_dir.join("config.toml")
    })
}
