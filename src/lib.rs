use chrono::{offset::Local, DateTime};
use clap::{command, Arg, ArgAction};
use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

pub struct Input {
    paths: Vec<PathBuf>,
    pub show_all: bool,
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
        show_all: matches.get_one::<bool>("all").unwrap().to_owned(),
        long: matches.get_one::<bool>("long").unwrap().to_owned(),
    }
}

pub fn execute(input: Input) {
    let files = find_files(input.paths);
    format_output(files, input.long, input.show_all);
}

fn find_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let access_closure = |path: PathBuf| {
        if path.metadata().is_err() {
            eprintln!(
                "lr: cannot access '{}': {}",
                path.display(),
                path.metadata().err().unwrap()
            );
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

    result.sort();

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

fn format_output(files: Vec<PathBuf>, long_option: bool, show_all: bool) {
    let extract_metadata = |file: PathBuf| match fs::metadata(&file) {
        Err(e) => {
            eprintln!(
                "lr: cannot access file's metadata '{}': {}",
                file.display(),
                e.kind()
            );
            None
        }
        Ok(metadata) => Some((file, metadata)),
    };

    files
        .into_iter()
        .filter_map(extract_metadata)
        .filter(|(file_path, _metadata)| {
            !file_path.to_string_lossy().starts_with("./.") || show_all
        })
        .for_each(|(file_path, metadata)| {
            if long_option {
                output_as_table(file_path, metadata);
            } else {
                println!("{}", file_path.display());
            }
        });
}

fn output_as_table(file_path: PathBuf, metadata: std::fs::Metadata) {
    let mut table = Table::new("{:>}{:>} {:<} {:<} {:<} {:<} {:<} {:<}");

    let file_type = if metadata.is_dir() { 'd' } else { '-' };
    let user = get_user_by_uid(metadata.uid()).unwrap();
    let group = get_group_by_gid(metadata.gid()).unwrap();
    let last_mod_time: DateTime<Local> = metadata.modified().unwrap().into();

    table.add_row(
        Row::new()
            .with_cell(file_type)
            .with_cell(metadata.mode())
            .with_cell(metadata.nlink())
            .with_cell(user.name().to_string_lossy())
            .with_cell(group.name().to_string_lossy())
            .with_cell(metadata.len())
            .with_cell(last_mod_time.format("%d/%m/%Y %T"))
            .with_cell(file_path.display()),
    );
    print!("{}", table);
}
