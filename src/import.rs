use model::DotFile;
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
  let current_dir =  env::current_dir().expect("Expected current directory to be available");
  let links = try!(get_links(log, dir));
  for link in links {
  }
  Ok(Vec::new())
}

fn get_links(log: &Logger, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
  let mut symlinks: Vec<PathBuf> = Vec::new();
  let entries = try!(fs::read_dir(path));
  for dir_entry_result in entries {
    let dir_entry: fs::DirEntry = try!(dir_entry_result);
    let entry_path = dir_entry.path();
    debug!(log, "Analyzing"; "dir_entry" => format!("{}", entry_path.display()));
    let metadata: fs::Metadata = try!(entry_path.symlink_metadata());
    if metadata.file_type().is_symlink(){
      symlinks.push(entry_path.to_path_buf());
    }
  }
  debug!(log, format!("Found {} symlinks", symlinks.len()));
  Ok(symlinks)
}
