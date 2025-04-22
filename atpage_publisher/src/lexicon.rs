// Ok so, buyers beware!
// Since my desire for code generation is greater than my self-love, I hacked around Atrium's code lexgen
// tool to generate Rust code starting from my Page lexicon.
//
// It kinda worked, but it also assumes you're generating code to be included in the atrium repo itself,
// so I had to copy-paste several things in order to make it work.
//
// Now that the skeleton is here, we can change it if needed!
//
// Cool right?
use bsky_sdk::api::types::{self, string::AtIdentifier, Collection};

const CREATE_RECORD_NDIS: &str = "com.atproto.repo.createRecord";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum KnownRecord {
    #[serde(rename = "industries.geesawra.website")]
    IndustriesGeesawraWebsitePage(Box<Record>),
}
impl From<Record> for KnownRecord {
    fn from(record: Record) -> Self {
        KnownRecord::IndustriesGeesawraWebsitePage(Box::new(record))
    }
}
impl From<Page> for KnownRecord {
    fn from(record_data: Page) -> Self {
        KnownRecord::IndustriesGeesawraWebsitePage(Box::new(record_data.into()))
    }
}

impl Collection for Page {
    const NSID: &'static str = "industries.geesawra.website";
    type Record = Record;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub embeds: core::option::Option<Vec<types::BlobRef>>,
}
pub type Record = types::Object<Page>;
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputData {
    ///The NSID of the record collection.
    pub collection: types::string::Nsid,
    ///The record itself. Must contain a $type field.
    pub record: KnownRecord,
    ///The handle or DID of the repo (aka, current account).
    pub repo: types::string::AtIdentifier,
    ///The Record Key.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub rkey: core::option::Option<String>,
    ///Compare and swap with the previous commit by CID.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub swap_commit: core::option::Option<types::string::Cid>,
    ///Can be set to 'false' to skip Lexicon schema validation of record data, 'true' to require it, or leave unset to validate only for known Lexicons.
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub validate: core::option::Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PageData {
    pub page: Page,
    pub id: AtIdentifier,
    pub rkey: Option<String>,
}

impl From<PageData> for InputData {
    fn from(value: PageData) -> Self {
        InputData {
            collection: Page::nsid(),
            record: value.page.into(),
            repo: value.id,
            rkey: Some(value.rkey.unwrap_or(tsid::create_tsid().to_string())),
            swap_commit: None,
            validate: None,
        }
    }
}

pub fn post_page(page: PageData) -> atrium_xrpc::XrpcRequest<(), InputData> {
    atrium_xrpc::XrpcRequest {
        method: http::Method::POST,
        nsid: CREATE_RECORD_NDIS.into(),
        parameters: None,
        input: Some(atrium_xrpc::InputDataOrBytes::Data(page.into())),
        encoding: Some(String::from("application/json")),
    }
}
