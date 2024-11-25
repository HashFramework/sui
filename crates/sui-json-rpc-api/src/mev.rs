use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

use sui_json_rpc_types::SuiTransactionBlockEffects;
use sui_json_rpc_types::{
    DynamicFieldPage, EventFilter, EventPage, ObjectsPage, Page, SuiEvent, SuiObjectResponse,
    SuiObjectResponseQuery, SuiTransactionBlockResponseQuery, TransactionBlocksPage,
    TransactionFilter,
};
use sui_open_rpc_macros::open_rpc;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::digests::TransactionDigest;
use sui_types::dynamic_field::DynamicFieldName;
use sui_types::event::EventID;
use sui_json_rpc_types::{MevBundle, MevStats, MevBundleResponse};

#[open_rpc(namespace = "suix", tag = "Extended API")]
#[rpc(server, client, namespace = "suix")]
pub trait MevApiServer {
    /// Submit a MEV bundle to the network
    #[method(name = "submitMevBundle")]
    async fn submit_mev_bundle(&self, bundle: MevBundle) -> RpcResult<MevBundleResponse>;

    /// Get all MEV bundles for a specific block number
    #[method(name = "getMevBundles")]
    async fn get_mev_bundles(&self, block_number: u64) -> RpcResult<Vec<MevBundle>>;

    /// Get MEV statistics
    #[method(name = "getMevStats")]
    async fn get_mev_stats(&self) -> RpcResult<MevStats>;

    /// Get pending MEV bundles for an address
    #[method(name = "getPendingBundles")]
    async fn get_pending_bundles(&self, address: SuiAddress) -> RpcResult<Vec<MevBundle>>;
}