
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
#[cfg(test)]
mod tests {
  use slog;
  use slog_stdlog;
  use slog::DrainExt;
  use yaml_rust::YamlLoader;
  use yaml_rust::{Yaml, YamlEmitter};
  use super::*;
  use spectral::prelude::*;
  use model::{DotFile, DotFileType};


  #[test]
  fn test_add_dotfile_to_config() {
    let s = "
files:
    ~/.tmux/plugins/tpm: tpm
    ~/.tmux.conf:
        src: tmux.conf
        type: copy
    ~/.vimrc:
        src: vimrc
        type: link
";
    let yaml_documents = YamlLoader::load_from_str(s).unwrap();
    let yaml_config = &yaml_documents[0];
    let logger = slog::Logger::root(slog_stdlog::StdLog.fuse(), o!());
    let new_files = [DotFile {
                       source: "test".to_string(),
                       target: "~/test".to_string(),
                       dot_file_type: DotFileType::LINK,
                     }];
    let new_config: Yaml = add_dotfiles_to_config(&logger, &yaml_config, &new_files);
    let mut out_str = String::new();
    {
      let mut emitter = YamlEmitter::new(&mut out_str);
      emitter.dump(&new_config).unwrap();
    }

    let expected = "---
files: 
  ~/.tmux/plugins/tpm: tpm
  ~/.tmux.conf: 
    src: tmux.conf
    type: copy
  ~/.vimrc: 
    src: vimrc
    type: link
  ~/test: 
    src: test
    type: LINK".to_string();

    assert_that(&out_str).is_equal_to(expected);
  }
}
