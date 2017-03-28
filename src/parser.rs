use yaml_rust;
use slog::{Logger};
use yaml_rust::Yaml;
use model::*;

pub fn parse_dot_files(log: &Logger, dot_files: &Yaml) -> Vec<DotFile> {
  let mut parsed_dot_files = Vec::new();
  info!(log, "Processing dotfiles");
  if let Yaml::Hash(entries) = dot_files.clone() {
    for (key, value) in entries {
      match (key, value) {
        (Yaml::String(target), Yaml::String(source)) =>
          parsed_dot_files.push(DotFile{ source: source.to_string(), target: target.to_string() , dot_file_type: DotFileType::LINK }),
        (Yaml::String(target), Yaml::Hash(settings)) =>
          parsed_dot_files.push(dot_file_from_settings(&log.new(o!("target" => target.clone())), target, &settings)),
        _ => {}
      }
    }
  } else {
    warn!(log, "Found no entries to process");
  }
  parsed_dot_files
}

fn dot_file_from_settings(log: &Logger, target: String, settings: &yaml_rust::yaml::Hash) -> DotFile {
  let mut dot_file = DotFile{ source: "<todo>".to_string(), target: target.to_string(), dot_file_type: DotFileType::LINK };
  for (key, value) in settings.clone() {
    match (key, value) {
      (Yaml::String(setting_key), Yaml::String(setting_value)) => {
        match setting_key.as_ref() {
          "src" => dot_file.source = setting_value.to_string(),
          "type" => dot_file.dot_file_type = dot_file_type_from_string(log, &setting_value),
          _ => {}
        }
      },
      _ => {}
    }
  }
  dot_file
}

fn dot_file_type_from_string(log: &Logger, s: &str) -> DotFileType {
  match s.to_lowercase().as_ref() {
    "copy" => DotFileType::COPY,
    "link" => DotFileType::LINK,
    x => {
      warn!(log, "could not parse file type. fallback to link."; "file_type" => x);
      DotFileType::LINK
    }
  }
}

#[cfg(test)]
mod tests {
  use slog;
  use slog_stdlog;
  use slog::{DrainExt};
  use yaml_rust::{YamlLoader};
  use yaml_rust::Yaml;
  use super::*;
  use spectral::prelude::*;


  #[test]
  fn parse_config() {
    let s =
      "
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
    let dot_files: &Yaml = &yaml_config["files"];
    let logger = slog::Logger::root(slog_stdlog::StdLog.fuse(), o!());
    let parsed_dot_files: Vec<DotFile> = parse_dot_files(&logger, dot_files);

    assert_that(&parsed_dot_files).has_length(3);
    assert_that(&parsed_dot_files).contains(&DotFile{ source: "tpm".to_string(), target: "~/.tmux/plugins/tpm".to_string() , dot_file_type: DotFileType::LINK });
    assert_that(&parsed_dot_files).contains(&DotFile{ source: "tmux.conf".to_string(), target: "~/.tmux.conf".to_string() , dot_file_type: DotFileType::COPY });
    assert_that(&parsed_dot_files).contains(&DotFile{ source: "vimrc".to_string(), target: "~/.vimrc".to_string() , dot_file_type: DotFileType::LINK });
  }
}
