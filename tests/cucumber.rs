#![feature(fnbox)]

#[macro_use]
extern crate cucumber_rust;

extern crate regex;
extern crate mktemp;
extern crate lndir;

use std::env::temp_dir;
use std::path::PathBuf;
use mktemp::Temp;


pub struct World {
  pub tmpdir: PathBuf,
}

impl cucumber_rust::World for World {}
impl std::default::Default for World {
  fn default() -> World {
      World { tmpdir: Temp::new_dir().unwrap().to_path_buf() }
  }
}


mod example_steps {
  use std::collections::BTreeMap;
  use std::path::PathBuf;
  use std::fs::create_dir_all;
  use std::fs::read_link;
  use std::fs::remove_dir_all;
  use std::fs::File;
  use std::ffi::OsStr;
  use lndir::lndir;

  fn parse_directory_tree(string: &String) -> Vec<PathBuf> {
    let mut lines = string.lines();

    let mut tree: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();

    let mut current_key: Option<PathBuf> = None;

    let root = if let Some(line) = lines.next() {
      PathBuf::from(line)
    } else {
      return Vec::new()
    };

    loop {
      if let Some(line) = lines.next() {
        let prefix: String = line.chars().take(4).collect();
        let mut string: String = line.chars().skip(4).collect();

        if prefix == "├── " || prefix == "└── " {
          let path = PathBuf::from(string.to_owned());
          current_key = Some(path.to_owned());
          tree.insert(path, vec![string]);
        } else if let Some(key) = &current_key {
          if let Some(lines) = tree.get_mut(key) {
            lines.push(string)
          }
        }
      } else {
        break;
      }
    }

    let mut paths: Vec<PathBuf> = Vec::new();

    for (path, lines) in tree {
      if lines.len() > 1 {
        let doc = lines.join("\n");
        let mut sub_paths = parse_directory_tree(&doc).iter().map(|sub_path| root.join(sub_path) ).collect();

        paths.append(&mut sub_paths);
      } else {
        paths.push(root.join(path.to_owned()));
      }
    }

    paths
  }

  steps! {
    world: ::World; // Any type that implements Default can be the world

    given "the directory structure" |world, step| {
      let docstring = step.to_owned().docstring.unwrap();

      let paths = parse_directory_tree(&docstring);

      for path in paths {
        let relative_path = path.strip_prefix(".").unwrap();
        let absolute_path = world.tmpdir.join(relative_path);

        let file_name = absolute_path.file_name().and_then(OsStr::to_str).unwrap();

        if file_name.ends_with(".d") {
          create_dir_all(&absolute_path).unwrap();
        } else {
          create_dir_all(&absolute_path.parent().unwrap()).unwrap();
          File::create(&absolute_path).unwrap();
        }
      }
    };

    when regex r"I run `lndir\s+([^`]*)`" |world, matches, step| {
      let mut arguments: Vec<PathBuf> = matches[1].split(" ").map(|arg| world.tmpdir.join(PathBuf::from(arg))).collect();

      let len = arguments.len();

      let destination = arguments.split_off(len - 1).first().unwrap().to_owned();
      let sources = arguments;

      lndir(sources, destination, None).unwrap();
    };

    then "the resulting directory structure is" |world, step| {
      let docstring = step.to_owned().docstring.unwrap();

      let paths = parse_directory_tree(&docstring);

      for path in paths {
        let parts: Vec<String> = path.to_str().unwrap().split(" → ").map(str::to_string).collect();

        let relative_path = PathBuf::from(parts[0].to_owned());
        let relative_path = relative_path.strip_prefix(".").unwrap();
        let relative_symlink_path = parts.get(1).to_owned();
        let absolute_symlink_path = relative_symlink_path.map(|path| world.tmpdir.join(path).canonicalize().unwrap());
        let absolute_path = world.tmpdir.join(relative_path);

        let file_name = absolute_path.file_name().and_then(OsStr::to_str).unwrap();

        if let Some(absolute_symlink_path) = absolute_symlink_path {
          if let Ok(symlink_path) = read_link(&absolute_path) {
            assert_eq!(symlink_path, absolute_symlink_path);
          } else {
            panic!("{} is not a symlink.", relative_path.to_string_lossy());
          }
        } else if file_name.ends_with(".d") {
          assert!(absolute_path.is_dir(), format!("{} is not a directory.", relative_path.to_string_lossy()));
        } else {
          assert!(absolute_path.is_file(), format!("{} is not a file.", relative_path.to_string_lossy()));
        }
      }
    };
  }
}

cucumber! {
  features: "./features"; // Path to our feature files
  world: ::World; // The world needs to be the same for steps and the main cucumber call
  steps: &[
    example_steps::steps // the `steps!` macro creates a `steps` function in a module
  ];
  before: || {
    // Called once before everything; optional.
  }
}
