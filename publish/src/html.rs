use dom_query::Selection;
use std::{path::PathBuf, string::FromUtf8Error};
use thiserror::{self, Error};

const EDITABLE_ATTRS: [&'static str; 2] = ["href", "src"];

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error")]
    IoError(#[from] std::io::Error),

    #[error("HTML content error")]
    FormatError(#[from] FromUtf8Error),

    #[error("Error")]
    AnyError(#[from] anyhow::Error),
}

pub fn walk_html(dir: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let dir_iter = std::fs::read_dir(dir)?;

    let mut ret = vec![];
    for dir in dir_iter {
        let dir = dir?;

        let metadata = dir.metadata()?;

        let path = dir.path();

        match metadata.is_dir() {
            true => {
                let mut walked = walk_html(path)?;
                ret.append(&mut walked)
            }
            false => {
                match path.extension() {
                    Some(ex) => match ex.to_str().unwrap() {
                        "html" | "htm" => ret.push(path),
                        _ => (),
                    },
                    None => (), // continue, no extension
                }
            }
        }
    }

    Ok(ret)
}

pub fn scan_html(
    f: PathBuf,
    editor: impl Fn(String) -> Result<String, Error>,
) -> Result<String, Error> {
    let content = std::fs::read(f)?;
    let content = String::from_utf8(content)?;
    let doc = dom_query::Document::from(content);

    let body = doc.select("body");
    let head = doc.select("head");

    scan_body(body.clone(), &editor)?;
    scan_body(head.clone(), &editor)?;

    Ok(doc.html().to_string())
}

fn scan_body(
    sel: Selection,
    editor: &impl Fn(String) -> Result<String, Error>,
) -> Result<(), Error> {
    for child in sel.children().iter() {
        if child.is("div") || child.is("span") {
            for deeper_child in child.children().iter() {
                scan_body(deeper_child.clone(), editor)?;
            }
            continue;
        }

        for attr in EDITABLE_ATTRS {
            replace_if_present(child.clone(), attr, editor)?;
        }
    }

    Ok(())
}

fn replace_if_present(
    sel: Selection,
    attr: &str,
    editor: impl Fn(String) -> Result<String, Error>,
) -> Result<(), Error> {
    if sel.has_attr(attr) {
        let curr_attr = sel.attr(attr).unwrap().to_string();
        if curr_attr.starts_with("http://") || curr_attr.starts_with("https://") {
            // bypass externally-referenced resources
            return Ok(());
        }
        sel.set_attr(attr, &editor(sel.attr(attr).unwrap().to_string())?)
    }

    Ok(())
}
