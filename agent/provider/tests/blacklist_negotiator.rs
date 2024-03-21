use serde_json::{json, Value};
use serial_test::serial;
use std::str::FromStr;
use test_case::test_case;

use ya_agreement_utils::agreement::expand;
use ya_agreement_utils::{OfferTemplate, ProposalView};
use ya_client_model::market::proposal::State;
use ya_client_model::NodeId;
use ya_manifest_test_utils::TestResources;
use ya_provider::market::negotiator::builtin::blacklist::Blacklist;
use ya_provider::market::negotiator::{NegotiationResult, NegotiatorComponent};
use ya_provider::provider_agent::AgentNegotiatorsConfig;
use ya_provider::rules::RulesManager;

static MANIFEST_TEST_RESOURCES: TestResources = TestResources {
    temp_dir: env!("CARGO_TARGET_TMPDIR"),
};

fn setup_rules_manager() -> RulesManager {
    let (_resource_cert_dir, test_cert_dir) = MANIFEST_TEST_RESOURCES.init_cert_dirs();

    let whitelist_file = test_cert_dir.join("whitelist.json");
    let rules_file_name = test_cert_dir.join("rules.json");

    RulesManager::load_or_create(&rules_file_name, &whitelist_file, &test_cert_dir)
        .expect("Can't load RulesManager")
}

fn create_demand(demand: Value) -> ProposalView {
    ProposalView {
        content: OfferTemplate {
            properties: expand(demand),
            constraints: "()".to_string(),
        },
        id: "0x0000000000000000000000000000000000000000".to_string(),
        issuer: Default::default(),
        state: State::Initial,
        timestamp: Default::default(),
    }
}

fn create_offer() -> ProposalView {
    ProposalView {
        content: OfferTemplate {
            properties: expand(serde_json::from_str(r#"{ "any": "thing" }"#).unwrap()),
            constraints: "()".to_string(),
        },
        id: "0x0000000000000000000000000000000000000000".to_string(),
        issuer: Default::default(),
        state: State::Initial,
        timestamp: Default::default(),
    }
}

fn load_node_descriptor(file: Option<&str>) -> Value {
    let (resource_cert_dir, _test_cert_dir) = MANIFEST_TEST_RESOURCES.init_cert_dirs();

    let desc = file
        .map(|node_descriptor_filename| {
            let data = std::fs::read(resource_cert_dir.join(node_descriptor_filename)).unwrap();
            serde_json::from_slice::<Value>(&data).unwrap()
        })
        .unwrap_or(Value::Null);

    json!({
        "golem": {
            "!exp": {
                "gap-31": {
                    "v0": {
                        "node": {
                            "descriptor": desc
                        }
                    }
                }

            }
        },
    })
}

fn expect_accept(result: NegotiationResult) {
    match result {
        NegotiationResult::Ready { .. } => {}
        NegotiationResult::Reject { message, .. } => {
            panic!("Expected negotiations accepted, got: {}", message)
        }
        NegotiationResult::Negotiating { .. } => {
            panic!("Expected negotiations accepted, got: Negotiating")
        }
    }
}

fn expect_reject(result: NegotiationResult, error: Option<&str>) {
    match result {
        NegotiationResult::Ready { .. } => panic!("Expected negotiations rejected, got: Ready"),
        NegotiationResult::Negotiating { .. } => {
            panic!("Expected negotiations rejected, got: Negotiating")
        }
        NegotiationResult::Reject { message, is_final } => {
            assert!(is_final);
            if let Some(expected_error) = error {
                if !message.contains(expected_error) {
                    panic!(
                        "Negotiations error message: \n {} \n doesn't contain expected message: \n {}",
                        message, expected_error
                    );
                }
            }
        }
    }
}

#[test_case(
    Some("node-descriptor-happy-path.signed.json");
    "Signed Requestors are passed"
)]
#[test_case(None; "Un-signed Requestors are passed")]
#[test_case(
    Some("node-descriptor-different-node.signed.json");
    "Incorrect NodeId signatures are ignored (passed)"
)]
#[test_case(
    Some("node-descriptor-invalid-signature.signed.json");
    "Invalid signatures are ignored (passed)"
)]
#[serial]
fn blacklist_negotiator_rule_disabled(node_descriptor: Option<&str>) {
    let rules_manager = setup_rules_manager();
    rules_manager.blacklist().disable().unwrap();
    let mut negotiator = Blacklist::new(AgentNegotiatorsConfig { rules_manager });

    let demand = create_demand(load_node_descriptor(node_descriptor));
    let offer = create_offer();

    let result = negotiator
        .negotiate_step(&demand, offer.clone())
        .expect("Negotiator shouldn't return error");
    expect_accept(result);
}

#[test_case(
    "node-descriptor-happy-path.signed.json",
    "Requestor's NodeId is on the blacklist";
    "Rejected because requestor is blacklisted"
)]
#[serial]
fn blacklist_negotiator_id_blacklisted(node_descriptor: &str, expected_err: &str) {
    let rules_manager = setup_rules_manager();
    rules_manager.blacklist().enable().unwrap();
    rules_manager
        .blacklist()
        .add_identity_rule(NodeId::from_str("0x0000000000000000000000000000000000000000").unwrap())
        .unwrap();

    let mut negotiator = Blacklist::new(AgentNegotiatorsConfig { rules_manager });

    let demand = create_demand(load_node_descriptor(Some(node_descriptor)));
    let offer = create_offer();

    let result = negotiator
        .negotiate_step(&demand, offer.clone())
        .expect("Negotiator shouldn't return error");
    expect_reject(result, Some(expected_err));
}
