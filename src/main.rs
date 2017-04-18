use clap::{Arg, App, AppSettings, SubCommand};
use slog::{LevelFilter, Level, DrainExt, Logger};
use yaml_rust::YamlLoader;
use std::io::prelude::*;
use std::fs::File;
use yaml_rust::{Yaml, YamlEmitter};
use std::error::Error;
use std::path::Path;
use std::process::exit;
use errors::DotcopterError;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_stdlog;
extern crate yaml_rust;
extern crate crypto;
extern crate regex;
#[macro_use]
extern crate clap;

#[cfg(test)]
extern crate spectral;

mod model;
mod config;
mod checksum;
mod files;
mod mutate;
mod import;
mod errors;

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

  let config = match load_config_file(&log, config_file) {
    Ok(content) => content,
    Err(e) => {
      error!(log, "Failed to load config file."; "error" => e.description());
      exit(1)
    }
  };

  let mut yaml_documents = match YamlLoader::load_from_str(&config) {
    Ok(yaml) => yaml,
    Err(e) => {
      error!(log, "Failed to parse config file."; "error" => e.description());
      exit(2)
    }
  };
  if yaml_documents.is_empty() {
    let s = "files:";
    yaml_documents = YamlLoader::load_from_str(s).unwrap();
  }

  let maybe_ln_matches = matches.subcommand_matches("ln");
  let maybe_cp_matches = matches.subcommand_matches("cp");
  let maybe_apply_matches = matches.subcommand_matches("apply");
  let maybe_import_matches = matches.subcommand_matches("import");
  if maybe_apply_matches.is_some() {
    let yaml_config = &yaml_documents[0];
    let dot_files: &Yaml = &yaml_config["files"];
    info!(log, "Liftoff! Applying configuration.");
    files::process_dot_files(&log, dot_files, force);
  } else if let Some(ln_matches) = maybe_ln_matches {
    let yaml_config = &yaml_documents[0];
    let link_target = ln_matches.value_of("link_target").unwrap();
    let link_name = ln_matches.value_of("link_name").unwrap();
    let log = log.new(o!("link_target" => link_target.to_string(), "link_name" => link_name.to_string()));
    info!(log, "Liftoff! Adding new link to configuration");
    let new_config = mutate::add_dotfiles_to_config(&log,
                                                    yaml_config,
                                                    &[model::DotFile {
                                                        target: link_name.to_string(),
                                                        source: link_target.to_string(),
                                                        dot_file_type: model::DotFileType::LINK,
                                                      }]);
    write_new_yaml(&log, &new_config, config_file);
  } else if let Some(cp_matches) = maybe_cp_matches {
    let yaml_config = &yaml_documents[0];
    let target = cp_matches.value_of("target").unwrap();
    let source = cp_matches.value_of("source").unwrap();
    let log = log.new(o!("target" => target.to_string(), "source" => source.to_string()));
    info!(log, "Liftoff! Adding new copy to configuration");
    let new_config = mutate::add_dotfiles_to_config(&log,
                                                    yaml_config,
                                                    &[model::DotFile {
                                                        target: target.to_string(),
                                                        source: source.to_string(),
                                                        dot_file_type: model::DotFileType::COPY,
                                                      }]);
    write_new_yaml(&log, &new_config, config_file);
  } else if let Some(import_matches) = maybe_import_matches {
    let dir = import_matches.value_of("dir").unwrap();
    let log = log.new(o!("import_directory" => dir.to_string()));
    let yaml_config = &yaml_documents[0];
    info!(log, "Liftoff! Importing to configuration");
    let dot_files = import::scan_dir(&log, dir);
    if !dot_files.is_empty() {
      let new_config = mutate::add_dotfiles_to_config(&log, yaml_config, &dot_files);
      write_new_yaml(&log, &new_config, config_file);
    }
  }
}


fn write_new_yaml(log: &Logger, document: &Yaml, config_file: &str) {
  let mut out_str = String::new();
  {
    let mut emitter = YamlEmitter::new(&mut out_str);
    match emitter.dump(document) {
      Ok(_) => {}
      Err(e) => {
        error!(log, "Failed to write config file."; "error" => format!("{:?}", e));
        exit(3);
      }
    }
  }
  out_str.push_str("\n");
  match write_config_file(config_file, &out_str) {
    Ok(_) => info!(log, "Successfully written configuration"),
    Err(e) => {
      error!(log, "Failed to write config file."; "error" => e.description());
      exit(4);
    }
  };
}

fn write_config_file(file: &str, content: &str) -> Result<(), DotcopterError> {
  let mut file = try!(File::create(file));
  try!(file.write_all(content.as_bytes()));
  Ok(())
}

fn load_config_file(log: &Logger, file: &str) -> Result<String, DotcopterError> {
  let path = Path::new(file);
  if path.exists() {
    let mut file = try!(File::open(file));
    let mut content = String::new();
    try!(file.read_to_string(&mut content));
    Ok(content)
  } else {
    warn!(log, "Configuration doesn't exit"; "file" => file);
    Ok("".to_string())
  }
}


fn create_app<'a>() -> App<'a, 'a> {
  App::new("dotcopter")
    .version(crate_version!())
    .setting(AppSettings::ColoredHelp)
    .author("Patrick Haun <bomgar85@googlemail.com>")
    .about("manages dotfiles installation")
    .arg(Arg::with_name("force")
           .short("f")
           .long("force")
           .takes_value(false))
    .arg(Arg::with_name("verbose")
           .long("verbose")
           .short("v")
           .help("debug output")
           .required(false)
           .takes_value(false))
    .arg(Arg::with_name("config_file").required(true))
    .subcommand(SubCommand::with_name("apply").about("applies a dotfile configuration"))
    .subcommand(SubCommand::with_name("ln")
                  .about("adds new link to configuration")
                  .arg(Arg::with_name("link_target").required(true))
                  .arg(Arg::with_name("link_name").required(true)))
    .subcommand(SubCommand::with_name("cp")
                  .about("adds new copy to configuration")
                  .arg(Arg::with_name("source").required(true))
                  .arg(Arg::with_name("target").required(true)))
    .subcommand(SubCommand::with_name("import")
                  .about("imports dotfiles from a folder into the configuration")
                  .arg(Arg::with_name("dir").required(true)))
}
