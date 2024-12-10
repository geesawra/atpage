use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use atrium_api::{
    agent::{store::MemorySessionStore, AtpAgent},
    com::atproto::repo::create_record,
};

use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use bsky_sdk::api::{types::string::AtIdentifier, xrpc::XrpcClient};
use envconfig::Envconfig;
use html::{scan_html, walk_html};
use http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
use tokio::sync::Mutex;
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

struct IdentityData {
    did: AtIdentifier,
    handle: AtIdentifier,
    client: ReqwestClient,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
}

impl IdentityData {
    /// Returns a logged-in ReqwestClient that can be used to perform POST requests.
    async fn login(ld: LoginCredential) -> Result<Self> {
        let c = ReqwestClientBuilder::new(ld.pds.clone()).build();

        let agent = AtpAgent::new(c.clone(), MemorySessionStore::default());

        let session = agent
            .login(ld.username, ld.password)
            .await
            .with_context(|| "Can't login with provided credentials")?;

        let mut headers = HeaderMap::new();

        let access_str = format!("Bearer {}", session.access_jwt.clone());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(access_str.as_str())
                .with_context(|| "Authorization token provided by PDS is not a valid string")?,
        );

        let rc = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .with_context(|| {
                "Can't construct HTTP client with the Authorization headers built after login"
            })?;

        let c = ReqwestClientBuilder::new(ld.pds.clone()).client(rc).build();

        Ok(IdentityData {
            did: AtIdentifier::Did(session.did.clone()),
            handle: AtIdentifier::Handle(session.handle.clone()),
            client: c,
            agent,
        })
    }
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

    let c = Arc::new(Mutex::new(IdentityData::login(ld).await?));

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

                let res = c
                    .agent
                    .api
                    .com
                    .atproto
                    .repo
                    .upload_blob(blob_content) // TODO(gsora): actually pass blob
                    .await
                    .unwrap();

                let blob_ref = match res.blob.clone() {
                    atrium_api::types::BlobRef::Typed(t) => {
                        let r = match t {
                            atrium_api::types::TypedBlobRef::Blob(b) => b,
                        };

                        let cid = r.r#ref.0;
                        cid.to_string_of_base(multibase::Base::Base32Lower).unwrap()
                    }
                    atrium_api::types::BlobRef::Untyped(u) => u.cid,
                };

                println!("Uploading {:?} to blob ref {}", blob_path, blob_ref);

                refs.lock().await.push(res.blob.clone());

                Ok(format_blob_uri(blob_ref, c.handle.clone()))
            });

            blob_ref
        })?;

        let page = lexicon::Page {
            content: page_content,
            embeds: Some(refs.lock().await.clone()),
        };

        let request = &lexicon::post_page(lexicon::PageData {
            page,
            id: c.lock().await.did.clone(),
        });

        let res = c
            .lock()
            .await
            .client
            .send_xrpc::<(), lexicon::InputData, create_record::Output, create_record::Error>(
                request,
            )
            .await
            .with_context(|| "Can't write webpage to PDS")?;

        match res {
            atrium_xrpc::OutputDataOrBytes::Data(data) => {
                println!("Created new ATPage at uri {}", data.uri);
            }
            atrium_xrpc::OutputDataOrBytes::Bytes(_) => {
                unimplemented!("cannot have bytes response here!")
            }
        };
    }
    Ok(())
}

fn format_blob_uri(blob: String, did: AtIdentifier) -> String {
    let did = match did {
        AtIdentifier::Did(d) => d.to_string(),
        AtIdentifier::Handle(h) => h.to_string(),
    };

    format!("/at/{}/blobs/{}", did, blob)
}
