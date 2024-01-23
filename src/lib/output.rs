use itertools::Itertools;
use meilisearch_sdk::{Client, TasksSearchQuery};
use osmpbfreader::{Node, OsmId, OsmObj};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

const MEILI_API_KEY: &str = "346cb8272d48ca98f3ea33b834a8467a7149eb8886a5580a0332eeac9b5abfcd";
const SEARCHABLE_ATTRIBUTES: [&str; 5] = [
    "street",
    "houseNumber",
    "postcode",
    "city",
    "country"
];

#[derive(Serialize, Deserialize)]
struct MeiliCoordinate {
    lon: f64,
    lat: f64,
}

#[derive(Serialize, Deserialize)]
struct MeiliObject {
    id: i64,

    #[serde(rename = "_geo")]
    geo_info: MeiliCoordinate,
    //todo: we need a match quality here. Infer from lowest osm type
}

trait FromOsm {
    fn from_osm(node: &Node) -> Self;
}

impl FromOsm for HashMap<String, String> {
    fn from_osm(node: &Node) -> Self {
        let mut map: HashMap<String, String> = HashMap::new();

        // first, populate every addr:* tag the node has
        for tag in node.tags.iter() {
            if tag.0.starts_with("addr") {
                let address_part = tag.0.split_terminator(':').last().unwrap().to_string();
                map.insert(address_part, tag.1.to_string());
            }
        }

        // if at least one tag exists, we add the id and coordinates
        if !map.is_empty() {
            map.insert(String::from("id"), node.id.0.to_string());

            let geo_info = MeiliCoordinate { lon: node.lon(), lat: node.lat() };
            map.insert(String::from("_geo"), serde_json::to_string(&geo_info).unwrap());
        }

        map
    }
}

pub async fn import_meili(items: BTreeMap<OsmId, OsmObj>) -> Result<(), Box<dyn Error>> {
    let client = Client::new("http://localhost:7700", Some(MEILI_API_KEY));

    // first, we need to import data. Creating search attributes first is not working
    let objects = items
        .values()
        .filter_map(|obj| {
            match obj {
                OsmObj::Node(obj) => {
                    let map = HashMap::from_osm(obj);
                    if map.is_empty() {
                        return None;
                    }

                    Some(map)
                }
                _ => None
            }
        });

    let bodies = client
        .index("addresses")
        .add_documents_in_batches(&objects.collect_vec(), None, None)
        .await;

    if bodies.is_ok() {
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
            .set_searchable_attributes(&SEARCHABLE_ATTRIBUTES)
            .await?;

        // geography objects are needed for geofencing. Set up a filter attribute
        client
            .index("addresses")
            .set_filterable_attributes(&["_geo"])
            .await?;
    }

    Ok(())
}
