mod atproto;
use atproto::{get_raw, is_at_url};
use wasm_bindgen::prelude::*;

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
pub async fn resolve(event: web_sys::FetchEvent) -> web_sys::Request {
    console_error_panic_hook::set_once();
    let u = event.request().url();

    console_log!("url: {}", u.clone());

    if !is_at_url(u.clone()) {
        console_log!("not an at url, fetching then returning: {}", u.clone());
        return event.request();
    }
    console_log!("fetch event url: {}", u);

    return event.request();
}
