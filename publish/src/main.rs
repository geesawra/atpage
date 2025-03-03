use anyhow::{anyhow, Context, Result};
use atrium_api::types::BlobRef;
use clap::Parser;
use html::{page_title, scan_html, scan_html_path, walk_html};
use shared::cli;
use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

mod atproto;
mod html;
mod lexicon;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    match cli::Command::parse() {
        cli::Command::Post { login_data, src } => post(login_data, src).await,
        cli::Command::Nuke(login_data) => nuke(login_data).await,
    }
}

async fn nuke(ld: cli::LoginData) -> Result<()> {
    let c = atproto::IdentityData::login(ld.username.clone(), ld.password.clone(), ld.pds.clone())
        .await?;

    c.nuke().await
}

async fn post(ld: cli::LoginData, src: String) -> Result<()> {
    let content_dir = PathBuf::from_str(&src.clone()).unwrap();

    let c = Arc::new(Mutex::new(
        atproto::IdentityData::login(ld.username.clone(), ld.password.clone(), ld.pds.clone())
            .await?,
    ));

    let pages = Arc::new(Mutex::new(HashMap::new()));
    let dedup = Arc::new(Mutex::new(HashMap::<String, (BlobRef, String)>::new()));

    // step 1: upload blobs as they appear alongside raw pages
    for f in walk_html(content_dir.clone())? {
        let refs = Arc::new(Mutex::new(vec![]));
        log::debug!("Processing blobs for page {:?}", f);

        let pages = pages.clone();

        let content = std::fs::read(f.clone())?;
        let content = String::from_utf8(content)?;

        let page_title = match page_title(content.clone()) {
            Some(title) => title,
            None => {
                return Err(anyhow!(
                    "found page {:?} without title, need one to create an atpage!",
                    f
                ));
            }
        };

        let page_content = scan_html(content.clone(), |src, is_a| {
            if is_a {
                // ignore <a> at this point
                return Ok(None);
            }

            let c = c.clone();
            let refs = refs.clone();
            let dedup = dedup.clone();
            let content_dir = content_dir.clone();

            futures::executor::block_on(async move {
                if let Some(blob) = dedup.lock().await.get(&src.clone()) {
                    let (blob, blob_ref) = blob;
                    refs.lock().await.push(blob.clone());
                    return Ok(Some(c.lock().await.format_blob_uri(blob_ref.clone())));
                }

                let mut src_path = PathBuf::from_str(&src.clone()).unwrap();
                if src_path.is_absolute() {
                    src_path = src_path.strip_prefix("/").unwrap().to_path_buf();
                }

                let blob_path = content_dir.join(src_path.clone());

                let blob_content = std::fs::read(blob_path.clone())
                    .with_context(|| format!("cannot open {:?}", blob_path.clone()))?;

                let c = c.lock().await;

                // TODO(gsora): check if a given blob_content is already on the PDS?
                let (blob, blob_ref) = c.upload_blob(blob_content).await?;

                log::debug!("Uploading {:?} to blob ref {}", blob_path, blob_ref);

                refs.lock().await.push(blob.clone());
                dedup
                    .lock()
                    .await
                    .insert(src.clone(), (blob, blob_ref.clone()));

                Ok(Some(c.format_blob_uri(blob_ref)))
            })
        })?;

        let page = lexicon::Page {
            title: page_title,
            content: page_content,
            embeds: Some(refs.lock().await.clone()),
        };

        let page_data = c.lock().await.generate_page_data(page);

        let stripped_path = to_html_path(f, content_dir.clone())?;

        pages.lock().await.insert(stripped_path, page_data.clone());
    }

    // step 2: overwrite <a> tags

    let mut index_address = String::new();

    for f in walk_html(content_dir.clone())? {
        log::debug!("Processing page upload: {}", f.display());

        let stripped_path = to_html_path(f.clone(), content_dir.clone())?;

        let page_data = {
            let maybe_page = pages.lock().await;
            match maybe_page.get(&stripped_path) {
                Some(p) => Some(p.clone()),
                None => None,
            }
        };

        let page_data = match page_data {
            Some(p) => p,
            None => continue,
        };

        let maybe_page_content = scan_html(page_data.page.content.clone(), |attr, is_a| {
            if !is_a {
                return Ok(None);
            }

            let c = c.clone();
            let pages = pages.clone();

            futures::executor::block_on(async move {
                if let Some(page) = pages.lock().await.get(&PathBuf::from_str(&attr).unwrap()) {
                    return Ok(Some(
                        c.lock().await.format_record_uri(page.rkey.clone().unwrap()),
                    ));
                };

                Ok(None)
            })
        });

        let page_content = maybe_page_content?;

        let new_page_data = lexicon::PageData {
            page: lexicon::Page {
                title: page_data.page.title.clone(),
                content: page_content,
                embeds: page_data.page.embeds.clone(),
            },
            id: page_data.id,
            rkey: page_data.rkey,
        };

        let res = c.lock().await.upload_page(new_page_data.clone()).await?;

        log::debug!(
            "Created new ATPage at uri {} with rkey {}",
            res.uri,
            new_page_data.rkey.unwrap(),
        );

        if f.ends_with("index.html") {
            index_address = res.uri.to_string();
        }
    }

    // not using log here, needs to be picked up by caller process;
    println!("{index_address}");

    Ok(())
}

fn to_html_path(p: PathBuf, fs_base: PathBuf) -> Result<PathBuf> {
    let stripped_path = p.strip_prefix(fs_base);
    let stripped_path = PathBuf::from_str("/")
        .unwrap()
        .join(PathBuf::from(stripped_path?));

    Ok(stripped_path)
}
