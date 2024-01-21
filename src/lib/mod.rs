//! A parser/filter for OSM protobuf bundles.

use filter::{Filter, Group};
use osmpbfreader::OsmPbfReader;
use std::error::Error;
use std::io::{Read, Seek};

pub mod filter;
mod geo;
pub mod items;
pub mod output;

/// Extract Objects from OSM
///
/// Objects (i.e. Nodes, Ways & Relations) will be extracted according to filter options. Some geographic properties (centroid, bounding boxes) are computed for all entities.
///
/// Filtering `groups` can be applied to select objects according to their tags.
pub async fn objects(
    file: impl Seek + Read,
    groups: Option<Vec<Group>>,
) -> Result<(), Box<dyn Error>> {
    let mut pbf = OsmPbfReader::new(file);

    let objs = match groups {
        Some(grps) => pbf.get_objs_and_deps(|obj| obj.filter(&grps))?,
        None => pbf.get_objs_and_deps(|_| true)?,
    };

    output::import_meili(objs).await?;

    Ok(())
}
