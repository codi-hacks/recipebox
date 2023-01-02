use std::{env, fs, path::Path, process};

use clap::Parser;
use inquire::Confirm;
use log::error;
use crate::args::Args;
use crate::cache::{generate_dashboard, generate_index};

pub fn check_for_directories() -> () {
    let args = Args::parse();

    //
    // Generate any missing directories
    //
    if !check_directory(&args.pages) {
        prompt_to_create_dir(
            &args.pages,
            "No directory for pages found. Would you like to generate the default pages?",
            "This will create a \"/pages\" folder in the current directory."
        )
    }

    if !check_directory(&args.cache) {
        prompt_to_create_dir(
            &args.cache,
            "No cache directory found. Would you like to generate it?",
            "This will create a \"/cache\" folder in the current directory."
        )
    }

    //
    // Generate any missing pages and their static html output
    //
    let index_path: String = Path::new(&args.pages).join("index.hbs").display().to_string();
    if !check_file(&index_path) {
        let index_template = include_bytes!("../../default_pages/index.hbs");
        prompt_to_create_file(
            &index_path,
            index_template,
            &format!("Template \"{}\" does not exist. Generate it?", &index_path),
            "This is the html used to render the home page."
        );
        if let Err(_) = generate_index() {
            error!("Could not write index template");
            process::exit(1);
        }
    }

    let dashboard_path: String = Path::new(&args.pages).join("dashboard.hbs").display().to_string();
    if !check_file(&dashboard_path) {
        let dashboard_template = include_bytes!("../../default_pages/dashboard.hbs");
        prompt_to_create_file(
            &dashboard_path,
            dashboard_template,
            &format!("Template \"{}\" does not exist. Generate it?", &dashboard_path),
            "This is the html for the admin dashboard."
        );
        if let Err(_) = generate_dashboard() {
            error!("Could not write dashboard template");
            process::exit(1);
        }
    }
}

fn check_directory(path: &str) -> bool {
    let mut pages_dir = match env::current_dir() {
        Ok(d) => d,
        _ => return false
    };
    pages_dir.push(path);
    let metadata = match fs::metadata(pages_dir) {
        Ok(d) => d,
        _ => return false
    };
    metadata.is_dir()
}

fn check_file(path: &str) -> bool {
    let mut pages_dir = match env::current_dir() {
        Ok(d) => d,
        _ => return false
    };
    pages_dir.push(path);
    let metadata = match fs::metadata(pages_dir) {
        Ok(d) => d,
        _ => return false
    };
    metadata.is_file()
}

fn prompt_to_create_dir(dir: &str, text: &str, subtext: &str) -> () {
    let answer = Confirm::new(text)
        .with_default(true)
        .with_help_message(subtext)
        .prompt();

    match answer {
        Ok(true) => {
            match fs::create_dir(dir) {
                Err(err) => {
                    error!("{}", err);
                },
                _ => {
                    return
                }
            }
        },
        _ => {
        }
    }
    process::exit(1)
}

fn prompt_to_create_file(file: &str, content: &[u8], text: &str, subtext: &str) -> () {
    let answer = Confirm::new(text)
        .with_default(true)
        .with_help_message(subtext)
        .prompt();

    match answer {
        Ok(true) => {
            match fs::write(file, content) {
                Err(err) => {
                    error!("{}", err);
                },
                _ => {
                    return
                }
            }
        },
        _ => {
        }
    }
    process::exit(1)
}
