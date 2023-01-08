use std::{fs, path::Path};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error as IoError;

use clap::Parser;
use handlebars::{Handlebars, RenderError};
use serde::Serialize;
use crate::args::Args;

pub enum CacheError {
    TemplateError(RenderError),
    FileError(IoError)
}

#[derive(Serialize)]
struct DashboardData {
    foo: usize
}

pub fn generate_dashboard() -> Result<usize, CacheError> {
    let args = Args::parse();

    let dashboard_page_path: String = Path::new(&args.pages).join("dashboard.hbs").display().to_string();
    let source = fs::read_to_string(dashboard_page_path)
        .map_err(|err| { CacheError::FileError(err) })?;


    let handlebars = Handlebars::new();

    let data = DashboardData {
        foo: 5
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err) })?;
    let output_path = Path::new(&args.cache).join("dashboard.html");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .map_err(|err| { CacheError::FileError(err) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err) })
}

#[derive(Serialize)]
struct IndexDataItem {
    title: String
}

#[derive(Serialize)]
struct IndexData {
    items: Vec<IndexDataItem>
}

pub fn generate_index() -> Result<usize, CacheError> {
    let args = Args::parse();

    let index_page_path: String = Path::new(&args.pages).join("index.hbs").display().to_string();
    let source = fs::read_to_string(index_page_path)
        .map_err(|err| { CacheError::FileError(err) })?;

    let handlebars = Handlebars::new();

    let dir_entries = fs::read_dir(&args.pages)
        .map_err(|err| { CacheError::FileError(err) })?;

    for entry in dir_entries {
        let path = entry
            .map_err(|err| { CacheError::FileError(err) })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str() == Some("md") {
                    println!("Name: {}", path.display());
                    //read md into data.items (IndexData struct instance)
                }
            }
        }
    }

    let data = IndexData {
        items: vec![
            IndexDataItem { title: String::from("Baked Mayonnaise") },
            IndexDataItem { title: String::from("Totally a Cheesecake (and not baked mayonnaise)") }
        ]
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err) })?;
    let output_path = Path::new(&args.cache).join("index.html");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .map_err(|err| { CacheError::FileError(err) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err) })
}
