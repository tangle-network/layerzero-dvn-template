use blueprint_sdk::runners::core::runner::BlueprintRunner;
use color_eyre::Result;

#[blueprint_sdk::main(env)]
async fn main() -> Result<()> {
    let signer = env.first_sr25519_signer()?;
    let client = env.client().await?;

    tracing::info!("Starting the event watcher ...");
    BlueprintRunner::new((), env).run().await?;

    tracing::info!("Exiting...");
    Ok(())
}
