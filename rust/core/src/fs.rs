use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::KelivoResult;

pub fn ensure_dir(path: &Path) -> KelivoResult<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("unable to create directory {}", path.display()))?;
    Ok(())
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> KelivoResult<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let mut file =
        File::create(path).with_context(|| format!("failed to create file {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write file {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush file {}", path.display()))?;
    Ok(())
}

pub fn read_file_to_string(path: &Path) -> KelivoResult<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open file {}", path.display()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .with_context(|| format!("failed to read file {}", path.display()))?;
    Ok(buf)
}

pub fn is_within(base: &Path, candidate: &Path) -> bool {
    match (base.canonicalize(), candidate.canonicalize()) {
        (Ok(base), Ok(candidate)) => candidate.starts_with(base),
        _ => false,
    }
}

pub fn normalize_slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn resolve_absolute(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("file://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(path)
    }
}
