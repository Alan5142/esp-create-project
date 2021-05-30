use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use dialoguer::{Confirm, Select};
use dialoguer::theme::ColorfulTheme;

const TEMPLATE_FILE: &str = "https://github.com/espressif/esp-idf-template/archive/refs/heads/master.zip";

const C_TEMPLATE: &str = r#"#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"


void app_main(void)
{
    // TODO Insert code
}
"#;

const CPP_TEMPLATE: &str = r#"#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"


extern "C" void app_main(void)
{
    // TODO Insert code
}
"#;

fn main() {
    let directory = env::args().nth(1).unwrap_or_else(|| "esp-new-project".into());

    let dir = Path::new(&directory);
    if dir.exists() && dir.read_dir().unwrap().next().is_some() {
        if Confirm::new()
            .with_prompt("? Directory not empty, delete?")
            .interact()
            .expect("An option is required")
        {
            if let Err(e) = fs::remove_dir_all(&dir) {
                eprintln!("Cannot delete directory contents, error: {}", e);
            }
        } else {
            return;
        }
    }

    let language_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("💻 Programming language? (default: C)")
        .item("C")
        .item("C++ 11")
        .item("C++ 14")
        .item("C++ 17")
        .default(0)
        .interact()
        .expect("An option is required");

    let use_git = Confirm::new()
        .with_prompt("Initialize git repo? (needs git)?")
        .interact()
        .expect("An option is required");

    if directory != "" && !Path::new(directory.as_str()).exists() {
        if let Err(e) = std::fs::create_dir_all(&directory) {
            eprintln!("Cannot create directory \"{}\", error: {}", &directory, e);
            return;
        }
    }

    let mut tmpfile = tempfile::tempfile().unwrap();
    print!("🌐 Downloading template");
    reqwest::blocking::get(TEMPLATE_FILE)
        .expect("Cannot download the template")
        .copy_to(&mut tmpfile).unwrap();
    println!("\r✔ Template downloaded       ");

    print!("🗄 Unziping file");
    let mut zip = zip::ZipArchive::new(tmpfile).unwrap();
    println!("\r✔ File unzipped");

    let prefix = zip.by_index(0).unwrap().enclosed_name().unwrap().to_owned();

    print!("📁 Writing files");
    for i in 1..zip.len() {
        let mut file = zip.by_index(i).unwrap();


        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = PathBuf::new().join(&directory).join(outpath.strip_prefix(&prefix).unwrap());
        if file.name().ends_with("/") {
            fs::create_dir_all(&outpath).unwrap();
            continue;
        }

        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(&p).unwrap();
            }
        }

        let mut outfile = fs::File::create(&outpath).unwrap();
        io::copy(&mut file, &mut outfile).expect("Cannot write file");
    }

    let project_language = match language_selection {
        0 => "",
        1 => "set(CMAKE_CXX_VERSION 11)",
        2 => "set(CMAKE_CXX_VERSION 14)",
        3 => "set(CMAKE_CXX_VERSION 17)",
        _ => {
            eprintln!("Invalid option");
            return;
        }
    };

    let mut c_file = Path::new(&directory).join("main/main.c");
    if language_selection == 0 {
        fs::write(c_file, C_TEMPLATE).expect("Cannot write C file");
    } else {
        fs::remove_file(&c_file).unwrap();
        c_file.pop();
        c_file.push("main.cpp");
        fs::write(c_file, CPP_TEMPLATE).unwrap();

        let cmake_file = Path::new(&directory)
            .join("main/CMakeLists.txt");
        let mut component_cmake = fs::read_to_string(&cmake_file)
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        component_cmake[4] = r#"set(COMPONENT_SRCS "main.cpp")"#.into();

        let new_cmake_file = component_cmake.join("\n");

        fs::write(cmake_file, new_cmake_file).unwrap();
    }
    let cmake_file = Path::new(&directory)
        .join("CMakeLists.txt");
    let mut cmake_list_file = fs::read_to_string(&cmake_file)
        .unwrap()
        .split("\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    cmake_list_file[4] = project_language.into();

    let new_cmake_file = cmake_list_file.join("\n");

    fs::write(&cmake_file, new_cmake_file).unwrap();

    println!("\r✔ Files written  ");

    if use_git {
        print!("⚙️Initializing git repo");
        Command::new("git")
            .args(&["init", directory.as_str()])
            .output()
            .expect("Failed to init git repo");
        println!("\r✔ Git repo initialized  ");
    }

    println!("Have fun!")
}

#[cfg(test)]
mod tests {}
