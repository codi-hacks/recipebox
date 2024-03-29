use std::collections::HashMap;
use std::{fs, path};
use std::io::Write;
use pulldown_cmark::{Parser, html, Options};
use form_urlencoded::parse as uri_parse;
use matter::matter;
use regex::Regex;
use serde::Serialize;
use crate::cache::CacheError;

#[derive(Serialize)]
pub struct Recipe {
    pub title: String,
    pub recipe: String
}

//fn decode_form_field() {}

impl Recipe {
    /// Parse requests with a Content-Type of `application/x-www-form-urlencoded`
    pub fn from_request(request_body: &Vec<u8>) -> Result<Recipe, &[u8]> {
        let form_data: HashMap<String, String> = HashMap::from_iter(uri_parse(request_body).into_owned());

        if form_data.get("title").is_none() {
            return Err(b"'title' field is missing.");
        }
        let title = String::from(form_data.get("title").unwrap());
        if title.trim().len() == 0 {
            return Err(b"'title' field provided with a blank value.");
        }

        if form_data.get("recipe").is_none() {
            return Err(b"'recipe' field is missing.");
        }
        let recipe = String::from(form_data.get("recipe").unwrap());
        if recipe.trim().len() == 0 {
            return Err(b"'recipe' field provided with a blank value.");
        }

        Ok(Recipe { title, recipe })
    }

    pub fn from_file(path: &path::PathBuf) -> Result<Recipe, CacheError> {
        let input = fs::read_to_string(path)
            .map_err(|err| { CacheError::FileError(err.to_string()) })?;
        if let Some((matter, markdown)) = matter(&input) {
            return Ok(Recipe {
                title: matter,
                recipe: markdown
            })
        }
        Err(CacheError::MarkdownError(
            String::from(format!(
                "{} does not contain frontmatter and/or markdown.",
                path.display()
            ))
        ))
    }

    /// Attempt to write markdown as an html file or return error
    pub fn markdown_to_html(&self) -> Result<(), std::io::Error> {
        // Strikethroughs are not part of the CommonMark standard
        // and we therefore must enable it explicitly.
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.recipe, options);

        // Pre-allocate enough for a typical html partial
        let mut html_output = String::with_capacity(4000);
        html::push_html(&mut html_output, parser);

        let mut filename = self.title.clone();
        filename.truncate(200);
        let filename = filename.trim().to_lowercase();
        // Sanitized name to make it filesystem-friendly
        let regex = Regex::new(r"[^a-z]").unwrap();
        let filename = regex.replace_all(&filename, "_");

        let mut file = fs::File::create_new(format!("{}.md", filename))?;
        file.write_all(self.recipe.as_bytes())?;
        let mut file = fs::File::create_new(format!("{}.html", filename))?;
        file.write_all(html_output.as_bytes())?;

        Ok(())
    }
}
