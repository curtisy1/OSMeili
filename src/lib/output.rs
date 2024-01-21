use itertools::Itertools;
use meilisearch_sdk::{Client, TasksSearchQuery};
use osmpbfreader::{OsmId, OsmObj, Tags};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    postcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "houseNumber")]
    house_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    #[serde(rename = "_geo")]
    geo_info: MeiliCoordinate,
}

fn get_address_part(tags: &Tags, address_part: &str) -> Option<String> {
    if tags.contains_key(address_part) {
        Some(tags.get(address_part).unwrap().to_string())
    } else {
        None
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
                    let geo_info = MeiliCoordinate { lon: obj.lon(), lat: obj.lat() };
                    let tags = &obj.tags;
                    let object = MeiliObject {
                        id: obj.id.0,
                        country: get_address_part(&tags, "addr:country"),
                        city: get_address_part(&tags, "addr:city"),
                        postcode: get_address_part(&tags, "addr:postcode"),
                        street: get_address_part(&tags, "addr:street"),
                        house_number: get_address_part(&tags, "addr:housenumber"),
                        geo_info,
                    };
                    Some(object)
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
