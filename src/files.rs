
use slog::{Logger};
use yaml_rust::Yaml;
use model::*;
use std::path::Path;
use std::fs;
use std::error::Error;
use config;
use checksum;
use std;
use std::env;

pub fn process_dot_files(log: &Logger, dot_files: &Yaml, force: bool) {
  if dot_files.is_badvalue() {
    warn!(log, "Empty files list");
  } else {
    for dot_file in config::parse_dot_files(&log, dot_files) {
      process_dot_file(&log, dot_file, force);
    }
  }
}


fn process_dot_file(log: &Logger, dot_file: DotFile, force: bool) {
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
    DotFileType::COPY => process_copy(log, source_path, target_path, force)
  }
}

fn resolve_home(log: &Logger, path: &str) -> String {
  if let Some(home_dir) = env::home_dir() {
    if path.starts_with("~") {
      let mut home_string = home_dir.into_os_string().into_string().expect("home_dir should be a valid string");
      home_string.push_str(&path[1..]);
      return home_string;
    } else {
      return path.to_string();
    }
  } else {
    warn!(log, "Home dir not set");
    return path.to_string();
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
        Err(e) => error!(log, "Failed to copy file"; "error" => e.description())
      }
    },
    Err(e) => error!(log, "Failed to copy dotfile"; "error" => e.description())
  }
}

fn copy_dot_file(source: &Path, target: &Path) -> Result<(), std::io::Error> {
  if let Some(parent) = target.parent() {
    try!(fs::create_dir_all(parent));
  }
  if target.exists() {
    if target.is_file() {
      try!(fs::remove_file(target))
    } else if target.is_dir() {
      try!(fs::remove_dir_all(target))
    }
  }
  try!(fs::copy(source, target));
  Ok(())
}

fn has_same_content(log: &Logger, source: &Path, target: &Path) -> Result<bool, std::io::Error> {
  if !target.exists() {
    Ok(false)
  } else if target.is_dir() || source.is_dir() {
    Ok(false) //TODO: ???
  } else {
    let source_hash = try!(checksum::hash(source));
    let target_hash = try!(checksum::hash(target));
    debug!(log, "Hashed files"; "target_hash" => target_hash, "source_hash" => source_hash);
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
          Err(e) => error!(log, "Failed to create link"; "error" => e.description()),
          Ok(_) => info!(log, "Link created successfully"),
        }
      }
    }
    Err(e) => error!(log, "Failed to check link existence"; "error" => e.description()),
  }
}

fn already_linked(source: &Path, target: &Path) -> Result<bool, std::io::Error> {
  if target.exists() {
    let canonicalized_target = try!(fs::canonicalize(target));
    let canonicalized_source = try!(fs::canonicalize(source));
    Ok(canonicalized_source == canonicalized_target)
  } else {
    Ok(false)
  }
}

fn link_dot_file(source: &Path, target: &Path) -> Result<(), std::io::Error> {
  if let Some(parent) = target.parent() {
    try!(fs::create_dir_all(parent));
  }
  if target.exists() {
    if target.is_file() {
      try!(fs::remove_file(target))
    } else if target.is_dir() {
      try!(fs::remove_dir_all(target))
    }
  }
  let canonicalized_source = try!(fs::canonicalize(source));
  try!(std::os::unix::fs::symlink(canonicalized_source, target));
  Ok(())
}

