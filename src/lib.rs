use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::collections::BTreeMap;

use options::Options;

use std::io;
use std::fs::{remove_file, remove_dir, create_dir_all, read_link};
use std::os::unix::fs::symlink;

pub mod argument_error;
pub mod options;

pub fn lndir(sources: Vec<PathBuf>, destination: PathBuf, options: Option<Options>) -> Result<(), io::Error> {
  let options = options.unwrap_or_default();

  //println!("options: {:?}\nsources: {:?}\ndestination: {:?}", options, sources, destination);

  // Ensure that destination directory exists.
  destination.read_dir()?;

  let mut entry_map: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();

  for source in sources {
    let entries = entries(&source, 1, options.max_depth)?;

    for entry in entries.iter() {
      let relative_entry = entry.strip_prefix(&source).unwrap().to_owned();

      if let Some(other_source) = entry_map.get(&relative_entry) {
        panic!("Found {} in both {} and {}.", relative_entry.to_string_lossy(), other_source.to_string_lossy(), source.to_string_lossy());
      }

      if options.with_rev_info || !is_rev_info(&relative_entry) {
        entry_map.insert(relative_entry, source.to_owned());
      }
    }
  }

  for (relative_path, source) in entry_map.iter() {
    let source_path = source.canonicalize()?.join(relative_path);
    let destination_path = destination.canonicalize()?.join(relative_path);

    create_dir_all(destination_path.parent().unwrap())?;

    if let Ok(symlink_metadata) = destination_path.symlink_metadata() {
      if symlink_metadata.file_type().is_dir() {
        remove_dir(&destination_path)?;
      } else {
        remove_file(&destination_path)?;
      }
    }

    if !options.ignore_links && source_path.symlink_metadata()?.file_type().is_symlink() {
      let link_source = read_link(&source_path).unwrap();
      symlink(&link_source, &destination_path)?;
    } else {
      symlink(&source_path, &destination_path)?;
    }

    if !options.silent {
      println!("{}:", source_path.to_str().unwrap());
    }
  }

  Ok(())
}

fn is_rev_info(path: &Path) -> bool {
  if let Some(file_name) = path.file_name().and_then(OsStr::to_str) {
    match file_name {
      "BitKeeper" => return true,
      "CVS"       => return true,
      "CVS.adm"   => return true,
      ".git"      => return true,
      ".hg"       => return true,
      "RCS"       => return true,
      "SCCS"      => return true,
      ".svn"      => return true,
      _ => return false,
    }
  }

  false
}

fn entries(dir: &Path, depth: u32, max_depth: Option<u32>) -> Result<Vec<PathBuf>, io::Error> {
  match max_depth {
    Some(max_depth) if depth > max_depth => return Ok(Vec::new()),
    _ => (),
  }

  let mut paths: Vec<PathBuf> = Vec::new();

  for entry in dir.read_dir()? {
    let child = entry?.path();

    if child.is_dir() {
      let mut child_entries = entries(&child, depth + 1, max_depth)?;

      if child_entries.is_empty() {
        paths.push(child);
      } else {
        paths.append(&mut child_entries);
      }
    } else {
      paths.push(child);
    }
  }

  //for path in &paths {
  //  match path.read_dir() {
  //    Ok(children) => {
  //      for child in children {
  //        println!("{:?}", child);
  //      }
  //    },
  //    Err(err) => {
  //      return Err(ArgumentError::new(
  //        format!("{}: {}", path.to_string_lossy(), err),
  //        Some(Box::new(err)),
  //      ))
  //    },
  //  }
  //}

  Ok(paths)
}
