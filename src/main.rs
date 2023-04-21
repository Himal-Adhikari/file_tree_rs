use std::{
    ffi::OsString,
    fs::{self, DirEntry, ReadDir},
};

#[derive(Debug)]
struct Dir {
    name: OsString,
    files: Vec<Files>,
}

impl Dir {
    fn new(name: OsString, entry: DirEntry) -> Self {
        let dir_name = entry.path();
        let files = get_all_files(fs::read_dir(dir_name).unwrap());
        Self { name, files }
    }
}

#[derive(Debug)]
enum Files {
    File(OsString),
    Directory(Dir),
}

fn main() {
    match fs::read_dir("") {
        Err(why) => eprintln!("{why}"),
        Ok(paths) => {
            let files = get_all_files(paths);
            display_tree(files, 0);
        }
    }
}

fn get_all_files(paths: ReadDir) -> Vec<Files> {
    let mut res = Vec::new();
    for path in paths {
        let entry = path.unwrap();
        let file_name = entry.file_name();
        let file_type = entry.file_type().unwrap();
        if file_type.is_file() {
            res.push(Files::File(file_name));
        } else if file_type.is_dir() {
            res.push(Files::Directory(Dir::new(file_name, entry)));
        }
    }
    res
}

fn display_tree(files: Vec<Files>, position: usize) {
    let mut file_position = 1;
    let total_files = files.len();
    for file in files {
        match file {
            Files::File(file_name) => {
                let ending_pattern = {
                    if file_position == total_files {
                        "└──"
                    } else {
                        "├──"
                    }
                };
                match position {
                    0 => {
                        println!("{} {}", ending_pattern, file_name.into_string().unwrap());
                    }
                    _ => {
                        println!(
                            "│{}{} {}",
                            " ".repeat((position - 1) * 3 + 2),
                            ending_pattern,
                            file_name.into_string().unwrap()
                        );
                    }
                }
            }
            Files::Directory(dir) => match position {
                0 => {
                    println!("├── {}", dir.name.into_string().unwrap());
                    display_tree(dir.files, position + 1);
                }
                _ => {
                    let no_of_files = dir.files.len();
                    let ending_pattern = {
                        if no_of_files > 0 || file_position == total_files {
                            "└──"
                        } else {
                            "├──"
                        }
                    };
                    println!(
                        "│{}{} {}",
                        " ".repeat((position - 1) * 3 + 2),
                        ending_pattern,
                        dir.name.into_string().unwrap()
                    );
                    display_tree(dir.files, position + 1);
                }
            },
        }
        file_position += 1;
    }
}
