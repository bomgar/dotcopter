use crate::checksum;
use crate::config;
use crate::errors::DotcopterError;
use crate::model::*;
use slog::Logger;
use slog::{debug, error, info, o, warn};
use std::fs;
use std::path::Path;
use yaml_rust::Yaml;

pub fn process_dot_files(log: &Logger, dot_files: &Yaml, force: bool) {
  if dot_files.is_badvalue() {
    warn!(log, "Empty files list");
  } else {
    for dot_file in config::parse_dot_files(log, dot_files) {
      process_dot_file(log, &dot_file, force);
    }
  }
}

fn process_dot_file(log: &Logger, dot_file: &DotFile, force: bool) {
  let log = &log.new(o!("target" => dot_file.target.clone(), "source" => dot_file.source.clone(), "type" => format!("{:?}", dot_file.dot_file_type)));
  debug!(log, "Process entry");
  let source_with_resolved_home = resolve_home(log, &dot_file.source);
  let target_with_resolved_home = resolve_home(log, &dot_file.target);
  let source_path = Path::new(&source_with_resolved_home);
  let target_path = Path::new(&target_with_resolved_home);
  if !source_path.exists() {
    warn!(log, "Source path does not exist");
    return;
  }
  match dot_file.dot_file_type {
    DotFileType::LINK => process_link(log, source_path, target_path, force),
    DotFileType::COPY => process_copy(log, source_path, target_path, force),
  }
}

fn resolve_home(log: &Logger, path: &str) -> String {
  if let Some(home_dir) = dirs::home_dir() {
    if let Some(stripped_path) = path.strip_prefix('~') {
      let mut home_string = home_dir.into_os_string().into_string().expect("home_dir should be a valid string");
      home_string.push_str(&stripped_path);
      home_string
    } else {
      path.to_string()
    }
  } else {
    warn!(log, "Home dir not set");
    path.to_string()
  }
}

fn process_copy(log: &Logger, source_path: &Path, target_path: &Path, force: bool) {
  match has_same_content(log, source_path, target_path) {
    Ok(true) => info!(log, "File already there"),
    Ok(false) => {
      if !force && target_path.exists() {
        error!(log, "Target already exists but has different content.");
        return;
      }
      match copy_dot_file(source_path, target_path) {
        Ok(_) => info!(log, "Copied file successfully"),
        Err(e) => error!(log, "Failed to copy file"; "error" => e.to_string()),
      }
    }
    Err(e) => error!(log, "Failed to copy dotfile"; "error" => e.to_string()),
  }
}

fn copy_dot_file(source: &Path, target: &Path) -> Result<(), DotcopterError> {
  if let Some(parent) = target.parent() {
    fs::create_dir_all(parent)?;
  }
  if target.exists() {
    if target.is_file() {
      fs::remove_file(target)?
    } else if target.is_dir() {
      fs::remove_dir_all(target)?
    }
  }
  fs::copy(source, target)?;
  Ok(())
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::if_same_then_else))]
fn has_same_content(log: &Logger, source: &Path, target: &Path) -> Result<bool, DotcopterError> {
  if !target.exists() {
    Ok(false)
  } else if target.is_dir() || source.is_dir() {
    Ok(false) //TODO: ???
  } else {
    let source_hash = checksum::hash(source)?;
    let target_hash = checksum::hash(target)?;
    debug!(log, "Hashed files"; "target_hash" => &target_hash, "source_hash" => &source_hash);
    Ok(source_hash == target_hash)
  }
}

fn process_link(log: &Logger, source_path: &Path, target_path: &Path, force: bool) {
  match already_linked(source_path, target_path) {
    Ok(true) => info!(log, "Link already exists"),
    Ok(false) => {
      if !force && target_path.exists() {
        error!(log, "Target exists but does not point to source")
      } else {
        let result = link_dot_file(source_path, target_path);
        match result {
          Err(e) => error!(log, "Failed to create link"; "error" => e.to_string()),
          Ok(_) => info!(log, "Link created successfully"),
        }
      }
    }
    Err(e) => error!(log, "Failed to check link existence"; "error" => e.to_string()),
  }
}

fn already_linked(source: &Path, target: &Path) -> Result<bool, DotcopterError> {
  if target.exists() {
    let canonicalized_target = fs::canonicalize(target)?;
    let canonicalized_source = fs::canonicalize(source)?;
    Ok(canonicalized_source == canonicalized_target)
  } else {
    Ok(false)
  }
}

fn link_dot_file(source: &Path, target: &Path) -> Result<(), DotcopterError> {
  if let Some(parent) = target.parent() {
    fs::create_dir_all(parent)?;
  }
  if target.exists() {
    if target.is_file() {
      fs::remove_file(target)?
    } else if target.is_dir() {
      fs::remove_dir_all(target)?
    }
  }
  let canonicalized_source = fs::canonicalize(source)?;
  std::os::unix::fs::symlink(canonicalized_source, target)?;
  Ok(())
}
