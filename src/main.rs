use blueprint_sdk::alloy::network::EthereumWallet;
use blueprint_sdk::alloy::primitives::address;
use blueprint_sdk::alloy::signers::local::PrivateKeySigner;
use blueprint_sdk::contexts::keystore::KeystoreContext;
use blueprint_sdk::crypto::sp_core::SpEcdsa;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::logging;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::stores::local_database::LocalDatabase;
use blueprint_sdk::utils::evm::get_wallet_provider_http;
use color_eyre::Result;
use layerzero_dvn_blueprint_template as blueprint;
use std::sync::Arc;

#[blueprint_sdk::main(env)]
async fn main() -> Result<()> {
    let data_dir = match env.data_dir.clone() {
        Some(dir) => dir,
        None => {
            logging::warn!("Data dir not specified, using default");
            blueprint::default_data_dir()
        }
    };

    let context = blueprint::DvnContext::new(env.clone(), data_dir);

    let keystore = env.keystore();
    let ecdsa_pub = keystore.first_local::<SpEcdsa>()?;
    let pair = keystore.get_secret::<SpEcdsa>(&ecdsa_pub)?;
    let signer = TanglePairSigner::new(pair.0);

    let wallet = EthereumWallet::from(signer.alloy_key()?);
    let provider = get_wallet_provider_http(&env.http_rpc_endpoint, wallet.clone());

    let endpoint_instance = blueprint::ILayerZeroEndpointV2::ILayerZeroEndpointV2Instance::new(
        address!("0000000000000000000000000000000000000000"), // TODO
        provider.clone(),
    );
    let store_packet = blueprint::StorePacketEventHandler::new(endpoint_instance, context.clone());

    let send_uln302_instance = blueprint::SendUln302::SendUln302Instance::new(
        address!("0000000000000000000000000000000000000000"), // TODO
        provider,
    );
    let process_packet = blueprint::ProcessPacketEventHandler::new(send_uln302_instance, context);

    logging::info!("Starting the event watcher ...");
    BlueprintRunner::new((), env)
        .job(store_packet)
        .job(process_packet)
        .run()
        .await?;

    logging::info!("Exiting...");
    Ok(())
}
