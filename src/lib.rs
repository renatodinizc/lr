use clap::{command, Arg, ArgAction};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Input {
  paths: Vec<PathBuf>,
  all: bool,
  long: bool,
}

pub fn get_args() -> Input {
  let matches = command!()
  .arg(
      Arg::new("all")
      .help("Do not ignore entries starting with '.'")
      .short('a')
      .long("all")
      .action(ArgAction::SetTrue),
  )
  .arg(
    Arg::new("long")
    .help("Use a long listing format")
    .short('l')
    .long("long")
    .action(ArgAction::SetTrue),
)
  .arg(
      Arg::new("paths")
      .help("List information about the FILEs (the current directory by default)")
      .action(ArgAction::Append)
      .default_value("."),
  )
  .get_matches();

  let paths = matches
      .get_many::<String>("paths")
      .unwrap()
      .map(|v| PathBuf::from(v))
      .collect::<Vec<PathBuf>>();

  Input {
      paths,
      all: matches.get_one::<bool>("all").unwrap().to_owned(),
      long: matches.get_one::<bool>("long").unwrap().to_owned(),
  }
}
