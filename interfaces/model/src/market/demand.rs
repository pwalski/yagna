/* 
 * Golem Market API
 *
 * Market API
 *
 * OpenAPI spec version: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Demand {
  #[serde(rename = "properties")]
  properties: Value,
  #[serde(rename = "constraints")]
  constraints: String
}

impl Demand {
  pub fn new(properties: Value, constraints: String) -> Demand {
    Demand {
      properties: properties,
      constraints: constraints
    }
  }

  pub fn set_properties(&mut self, properties: Value) {
    self.properties = properties;
  }

  pub fn with_properties(mut self, properties: Value) -> Demand {
    self.properties = properties;
    self
  }

  pub fn properties(&self) -> &Value {
    &self.properties
  }


  pub fn set_constraints(&mut self, constraints: String) {
    self.constraints = constraints;
  }

  pub fn with_constraints(mut self, constraints: String) -> Demand {
    self.constraints = constraints;
    self
  }

  pub fn constraints(&self) -> &String {
    &self.constraints
  }


}



