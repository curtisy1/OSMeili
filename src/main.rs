use clap::Parser;
use settings::Settings;
use std::error::Error;

mod filter;
mod geo;
mod importer;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Settings::parse();
    importer::import_meili(args).await?;

    Ok(())
}
