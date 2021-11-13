/*
Copyright (c) 2021 Alan Ramírez Herrera

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
OR OTHER DEALINGS IN THE SOFTWARE.
 */

mod templates;

use anyhow::Context;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use zip::ZipArchive;

/// Prompts if the selected directory should be deleted
///
/// # Arguments
/// * `path` - The path to the directory to be deleted
///
/// # Returns
/// `true` if the directory should be deleted, `false` otherwise
///
/// # Errors
/// If the user cancels the operation
fn prompt_directory_delete(path: &Path) -> anyhow::Result<bool> {
    if Confirm::new()
        .with_prompt("Directory not empty, delete?")
        .interact()
        .context("Failed to prompt for directory deletion")?
    {
        if let Err(e) = fs::remove_dir_all(&path) {
            eprintln!("Cannot delete directory contents, error: {}", e);
            return Ok(false);
        }
        return Ok(true);
    }
    Ok(false)
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ProgrammingLanguage {
    Unknown,
    C,
    Cpp11,
    Cpp14,
    Cpp17,
}

impl From<usize> for ProgrammingLanguage {
    fn from(lang: usize) -> Self {
        match lang {
            0 => ProgrammingLanguage::C,
            1 => ProgrammingLanguage::Cpp11,
            2 => ProgrammingLanguage::Cpp14,
            3 => ProgrammingLanguage::Cpp17,
            _ => ProgrammingLanguage::Unknown,
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Get selected directory
    let directory = env::args()
        .nth(1)
        .unwrap_or_else(|| "esp-new-project".into());

    let dir = Path::new(&directory);
    if dir.exists() && dir.read_dir().unwrap().next().is_some() && !prompt_directory_delete(dir)? {
        return Ok(());
    }

    let language_selection = prompt_programming_language()?;

    let use_git = prompt_use_git()?;

    if !directory.is_empty() && !Path::new(directory.as_str()).exists() {
        fs::create_dir_all(dir)
            .context(format!("Failed to create directory \"{}\"", &directory))?;
    }

    // Create a temp file to download the template
    let mut tmp_file = tempfile::tempfile().unwrap();

    download_template(&mut tmp_file)?;

    // Unzip the template
    print!("🗄 Unziping file");
    io::stdout().flush().unwrap();
    let mut zip = zip::ZipArchive::new(tmp_file).unwrap();
    println!("\r✔ File unzipped");

    let prefix = zip.by_index(0).unwrap().enclosed_name().unwrap().to_owned();

    // Write the zip contents to the directory
    print!("📁 Writing files");
    extract_zip(&directory, &mut zip, &prefix)?;

    replace_main_file(&directory, language_selection)?;

    let project_language = match language_selection {
        ProgrammingLanguage::C => "",
        ProgrammingLanguage::Cpp11 => "set(CMAKE_CXX_VERSION 11)",
        ProgrammingLanguage::Cpp14 => "set(CMAKE_CXX_VERSION 14)",
        ProgrammingLanguage::Cpp17 => "set(CMAKE_CXX_VERSION 17)",
        _ => {
            eprintln!("Invalid option");
            return Ok(());
        }
    };
    set_cmake_programming_language(&directory, project_language)?;

    println!("\r✔ Files written  ");

    if use_git {
        initialize_git_repo(&directory)?;
    }

    println!("😁 Have fun!");
    Ok(())
}

fn download_template(tmp_file: &mut File) -> anyhow::Result<()> {
    // Download the template
    print!("🌐 Downloading template");
    io::stdout().flush().unwrap();
    let mut res = ureq::get(templates::TEMPLATE_FILE)
        .call()
        .context("Cannot download the template")?
        .into_reader();
    io::copy(&mut res, tmp_file).context("Cannot copy the template to temp file")?;
    println!("\r✔ Template downloaded       ");
    Ok(())
}

/// Intializes the git repository in the selected directory
///
/// # Arguments
/// * `directory` - The directory to initialize the git repository in
/// * `use_git` - Whether to initialize the git repository
fn initialize_git_repo(directory: &str) -> anyhow::Result<()> {
    print!("⚙️Initializing git repo");
    Command::new("git")
        .args(&["init", directory])
        .output()
        .context("Failed to init git repo")?;
    println!("\r✔ Git repo initialized  ");
    Ok(())
}

/// Prompts the user for the programming language to use
///
/// # Returns
/// The programming language selected by the user
///
/// # Errors
/// If the user cancels the operation
fn prompt_programming_language() -> anyhow::Result<ProgrammingLanguage> {
    let selected_language = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("💻 Programming language? (default: C)")
        .item("C")
        .item("C++ 11")
        .item("C++ 14")
        .item("C++ 17")
        .default(0)
        .interact()
        .context("Failed to prompt for programming language")?;

    Ok(ProgrammingLanguage::from(selected_language))
}

/// Prompts the user to initialize a git repository on the new project
///
/// # Returns
/// `true` if the user wants to initialize a git repository, `false` otherwise
///
/// # Errors
/// If the user cancels the operation
fn prompt_use_git() -> anyhow::Result<bool> {
    Confirm::new()
        .with_prompt("Initialize git repo? (needs git)?")
        .interact()
        .context("Failed to prompt for git initialization")
}

/// Sets the programming language in the CMakeLists.txt file
///
/// # Arguments
/// * `directory` - The directory that contains the project
/// * `language` - The programming language CMake template to use
///
/// # Errors
/// If the file cannot be found or the file cannot be written
fn set_cmake_programming_language(directory: &str, project_language: &str) -> anyhow::Result<()> {
    let cmake_file = Path::new(&directory).join("CMakeLists.txt");
    let mut cmake_list_file = fs::read_to_string(&cmake_file)
        .context("Cannot find CMakeLists.txt")?
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    cmake_list_file[4] = project_language.into();

    let new_cmake_file = cmake_list_file.join("\n");

    fs::write(&cmake_file, new_cmake_file)
        .context("Cannot write CMakeLists.txt to set programming language")?;

    Ok(())
}

/// Replaces the main file with the selected programming language
///
/// # Arguments
/// * `directory` - The directory to write the file to
/// * `language_selection` - The programming language to use
///
/// # Returns
/// `Ok(())` if the file was written successfully, `Err(anyhow::Error)` otherwise
fn replace_main_file(
    directory: &str,
    language_selection: ProgrammingLanguage,
) -> anyhow::Result<()> {
    let mut c_file = Path::new(&directory).join("main/main.c");
    if language_selection == ProgrammingLanguage::C {
        fs::write(c_file, templates::C_TEMPLATE).context("Cannot write C file")?;
    } else {
        // Remove main C file and replace with a C++ file
        fs::remove_file(&c_file).unwrap();
        c_file.pop();
        c_file.push("main.cpp");
        fs::write(c_file, templates::CPP_TEMPLATE).context("Cannot write cpp file")?;

        // Tell CMake to use the new main.cpp file
        let cmake_file = Path::new(&directory).join("main/CMakeLists.txt");
        let mut component_cmake = fs::read_to_string(&cmake_file)
            .unwrap()
            .split('\n')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        component_cmake[4] = r#"set(COMPONENT_SRCS "main.cpp")"#.into();

        let new_cmake_file = component_cmake.join("\n");

        fs::write(cmake_file, new_cmake_file).context("Cannot write CMakeLists.txt")?;
    }
    Ok(())
}

/// Extracts the zip template file to the directory
///
/// # Arguments
/// * `directory` - The directory to extract the template to
/// * `zip` - The zip archive to extract
/// * `prefix` -
///
/// # Returns
/// `Ok(())` if the extraction was successful, `Err(anyhow::Error)` otherwise
fn extract_zip(directory: &str, zip: &mut ZipArchive<File>, prefix: &Path) -> anyhow::Result<()> {
    for i in 1..zip.len() {
        let mut file = zip.by_index(i).unwrap();

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = PathBuf::new()
            .join(&directory)
            .join(outpath.strip_prefix(&prefix).unwrap());
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
            continue;
        }

        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(&p).unwrap();
            }
        }

        let mut outfile = fs::File::create(&outpath).unwrap();
        io::copy(&mut file, &mut outfile)
            .context(format!("Failed to unzip file \"{}\"", file.name()))?;
    }
    Ok(())
}
