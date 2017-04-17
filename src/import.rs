use model::{DotFile, DotFileType};
use std::path::Path;
use std::path::PathBuf;
use slog::Logger;
use std::fs;
use std::env;
use std::error::Error;
use regex::Regex;
use errors::DotcopterError;

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

fn get_dot_files(log: &Logger, dir: &Path) -> Result<Vec<DotFile>, DotcopterError> {
  let current_dir = try!(env::current_dir());
  let links = try!(get_links(log, dir));
  let mut dot_files: Vec<DotFile> = Vec::new();
  for link in links {
    let log = log.new(o!("link" => format!("{}", link.display())));
    if link.exists() {
      debug!(log, "Analyzing link");
      if try!(link_points_into_dir(&log, &link, &current_dir)) {
        info!(log, "Found dotfile");
        if let Ok(target) = link.clone().into_os_string().into_string() {
          let source_path = try!(link_target_to_relative_path(&link, &current_dir));
          if let Ok(source) = source_path.into_os_string().into_string() {
            let dot_file = DotFile {
              source: source,
              target: try!(replace_home_with_tilde(&log, &target)),
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

fn replace_home_with_tilde(log: &Logger, path: &str) -> Result<String, DotcopterError> {
  if let Some(home_dir) = env::home_dir() {
    replace_path_with_tilde(path, home_dir)
  } else {
    warn!(log, "Home dir not set");
    Ok(path.to_string())
  }
}

fn replace_path_with_tilde(path: &str, path_to_replace: PathBuf) -> Result<String, DotcopterError> {
  let replace_string = path_to_replace
    .into_os_string()
    .into_string()
    .expect("path should be a valid string");
  let mut pattern: String = "^".to_string();
  pattern.push_str(&replace_string);
  let regex = try!(Regex::new(&pattern));
  Ok(regex.replace_all(&path, "~").into_owned())
}

fn link_target_to_relative_path(link: &Path, current_dir: &Path) -> Result<PathBuf, DotcopterError> {
  let canonicalized_link = try!(link.canonicalize());
  Ok(try!(canonicalized_link.strip_prefix(current_dir)).to_path_buf())
}

fn link_points_into_dir(log: &Logger, link: &Path, dir: &Path) -> Result<bool, DotcopterError> {
  let canonicalized_link = try!(link.canonicalize());
  debug!(log, "Check if link points to dir";
         "canonicalized_link" => format!("{}", canonicalized_link.display()),
         "dir_to_check" => format!("{}", dir.display()));
  Ok(canonicalized_link.starts_with(dir))
}

fn get_links(log: &Logger, path: &Path) -> Result<Vec<PathBuf>, DotcopterError> {
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

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;


  #[test]
  fn test_replace_path_with_tilde() {
    let home_dir = Path::new("/home/blubb").to_path_buf();

    let replaced_string = replace_path_with_tilde("/home/blubb/moep/home/blubb/test.txt", home_dir).expect("should succeed");
    assert_that(&replaced_string).is_equal_to("~/moep/home/blubb/test.txt".to_string());

  }
}
