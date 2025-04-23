mod atproto;
use std::collections::HashMap;

use atproto::parse_at_url;
use js_sys::Uint8Array;
use shared::atproto::ATURL;
use wasm_bindgen::prelude::*;
use web_sys::{Response, ResponseInit};

static CACHED_DID_DATA: tokio::sync::OnceCell<(String, String)> =
    tokio::sync::OnceCell::const_new();

#[wasm_bindgen(start)]
pub fn init_wasm_log() {
    #[cfg(debug_assertions)]
    let wlogger_conf = wasm_logger::Config::default();

    #[cfg(not(debug_assertions))]
    let wlogger_conf = wasm_logger::Config::new(log::Level::Info);

    wasm_logger::init(wlogger_conf);
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn is_at(event: web_sys::FetchEvent) -> bool {
    let u = event.request().url();
    parse_at_url(u.clone()).is_some()
}

#[derive(Debug)]
pub enum Error {
    NotATURI(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotATURI(u) => write!(f, "not an AT URI: {}", u),
        }
    }
}

impl std::error::Error for Error {}

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        JsValue::from(self.to_string())
    }
}

#[wasm_bindgen]
pub async fn resolve(event: web_sys::FetchEvent) -> Result<web_sys::Response, Error> {
    let u = event.request().url();

    log::debug!("fetching: {}", u);

    let atu = match parse_at_url(u.clone()) {
        Some(s) => s,
        None => return Err(Error::NotATURI(u)),
    }
    .unwrap();

    let (did, pds) = did_pds(&atu).await;

        match atu.blob {
        true => blob(pds, did, atu).await,
        false => page(pds, did, atu).await,
    }
}

async fn blob(pds: String, did: String, atu: ATURL) -> Result<web_sys::Response, Error> {
    log::debug!("processing blob!");
    let data = atproto::data(pds.clone(), did.clone(), atu.key, atu.blob)
        .await
        .expect_throw("object not found");

    let ri = ResponseInit::new();
    ri.set_status(200);

    log::debug!("got blob, creating Uint8Array");

    let buffer = Uint8Array::new(&data);

    log::debug!("blob: {:?}", buffer);

    Ok(Response::new_with_opt_js_u8_array_and_init(Some(&buffer), &ri).unwrap())
}

async fn page(pds: String, did: String, atu: ATURL) -> Result<web_sys::Response, Error> {
    log::debug!("processing page!");
    let data = atproto::data(pds.clone(), did.clone(), atu.collection, atu.blob)
        .await
        .expect_throw("object not found");
    let webpage = atproto::webpage(data.clone(), atu.key)
        .await
        .expect_throw("can't find webpages");

    let mut headers = HashMap::new();
    headers.insert("Content-Type", "text/html; charset=utf-8");
    let ri = ResponseInit::new();
    ri.set_status(200);
    ri.set_headers(&serde_wasm_bindgen::to_value(&headers).unwrap());

    Ok(Response::new_with_opt_str_and_init(Some(&webpage.content), &ri).unwrap())
}

async fn did_pds(atu: &ATURL) -> (String, String) {
    CACHED_DID_DATA
        .get_or_init(async || {
            log::debug!("solving did...");
            let did = atu.did.clone();
            let did = match atu.needs_resolution {
                true => atproto::solve_did(did)
                    .await
                    .expect_throw("can't solve did"),
                false => did,
            };

            log::debug!("solving pds...");
            let pds = atproto::solve_pds(did.clone())
                .await
                .expect_throw("can't find pds for did");

            (did, pds)
        })
        .await
        .clone()
}
