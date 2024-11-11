use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const PLC_DIRECTORY: &'static str = "https://plc.directory";
const BSKY_SOCIAL: &'static str = "https://bsky.social";
const COLLECTION: &'static str = "industries.geesawra.webpages";

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    NoDIDFound(String),
    NoPDSFound(String),
    JSError(JsValue),
    JSSerdeError(serde_wasm_bindgen::Error),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::JSError(value)
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::JSSerdeError(value)
    }
}

#[allow(dead_code)]
pub struct Webpage {
    pub date: String,
    pub title: String,
    pub content: String,
}

fn plc_url(did: String) -> String {
    format!("{}/{}", PLC_DIRECTORY, did)
}

fn bsky_url(method: String) -> String {
    format!("{}/{}", xrpc_url(BSKY_SOCIAL), method)
}

fn xrpc_url(base: &str) -> String {
    format!("{}/xrpc", base)
}

fn pds_url(pds: String, method: String) -> String {
    format!("{}/{}", xrpc_url(&pds), method)
}

fn url(base: String, args: &[(String, String)]) -> String {
    if args.is_empty() {
        return base;
    }

    let args = args
        .into_iter()
        .map(|e| format!("{}={}&", e.0, e.1))
        .collect::<String>();

    format!("{}?{}", base, args)
}

pub fn is_at_url(u: String) -> bool {
    u.starts_with("at://")
}

pub async fn solve_did(handle: String) -> Result<String, Error> {
    let params = [("handle".to_string(), handle.clone())];
    let u = url(
        bsky_url("com.atproto.identity.resolveHandle".to_string()),
        &params,
    );

    let data = get(u).await?;

    let resp: HashMap<String, String> = serde_wasm_bindgen::from_value(data)?;

    if let Some(did) = resp.get("did") {
        Ok(did.clone())
    } else {
        Err(Error::NoDIDFound(handle))
    }
}

pub async fn solve_pds(did: String) -> Result<String, Error> {
    let data = get(plc_url(did.clone())).await?;

    let resp: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

    Ok(
        match resp
            .get("service")
            .and_then(|e| e.get(0))
            .and_then(|e| e.get("serviceEndpoint"))
            .and_then(|e| e.as_str())
        {
            Some(s) => s.to_string(),
            None => return Err(Error::NoPDSFound(did)),
        },
    )
}

pub async fn webpage(pds: String, did: String) -> Result<Webpage, Error> {
    let u = url(
        pds_url(pds, "com.atproto.repo.listRecords".to_string()),
        &[
            ("repo".to_string(), did.clone()),
            ("collection".to_string(), COLLECTION.to_string()),
        ],
    );

    let data = get(u).await?;

    let resp: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

    let page = resp
        .get("records")
        .and_then(|e| e.get(0)) // grab only the first record ever
        .and_then(|e| e.get("value"))
        .and_then(|e| e.get("record"))
        .unwrap(); // from now on we have the real record

    let content = page
        .get("content")
        .and_then(|e| e.as_str())
        .unwrap()
        .to_string();
    let title = page
        .get("title")
        .and_then(|e| e.as_str())
        .unwrap()
        .to_string();
    let date = page
        .get("date")
        .and_then(|e| e.as_str())
        .unwrap()
        .to_string();

    Ok(Webpage {
        date,
        title,
        content,
    })
}

#[wasm_bindgen]
pub async fn get(url: String) -> Result<JsValue, JsValue> {
    let resp: Response = get_raw(url).await?.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}

#[wasm_bindgen]
pub async fn get_raw(url: String) -> Result<Response, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request.headers().set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    Ok(resp)
}
