mod atproto;
use std::collections::HashMap;

use atproto::parse_at_url;
use wasm_bindgen::prelude::*;
use web_sys::{Response, ResponseInit};

#[wasm_bindgen]
pub async fn resolve(event: web_sys::FetchEvent) -> web_sys::Response {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    let u = event.request().url();

    if let Some(atu) = parse_at_url(u.clone()) {
        let atu = atu.unwrap();

        let did = atproto::solve_did(atu.did)
            .await
            .expect_throw("can't solve did");
        let pds = atproto::solve_pds(did.clone())
            .await
            .expect_throw("can't find pds for did");

        let data = atproto::data(pds, did, atu.collection)
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

        log::debug!("ri: {:?}", ri);

        let r = Response::new_with_opt_str_and_init(Some(&webpage.content), &ri).unwrap();
        r
    } else {
        log::debug!("not an at url, fetching then returning: {}", u.clone());

        return atproto::get_raw_worker(u.clone(), web_sys::RequestMode::NoCors)
            .await
            .unwrap();
    }
}
