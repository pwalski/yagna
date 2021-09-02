/*
    Payment driver for yagna using erc20.

    This file only contains constants and imports.
*/

// Public
pub const DRIVER_NAME: &'static str = "polygon";

pub const RINKEBY_NETWORK: &'static str = "rinkeby";
pub const RINKEBY_TOKEN: &'static str = "tGLM";
pub const RINKEBY_PLATFORM: &'static str = "erc20-rinkeby-tglm";

pub const GOERLI_NETWORK: &'static str = "goerli";
pub const GOERLI_TOKEN: &'static str = "tGLM";
pub const GOERLI_PLATFORM: &'static str = "erc20-goerli-tglm";

pub const MAINNET_NETWORK: &'static str = "mainnet";
pub const MAINNET_TOKEN: &'static str = "GLM";
pub const MAINNET_PLATFORM: &'static str = "erc20-mainnet-glm";

pub use service::PolygonService as PaymentDriverService;

// Private
#[macro_use]
extern crate log;

mod dao;
mod driver;
pub mod erc20;
mod network;
mod service;