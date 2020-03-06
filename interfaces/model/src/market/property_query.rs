/*
 * Yagna Market API
 *
 *  ## Yagna Market The Yagna Market is a core component of the Yagna Network, which enables computational Offers and Demands circulation. The Market is open for all entities willing to buy computations (Demands) or monetize computational resources (Offers). ## Yagna Market API The Yagna Market API is the entry to the Yagna Market through which Requestors and Providers can publish their Demands and Offers respectively, find matching counterparty, conduct negotiations and make an agreement.  This version of Market API conforms with capability level 1 of the <a href=\"https://docs.google.com/document/d/1Zny_vfgWV-hcsKS7P-Kdr3Fb0dwfl-6T_cYKVQ9mkNg\"> Market API specification</a>.  Market API contains two roles: Requestors and Providers which are symmetrical most of the time (excluding agreement phase).
 *
 * The version of the OpenAPI document: 1.4.2
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyQuery {
    #[serde(rename = "issuerProperties", skip_serializing_if = "Option::is_none")]
    pub issuer_properties: Option<serde_json::Value>,
    #[serde(rename = "queryId", skip_serializing_if = "Option::is_none")]
    pub query_id: Option<String>,
    #[serde(rename = "queriedProperties", skip_serializing_if = "Option::is_none")]
    pub queried_properties: Option<Vec<String>>,
}

impl PropertyQuery {
    pub fn new() -> PropertyQuery {
        PropertyQuery {
            issuer_properties: None,
            query_id: None,
            queried_properties: None,
        }
    }
}