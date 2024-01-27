use crate::filter::Filter;
use crate::settings::Settings;
use futures::{stream, StreamExt};
use itertools::Itertools;
use meilisearch_sdk::{Client, TasksSearchQuery};
use osm_io::osm::model::element::Element;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use osm_io::osm::pbf::reader::Reader as PbfReader;

#[derive(Serialize, Deserialize)]
struct MeiliCoordinate {
    lon: f64,
    lat: f64,
}

pub trait FromOsm {
    fn from_element(node: osm_io::osm::model::node::Node) -> Self;
}

impl FromOsm for HashMap<String, String> {
    //todo: we need a match quality here. Infer from lowest osm type
    fn from_element(node: osm_io::osm::model::node::Node) -> Self {
        let mut map: HashMap<String, String> = HashMap::new();

        // first, populate every addr:* tag the node has
        for tag in node.tags().iter() {
            let key = tag.k();
            if key.starts_with("addr") {
                let address_part = key.split_terminator(':').last().unwrap().to_string();
                map.insert(address_part, tag.v().to_string());
            }
        }

        // if at least one tag exists, we add the id and coordinates
        if !map.is_empty() {
            map.insert(String::from("id"), node.id().to_string());

            let coord = node.coordinate();
            let geo_info = MeiliCoordinate {
                lon: coord.lon(),
                lat: coord.lat(),
            };
            map.insert(
                String::from("_geo"),
                serde_json::to_string(&geo_info).unwrap(),
            );
        }

        map
    }
}

fn filter_osm(element: Element) -> Option<HashMap<String, String>> {
    match element {
        Element::Node { node } => {
            let map = HashMap::from_element(node);
            if map.is_empty() {
                return None;
            }

            Some(map)
        }
        _ => None,
    }
}

pub async fn import_meili(settings: Settings) -> Result<(), Box<dyn Error>> {
    let input_path = PathBuf::from(settings.source);
    let pbf = PbfReader::new(&input_path)?;
    let client = Client::new("http://localhost:7700", Some(settings.meili_key));

    println!("Starting meili import");
    // first, we need to import data. Creating search attributes first is not working
    let chunks = pbf
        .elements()?
        .filter_map(|elem| {
            if elem.filter(&settings.tags) {
                filter_osm(elem)
            } else {
                None
            }
        })
        .chunks(settings.import_chunk_size);

    let meili_index = &settings.meili_node_index_name;
    let bodies = stream::iter(&chunks)
        .map(|chunk| {
            let client = &client;
            async move {
                let objects = chunk.collect_vec();
                client
                    .index(meili_index)
                    .add_or_replace(&objects, None)
                    .await
            }
        })
        .buffer_unordered(settings.import_parallel_requests);

    let import_successful = bodies
        .all(|b| async move {
            match b {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("failed to push document to meili: {}", e);
                    false
                }
            }
        })
        .await;

    if import_successful {
        let mut has_pending_task = true;
        while has_pending_task {
            // check if we have processing tasks remaining
            let mut query = TasksSearchQuery::new(&client);
            let pending_tasks = query
                .with_statuses(["processing"])
                .with_limit(1)
                .execute()
                .await;
            if !pending_tasks.is_ok() || pending_tasks.unwrap().next == None {
                has_pending_task = false;
            } else {
                // wait 5 seconds before polling for open tasks again
                sleep(Duration::from_millis(5000)).await;
            }
        }

        // when above is done, create searchable attributes. Exclude the ids here as they mess up house numbers
        client
            .index("addresses")
            .set_searchable_attributes(&settings.meili_node_searchable_values)
            .await?;

        // geography objects are needed for geofencing. Set up a filter attribute
        client
            .index("addresses")
            .set_filterable_attributes(&["_geo"])
            .await?;
    }

    Ok(())
}
