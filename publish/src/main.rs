use anyhow::{anyhow, Context, Result};
use atrium_api::{
    agent::{store::MemorySessionStore, AtpAgent},
    com::atproto::repo::create_record,
};

use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use bsky_sdk::api::{types::string::AtIdentifier, xrpc::XrpcClient};
use envconfig::Envconfig;
use http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
mod lexicon;

#[derive(Envconfig)]
struct LoginData {
    #[envconfig(from = "PUBLISH_USERNAME")]
    username: String,

    #[envconfig(from = "PUBLISH_PASSWORD")]
    password: String,

    #[envconfig(from = "PUBLISH_PDS", default = "https://bsky.app")]
    pds: String,
}

struct IdentityData {
    did: AtIdentifier,
    client: ReqwestClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    let ld = match LoginData::init_from_env() {
        Ok(ld) => ld,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(42);
        }
    };

    let c = login(ld).await?;

    let page = lexicon::Page {
        content: "meme".to_string(),
        embeds: None,
    };

    let request = &lexicon::post_page(lexicon::PageData { page, id: c.did });

    Ok(c.client
        .send_xrpc::<(), lexicon::InputData, create_record::Output, create_record::Error>(request)
        .await
        .map(|_| ())
        .with_context(|| "Can't write webpage to PDS")?)
}

/// Returns a logged-in ReqwestClient that can be used to perform POST requests.
async fn login(ld: LoginData) -> Result<IdentityData> {
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
        client: c,
    })
}
