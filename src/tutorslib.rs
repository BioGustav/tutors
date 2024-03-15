use std::fs::{create_dir, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

const COUNTED_FILES: [&str; 1] = ["java"];
const DEFAULT_MAX_POINTS: u8 = 25;
const IGNORED_NAMES: [&str; 6] = ["__macosx", ".git", ".idea", ".ds_store", ".iml", ".class"];
const NAME_PATTERN: &str = r"([^\d_]*)";
const TUTOR_PATTERN: &str = r"// Tutor: (-)?(\d*(\.\d)?)";

pub fn count(path: &Path, target_dir: &Path, max_points: &Option<u8>, _debug: bool) -> Result<()> {
    //dbg!(&path, max_points);
    let max_points = match max_points {
        Some(points) => points,
        None => &DEFAULT_MAX_POINTS,
    };

    if !target_dir.exists() {
        create_dir(target_dir)?;
    }

    let name_re = Regex::new(NAME_PATTERN)?;

    let folders = WalkDir::new(path).max_depth(1).into_iter().skip(1);
    let mut result = File::create(target_dir.join("result.csv"))?;

    for folder in folders.flatten() {
        let folder = folder.path();
        let folder_name = match folder.file_stem() {
            Some(name) => name.to_str().unwrap(),
            None => continue,
        };
        let name = name_re
            .captures(folder_name)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        let file_walker = WalkDir::new(folder).into_iter();
        let mut points = *max_points as f32;
        for dir in file_walker.flatten() {
            if dir
                .path()
                .extension()
                .is_some_and(|ext| COUNTED_FILES.contains(&ext.to_str().unwrap()))
            {
                let file = File::open(dir.path())?;
                let file = BufReader::new(file);

                points -= calculate_deduction(file)?;
            }
        }
        points = if points < 0. { 0. } else { points };
        result.write_all(format!("{},{}\n", name, points).as_bytes())?;
    }

    Ok(())
}

pub fn unzip(
    path: &PathBuf,
    single: bool,
    flatten: bool,
    target: Option<&PathBuf>,
    debug: bool,
) -> Result<()> {
    let file_name = Path::new(".").join(path.file_stem().unwrap());
    let target = match target {
        Some(path_buf) => path_buf.as_path(),
        None => &file_name,
    };

    let mut archive = ZipArchive::new(File::open(path)?)?;
    archive.extract(target)?;

    // No more work to be done in single mode
    if single {
        std::fs::remove_file(path)?;
        return Ok(());
    }

    let walkdir = WalkDir::new(target).into_iter();

    for entry in walkdir.flatten() {
        if is_zip_file(&entry) {
            let path = entry.path().to_path_buf();
            let parent = path.parent().unwrap();
            let target = &parent.to_path_buf();

            dbglog!(debug, "Unzipping", "path", target.to_str().unwrap_or(""));

            unzip(&path, true, flatten, Some(target), debug)?;

            clean_dirs(target, debug)?;

            if flatten {
                flatten_dirs(target, None, debug)?;
            }
        }
    }

    Ok(())
}

pub fn _generate_report() -> Result<()> {
    Err(anyhow::anyhow!("Not yet implemented"))
}

pub fn stats() -> Result<()> {
    Err(anyhow::anyhow!("Not yet implemented"))
}
pub fn zipit(_name: Option<String>, _paths: Vec<PathBuf>) -> Result<()> {
    Err(anyhow::anyhow!("Not yet implemented"))
}

fn calculate_deduction(file: BufReader<File>) -> Result<f32> {
    let tut_re = Regex::new(TUTOR_PATTERN)?;
    let mut result = 0f32;

    for line in file.lines().map_while(Result::ok) {
        tut_re.captures_iter(&line).for_each(|cap| {
            if let Some(deduction) = cap.get(2) {
                let deduction = deduction.as_str().parse::<f32>().unwrap_or(0.);
                result += deduction;
            }
        });
    }

    Ok(result)
}

fn clean_dirs(path: &Path, debug: bool) -> Result<()> {
    let walkdir = WalkDir::new(path).into_iter().flatten().filter(|entry| {
        IGNORED_NAMES.iter().any(|name| {
            entry
                .file_name()
                .to_str()
                .unwrap()
                .to_lowercase()
                .contains(name)
        })
    });
    for entry in walkdir {
        dbglog!(
            debug,
            "Removing",
            "path",
            entry.path().to_str().unwrap_or("")
        );

        if entry.path().is_file() {
            std::fs::remove_file(entry.path())?;
        } else {
            std::fs::remove_dir_all(entry.path())?;
        }
    }

    Ok(())
}
fn flatten_dirs(path: &Path, to: Option<&Path>, debug: bool) -> Result<()> {
    let walkdir = WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .skip(1); // Skip the root directory

    for entry in walkdir {
        dbglog!(
            debug,
            "Flatten",
            "path",
            entry.path().to_str().unwrap_or("")
        );

        let to = to.unwrap_or_else(|| entry.path().parent().unwrap_or(Path::new("/")));
        flatten_dirs(entry.path(), Some(to), debug)?;
        move_files(entry.path(), to, debug)?;
        std::fs::remove_dir_all(entry.path())?;
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

fn log(command: &str, args: Vec<(&str, &str)>) {
    let mut prefix = "";
    let args = args.iter().fold(String::new(), |mut acc, (key, value)| {
        acc.push_str(&format!("{}{}: {}", prefix, key, value));
        prefix = ", ";
        acc
    });
    println!("{:9}: {}", command, args);
}

fn move_files(path: &Path, to: &Path, debug: bool) -> Result<()> {
    WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter(not_ignored)
        .for_each(|entry| {
            let from = entry.path();
            std::fs::copy(from, to.join(entry.file_name())).unwrap();

            dbglog!(
                debug,
                "Moving",
                "from",
                from.to_str().unwrap_or(""),
                "to",
                to.to_str().unwrap_or("")
            );
        });
    Ok(())
}

fn not_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            !IGNORED_NAMES
                .iter()
                .any(|name| s.to_lowercase().contains(name))
        })
        .unwrap_or(false)
}
