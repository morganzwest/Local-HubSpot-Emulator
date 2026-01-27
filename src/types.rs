use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PromoteSelector {
    #[serde(rename = "type")]
    pub selector_type: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    #[serde(rename = "hubspot_token")]
    pub hubspot_token: String,

    pub workflow_id: String,
    pub selector: PromoteSelector,
    pub runtime: Option<String>,
    pub source_code: String,

    #[serde(default)]
    pub force: bool,

    #[serde(default)]
    pub dry_run: bool,
}