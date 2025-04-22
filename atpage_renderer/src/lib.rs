mod atproto;
use std::collections::HashMap;

use atproto::parse_at_url;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{Response, ResponseInit};

static CACHED_DID_DATA: tokio::sync::OnceCell<(String, String)> =
    tokio::sync::OnceCell::const_new();

#[wasm_bindgen]
pub fn init_wasm_log() {
    #[cfg(debug_assertions)]
    let wlogger_conf = wasm_logger::Config::default();

    #[cfg(not(debug_assertions))]
    let wlogger_conf = wasm_logger::Config::new(log::Level::Info);

    wasm_logger::init(wlogger_conf);
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn resolve(event: web_sys::FetchEvent) -> web_sys::Response {
    let u = event.request().url();

    log::debug!("fetching: {}", u);

    if let Some(atu) = parse_at_url(u.clone()) {
        let atu = atu.unwrap();

        let (did, pds) = CACHED_DID_DATA
            .get_or_init(async || {
                log::debug!("solving did...");
                let did = match atu.needs_resolution {
                    true => atproto::solve_did(atu.did)
                        .await
                        .expect_throw("can't solve did"),
                    false => atu.did,
                };

                log::debug!("solving pds...");
                let pds = atproto::solve_pds(did.clone())
                    .await
                    .expect_throw("can't find pds for did");

                (did, pds)
            })
            .await;

        let r = match atu.blob {
            true => {
                log::debug!("processing blob!");
                let data = atproto::data(pds.clone(), did.clone(), atu.key, atu.blob)
                    .await
                    .expect_throw("object not found");

                let ri = ResponseInit::new();
                ri.set_status(200);

                log::debug!("got blob, creating Uint8Array");

                let buffer = Uint8Array::new(&data);

                log::debug!("blob: {:?}", buffer);

                Response::new_with_opt_js_u8_array_and_init(Some(&buffer), &ri).unwrap()
            }
            false => {
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

                Response::new_with_opt_str_and_init(Some(&webpage.content), &ri).unwrap()
            }
        };

        r
    } else {
        log::debug!("not an at url, fetching then returning: {}", u.clone());

        return atproto::get_raw_worker(u.clone(), web_sys::RequestMode::NoCors)
            .await
            .unwrap();
    }
}
