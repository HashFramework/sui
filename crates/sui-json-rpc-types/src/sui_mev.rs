use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sui_types::base_types::SuiAddress;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MevBundle {
    pub transactions: Vec<String>,  // Transaction bytes in base64
    pub block_number: Option<u64>,  // Target block number
    pub timestamp: u64,            // Bundle submission timestamp
    pub sender: SuiAddress,        // Bundle sender
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MevBundleResponse {
    pub bundle_hash: String,
    pub accepted: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MevStats {
    pub total_bundles: u64,
    pub accepted_bundles: u64,
    pub rejected_bundles: u64,
    pub pending_bundles: u64,
}