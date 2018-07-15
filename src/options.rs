#[derive(Default, Debug)]
pub struct Options {
  pub silent: bool,
  pub ignore_links: bool,
  pub with_rev_info: bool,
  pub max_depth: Option<u32>,
}

impl Options {
  pub fn new() -> Self {
    Options {
      silent: false,
      ignore_links: false,
      with_rev_info: false,
      max_depth: None,
    }
  }
}
