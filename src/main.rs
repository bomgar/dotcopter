extern crate clap;
use clap::{Arg, App, AppSettings};
use slog::{Logger, LevelFilter, Level, DrainExt};
use yaml_rust::YamlLoader;
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::Yaml;
use model::*;
use std::path::Path;
use std::fs;
use std::error::Error;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_stdlog;
extern crate yaml_rust;
extern crate crypto;

#[cfg(test)]
extern crate spectral;

mod model;
mod parser;
mod checksum;

fn main() {
  let matches: clap::ArgMatches = create_app().get_matches();
  let stream = slog_term::streamer().full().build();
  let verbose: bool = matches.is_present("verbose");
  let force: bool = matches.is_present("force");
  let log = if verbose {
    slog::Logger::root(stream.fuse(), o!())
  } else {
    slog::Logger::root(LevelFilter::new(stream, Level::Info).fuse(), o!())
  };


  let config_file = matches.value_of("config_file").unwrap();
  info!(log, "Starting engine"; "config_file" => config_file);

  let config = match load_config_file(config_file) {
    Ok(content) => content,
    Err(e) => {
      error!(log, "Failed to load config file."; "error" => e.description());
      panic!(1)
    }
  };

  let yaml_documents = match YamlLoader::load_from_str(&config) {
    Ok(yaml) => yaml,
    Err(e) => {
      error!(log, "Failed to parse config file."; "error" => e.description());
      panic!(2)
    }
  };
  let yaml_config = &yaml_documents[0];
  let dot_files: &Yaml = &yaml_config["files"];
  if dot_files.is_badvalue() {
    warn!(log, "Empty files list");
  }

  info!(log, "Parsed config file. Liftoff!");
  for dot_file in parser::parse_dot_files(&log, dot_files) {
    process_dot_file(&log, dot_file, force);
  }
}


fn process_dot_file(log: &Logger, dot_file: DotFile, force: bool) {
  let log = &log.new(o!("target" => dot_file.target.clone(), "source" => dot_file.source.clone(), "type" => format!("{:?}", dot_file.dot_file_type)));
  info!(log, "Process entry");
  let source_path = Path::new(&dot_file.source);
  let target_path = Path::new(&dot_file.target);
  if !source_path.exists() {
    warn!(log, "Source path does not exist");
    return;
  }
  match dot_file.dot_file_type {
    DotFileType::LINK => process_link(log, source_path, target_path, force),
    DotFileType::COPY => process_copy(log, source_path, target_path, force)
  }
}

fn process_copy(log: &Logger, source_path: &Path, target_path: &Path, force: bool) {
  match has_same_content(source_path, target_path) {
    Ok(true) => {},
    Ok(false) => {},
    Err(e) => error!(log, "Failed to copy dotfile"; "error" => e.description())
  }
}

fn has_same_content(source_path: &Path, target_path: &Path) -> Result<bool, std::io::Error> {
  if !target_path.exists() {
    Ok(false)
  } else if target_path.is_dir() {
    Ok(false) //TODO: ???
  } else {
    Ok(true)
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

fn load_config_file(file: &str) -> Result<String, std::io::Error> {
  let mut file = try!(File::open(file));
  let mut content = String::new();
  try!(file.read_to_string(&mut content));
  return Ok(content);
}

fn create_app<'a>() -> App<'a, 'a> {
  App::new("dotcopter")
    .version("0.1")
    .setting(AppSettings::ColoredHelp)
    .author("Patrick Haun <bomgar85@googlemail.com>")
    .about("manages dotfiles installation")
    .arg(Arg::with_name("force").short("f").long("force").takes_value(false))
    .arg(Arg::with_name("verbose")
           .long("verbose")
           .short("v")
           .help("debug output")
           .required(false)
           .takes_value(false))
    .arg(Arg::with_name("config_file").required(true))
}
