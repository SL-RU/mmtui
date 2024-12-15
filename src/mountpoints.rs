use std::{io::BufRead, path::PathBuf};

#[derive(Debug)]
pub struct MountPoint {
    pub dev: String,
    pub path: Option<String>,
    pub fs: String,
}

impl MountPoint {
    pub fn collect_from_file(path: &str) -> Vec<MountPoint> {
        std::io::BufReader::new(std::fs::File::open(PathBuf::from(path)).unwrap())
            .lines()
            .map_while(Result::ok)
            .filter_map(|l| {
                let mut parts = l.split_whitespace();
                Some(MountPoint {
                    dev: parts
                        .next()
                        .and_then(|d| if !d.starts_with('#') { Some(d) } else { None })?
                        .into(),
                    path: Some(parts.next()?.to_string()),
                    fs: parts.next()?.into(),
                })
            })
            .filter(|p| {
                p.fs != "tmpfs" && p.fs != "swap"
                    && p.path.clone().is_some_and(|p| {
                        !p.starts_with("/sys")
                            && !p.starts_with("/tmp")
                            && !p.starts_with("/run")
                            && !p.starts_with("/proc")
                            && !p.starts_with("/dev")
                    })
            })
            .collect()
    }

    pub fn collect() -> Vec<MountPoint> {
        let mnt = Self::collect_from_file("/proc/self/mounts");
        let fstab = Self::collect_from_file("/etc/fstab");

        let fstab: Vec<MountPoint> = fstab
            .into_iter()
            .filter(|p| !mnt.iter().any(|f| f.path == p.path))
            .map(|p| MountPoint {
                dev: p.dev,
                path: None,
                fs: p.fs,
            })
            .collect();

        mnt.into_iter().chain(fstab).collect()
    }
}
