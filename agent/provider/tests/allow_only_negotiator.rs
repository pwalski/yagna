mod utils;

use serial_test::serial;
use test_case::test_case;

use ya_provider::market::negotiator::builtin::allow_only::AllowOnly;
use ya_provider::market::negotiator::NegotiatorComponent;
use ya_provider::provider_agent::AgentNegotiatorsConfig;

use crate::utils::rules::{
    create_demand, create_offer, expect_accept, load_node_descriptor, setup_certificates_rules,
    setup_identity_rules, setup_rules_manager,
};

#[test_case(
    Some("node-descriptor-happy-path.signed.json"),
    &["partner-certificate.signed.json"],
    &[];
    "Signed Requestors on the allow-list are passed"
)]
#[test_case(
    Some("node-descriptor-happy-path.signed.json"),
    &[],
    &[];
    "Signed Requestors not on the allow-list are passed"
)]
#[test_case(
    None,
    &["partner-certificate.signed.json"],
    &[];
    "Un-signed Requestors are passed"
)]
#[test_case(
    Some("node-descriptor-happy-path.signed.json"),
    &[],
    &["0x0000000000000000000000000000000000000000"];
    "Signed Requestors with identity on the allow-list are passed"
)]
#[test_case(
    None,
    &[],
    &["0x0000000000000000000000000000000000000001"];
    "Signed Requestors with identity not on the allow-list are passed"
)]
#[test_case(
    Some("node-descriptor-different-node.signed.json"),
    &[],
    &[];
    "Mismatching NodeId is ignored (passed)"
)]
#[test_case(
    Some("node-descriptor-invalid-signature.signed.json"),
    &[],
    &[];
    "Invalid signatures are ignored (passed)"
)]
#[serial]
fn allowonly_negotiator_rule_disabled(
    node_descriptor: Option<&str>,
    allow_certs: &[&str],
    allow_ids: &[&str],
) {
    let rules_manager = setup_rules_manager();
    rules_manager.allow_only().disable().unwrap();

    setup_certificates_rules(rules_manager.allow_only(), allow_certs);
    setup_identity_rules(rules_manager.allow_only(), allow_ids);

    let mut negotiator = AllowOnly::new(AgentNegotiatorsConfig { rules_manager });
    let demand = create_demand(load_node_descriptor(node_descriptor));

    let result = negotiator
        .negotiate_step(&demand, create_offer())
        .expect("Negotiator shouldn't return error");
    expect_accept(result);
}
