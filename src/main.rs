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
            let mut are_final = if files.len() == 1 {
                vec![true]
            } else {
                vec![false]
            };
            display_tree(files, 0, &mut are_final);
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

fn display_tree(files: Vec<Files>, position: usize, mut are_final: &mut Vec<bool>) {
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
                if position == 0 {
                    println!("{} {}", ending_pattern, file_name.into_string().unwrap());
                } else {
                    let mut res = String::new();
                    for &bl in are_final.iter().take(position) {
                        if !bl {
                            res.push_str("│  ");
                        } else {
                            res.push_str("   ");
                        }
                    }
                    println!(
                        "{}{} {}",
                        res,
                        ending_pattern,
                        file_name.into_string().unwrap()
                    );
                    // println!(
                    //     "│{}{} {}",
                    //     " ".repeat((position - 1) * 3 + 2),
                    //     ending_pattern,
                    //     file_name.into_string().unwrap()
                    // );
                }
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
                    match are_final.get(position + 1) {
                        Some(_) => (),
                        None => {
                            are_final.push(dir.files.len() > 1);
                        } // Default value, will be changed later
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
                    let mut starting_pattern = String::new();
                    for &is_final in are_final.iter().take(position) {
                        if !is_final {
                            starting_pattern.push_str("│  ");
                        } else {
                            starting_pattern.push_str("   ");
                        }
                    }

                    println!(
                        "{}{} {}",
                        starting_pattern,
                        ending_pattern,
                        dir.name.into_string().unwrap()
                    );
                    // let starting_char = {
                    //     if are_final[0] {
                    //         " "
                    //     } else {
                    //         "│"
                    //     }
                    // };
                    // println!(
                    //     "{}{}{} {}",
                    //     starting_char,
                    //     " ".repeat((position - 1) * 3 + 2),
                    //     ending_pattern,
                    //     dir.name.into_string().unwrap()
                    // );
                    match are_final.get(position + 1) {
                        Some(_) => (),
                        None => are_final.push(false), // Default value, will be changed later
                    }
                    display_tree(dir.files, position + 1, are_final);
                }
            }
        }
        file_position += 1;
    }
}
