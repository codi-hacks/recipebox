use std::{fs, path::Path, vec::Vec};
use std::fs::OpenOptions;
use std::io::prelude::*;

use clap::Parser;
use handlebars::Handlebars;
use serde::Serialize;
use crate::args::Args;
use crate::forms::recipe::Recipe;

#[derive(Serialize)]
pub enum CacheError {
    TemplateError(String),
    FileError(String),
    MarkdownError(String)
}

#[derive(Serialize)]
struct DashboardData {
    foo: usize
}

pub fn generate_dashboard() -> Result<usize, CacheError> {
    let args = Args::parse();

    let dashboard_page_path: String = Path::new(&args.pages).join("dashboard.hbs").display().to_string();
    let source = fs::read_to_string(dashboard_page_path)
        .map_err(|err| { CacheError::FileError(err.to_string()) })?;


    let handlebars = Handlebars::new();

    let data = DashboardData {
        foo: 5
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err.to_string()) })?;
    let output_path = Path::new(&args.cache).join("dashboard.html");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .map_err(|err| { CacheError::FileError(err.to_string()) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err.to_string()) })
}

#[derive(Serialize)]
struct IndexData {
    recipes: Vec<Recipe>,
    errors: Vec<CacheError>
}

pub fn generate_index() -> Result<usize, CacheError> {
    let args = Args::parse();

    let index_page_path = Path::new(&args.pages).join("index.hbs");
    let source = fs::read_to_string(index_page_path)
        .map_err(|err| { CacheError::FileError(err.to_string()) })?;

    let handlebars = Handlebars::new();

    let dir_entries = fs::read_dir(&args.pages)
        .map_err(|err| { CacheError::FileError(err.to_string()) })?;

    let mut recipes = Vec::new();
    let mut errors = Vec::new();
    // Scan directory from markdown files and index them
    for entry in dir_entries {
        let path = entry
            .map_err(|err| { CacheError::FileError(err.to_string()) })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str() == Some("md") {
                    match Recipe::from_file(&path) {
                        Ok(recipe) => recipes.push(recipe),
                        Err(error) => errors.push(error)
                    }
                }
            }
        }
    }

    let data = IndexData {
        recipes,
        errors
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err.to_string()) })?;
    let output_path = Path::new(&args.cache).join("index.html");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .map_err(|err| { CacheError::FileError(err.to_string()) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err.to_string()) })
}
