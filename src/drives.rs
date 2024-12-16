use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
    sync::{Arc, Mutex},
};

use crate::mountpoints;

#[derive(Debug, Clone)]
pub struct Block {
    pub object_path: String,
    pub dev: String,
    pub label: String,
    pub mount: Option<String>,
    pub fstype: String,
    pub mounted: bool,
}

#[derive(Debug, Clone)]
pub struct Drive {
    pub id: String,
    pub object_path: String,
    pub model: String,
    pub ejectable: bool,
    pub blocks: Vec<Block>,
}

pub async fn collect_drives_from_udisk() -> udisks2::Result<Vec<Drive>> {
    let mut drives: Vec<Drive> = Vec::new();
    let client = udisks2::Client::new().await?;
    let objects = client
        .object_manager()
        .get_managed_objects()
        .await
        .into_iter()
        .flatten()
        .filter_map(|(object_path, _)| {
            let Ok(obj) = client.object(object_path.clone()) else {
                return None;
            };
            Some((object_path, obj))
        });

    for (path, i) in objects {
        let path = path.to_string();
        if let Ok(drv) = i.drive().await {
            let drv = Drive {
                object_path: path,
                id: drv.id().await?,
                model: drv.model().await?,
                ejectable: drv.ejectable().await?,
                blocks: Vec::new(),
            };

            if let Some(d) = drives.iter_mut().find(|i| i.object_path == drv.object_path) {
                d.model = drv.model;
                d.ejectable = drv.ejectable;
                d.id = drv.id;
            } else {
                drives.push(drv);
            }
        } else if let Ok(blk) = i.block().await {
            let drv_path = blk.drive().await?.to_string();
            let block = Block {
                object_path: path,
                dev: String::from_utf8_lossy(&blk.device().await?)
                    .chars()
                    .filter(|c| c != &'\0')
                    .collect::<String>(),
                label: blk.id_label().await?,
                mount: None,
                fstype: blk.id_type().await?,
                mounted: false,
            };

            if let Some(d) = drives.iter_mut().find(|i| i.object_path == drv_path) {
                d.blocks.push(block);
            } else {
                drives.push(Drive {
                    object_path: drv_path,
                    id: String::new(),
                    model: String::new(),
                    ejectable: false,
                    blocks: vec![block],
                });
            }
        }
    }

    Ok(drives)
}

pub async fn collect_all() -> udisks2::Result<Vec<Drive>> {
    let mut drives = collect_drives_from_udisk().await?;
    let mounts = mountpoints::MountPoint::collect();

    let mut fstab = Drive {
        id: "fstab".to_owned(),
        object_path: "fstab".to_owned(),
        model: "fstab".to_owned(),
        ejectable: false,
        blocks: Vec::new(),
    };

    for i in mounts {
        let block = drives
            .iter_mut()
            .find(|d| d.blocks.iter().find(|b| b.dev == i.dev).is_some())
            .and_then(|d| d.blocks.iter_mut().find(|b| b.dev == i.dev));
        if let Some(block) = block {
            block.mount = i.path;
            block.mounted = i.mounted;
        } else {
            fstab.blocks.push(Block {
                object_path: String::new(),
                dev: i.dev,
                label: String::new(),
                mount: i.path,
                fstype: i.fs,
                mounted: i.mounted,
            });
        }
    }

    drives.push(fstab);
    drives.sort_by_cached_key(|b| b.object_path.clone());
    for i in &mut drives {
        i.blocks.sort_by_cached_key(|b| b.dev.clone());
    }

    Ok(drives)
}

pub async fn mount(block: &Block) -> udisks2::Result<()> {
    let mut drives: Vec<Drive> = Vec::new();
    let client = udisks2::Client::new().await?;

    client
        .object(block.object_path.clone())?
        .filesystem()
        .await?
        .mount(HashMap::new())
        .await?;

    //    client.part

    Ok(())
}

pub async fn unmount(block: &Block) -> udisks2::Result<()> {
    let mut drives: Vec<Drive> = Vec::new();
    let client = udisks2::Client::new().await?;

    client
        .object(block.object_path.clone())?
        .filesystem()
        .await?
        .unmount(HashMap::new())
        .await?;
    //    client.part

    Ok(())
}
