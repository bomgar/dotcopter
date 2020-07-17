use std::error;
use std::fmt;
use std::io;
use std::path;

#[derive(Debug)]
pub enum DotcopterError {
  IO(io::Error),
  Regex(regex::Error),
  StripPrefix(path::StripPrefixError),
}

macro_rules! dotcopter_error_from {
  ($error: ty, $dotcopter_error: ident) => {
    impl From<$error> for DotcopterError {
      fn from(err: $error) -> DotcopterError {
        DotcopterError::$dotcopter_error(err)
      }
    }
  };
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
    #[allow(deprecated)]
    match *self {
      DotcopterError::IO(ref err) => err.description(),
      DotcopterError::Regex(ref err) => err.description(),
      DotcopterError::StripPrefix(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&dyn error::Error> {
    match *self {
      DotcopterError::IO(ref err) => Some(err),
      DotcopterError::Regex(ref err) => Some(err),
      DotcopterError::StripPrefix(ref err) => Some(err),
    }
  }
}

dotcopter_error_from!(io::Error, IO);
dotcopter_error_from!(regex::Error, Regex);
dotcopter_error_from!(path::StripPrefixError, StripPrefix);
