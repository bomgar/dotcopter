
use slog::Logger;
use yaml_rust::yaml;
use yaml_rust::Yaml;
use model;

pub fn add_dotfile_to_config(log: &Logger, config: &Yaml, dotfile: model::DotFile) -> Yaml {
  let mut new_hash = if let Yaml::Hash(config_hash) = config.clone() {
    config_hash
  } else {
    warn!(log, "Configuration seems invalid. Overwriting it.");
    yaml::Hash::new()
  };
  let new_files = add_dotfile_to_files(&config["files"], dotfile);
  new_hash.insert(Yaml::String("files".to_string()), new_files);
  Yaml::Hash(new_hash)
}

fn add_dotfile_to_files(files: &Yaml, dotfile: model::DotFile) -> Yaml {
  let mut new_hash = if let Yaml::Hash(files_hash) = files.clone() {
    files_hash
  } else {
    yaml::Hash::new()
  };
  new_hash.insert(Yaml::String(dotfile.target.to_string()),
                  Yaml::String(dotfile.source.to_string()));
  Yaml::Hash(new_hash)
}
