use anyhow::{Context, Result};
use envconfig::Envconfig;
use html::{scan_html, walk_html};
use std::{path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

mod atproto;
mod html;
mod lexicon;

#[derive(Envconfig)]
struct LoginCredential {
    #[envconfig(from = "PUBLISH_USERNAME")]
    username: String,

    #[envconfig(from = "PUBLISH_PASSWORD")]
    password: String,

    #[envconfig(from = "PUBLISH_PDS", default = "https://bsky.app")]
    pds: String,

    #[envconfig(from = "PUBLISH_SRC")]
    src: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let ld = match LoginCredential::init_from_env() {
        Ok(ld) => ld,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(42);
        }
    };

    let content_dir = PathBuf::from_str(&ld.src).unwrap();

    let c = Arc::new(Mutex::new(
        atproto::IdentityData::login(ld.username.clone(), ld.password.clone(), ld.pds.clone())
            .await?,
    ));

    for f in walk_html(content_dir.clone())? {
        let refs = Arc::new(Mutex::new(vec![]));
        println!("Processing {:?}", f);

        let page_content = scan_html(f, |src| {
            let c = c.clone();
            let refs = refs.clone();
            let content_dir = content_dir.clone();

            let blob_ref = futures::executor::block_on(async move {
                let mut content_dir = content_dir.clone();
                content_dir.push(src.clone());
                let blob_path = content_dir;

                let blob_content = std::fs::read(blob_path.clone())
                    .with_context(|| format!("cannot open {:?}", blob_path.clone()))?;

                let c = c.lock().await;

                let (blob, blob_ref) = c.upload_blob(blob_content).await?;

                println!("Uploading {:?} to blob ref {}", blob_path, blob_ref);

                refs.lock().await.push(blob);

                Ok(c.format_blob_uri(blob_ref))
            });

            blob_ref
        })?;

        let page = lexicon::Page {
            content: page_content,
            embeds: Some(refs.lock().await.clone()),
        };

        let res = c.lock().await.upload_page(page).await?;

        println!("Created new ATPage at uri {}", res.uri);
    }
    Ok(())
}
