#[derive(Debug, PartialEq)]
pub enum DotFileType {
  LINK,
  COPY,
}

#[derive(Debug, PartialEq)]
pub struct DotFile {
  pub source: String,
  pub target: String,
  pub dot_file_type: DotFileType,
}
