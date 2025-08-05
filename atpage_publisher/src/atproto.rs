use anyhow::{anyhow, Context, Result};
use atrium_api::{
    agent::atp_agent::{store::MemorySessionStore, AtpAgent},
    com::{
        self,
        atproto::repo::{create_record, delete_record, list_records},
    },
    types::{
        string::{AtIdentifier, RecordKey},
        BlobRef, Collection,
    },
};
use atrium_xrpc::XrpcClient;
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
use shared::atproto::ATURL;

use crate::lexicon;
pub(crate) struct IdentityData {
    pub did: AtIdentifier,
    pub handle: AtIdentifier,
    client: ReqwestClient,
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
}

impl IdentityData {
    pub fn format_blob_uri(&self, blob: String) -> String {
        let did = match self.handle.clone() {
            AtIdentifier::Did(d) => d.to_string(),
            AtIdentifier::Handle(h) => h.to_string(),
        };
        format!("/at/{}/blobs/{}", did.to_string(), blob)
    }

    pub fn format_record_uri(&self, rkey: String) -> String {
        let did = match self.handle.clone() {
            AtIdentifier::Did(d) => d.to_string(),
            AtIdentifier::Handle(h) => h.to_string(),
        };
        format!(
            "/at/{}/industries.geesawra.website/{}",
            did.to_string(),
            rkey
        )
    }

    pub fn generate_page_data(&self, page: lexicon::Page) -> lexicon::PageData {
        let rkey = sha256::digest(page.title.clone());
        lexicon::PageData {
            page,
            id: self.did.clone(),
            rkey: Some(rkey),
        }
    }

    pub async fn upload_page(&self, page_data: lexicon::PageData) -> Result<create_record::Output> {
        let request = &lexicon::post_page(page_data);

        let res = self
            .client
            .send_xrpc::<(), lexicon::InputData, create_record::Output, create_record::Error>(
                &request,
            )
            .await
            .with_context(|| "Can't write webpage to PDS")?;

        match res {
            atrium_xrpc::OutputDataOrBytes::Data(data) => {
                // println!("Created new ATPage at uri {}", data.uri);
                Ok(data)
            }
            atrium_xrpc::OutputDataOrBytes::Bytes(_) => {
                Err(anyhow!("received bytes from post_page call, impossible!"))
            }
        }
    }

    pub async fn nuke(&self) -> Result<Vec<String>> {
        let mut deleted = vec![];
        loop {
            let records = self
                .agent
                .api
                .com
                .atproto
                .repo
                .list_records(
                    list_records::ParametersData {
                        collection: lexicon::Page::nsid(),
                        cursor: None,
                        limit: None,
                        repo: self.did.clone(),
                        reverse: None,
                    }
                    .into(),
                )
                .await?;

            if records.records.is_empty() {
                break;
            }

            for r in records.records.iter() {
                let ru: ATURL = r.uri.clone().try_into()?;

                self.agent
                    .api
                    .com
                    .atproto
                    .repo
                    .delete_record(
                        delete_record::InputData {
                            collection: lexicon::Page::nsid(),
                            repo: self.did.clone(),
                            rkey: RecordKey::new(ru.key.clone()).unwrap(),
                            swap_commit: None,
                            swap_record: None,
                        }
                        .into(),
                    )
                    .await?;

                deleted.push(r.uri.clone());
            }
        }

        Ok(deleted)
    }

    pub async fn upload_blob(
        &self,
        data: Vec<u8>,
        ext: Option<String>,
    ) -> Result<(BlobRef, String)> {
        let mt = match infer::get(&data) {
            Some(mt) => Some(mt.mime_type().to_owned()),
            None => {
                let m = mime_guess::from_ext(&ext.unwrap_or_default());
                Some(m.first_or_text_plain().to_string())
            }
        };

        let res = self.upload_blob_raw(data, mt).await?;

        match res.blob.clone() {
            atrium_api::types::BlobRef::Typed(t) => {
                let r = match t {
                    atrium_api::types::TypedBlobRef::Blob(b) => b,
                };

                let cid = r.r#ref.0;
                Ok((
                    res.blob.clone(),
                    cid.to_string_of_base(multibase::Base::Base32Lower)?,
                ))
            }
            atrium_api::types::BlobRef::Untyped(u) => Ok((res.blob.clone(), u.cid)),
        }
    }

    /// Returns a logged-in ReqwestClient that can be used to perform POST requests.
    pub async fn login(username: String, password: String, pds: String) -> Result<Self> {
        let c = ReqwestClientBuilder::new(pds.clone()).build();

        let agent = AtpAgent::new(c.clone(), MemorySessionStore::default());

        let session = agent
            .login(username, password)
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

        let c = ReqwestClientBuilder::new(pds).client(rc).build();

        Ok(IdentityData {
            did: AtIdentifier::Did(session.did.clone()),
            handle: AtIdentifier::Handle(session.handle.clone()),
            client: c,
            agent,
        })
    }

    pub async fn upload_blob_raw(
        &self,
        input: Vec<u8>,
        mime_type: Option<String>,
    ) -> atrium_xrpc::Result<
        com::atproto::repo::upload_blob::Output,
        com::atproto::repo::upload_blob::Error,
    > {
        let response = self
            .client
            .send_xrpc::<(), Vec<u8>, _, _>(&atrium_xrpc::XrpcRequest {
                method: http::Method::POST,
                nsid: com::atproto::repo::upload_blob::NSID.into(),
                parameters: None,
                input: Some(atrium_xrpc::InputDataOrBytes::Bytes(input)),
                encoding: Some(mime_type.unwrap_or("*/*".to_owned())),
            })
            .await?;
        match response {
            atrium_xrpc::OutputDataOrBytes::Data(data) => Ok(data),
            _ => Err(atrium_xrpc::Error::UnexpectedResponseType),
        }
    }
}
