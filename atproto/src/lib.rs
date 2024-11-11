pub mod atproto;
use js_sys::global;
use wasm_bindgen::prelude::*;
use web_sys::{ServiceWorker, ServiceWorkerGlobalScope};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub async fn on_load() {
    console_error_panic_hook::set_once();
    load_page_content().await
}

pub async fn load_page_content() {
    let did = atproto::solve_did("geesawra.industries".to_string())
        .await
        .expect_throw("can't solve did");
    let pds = atproto::solve_pds(did.clone())
        .await
        .expect_throw("can't find pds for did");

    let webpage = atproto::webpage(pds, did)
        .await
        .expect_throw("can't find webpages");

    let window = web_sys::window().expect_throw("no global `window` exists");
    let document = window
        .document()
        .expect_throw("should have a document on window");
    document
        .document_element()
        .unwrap()
        .set_inner_html(&webpage.content);
    document.set_title(&webpage.title)
}
