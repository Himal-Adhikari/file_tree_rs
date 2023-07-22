use std::{
    ffi::OsString,
    fs::{self, DirEntry, ReadDir},
    io::{self, Write},
};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    path: Option<String>,

    /// Show hidden files
    #[arg(short = 'a', long = "all")]
    hidden: bool,
}

#[derive(Debug)]
struct Dir {
    name: OsString,
    files: Vec<Files>,
}

impl Dir {
    fn new(name: OsString, entry: DirEntry, read_hidden_files: bool) -> std::io::Result<Self> {
        let file = fs::read_dir(entry.path())?;
        let files = get_all_files(file, read_hidden_files);
        Ok(Self { name, files })
    }
}

#[derive(Debug)]
enum Files {
    File(OsString),
    Directory(Dir),
    Symlink(OsString),
}

fn main() {
    let cli = Cli::parse();

    let path = match cli.path {
        Some(path) => path,
        None => String::from("."),
    };

    match fs::read_dir(&path) {
        Err(why) => eprintln!("Invalid directory name \"{path}\": {why}"),
        Ok(paths) => {
            let files = get_all_files(paths, cli.hidden);
            let mut are_final = vec![files.len() == 1];
            display_tree(files, 0, &mut are_final);
        }
    }
}

fn get_all_files(paths: ReadDir, read_hidden_files: bool) -> Vec<Files> {
    let mut res = Vec::new();
    for path in paths {
        let entry = match path {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        let mut file_name = entry.file_name();
        let file_type = match entry.file_type() {
            Ok(file) => file,
            Err(e) => {
                eprintln!(
                    "Couldn't detect file_type of {}: {}",
                    file_name.into_string().unwrap(),
                    e
                );
                continue;
            }
        };
        if file_name.to_str().unwrap().starts_with('.') && !read_hidden_files {
            continue;
        }
        if file_type.is_file() {
            res.push(Files::File(file_name));
        } else if file_type.is_dir() {
            match Dir::new(file_name.clone(), entry, read_hidden_files) {
                Ok(dir) => {
                    res.push(Files::Directory(dir));
                }
                Err(e) => {
                    eprintln!(
                        "Couldn't read contents of directory {}: {e}",
                        file_name.into_string().unwrap()
                    );
                }
            }
        } else if file_type.is_symlink() {
            let link_path = match std::fs::read_link(entry.path()) {
                Ok(link_path) => link_path,
                Err(e) => {
                    eprintln!(
                        "Couldn't read symlink {}: {}",
                        entry.path().to_string_lossy(),
                        e
                    );
                    continue;
                }
            };
            let link_path = link_path.into_os_string();
            file_name.push(" => ");
            file_name.push(link_path);
            res.push(Files::Symlink(file_name));
        }
    }
    res
}

fn display_tree(files: Vec<Files>, position: usize, are_final: &mut Vec<bool>) {
    let mut file_position = 1;
    let total_files = files.len();
    for file in files {
        match file {
            Files::File(file_name) | Files::Symlink(file_name) => {
                let mut buffer = String::new();
                let ending_pattern = if file_position == total_files {
                    are_final[position] = true;
                    "└── "
                } else {
                    "├── "
                };
                for &is_final in are_final.iter().take(position) {
                    if is_final {
                        buffer.push_str("   ");
                    } else {
                        buffer.push_str("│  ");
                    }
                }
                buffer.push_str(ending_pattern);
                buffer.push_str(&file_name.into_string().unwrap());
                buffer.push('\n');
                print_buffer(buffer);
            }
            Files::Directory(dir) => {
                let mut buffer = String::new();
                are_final[position] = file_position == total_files;
                let no_of_files = dir.files.len();
                let ending_pattern =
                    if file_position == total_files || (no_of_files > 0 && are_final[position]) {
                        "└── "
                    } else {
                        "├── "
                    };
                for &is_final in are_final.iter().take(position) {
                    if is_final {
                        buffer.push_str("   ");
                    } else {
                        buffer.push_str("│  ");
                    }
                }
                buffer.push_str(ending_pattern);
                buffer.push_str(&dir.name.into_string().unwrap());
                buffer.push('\n');
                print_buffer(buffer);
                match are_final.get(position + 1) {
                    None => are_final.push(dir.files.len() > 1),
                    Some(_) => are_final[position + 1] = dir.files.len() > 1,
                }
                display_tree(dir.files, position + 1, are_final);
            }
        }
        file_position += 1;
    }
}

fn print_buffer(buffer: String) {
    if let Err(e) = io::stdout().write_all(buffer.as_bytes()) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        } else {
            eprintln!("{e:#?}");
        }
    }
}
