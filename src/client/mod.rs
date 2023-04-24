use crate::consts::BHTAFEL_BASE_URL;

mod query;

pub struct BahnClient {
    client: reqwest::Client,
    bhtafel_url: String,
}

impl Default for BahnClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            bhtafel_url: BHTAFEL_BASE_URL.to_owned(),
        }
    }
}

pub(crate) trait ToApiType {
    type ApiType;

    fn to_api_type(&self) -> Self::ApiType;
}
