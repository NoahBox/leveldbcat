use super::Entry;
use rusty_leveldb::{DB, LdbIterator, Options};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use tempfile::{Builder, TempDir};

const LOCK_FILE_NAMES: [&str; 2] = ["LOCK", ".LOCK"];

pub fn load_entries(db_path: &Path) -> Result<Vec<Entry>, String> {
    load_entries_with_options(db_path, false)
}

pub fn load_entries_ignoring_lock_file(db_path: &Path) -> Result<Vec<Entry>, String> {
    load_entries_with_options(db_path, true)
}

pub fn persisted_lock_file_name(db_path: &Path) -> Result<Option<&'static str>, String> {
    validate_db_path(db_path)?;

    Ok(LOCK_FILE_NAMES
        .into_iter()
        .find(|file_name| db_path.join(file_name).is_file()))
}

fn load_entries_with_options(db_path: &Path, ignore_lock_file: bool) -> Result<Vec<Entry>, String> {
    validate_db_path(db_path)?;

    let temp_dir = copy_db_to_temp_dir(db_path, ignore_lock_file)?;

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

fn copy_db_to_temp_dir(db_path: &Path, ignore_lock_file: bool) -> Result<TempDir, String> {
    let temp_dir = Builder::new()
        .prefix("leveldb-reader-")
        .tempdir()
        .map_err(|error| format!("Failed to create temporary workspace: {error}"))?;

    copy_directory_contents(db_path, temp_dir.path(), db_path, ignore_lock_file)?;

    Ok(temp_dir)
}

fn copy_directory_contents(
    source: &Path,
    target: &Path,
    root_source: &Path,
    ignore_lock_file: bool,
) -> Result<(), String> {
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

        if ignore_lock_file
            && source == root_source
            && file_type.is_file()
            && is_lock_file_name(&entry.file_name())
        {
            continue;
        }

        if file_type.is_dir() {
            fs::create_dir_all(&target_path).map_err(|error| {
                format!(
                    "Failed to create temporary directory {}: {error}",
                    target_path.display()
                )
            })?;
            copy_directory_contents(&source_path, &target_path, root_source, ignore_lock_file)?;
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

fn is_lock_file_name(file_name: &OsStr) -> bool {
    LOCK_FILE_NAMES
        .iter()
        .any(|candidate| file_name == OsStr::new(candidate))
}

#[cfg(test)]
mod tests {
    use super::{copy_db_to_temp_dir, persisted_lock_file_name};
    use std::fs;
    use tempfile::Builder;

    #[test]
    fn detects_lock_file() {
        let db_dir = sample_db_dir();
        fs::write(db_dir.path().join("LOCK"), b"locked").unwrap();

        assert_eq!(
            persisted_lock_file_name(db_dir.path()).unwrap(),
            Some("LOCK")
        );
    }

    #[test]
    fn detects_dot_lock_file() {
        let db_dir = sample_db_dir();
        fs::write(db_dir.path().join(".LOCK"), b"locked").unwrap();

        assert_eq!(
            persisted_lock_file_name(db_dir.path()).unwrap(),
            Some(".LOCK")
        );
    }

    #[test]
    fn skips_root_lock_file_when_requested() {
        let db_dir = sample_db_dir();
        fs::write(db_dir.path().join("LOCK"), b"locked").unwrap();
        fs::write(db_dir.path().join("MANIFEST-000001"), b"manifest").unwrap();

        let temp_dir = copy_db_to_temp_dir(db_dir.path(), true).unwrap();

        assert!(!temp_dir.path().join("LOCK").exists());
        assert!(temp_dir.path().join("CURRENT").is_file());
        assert!(temp_dir.path().join("MANIFEST-000001").is_file());
    }

    fn sample_db_dir() -> tempfile::TempDir {
        let db_dir = Builder::new()
            .prefix("leveldb-load-test-")
            .tempdir()
            .unwrap();
        fs::write(db_dir.path().join("CURRENT"), b"MANIFEST-000001\n").unwrap();
        db_dir
    }
}
