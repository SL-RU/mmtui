use crate::config::Config;
use std::{io::BufRead, path::PathBuf};

#[derive(Debug)]
pub struct MountPoint {
    pub dev: String,
    pub path: Option<String>,
    pub fs: String,
    pub mounted: bool,
}

impl MountPoint {
    pub fn collect_from_file(path: &str, config: &Config) -> Vec<MountPoint> {
        std::io::BufReader::new(std::fs::File::open(PathBuf::from(path)).unwrap())
            .lines()
            .map_while(Result::ok)
            .filter_map(|l| {
                let mut parts = l.split_whitespace();
                Some(MountPoint {
                    dev: parts
                        .next()
                        .and_then(|d| if d.starts_with('#') { None } else { Some(d) })?
                        .into(),
                    path: Some(parts.next()?.to_string()),
                    fs: parts.next()?.into(),
                    mounted: false,
                })
            })
            .filter(|p| !config.fstype_ignore.contains(&p.fs))
            .filter(|p| {
                if let Some(p) = &p.path {
                    !config
                        .path_ignore
                        .iter()
                        .any(|ignore| p.starts_with(ignore))
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn collect(config: &Config) -> Vec<MountPoint> {
        let mnt = Self::collect_from_file("/proc/self/mounts", config);
        let fstab = Self::collect_from_file("/etc/fstab", config);

        let fstab: Vec<MountPoint> = fstab
            .into_iter()
            .filter(|p| !mnt.iter().any(|f| f.path == p.path))
            .map(|p| MountPoint {
                mounted: false,
                ..p
            })
            .collect();

        mnt.into_iter()
            .map(|m| MountPoint { mounted: true, ..m })
            .chain(fstab)
            .collect()
    }
}
