use super::Entry;
use rusty_leveldb::{DB, LdbIterator, Options};
use std::fs;
use std::path::Path;
use tempfile::{Builder, TempDir};

pub fn load_entries(db_path: &Path) -> Result<Vec<Entry>, String> {
    validate_db_path(db_path)?;

    let temp_dir = copy_db_to_temp_dir(db_path)?;

    let mut options = Options::default();
    options.create_if_missing = false;

    let mut db = DB::open(temp_dir.path(), options)
        .map_err(|error| format!("Failed to open LevelDB copy: {error}"))?;

    let mut iter = db
        .new_iter()
        .map_err(|error| format!("Failed to create iterator: {error}"))?;

    iter.seek_to_first();

    let mut entries = Vec::new();
    while let Some((key, value)) = iter.next() {
        entries.push(Entry {
            key_bytes: key,
            value_bytes: value,
        });
    }

    drop(iter);
    db.close()
        .map_err(|error| format!("Failed to close LevelDB copy cleanly: {error}"))?;

    Ok(entries)
}

fn validate_db_path(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Err(format!("Path does not exist: {}", db_path.display()));
    }

    if !db_path.is_dir() {
        return Err(format!("Path is not a directory: {}", db_path.display()));
    }

    if !db_path.join("CURRENT").is_file() {
        return Err(format!(
            "Directory does not look like a LevelDB folder (missing CURRENT): {}",
            db_path.display()
        ));
    }

    Ok(())
}

fn copy_db_to_temp_dir(db_path: &Path) -> Result<TempDir, String> {
    let temp_dir = Builder::new()
        .prefix("leveldb-reader-")
        .tempdir()
        .map_err(|error| format!("Failed to create temporary workspace: {error}"))?;

    copy_directory_contents(db_path, temp_dir.path())?;

    Ok(temp_dir)
}

fn copy_directory_contents(source: &Path, target: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source)
        .map_err(|error| format!("Failed to read {}: {error}", source.display()))?
    {
        let entry =
            entry.map_err(|error| format!("Failed to enumerate {}: {error}", source.display()))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "Failed to read file type for {}: {error}",
                source_path.display()
            )
        })?;

        if file_type.is_dir() {
            fs::create_dir_all(&target_path).map_err(|error| {
                format!(
                    "Failed to create temporary directory {}: {error}",
                    target_path.display()
                )
            })?;
            copy_directory_contents(&source_path, &target_path)?;
            continue;
        }

        if file_type.is_file() {
            fs::copy(&source_path, &target_path).map_err(|error| {
                format!(
                    "Failed to copy {} to {}: {error}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}
