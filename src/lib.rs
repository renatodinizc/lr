use clap::{command, Arg, ArgAction};
use std::{fmt::Debug, fs, path::PathBuf};

#[derive(Debug)]
pub struct Input {
    paths: Vec<PathBuf>,
    pub all: bool,
    pub long: bool,
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
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();

    Input {
        paths,
        all: matches.get_one::<bool>("all").unwrap().to_owned(),
        long: matches.get_one::<bool>("long").unwrap().to_owned(),
    }
}

pub fn execute(input: Input) {
    let files = find_files(input.paths);

    println!("{:#?}", files);
}

fn find_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let access_closure = |path: PathBuf| {
        if path.metadata().is_err() {
            eprintln!("lr: cannot find '{}':", path.display());
            None
        } else {
            Some(path)
        }
    };

    let mut result: Vec<PathBuf> = vec![];

    paths
        .into_iter()
        .filter_map(access_closure)
        .for_each(|path| {
            if path.is_dir() {
                result.extend(read_contents(path));
            } else {
                result.push(path)
            }
        });

    result
}

fn read_contents(dir: PathBuf) -> Vec<PathBuf> {
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!(
                "lr: cannot open directory '{}': {}",
                dir.display(),
                e.kind()
            );
            return vec![];
        }
    };

    entries
        .filter_map(|e| match e {
            Ok(entry) => Some(entry.path()),
            Err(_e) => None,
        })
        .collect::<Vec<PathBuf>>()
}
