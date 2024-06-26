use anyhow::bail;
use std::collections::HashSet;

use crate::market::negotiator::factory::LimitAgreementsNegotiatorConfig;
use crate::market::negotiator::{
    AgreementResult, NegotiationResult, NegotiatorComponent, ProposalView,
};

/// Negotiator that can limit number of running agreements.
pub struct MaxAgreements {
    active_agreements: HashSet<String>,
    max_agreements: u32,
}

impl MaxAgreements {
    pub fn new(config: &LimitAgreementsNegotiatorConfig) -> MaxAgreements {
        MaxAgreements {
            max_agreements: config.max_simultaneous_agreements,
            active_agreements: HashSet::new(),
        }
    }

    pub fn has_free_slot(&self) -> bool {
        self.active_agreements.len() < self.max_agreements as usize
    }
}

impl NegotiatorComponent for MaxAgreements {
    fn negotiate_step(
        &mut self,
        demand: &ProposalView,
        offer: ProposalView,
    ) -> anyhow::Result<NegotiationResult> {
        if self.has_free_slot() {
            Ok(NegotiationResult::Ready { offer })
        } else {
            log::info!(
                "'MaxAgreements' negotiator: Reject proposal [{}] due to limit.",
                demand.id,
            );
            Ok(NegotiationResult::Reject {
                message: format!(
                    "No capacity available. Reached Agreements limit: {}",
                    self.max_agreements
                ),
                is_final: false,
            })
        }
    }

    fn on_agreement_terminated(
        &mut self,
        agreement_id: &str,
        _result: &AgreementResult,
    ) -> anyhow::Result<()> {
        self.active_agreements.remove(agreement_id);

        let free_slots = self.max_agreements as usize - self.active_agreements.len();
        log::info!("Negotiator: {} free slot(s) for agreements.", free_slots);
        Ok(())
    }

    fn on_agreement_approved(&mut self, agreement_id: &str) -> anyhow::Result<()> {
        if self.has_free_slot() {
            self.active_agreements.insert(agreement_id.to_string());
            Ok(())
        } else {
            self.active_agreements.insert(agreement_id.to_string());
            bail!(
                "Agreement [{}] approved despite not available capacity.",
                agreement_id
            )
        }
    }
}
