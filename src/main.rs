extern crate clap;
use clap::{Arg, App, AppSettings};
use slog::{Logger, LevelFilter, Level, DrainExt};
use yaml_rust::{YamlLoader};
use std::io::prelude::*;
use std::fs::File;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate yaml_rust;

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

  let yaml_config = match YamlLoader::load_from_str(&config) {
    Ok(yaml) => yaml,
    Err(e) => {
      error!(log, "Failed to parse config file."; "error" => format!("{:?}", e));
      panic!(2)
    }
  };

  info!(log, "Parsed config file. Liftoff!");
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
