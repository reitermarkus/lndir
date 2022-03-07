#![cfg_attr(test, allow(dead_code))]

extern crate lndir;

use std::vec::Vec;
use std::path::PathBuf;
use std::env;

use lndir::lndir;
use lndir::options::Options;
use lndir::argument_error::ArgumentError;

fn parse_args() -> Result<(Options, Vec<PathBuf>, PathBuf), ArgumentError> {
  let mut options = Options::new();
  let mut paths: Vec<PathBuf> = Vec::new();

  let mut iterator = env::args().skip(1);
  let mut stop_option_parsing = false;

  while let Some(arg) = iterator.next() {
    match arg.as_ref() {
      "-silent" if !stop_option_parsing => {
        options.silent = true;
      },
      "-ignorelinks" if !stop_option_parsing  => {
        options.ignore_links = true;
      },
      "-withrevinfo" if !stop_option_parsing  => {
        options.with_rev_info = true;
      },
      "-maxdepth" if !stop_option_parsing  => {
        if let Some(value) = iterator.next() {
          options.max_depth = match value.parse() {
            Ok(max_depth) => Some(max_depth),
            Err(err) => return Err(ArgumentError::new(
              format!("failed to parse -maxdepth argument \"{}\": {}", value, err),
              Some(Box::new(err)),
            )),
          }
        } else {
          return Err(ArgumentError::new(
            "no value specified for -maxdepth",
            None,
          ))
        }
      },
      "--" if !stop_option_parsing => {
        stop_option_parsing = true;
      },
      _ => {
        stop_option_parsing = true;

        paths.push(PathBuf::from(arg));
      },
    }
  }

  let destination = if paths.len() < 2 {
    env::current_dir().unwrap()
  } else {
    let len = paths.len();

    paths.split_off(len - 1).first().map(|d| d.to_owned())
      .ok_or_else(|| ArgumentError::new(
        "no destination directory specified",
        None,
      ))?
  };

  let sources = paths;

  if sources.is_empty() {
    return Err(ArgumentError::new(
      "no source directory specified",
      None,
    ))
  }

  Ok((options, sources, destination))
}

fn main() -> Result<(), ArgumentError> {
  let program = env::args().next().unwrap();

  let (options, sources, destination) = parse_args()?;

  println!("program: {:?}", program);

  lndir(sources, destination, Some(options)).unwrap();

  Ok(())
}
