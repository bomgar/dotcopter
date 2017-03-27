extern crate clap;
use clap::{Arg, App, AppSettings};
use slog::{Logger, LevelFilter, Level, DrainExt};
use yaml_rust::{YamlLoader};
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::Yaml;
use std::collections::BTreeMap;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate yaml_rust;


#[derive(Debug)]
enum DotFileType {
  LINK,
  COPY
}

#[derive(Debug)]
struct DotFile {
  source: String,
  target: String,
  dot_file_type: DotFileType
}

fn main() {
  let matches: clap::ArgMatches = create_app().get_matches();
  let stream = slog_term::streamer().full().build();
  let verbose: bool = matches.is_present("verbose");
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
      error!(log, "Failed to load config file."; "error" => format!("{:?}", e));
      panic!(1)
    }
  };

  let yaml_documents = match YamlLoader::load_from_str(&config) {
    Ok(yaml) => yaml,
    Err(e) => {
      error!(log, "Failed to parse config file."; "error" => format!("{:?}", e));
      panic!(2)
    }
  };
  let yaml_config = &yaml_documents[0];
  let dot_files: &Yaml = &yaml_config["files"];
  if dot_files.is_badvalue() {
    warn!(log, "Empty files list");
  }

  info!(log, "Parsed config file. Liftoff!");
  process_dot_files(&log, dot_files);
}

fn process_dot_files(log: &Logger, dot_files: &Yaml) {
  info!(log, "Processing dotfiles");
  if let Yaml::Hash(entries) = dot_files.clone() {
    for (key, value) in entries {
      match (key, value) {
        (Yaml::String(target), Yaml::String(source)) =>
          process_dot_file(log, DotFile{ source: source.to_string(), target: target.to_string() , dot_file_type: DotFileType::LINK }),
        (Yaml::String(target), Yaml::Hash(settings)) =>
          process_dot_file(log, dot_file_from_settings(target, &settings)),
        _ => {}
      }
    }
  } else {
    warn!(log, "Found no entries to process");
  }
}

fn dot_file_from_settings(target: String, settings: &yaml_rust::yaml::Hash) -> DotFile {
  DotFile{ source: "<todo>".to_string(), target: target.to_string(), dot_file_type: DotFileType::LINK }
}

fn process_dot_file(log: &Logger, dot_file: DotFile) {
  info!(log, "Process entry"; "target" => dot_file.target, "source" => dot_file.source, "type" => format!("{:?}", dot_file.dot_file_type));
}

fn load_config_file(file: &str) -> Result<String, std::io::Error> {
  let mut file = try!(File::open(file));
  let mut content = String::new();
  try!(file.read_to_string(&mut content));
  return Ok(content)
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
