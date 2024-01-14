use lib::output::Output;
use lib::{boundaries, filter, objects, streets};
use std::error::Error;
use std::fs::File;
use std::io;
use structopt::StructOpt;

mod lib;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Objects {
        #[structopt(short, long)]
        tags: Option<String>,
        #[structopt(short, long)]
        retain_coordinates: bool,
        #[structopt(short, long)]
        meili: bool,
    },
    Streets {
        #[structopt(short, long)]
        geojson: bool,
        #[structopt(short, long)]
        name: Option<String>,
        #[structopt(short, long)]
        boundary: Option<u8>,
        #[structopt(short, long)]
        meili: bool,
    },
    Boundaries {
        #[structopt(short, long)]
        geojson: bool,
        #[structopt(short, long)]
        levels: Option<Vec<u8>>,
        #[structopt(short, long)]
        meili: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    let args = Cli::from_args();

    let file = File::open(args.path)?;

    match args.cmd {
        Command::Objects {
            tags,
            retain_coordinates,
            meili
        } => {
            let objects = if let Some(tags) = tags {
                let groups = filter::parse(&tags);
                objects(file, Some(&groups), retain_coordinates)?
            } else {
                objects(file, None, retain_coordinates)?
            };

            if meili {
                objects.write_meili()?;
            } else {
                objects.write_json_lines(&mut handle)?;
            }
        }
        Command::Streets {
            geojson,
            name,
            boundary,
            meili
        } => {
            let streets = streets(file, name.as_deref(), boundary)?;
            if geojson {
                streets.write_geojson(&mut handle)?;
            } else if meili {
                streets.write_meili()?;
            } else {
                streets.write_json_lines(&mut handle)?;
            }
        }
        Command::Boundaries { levels, geojson, meili } => {
            let boundaries = boundaries(file, levels)?;
            if geojson {
                boundaries.write_geojson(&mut handle)?;
            } else if meili {
                boundaries.write_meili()?;
            } else {
                boundaries.write_json_lines(&mut handle)?;
            }
        },
    }
    Ok(())
}
