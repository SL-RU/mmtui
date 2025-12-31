use serde::Deserialize;
use std::fs;
use std::vec::Vec;
use xdg::BaseDirectories;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub fstype_ignore: Vec<String>,
    pub path_ignore: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fstype_ignore: [
                "tmpfs",
                "ramfs",
                "swap",
                "devtmpfs",
                "devpts",
                "hugetlbfs",
                "mqueue",
                "fuse.portal",
                "fuse.gvfsd-fuse",
            ]
            .map(String::from)
            .to_vec(),
            path_ignore: ["/tmp", "/sys", "/proc"].map(String::from).to_vec(),
        }
    }
}

impl Config {
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    fn load() -> Option<Self> {
        let basedirs = BaseDirectories::with_prefix("mmtui");
        let path = basedirs.find_config_file("mmtui.toml")?;

        println!("Load config file from: {}", path.display());

        let data = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to read config file using default config:");
                eprintln!("{err}");
                return None;
            }
        };

        match toml::from_str(&data) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("Failed to parse config file using default config:");
                eprintln!("{err}");
                None
            }
        }
    }
}
