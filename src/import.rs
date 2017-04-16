use model::{DotFile, DotFileType};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std;
use slog::Logger;
use std::fs;
use std::env;
use std::error::Error;

pub fn scan_dir(log: &Logger, dir: &str) -> Vec<DotFile> {
  let path = Path::new(dir);
  if path.is_dir() {
    match get_dot_files(log, &path) {
      Ok(links) => links,
      Err(e) => {
        error!(log, "Failed to get symlinks"; "error" => e.description());
        Vec::new()
      }
    }
  } else {
    error!(log, "Parameter is not a directory");
    Vec::new()
  }
}

fn get_dot_files(log: &Logger, dir: &Path) -> Result<Vec<DotFile>, std::io::Error> {
  let current_dir = env::current_dir().expect("Expected current directory to be available");
  let links = try!(get_links(log, dir));
  let mut dot_files: Vec<DotFile> = Vec::new();
  for link in links {
    let log = log.new(o!("link" => format!("{}", link.display())));
    if link.exists() {
      debug!(log, "Analyzing link");
      if try!(link_points_into_dir(&log, &link, &current_dir)) {
        debug!(log, "Found dotfile");
        if let Ok(target) = link.clone().into_os_string().into_string() {
          let source_path = try!(link_target_to_relative_path(&link, &current_dir));
          if let Ok(source) = source_path.into_os_string().into_string() {
            let dot_file = DotFile {
              source: source,
              target: replace_home_with_tilde(&log, &target),
              dot_file_type: DotFileType::LINK,
            };
            dot_files.push(dot_file);
          }
        }
      } else {
        debug!(log, "Skip link")
      }
    } else {
      warn!(log, "Link broken");
    }
  }
  Ok(dot_files)
}

fn replace_home_with_tilde(log: &Logger, path: &str) -> String {
  if let Some(home_dir) = env::home_dir() {
    let home_string = home_dir.into_os_string().into_string().expect("home_dir should be a valid string");
    //TODO regex to only replace prefix
    path.replace(&home_string, "~")
  } else {
    warn!(log, "Home dir not set");
    path.to_string()
  }
}

fn link_target_to_relative_path(link: &Path, current_dir: &Path) -> Result<PathBuf, std::io::Error> {
  let canonicalized_link = try!(link.canonicalize());
  Ok(canonicalized_link
       .strip_prefix(current_dir)
       .expect("Should be checked already")
       .to_path_buf())
}

fn link_points_into_dir(log: &Logger, link: &Path, dir: &Path) -> Result<bool, std::io::Error> {
  let canonicalized_link = try!(link.canonicalize());
  debug!(log, "Check if link points to dir";
         "canonicalized_link" => format!("{}", canonicalized_link.display()),
         "dir_to_check" => format!("{}", dir.display()));
  Ok(canonicalized_link.starts_with(dir))
}

fn get_links(log: &Logger, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
  let mut symlinks: Vec<PathBuf> = Vec::new();
  let entries = try!(fs::read_dir(path));
  for dir_entry_result in entries {
    let dir_entry: fs::DirEntry = try!(dir_entry_result);
    let entry_path = dir_entry.path();
    debug!(log, "Analyzing"; "dir_entry" => format!("{}", entry_path.display()));
    let metadata: fs::Metadata = try!(entry_path.symlink_metadata());
    if metadata.file_type().is_symlink() {
      symlinks.push(entry_path.to_path_buf());
    }
  }
  debug!(log, format!("Found {} symlinks", symlinks.len()));
  Ok(symlinks)
}