use color_eyre::Result;
use gadget_sdk as sdk;
use gadget_sdk::runners::BlueprintRunner;


#[sdk::main(env)]
async fn main() -> Result<()> {
    let signer = env.first_sr25519_signer()?;
    let client = env.client().await?;

    tracing::info!("Starting the event watcher ...");
    BlueprintRunner::new((), env)
        .run()
        .await?;

    tracing::info!("Exiting...");
    Ok(())
}
