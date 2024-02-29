mod owner;

use chrono::{offset::Local, DateTime};
use clap::{command, Arg, ArgAction};
use owner::format_mode;
use std::{
    fs::{self, Metadata},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
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

    let strip_dir_rel_position = |object: (PathBuf, Metadata)| {
        let prefix_path_str = object
            .0
            .parent()
            .map(|p| p.to_string_lossy().into_owned() + "/");

        let path_str = object.0.to_string_lossy();

        let stripped_path = if let Some(prefix) = prefix_path_str {
            path_str
                .strip_prefix(&prefix)
                .unwrap_or(&path_str)
                .to_owned()
        } else {
            path_str.into_owned()
        };

        (stripped_path, object.1)
    };

    let hidden_files = |object: &(String, Metadata)| !object.0.starts_with('.') || show_all;

    files
        .into_iter()
        .filter_map(extract_metadata)
        .map(strip_dir_rel_position)
        .filter(hidden_files)
        .for_each(|(file_path, metadata)| match long_option {
            true => output_as_table(file_path, metadata, long_option),
            false => {
                if metadata.is_dir() {
                    print_colored_dir_path(file_path, long_option)
                } else {
                    print!("{}  ", file_path)
                }
            }
        });
}

fn output_as_table(file_path: String, metadata: Metadata, long_option: bool) {
    let file_type = if metadata.is_dir() { 'd' } else { '-' };
    let user = get_user_by_uid(metadata.uid()).unwrap();
    let group = get_group_by_gid(metadata.gid()).unwrap();
    let last_mod_time: DateTime<Local> = metadata.modified().unwrap().into();

    if metadata.is_dir() {
        print!(
            "{}{} {} {} {} {} {} ",
            file_type,
            format_mode(metadata.mode()),
            metadata.nlink(),
            user.name().to_string_lossy(),
            group.name().to_string_lossy(),
            metadata.len(),
            last_mod_time.format("%b %d %y %H:%M"),
        );
        print_colored_dir_path(file_path, long_option);
    } else {
        println!(
            "{}{} {} {} {} {} {} {}",
            file_type,
            format_mode(metadata.mode()),
            metadata.nlink(),
            user.name().to_string_lossy(),
            group.name().to_string_lossy(),
            metadata.len(),
            last_mod_time.format("%b %d %y %H:%M"),
            file_path
        );
    }
}

fn print_colored_dir_path(dir_name: String, long_option: bool) {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    let mut stdout_lock = stdout.lock();

    stdout_lock
        .set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))
        .unwrap();

    if long_option {
        println!("{}", dir_name);
    } else {
        print!("{}  ", dir_name);
    }
    stdout_lock.reset().unwrap();
}
