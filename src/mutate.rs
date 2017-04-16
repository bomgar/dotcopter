
use slog::Logger;
use yaml_rust::yaml;
use yaml_rust::Yaml;
use model;

pub fn add_dotfiles_to_config(log: &Logger, config: &Yaml, dotfiles: &[model::DotFile]) -> Yaml {
  let mut new_hash = if let Yaml::Hash(config_hash) = config.clone() {
    config_hash
  } else {
    warn!(log, "Configuration seems invalid. Overwriting it.");
    yaml::Hash::new()
  };
  let new_files = add_dotfiles_to_files(&config["files"], dotfiles);
  new_hash.insert(Yaml::String("files".to_string()), new_files);
  Yaml::Hash(new_hash)
}

fn add_dotfiles_to_files(files: &Yaml, dotfiles: &[model::DotFile]) -> Yaml {
  let mut new_hash = if let Yaml::Hash(files_hash) = files.clone() {
    files_hash
  } else {
    yaml::Hash::new()
  };
  for dotfile in dotfiles {
    let mut prop_hash = yaml::Hash::new();
    prop_hash.insert(Yaml::String("src".to_string()),
                     Yaml::String(dotfile.source.to_string()));
    prop_hash.insert(Yaml::String("type".to_string()),
                     Yaml::String(format!("{:?}", dotfile.dot_file_type)));
    new_hash.insert(Yaml::String(dotfile.target.to_string()),
                    Yaml::Hash(prop_hash));
  }
  Yaml::Hash(new_hash)
}
