//! A parser/filter for OSM protobuf bundles.

use filter::Group;
use std::error::Error;

pub mod filter;
mod geo;
pub mod output;

/// Extract Objects from OSM
///
/// Objects (i.e. Nodes, Ways & Relations) will be extracted according to filter options. Some geographic properties (centroid, bounding boxes) are computed for all entities.
///
/// Filtering `groups` can be applied to select objects according to their tags.
pub async fn objects(file: String, groups: Option<Vec<Group>>) -> Result<(), Box<dyn Error>> {
    output::import_meili(file, groups).await
}
