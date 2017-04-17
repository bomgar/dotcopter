use std::error;
use std::fmt;
use std::io;
use regex;
use std::path;

#[derive(Debug)]
pub enum DotcopterError {
  IO(io::Error),
  Regex(regex::Error),
  StripPrefix(path::StripPrefixError),
}

impl fmt::Display for DotcopterError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      DotcopterError::IO(ref err) => write!(f, "IO error: {}", err),
      DotcopterError::Regex(ref err) => write!(f, "Regex error: {}", err),
      DotcopterError::StripPrefix(ref err) => write!(f, "Strip prefix error: {}", err),
    }
  }
}

impl error::Error for DotcopterError {
  fn description(&self) -> &str {
    match *self {
      DotcopterError::IO(ref err) => err.description(),
      DotcopterError::Regex(ref err) => err.description(),
      DotcopterError::StripPrefix(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match *self {
      DotcopterError::IO(ref err) => Some(err),
      DotcopterError::Regex(ref err) => Some(err),
      DotcopterError::StripPrefix(ref err) => Some(err),
    }
  }
}

impl From<io::Error> for DotcopterError {
  fn from(err: io::Error) -> DotcopterError {
    DotcopterError::IO(err)
  }
}

impl From<regex::Error> for DotcopterError {
  fn from(err: regex::Error) -> DotcopterError {
    DotcopterError::Regex(err)
  }
}

impl From<path::StripPrefixError> for DotcopterError {
  fn from(err: path::StripPrefixError) -> DotcopterError {
    DotcopterError::StripPrefix(err)
  }
}
