use core::convert::Infallible;
use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, read_link, File};
use std::ffi::OsStr;
use std::os::unix::fs::symlink;

use async_trait::async_trait;
use cucumber::{given, then, when, WorldInit, gherkin::Step};
use mktemp::Temp;

use lndir::lndir;

#[derive(Debug, WorldInit)]
pub struct World {
  pub tmpdir: PathBuf,
  pub paths: Vec<PathBuf>,
}

#[async_trait(?Send)]
impl cucumber::World for World {
  type Error = Infallible;

  async fn new() -> Result<Self, Self::Error> {
    Ok(Self {
      tmpdir: Temp::new_dir().expect("Failed to create temporary directory").to_path_buf(),
      paths: vec![],
    })
  }
}

fn parse_directory_tree(string: &String) -> Vec<PathBuf> {
  let mut lines = string.trim().lines();

  let mut paths = vec![];

  let mut root = if let Some(line) = lines.next() {
    PathBuf::from(line)
  } else {
    return paths
  };

  let mut lines = lines.peekable();

  let mut current_level = 0;

  while let Some(mut line) = lines.next() {
    let mut level = 0;

    while let Some(s) = line.strip_prefix("│   ") {
      line = s;
      level += 1;
    }

    for _ in 0..(current_level - level) {
      if let Some(parent) = root.parent() {
        root = parent.to_owned();
        current_level -= 1;
      } else {
        return paths
      }
    }

    if let Some(s) = line.strip_prefix("├── ").or_else(|| line.strip_prefix("└── ")) {
      root = root.join(s);
      paths.push(root.clone());
      current_level += 1;
    }
  }

  paths
}

fn split_path(path: &Path) -> (PathBuf, Option<PathBuf>) {
  let parts: Vec<PathBuf> = path.to_str().unwrap().split(" → ").map(PathBuf::from).collect();

  let relative_path = match parts[0].strip_prefix(".") {
    Ok(relative_path) => relative_path.to_owned(),
    Err(err) => panic!("Failed to strip prefix from path {}: {}", path.to_str().unwrap(), err),
  };
  let relative_symlink_path = parts.get(1).map(PathBuf::to_owned);

  (relative_path, relative_symlink_path)
}

#[given(expr = "the directory structure")]
async fn directory_structure(world: &mut World, step: &Step) {
  let docstring = step.to_owned().docstring.unwrap();

  world.paths = parse_directory_tree(&docstring);

  for path in &world.paths {
    let (relative_path, relative_symlink_path) = split_path(&path);

    let absolute_path = world.tmpdir.join(&relative_path);

    let file_name = absolute_path.file_name().and_then(OsStr::to_str).unwrap();

    if let Some(relative_symlink_path) = relative_symlink_path {
      create_dir_all(&absolute_path.parent().unwrap()).unwrap();
      symlink(&relative_symlink_path, &absolute_path).unwrap();
    } else if file_name.ends_with(".d") {
      create_dir_all(&absolute_path).unwrap();
    } else {
      create_dir_all(&absolute_path.parent().unwrap()).unwrap();
      File::create(&absolute_path).unwrap();
    }
  }
}

#[when(regex = r"I run `lndir\s+([^`]*)`")]
async fn run_lndir(world: &mut World, matches: &[String]) {
  let mut arguments: Vec<PathBuf> = matches[0].split(" ").map(|arg| world.tmpdir.join(PathBuf::from(arg))).collect();

  let len = arguments.len();

  let destination = arguments.split_off(len - 1).first().unwrap().to_owned();
  let sources = arguments;

  lndir(sources, destination, None).unwrap();
}

#[then(expr = "the resulting directory structure is")]
async fn resulting_directory_structure(world: &mut World) {
  for path in &world.paths {
   let (relative_path, relative_symlink_path) = split_path(&path);

   let absolute_path = world.tmpdir.join(relative_path.to_owned());
   let dir = absolute_path.parent().unwrap();
   let absolute_symlink_path = relative_symlink_path.to_owned().map(|path| dir.join(path));

 let file_name = absolute_path.file_name().and_then(OsStr::to_str).unwrap();

   if let Some(relative_symlink_path) = relative_symlink_path {
     if let Ok(symlink_path) = read_link(&absolute_path) {
       if relative_symlink_path.starts_with("..") {
         assert_eq!(dir.join(symlink_path).canonicalize().unwrap(), absolute_symlink_path.unwrap().canonicalize().unwrap());
       } else {
         assert_eq!(symlink_path, relative_symlink_path);
       }
     } else {
       panic!("{} is not a symlink.", relative_path.to_string_lossy());
     }
   } else if file_name.ends_with(".d") {
     assert!(absolute_path.is_dir(), "{} is not a directory.", relative_path.to_string_lossy());
   } else {
     assert!(absolute_path.is_file(), "{} is not a file.", relative_path.to_string_lossy());
   }
  }
}

#[tokio::main]
async fn main() {
  World::run("features").await
}
