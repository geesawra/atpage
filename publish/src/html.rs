use dom_query::Selection;
use std::{path::PathBuf, string::FromUtf8Error};
use thiserror::{self, Error};

const EDITABLE_ATTRS: [&'static str; 2] = ["href", "src"];

type EditRet = Result<Option<String>, Error>;

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

/// scan_html_path calls scan_html over the content of f.
pub fn scan_html_path(
    f: PathBuf,
    editor: impl Fn(String, bool) -> EditRet,
) -> Result<String, Error> {
    let content = std::fs::read(f)?;
    let content = String::from_utf8(content)?;

    scan_html(content, editor)
}

/// scan_html scans the HTML contained in data, and runs editor on the content of the tree.
/// editor implementors will receive the content of either an src or href tag attribute, and
/// a boolean that's true if the attribute is on an <a> tag.
pub fn scan_html(data: String, editor: impl Fn(String, bool) -> EditRet) -> Result<String, Error> {
    let doc = dom_query::Document::from(data);
    let body = doc.select("body");
    let head = doc.select("head");

    walk_tree(body.clone(), &editor)?;
    walk_tree(head.clone(), &editor)?;

    Ok(doc.html().to_string())
}

/// page_title returns the HTML title extracted from <title> tags.
pub fn page_title(data: String) -> Option<String> {
    let doc = dom_query::Document::from(data);
    let title = doc.select("head").select("title").inner_html().to_string();

    match title.len() {
        0 => None,
        _ => Some(title),
    }
}

fn walk_tree(sel: Selection, editor: &impl Fn(String, bool) -> EditRet) -> Result<(), Error> {
    for child in sel.children().iter() {
        let deeper_child = child.children();
        if !deeper_child.is_empty() {
            for dc in deeper_child.iter() {
                walk_tree(dc.clone(), editor)?;
            }
            continue;
        }

        for attr in EDITABLE_ATTRS {
            replace_if_present(child.clone(), attr, editor)?;
        }
    }

    for attr in EDITABLE_ATTRS {
        replace_if_present(sel.clone(), attr, editor)?;
    }

    Ok(())
}

fn replace_if_present(
    sel: Selection,
    attr: &str,
    editor: impl Fn(String, bool) -> EditRet,
) -> Result<(), Error> {
    if sel.has_attr(attr) {
        let curr_attr = sel.attr(attr).unwrap().to_string();
        if curr_attr.starts_with("http://") || curr_attr.starts_with("https://") {
            // bypass externally-referenced resources
            return Ok(());
        }

        let is_a = sel.is("a");

        if let Some(new_attr) = editor(sel.attr(attr).unwrap().to_string(), is_a)? {
            sel.set_attr(attr, &new_attr)
        }
    }

    Ok(())
}
