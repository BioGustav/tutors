use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use zip::ZipArchive;

use crate::tutors_csv::Record;

const COUNTED_FILES: [&str; 1] = ["java"];
const DEFAULT_MAX_POINTS: u8 = 25;
const IGNORED_NAMES: [&str; 6] = ["__macosx", ".git", ".idea", ".ds_store", ".iml", ".class"];
const _INDEX_FEEDBACK: usize = 10;
const _INDEX_MAX_POINTS: usize = 6;
const _INDEX_POINTS: usize = 5;
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

pub fn fill_table(
    table_path: &Path,
    dir_path: &Path,
    result_path: &Path,
    _debug: bool,
) -> Result<()> {
    if !table_path.exists()
        || !table_path.is_file()
        || !table_path.extension().is_some_and(|ext| ext == "csv")
    {
        return Err(anyhow::anyhow!("Table path not valid"));
    }

    let dirs = get_dirs(dir_path)?;

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_path(result_path)?;

    read_table(table_path)?
        .iter_mut()
        .flat_map(|record| match dirs.get(&record.id) {
            Some(dir) => Some((record, dir)),
            _ => None,
        })
        .map(|(r, d)| {
            let deduction = sum_deduction(d).unwrap_or(0f32);
            let points = 0f32.max(r.max_points - deduction);
            r.points = Some(points);
            r
        })
        .for_each(|record| {
            wtr.serialize(record).unwrap();
        });

    Ok(())
}

fn sum_deduction(dir_path: &Path) -> Result<f32> {
    let file_walker = WalkDir::new(dir_path)
        .into_iter()
        .flatten()
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| COUNTED_FILES.contains(&ext.to_str().unwrap_or_default()))
        });

    let mut points = 0f32;
    for file in file_walker {
        let file = File::open(file)?;
        let file = BufReader::new(file);

        points -= calculate_deduction(file)?;
    }

    Ok(points)
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
                let deduction = deduction.as_str().parse::<f32>().unwrap_or(0f32);
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

fn read_table(table_path: &Path) -> Result<Vec<Record>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_path(table_path)?;

    let vec = reader
        .deserialize::<Record>()
        .filter_map(Result::ok)
        .collect();

    Ok(vec)
}

const ID_PATTERN: &str = r"([\d]+)";
fn get_dirs(dir_path: &Path) -> Result<HashMap<String, PathBuf>> {
    let re = Regex::new(ID_PATTERN)?;
    let walkdir = WalkDir::new(dir_path).max_depth(1).into_iter();

    let map: HashMap<_, _> = walkdir
        .skip(1)
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .flat_map(|entry| {
            let entry = entry.path();
            let name = entry.to_str().unwrap();
            let cap = re.captures(name).unwrap().get(1);
            match cap {
                Some(cap) => Some((cap.as_str().to_string(), entry.to_path_buf())),
                _ => None,
            }
        })
        .collect();

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_count() -> Result<()> {
        todo!()
    }
    fn test_unzip() -> Result<()> {
        todo!()
    }
    fn test_fill_table() -> Result<()> {
        todo!()
    }
    fn test_stats() -> Result<()> {
        todo!()
    }
    fn test_zipit() -> Result<()> {
        todo!()
    }
}
