use std::{fs, path::Path};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error as IoError;
use handlebars::{Handlebars, RenderError};
use serde::Serialize;

pub enum CacheError {
    TemplateError(RenderError),
    FileError(IoError)
}

#[derive(Serialize)]
struct DashboardData {
    foo: usize
}

pub fn generate_dashboard() -> Result<usize, CacheError> {
    let source = fs::read_to_string("./pages/dashboard.hbs")
        .map_err(|err| { CacheError::FileError(err) })?;


    let handlebars = Handlebars::new();

    let data = IndexData {
        foo: 5
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err) })?;
    let path = Path::new("./cache/dashboard.html");


    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| { CacheError::FileError(err) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err) })
}

#[derive(Serialize)]
struct IndexData {
    foo: usize
}

pub fn generate_index() -> Result<usize, CacheError> {
    let source = fs::read_to_string("./pages/index.hbs")
        .map_err(|err| { CacheError::FileError(err) })?;


    let handlebars = Handlebars::new();

    let data = IndexData {
        foo: 5
    };

    let output = handlebars.render_template(&source, &data)
        .map_err(|err| { CacheError::TemplateError(err) })?;
    let path = Path::new("./cache/index.html");


    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| { CacheError::FileError(err) })?;

    file.write(output.as_bytes())
        .map_err(|err| { CacheError::FileError(err) })
}
