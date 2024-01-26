use clap::Parser;
use lib::{filter::Group, objects};
use std::error::Error;

mod lib;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// the source of the .pbf extract.
    /// Can be either a local file or uri (e.g. GeoFabrik)
    #[clap(short, long, env, required = true)]
    source: String,
    /// specify tags to filter on.
    /// By default, addr will be used
    ///
    /// for a list of addr:* tags, see https://wiki.openstreetmap.org/wiki/Key:addr:*
    #[clap(short, long, env, value_parser = clap::value_parser!(Group), default_value = "addr")]
    tags: Option<Vec<Group>>,
    /// the source to use for replication.
    /// Like source, it can be set to either local or remote extracts
    #[clap(short, long, env)]
    replication_source: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    objects(args.source, args.tags).await?;

    Ok(())
}
