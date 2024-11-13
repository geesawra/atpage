use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Url, WorkerGlobalScope};

const PLC_DIRECTORY: &'static str = "https://plc.directory";
const BSKY_SOCIAL: &'static str = "https://bsky.social";

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    NoDIDFound(String),
    NoPDSFound(String),
    JSError(JsValue),
    JSSerdeError(serde_wasm_bindgen::Error),
    MalformedATURL(String),
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
    let u = Url::new(&PLC_DIRECTORY).unwrap();

    u.set_pathname(&did);

    u.href()
}

fn bsky_url(method: String) -> String {
    log::debug!("bsky url: {}", &xrpc_url(&BSKY_SOCIAL));
    let u = Url::new_with_base(&method, &xrpc_url(&BSKY_SOCIAL)).unwrap();

    u.href()
}

fn xrpc_url(base: &str) -> String {
    let u = Url::new(base).unwrap();

    u.set_pathname("/xrpc/");

    u.href()
}

fn pds_url(pds: String, method: String) -> String {
    let u = Url::new_with_base(&method, &xrpc_url(&pds)).unwrap();

    u.href()
}

fn url(base: String, args: &[(String, String)]) -> String {
    if args.is_empty() {
        return base;
    }

    let bu = Url::new(&base).unwrap();

    args.into_iter()
        .for_each(|(k, v)| bu.search_params().set(k, v));

    bu.href()
}

pub fn parse_at_url(u: String) -> Option<Result<ATURL, Error>> {
    let jsu = Url::new(&u).unwrap();

    jsu.pathname()
        .strip_prefix("/at/")
        .map(|e| TryFrom::try_from(e.to_string()))
}

pub struct ATURL {
    pub did: String,
    pub collection: String,
    pub key: String,
}

impl TryFrom<String> for ATURL {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let comp = value.split("/").collect::<Vec<&str>>();

        if comp.len() != 3 {
            return Err(Error::MalformedATURL(value));
        }

        Ok(ATURL {
            did: comp[0].to_string(),
            collection: comp[1].to_string(),
            key: comp[2].to_string(),
        })
    }
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

pub async fn data(pds: String, did: String, collection: String) -> Result<JsValue, Error> {
    let u = url(
        pds_url(pds, "com.atproto.repo.listRecords".to_string()),
        &[
            ("repo".to_string(), did.clone()),
            ("collection".to_string(), collection),
        ],
    );

    Ok(get(u).await?)
}

pub async fn webpage(data: JsValue, key: String) -> Result<Webpage, Error> {
    let resp: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

    let arr = resp.get("records").and_then(|e| e.as_array()).unwrap();

    let mut value = None;
    for e in arr {
        let e_cloned = e.clone();
        let uri = e_cloned.get("uri").unwrap();
        let uri = uri.as_str().unwrap();
        if uri.ends_with(&key) {
            value = Some(e);
            break;
        }
    }

    let page = value // grab only the first record ever
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
    let resp: Response = get_raw_worker(url, RequestMode::Cors)
        .await?
        .dyn_into()
        .unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}

#[wasm_bindgen]
pub async fn get_raw_worker(url: String, req_mode: RequestMode) -> Result<Response, JsValue> {
    use wasm_bindgen::JsCast;

    let worker = js_sys::global().dyn_into::<WorkerGlobalScope>().unwrap();

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(req_mode);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request.headers().set("Accept", "application/json")?;

    let resp_value = JsFuture::from(worker.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    Ok(resp)
}
