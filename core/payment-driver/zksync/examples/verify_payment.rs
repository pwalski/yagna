use structopt::StructOpt;
use ya_zksync_driver::zksync::wallet;
use std::str::FromStr;
use ya_payment_driver::db::models::Network;

#[derive(Clone, Debug, StructOpt)]
struct Args {
    #[structopt(short, long, default_value = "d061e06bb9805818d2ec69068f00363337f27f9c3f0b0c627dfd2ea3fc87600a")]
    tx_hash: String,
    #[structopt(short, long, default_value = "rinkeby")]
    network: String,
}


#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_owned());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();

    let args: Args = Args::from_args();
    log::info!("Calling verify_tx. tx_hash={}, network={}", &args.tx_hash, &args.network);

    let network = Network::from_str(&args.network).unwrap();

    let result = wallet::verify_tx(&args.tx_hash, network).await?;

    log::info!("result = {:?}", result);

    Ok(())
}
