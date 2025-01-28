use blueprint::DvnContext;
use blueprint_sdk::alloy::network::{Ethereum, EthereumWallet, NetworkWallet};
use blueprint_sdk::alloy::primitives::{Address, U256};
use blueprint_sdk::alloy::providers::{Provider, RootProvider};
use blueprint_sdk::alloy::sol;
use blueprint_sdk::alloy::transports::BoxTransport;
use blueprint_sdk::logging::setup_log;
use blueprint_sdk::tangle_subxt::subxt::tx::Signer;
use blueprint_sdk::testing::tempfile;
use blueprint_sdk::testing::utils::anvil::{get_receipt, start_anvil_container, Container};
use blueprint_sdk::testing::utils::harness::TestHarness;
use blueprint_sdk::testing::utils::runner::TestEnv;
use blueprint_sdk::testing::utils::tangle::TangleTestHarness;
use blueprint_sdk::tokio;
use blueprint_sdk::utils::evm::get_wallet_provider_http;
use color_eyre::Report;
use color_eyre::Result;
use layerzero_dvn_blueprint_template as blueprint;

#[tokio::test(flavor = "multi_thread")]
async fn dvn() -> Result<()> {
    setup_log();

    let tempdir = tempfile::tempdir()?;
    let temp_dir_path = tempdir.path().to_path_buf();

    let harness = TangleTestHarness::setup(tempdir).await?;

    let dvn_env = spinup_anvil_testnets(&harness).await?;

    let ctx = DvnContext::new(harness.env().clone(), temp_dir_path.clone());

    let endpoint_instance = blueprint::ILayerZeroEndpointV2::ILayerZeroEndpointV2Instance::new(
        dvn_env.contracts.endpoint_v2,
        dvn_env.provider.clone(),
    );
    let store_packet_handler =
        blueprint::StorePacketEventHandler::new(endpoint_instance, ctx.clone());

    let send_uln302_instance = blueprint::SendUln302::SendUln302Instance::new(
        dvn_env.contracts.send_uln_302,
        dvn_env.provider.clone(),
    );
    let process_packet_handler =
        blueprint::ProcessPacketEventHandler::new(send_uln302_instance, ctx.clone());

    // Setup service
    let (mut test_env, service_id) = harness.setup_services().await?;
    test_env.add_job(store_packet_handler);
    test_env.add_job(process_packet_handler);

    tokio::spawn(async move {
        test_env.run_runner().await.unwrap();
    });

    // Pass the arguments
    // TODO: Create arguments

    // Execute job and verify result
    let results = harness
        .execute_job(service_id, 0, Vec::new(), Vec::new())
        .await?;

    assert_eq!(results.service_id, service_id);

    Ok(())
}

const TESTNET1_STATE_PATH: &str = "./test_assets/testnet1-state.json";
const TESTNET2_STATE_PATH: &str = "./test_assets/testnet2-state.json";

struct DvnTestEnv {
    _origin_container: Container,
    _dest_container: Container,
    provider: RootProvider<BoxTransport>,
    contracts: ContractAddresses,
}

async fn spinup_anvil_testnets(harness: &TangleTestHarness) -> Result<DvnTestEnv> {
    let (origin_container, origin_http, _) =
        start_anvil_container(TESTNET1_STATE_PATH, false).await;
    let (provider, contracts) = deploy_endpoint_v2(&origin_http, harness).await?;

    let (dest_container, dest_http, _) = start_anvil_container(TESTNET2_STATE_PATH, false).await;
    deploy_endpoint_v2(&dest_http, harness).await?;

    Ok(DvnTestEnv {
        _origin_container: origin_container,
        _dest_container: dest_container,
        provider,
        contracts,
    })
}

struct ContractAddresses {
    endpoint_v2: Address,
    send_uln_302: Address,
    receive_uln_302: Address,
    read_lib_1002: Address,
}

sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug)]
    EndpointV2,
    "./contracts/out/EndpointV2.sol/EndpointV2.json"
);

sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug)]
    ReceiveUln302,
    "./contracts/out/ReceiveUln302.sol/ReceiveUln302.json"
);

sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    #[derive(Debug)]
    ReadLib1002,
    "./contracts/out/ReadLib1002.sol/ReadLib1002.json"
);

async fn deploy_endpoint_v2(
    http_endpoint: &str,
    harness: &TangleTestHarness,
) -> Result<(RootProvider<BoxTransport>, ContractAddresses)> {
    macro_rules! try_deploy {
        ($builder:expr) => {{
            let receipt = get_receipt($builder).await?;
            match receipt.contract_address {
                Some(address) => address,
                None => {
                    return Err(Report::msg("Contract address not found in receipt"));
                }
            }
        }};
    }

    let wallet = EthereumWallet::new(harness.alloy_key.clone());
    let signer = NetworkWallet::<Ethereum>::default_signer_address(&wallet);

    let provider = get_wallet_provider_http(http_endpoint, wallet);

    let endpoint_v2 = try_deploy!(EndpointV2::deploy_builder(&provider, 31337, signer));
    let send_uln_302 = try_deploy!(blueprint::SendUln302::deploy_builder(
        &provider,
        endpoint_v2,
        U256::from(0),
        U256::from(0),
    ));
    let receive_uln_302 = try_deploy!(ReceiveUln302::deploy_builder(&provider, endpoint_v2));
    let read_lib_1002 = try_deploy!(ReadLib1002::deploy_builder(
        &provider,
        endpoint_v2,
        U256::from(0),
        U256::from(0)
    ));

    Ok((
        provider,
        ContractAddresses {
            endpoint_v2,
            send_uln_302,
            receive_uln_302,
            read_lib_1002,
        },
    ))
}
