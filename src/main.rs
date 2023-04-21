use std::{
    ffi::OsString,
    fs::{self, DirEntry, ReadDir},
};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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
}

fn main() {
    let cli = Cli::parse();

    let path = match cli.path {
        Some(path) => path,
        None => String::from(""),
    };

    match fs::read_dir(path) {
        Err(why) => eprintln!("{why}"),
        Ok(paths) => {
            let files = get_all_files(paths, cli.hidden);
            let mut are_final = if files.len() == 1 {
                vec![true]
            } else {
                vec![false]
            };
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
        let file_name = entry.file_name();
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
        }
    }
    res
}

fn display_tree(files: Vec<Files>, position: usize, are_final: &mut Vec<bool>) {
    let mut file_position = 1;
    let total_files = files.len();
    for file in files {
        match file {
            Files::File(file_name) => {
                let ending_pattern = {
                    if file_position == total_files {
                        are_final[position] = true;
                        "└──"
                    } else {
                        are_final[position] = false;
                        "├──"
                    }
                };
                for &is_final in are_final.iter().take(position) {
                    if is_final {
                        print!("   ")
                    } else {
                        print!("│  ")
                    }
                }
                println!("{} {}", ending_pattern, file_name.into_string().unwrap());
            }
            Files::Directory(dir) => {
                if position == 0 {
                    let ending_pattern = {
                        if file_position == total_files {
                            are_final[position] = true;
                            "└──"
                        } else {
                            are_final[position] = false;
                            "├──"
                        }
                    };
                    println!("{} {}", ending_pattern, dir.name.into_string().unwrap());
                    if are_final.get(position + 1).is_none() {
                        are_final.push(dir.files.len() > 1);
                    }
                    display_tree(dir.files, position + 1, are_final);
                } else {
                    if file_position == total_files {
                        are_final[position] = true;
                    } else {
                        are_final[position] = false;
                    }
                    let no_of_files = dir.files.len();
                    let ending_pattern = {
                        if (no_of_files > 0 && are_final[position]) || file_position == total_files
                        {
                            "└──"
                        } else {
                            "├──"
                        }
                    };
                    for &is_final in are_final.iter().take(position) {
                        if is_final {
                            print!("   ")
                        } else {
                            print!("│  ")
                        }
                    }

                    println!("{} {}", ending_pattern, dir.name.into_string().unwrap());
                    if are_final.get(position + 1).is_none() {
                        are_final.push(dir.files.len() > 1);
                    }
                    display_tree(dir.files, position + 1, are_final);
                }
            }
        }
        file_position += 1;
    }
}
