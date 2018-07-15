use std::error::Error;

#[derive(Debug)]
pub struct ArgumentError {
  description: String,
  cause: Option<Box<Error>>,
}

impl ArgumentError {
  pub fn new<S: Into<String>>(description: S, cause: Option<Box<Error>>) -> Self {
    Self { description: description.into(), cause: cause }
  }
}
use std::fmt;
impl fmt::Display for ArgumentError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ArgumentError")
  }
}

impl Error for ArgumentError {
  fn description(&self) -> &str {
    &self.description
  }

  fn cause(&self) -> Option<&Error> {
    if let Some(ref err) = self.cause {
      Some(err.as_ref())
    } else {
      None
    }
  }
}
