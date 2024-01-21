use clap::{Parser};
use lib::{filter::{Group}, objects};
use std::error::Error;
use std::fs::File;

mod lib;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// the source of the .pbf extract.
    /// Can be either a local file or a uri (e.g. GeoFabrik)
    #[clap(short, long, env, required = true)]
    source: String,
    /// specify tags to filter on.
    /// By default addr:country, addr:city, addr:postcode, addr:street, addr:housenumber will be used
    ///
    /// for a list of addr:* tags, see https://wiki.openstreetmap.org/wiki/Key:addr:*
    #[clap(short, long, env, value_parser = clap::value_parser!(Group), default_value = "addr:city,addr:city,addr:country,addr:housenumber,addr:postcode")]
    tags: Option<Vec<Group>>,
    /// the source to use for replication.
    /// Like source, it can be set to either local or remote extracts
    #[clap(short, long, env)]
    replication_source: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let file = File::open(args.source)?;
    objects(file, args.tags).await?;

    Ok(())
}
