use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub fn hash(path: &Path) -> Result<String, std::io::Error> {
  let file = File::open(path)?;

  let mut hash = Sha256::new();
  let mut reader = BufReader::new(file);
  let mut buffer: [u8; 1024] = [0; 1024];

  loop {
    let len = reader.read(&mut buffer)?;
    if len == 0 {
      break;
    }
    hash.input(&buffer[..len]);
  }
  Ok(hash.result_str())
}
