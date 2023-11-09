#![allow(unused_variables)]

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

pub fn unzip(path: &Path, single: bool, target: Option<PathBuf>) -> Result<()> {
    //dbg!(&path, &single, &target);

    let file_name = path.file_stem().unwrap();
    let target = match target {
        Some(path_buf) => path_buf,
        None => Path::new(".").join(file_name),
    };

    let mut archive = ZipArchive::new(File::open(path)?)?;
    archive.extract(&target)?;

    // No more work to be done in single mode
    if single {
        return Ok(());
    }

    let walkdir = WalkDir::new(&target).into_iter();

    for entry in walkdir.flatten() {
        if is_zip_file(&entry) {
            let path = entry.path();
            let parent = path.parent().unwrap();
            let target = parent.join(Path::new("korrektur"));

            unzip(path, true, Some(target))?;
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn is_zip_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".zip"))
        .unwrap_or(false)
}

pub fn zipit(name: Option<String>, paths: Vec<PathBuf>) -> Result<()> {
    todo!()
}

pub fn stats() -> Result<()> {
    todo!()
}

pub fn count(path: &Path) -> Result<()> {




    Ok(())
}
