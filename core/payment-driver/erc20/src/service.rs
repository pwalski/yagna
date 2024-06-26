/*
    The service that binds this payment driver into yagna via GSB.
*/

use std::sync::Arc;
use std::{env, path::PathBuf, str::FromStr};
// External crates
use erc20_payment_lib::config;
use erc20_payment_lib::config::{
    AdditionalOptions, LockContractSettings, MultiContractSettings, RpcSettings,
    WrapperContractSettings,
};
use erc20_payment_lib::runtime::{PaymentRuntime, PaymentRuntimeArgs};
use ethereum_types::H160;

// Workspace uses
use ya_payment_driver::bus;

// Local uses
use crate::{driver::Erc20Driver, signer::IdentitySigner};

pub struct Erc20Service;

impl Erc20Service {
    pub async fn gsb(path: PathBuf) -> anyhow::Result<()> {
        log::debug!("Connecting Erc20Service to gsb...");

        // TODO: Read and validate env
        log::debug!("Environment variables validated");

        // Init database

        {
            let additional_options = AdditionalOptions {
                keep_running: true,
                generate_tx_only: false,
                skip_multi_contract_check: false,
                contract_use_direct_method: true,
                contract_use_unpacked_method: false,
                use_transfer_for_single_payment: true,
                skip_service_loop: false,
            };

            let mut config = config::Config::load_from_str(include_str!("../config-payments.toml"))
                .expect("Default erc20 config doesn't parse");

            // Load config from file if it exists giving the possibility of overwriting the default config
            if tokio::fs::try_exists(&path.join("config-payments.toml"))
                .await
                .unwrap_or(false)
            {
                log::warn!(
                    "Config file found in {}",
                    &path.join("config-payments.toml").display()
                );
                log::warn!(
                    "Format of the file may change in the future releases, use with caution!"
                );

                match config::Config::load(&path.join("config-payments.toml")).await {
                    Ok(config_from_file) => {
                        log::info!("Config file loaded successfully, overwriting default config");
                        config = config_from_file;
                    }
                    Err(err) => {
                        log::error!("Config file found but failed to load from file - using default config. Error: {err}")
                    }
                }
            } else {
                log::debug!(
                    "Config file not found in {}, using default config",
                    &path.join("config-payments.toml").display()
                );
            }

            let sendout_interval_env = "ERC20_SENDOUT_INTERVAL_SECS";
            if let Ok(sendout_interval) = env::var(sendout_interval_env) {
                match sendout_interval.parse::<u64>() {
                    Ok(sendout_interval_secs) => {
                        log::info!("erc20 gather interval set to {sendout_interval_secs}s");
                        config.engine.gather_interval = sendout_interval_secs;
                    },
                    Err(e) => log::warn!("Value {sendout_interval} for {sendout_interval_env} is not a valid integer: {e}"),
                }
            }

            for (network, chain) in &mut config.chain {
                let prefix = network.to_ascii_uppercase();
                let symbol = chain.token.symbol.to_ascii_uppercase();

                let rpc_env = format!("{prefix}_GETH_ADDR");
                let priority_fee_env = format!("{prefix}_PRIORITY_FEE");
                let max_fee_per_gas_env = format!("{prefix}_MAX_FEE_PER_GAS");
                let token_addr_env = format!("{prefix}_{symbol}_CONTRACT_ADDRESS");
                let wrapper_contract_env = format!("{prefix}_WRAPPER_CONTRACT_ADDRESS");
                let multi_payment_addr_env = format!("{prefix}_MULTI_PAYMENT_CONTRACT_ADDRESS");
                let lock_payment_addr_env = format!("{prefix}_LOCK_PAYMENT_CONTRACT_ADDRESS");
                let confirmations_env = format!("ERC20_{prefix}_REQUIRED_CONFIRMATIONS");

                if let Ok(addr) = env::var(&rpc_env) {
                    chain.rpc_endpoints = addr
                        .split(',')
                        .map(|s| RpcSettings {
                            names: Some(s.to_string()),
                            endpoints: Some(s.to_string()),
                            skip_validation: None,
                            backup_level: None,
                            verify_interval_secs: None,
                            min_interval_ms: None,
                            max_timeout_ms: None,
                            allowed_head_behind_secs: None,
                            max_consecutive_errors: None,
                            dns_source: None,
                            json_source: None,
                        })
                        .collect();
                    log::info!(
                        "{} rpc endpoints set to {:?}",
                        network,
                        &chain.rpc_endpoints
                    )
                }
                if let Ok(fee) = env::var(&priority_fee_env) {
                    match rust_decimal::Decimal::from_str(&fee) {
                        Ok(fee) => {
                            log::info!("{network} priority fee set to {fee}");
                            chain.priority_fee = fee;
                        }
                        Err(e) => log::warn!(
                            "Value {fee} for {priority_fee_env} is not a valid decimal: {e}"
                        ),
                    }
                }
                if let Ok(max_fee) = env::var(&max_fee_per_gas_env) {
                    match rust_decimal::Decimal::from_str(&max_fee) {
                        Ok(max_fee) => {
                            log::info!("{network} max fee per gas set to {max_fee}");
                            chain.max_fee_per_gas = max_fee;
                        }
                        Err(e) => log::warn!(
                            "Value {max_fee} for {max_fee_per_gas_env} is not a valid decimal: {e}"
                        ),
                    }
                }
                if let Ok(addr) = env::var(&token_addr_env) {
                    match H160::from_str(&addr) {
                        Ok(parsed) => {
                            log::info!("{network} token address set to {addr}");
                            chain.token.address = parsed;
                        }
                        Err(e) => {
                            log::warn!(
                                "Value {addr} for {token_addr_env} is not valid H160 address: {e}"
                            );
                        }
                    };
                }
                if let Ok(confirmations) = env::var(&confirmations_env) {
                    match confirmations.parse::<u64>() {
                        Ok(parsed) => {
                            log::info!("{network} required confirmations set to {parsed}");
                            chain.confirmation_blocks = parsed;
                        }
                        Err(e) => {
                            log::warn!(
                                "Value {confirmations} for {confirmations} is not valid u64: {e}"
                            );
                        }
                    };
                }
                if let Ok(multi_payment_addr) = env::var(&multi_payment_addr_env) {
                    match H160::from_str(&multi_payment_addr) {
                        Ok(parsed) => {
                            log::info!(
                                "{network} multi payment contract address set to {multi_payment_addr}"
                            );
                            chain.multi_contract = Some(MultiContractSettings {
                                address: parsed,
                                max_at_once: 10,
                            })
                        }
                        Err(e) => {
                            log::warn!(
                                "Value {multi_payment_addr} for {multi_payment_addr_env} is not valid H160 address: {e}"
                            );
                        }
                    };
                }
                if let Ok(lock_payment_addr) = env::var(&lock_payment_addr_env) {
                    match H160::from_str(&lock_payment_addr) {
                        Ok(parsed) => {
                            log::info!(
                                "{network} lock payment contract address set to {lock_payment_addr}"
                            );
                            chain.lock_contract = Some(LockContractSettings { address: parsed })
                        }
                        Err(e) => {
                            log::warn!(
                                "Value {lock_payment_addr} for {lock_payment_addr_env} is not valid H160 address: {e}"
                            );
                        }
                    };
                }
                if let Ok(wrapper_contract_addr) = env::var(&wrapper_contract_env) {
                    match H160::from_str(&wrapper_contract_addr) {
                        Ok(parsed) => {
                            if parsed == H160::zero() {
                                log::info!(
                                    "{network} wrapper contract address set to {wrapper_contract_addr}, setting to None"
                                );
                                chain.wrapper_contract = None;
                            } else {
                                log::info!(
                                    "{network} wrapper contract address set to {wrapper_contract_addr}"
                                );
                                chain.wrapper_contract =
                                    Some(WrapperContractSettings { address: parsed })
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Value {wrapper_contract_addr} for {wrapper_contract_env} is not valid H160 address: {e}"
                            );
                        }
                    };
                }
            }

            log::debug!("Starting payment engine: {:#?}", config);
            let signer = IdentitySigner;

            let (sender, recv) = tokio::sync::mpsc::channel(16);

            let pr = PaymentRuntime::new(
                PaymentRuntimeArgs {
                    secret_keys: vec![],
                    db_filename: path.join("erc20.sqlite"),
                    config,
                    conn: None,
                    options: Some(additional_options),
                    mspc_sender: Some(sender),
                    broadcast_sender: None,
                    extra_testing: None,
                },
                Arc::new(Box::new(signer)),
            )
            .await?;

            //let signer = IdentitySigner;
            //pr.add_account(PaymentAccount::new(H160::from_low_u64_be(0), Box::new(signer)))
            //    .await?;

            log::debug!("Bind erc20 driver");
            let driver = Erc20Driver::new(pr, recv);
            driver.load_active_accounts().await;
            bus::bind_service(driver).await?;

            log::info!("Successfully connected Erc20Service to gsb.");
            Ok(())
        }
    }
}
