/*
 * Market API
 *
 * OpenAPI spec version: 1.0.0
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DemandEvent {
    #[serde(rename = "eventType")]
    event_type: String,
    #[serde(rename = "requestorId")]
    requestor_id: String,
    #[serde(rename = "demand")]
    demand: Option<crate::market::Proposal>,
}

impl DemandEvent {
    pub fn new(event_type: String, requestor_id: String) -> DemandEvent {
        DemandEvent {
            event_type: event_type,
            requestor_id: requestor_id,
            demand: None,
        }
    }

    pub fn set_event_type(&mut self, event_type: String) {
        self.event_type = event_type;
    }

    pub fn with_event_type(mut self, event_type: String) -> DemandEvent {
        self.event_type = event_type;
        self
    }

    pub fn event_type(&self) -> &String {
        &self.event_type
    }

    pub fn set_requestor_id(&mut self, requestor_id: String) {
        self.requestor_id = requestor_id;
    }

    pub fn with_requestor_id(mut self, requestor_id: String) -> DemandEvent {
        self.requestor_id = requestor_id;
        self
    }

    pub fn requestor_id(&self) -> &String {
        &self.requestor_id
    }

    pub fn set_demand(&mut self, demand: crate::market::Proposal) {
        self.demand = Some(demand);
    }

    pub fn with_demand(mut self, demand: crate::market::Proposal) -> DemandEvent {
        self.demand = Some(demand);
        self
    }

    pub fn demand(&self) -> Option<&crate::market::Proposal> {
        self.demand.as_ref()
    }

    pub fn reset_demand(&mut self) {
        self.demand = None;
    }
}
