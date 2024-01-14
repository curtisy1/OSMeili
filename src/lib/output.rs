use super::geo::Length;
use super::geojson::{Entity, Geometry};
use super::items::osm::{GeoInfo, Object};
use super::items::{AdminBoundary, Street};
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use futures::executor::block_on;

use meilisearch_sdk::{
    client::*,
};

const MEILI_API_KEY: &str = "432cdb366925b78221ec1ad2603c39318a77a4241347ce039dd34e05c68c8506";

pub trait Output {
    fn write_geojson(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>>;
    fn write_json_lines(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>>;
    fn write_meili(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Serialize, Deserialize)]
struct JSONBBox {
    sw: [f64; 2],
    ne: [f64; 2],
}

#[derive(Serialize, Deserialize)]
struct JSONBoundary {
    name: String,
    admin_level: u8,
    bbox: JSONBBox,
}

impl Output for Vec<AdminBoundary> {
    fn write_json_lines(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        for boundary in self.iter() {
            let name = boundary.name.clone();
            let admin_level = boundary.admin_level;
            let (sw, ne) = boundary.geometry.sw_ne();
            let bbox = JSONBBox { sw, ne };
            let json_boundary = JSONBoundary {
                name,
                admin_level,
                bbox,
            };
            let json = to_string(&json_boundary)?;
            writeln!(writer, "{}", json)?;
        }
        Ok(())
    }

    fn write_geojson(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        let features = self
            .iter()
            .map(|boundary| {
                let coordinates = boundary.geometry.coordinates();
                let geometry = Geometry::MultiPolygon { coordinates };
                let properties = vec![
                    (String::from("name"), boundary.name.clone()),
                    (
                        String::from("admin_level"),
                        boundary.admin_level.to_string(),
                    ),
                ]
                    .into_iter()
                    .collect();
                Entity::Feature {
                    geometry,
                    properties,
                }
            })
            .collect();
        let feature_collection = Entity::FeatureCollection { features };
        let string = to_string(&feature_collection)?;
        writeln!(writer, "{}", string)?;
        Ok(())
    }

    fn write_meili(&self) -> Result<(), Box<dyn Error>> {
        let client = Client::new("http://localhost:7700", Some(MEILI_API_KEY));

        let boundaries: Vec<JSONBoundary> = self.iter().map(|boundary| {
            let name = boundary.name.clone();
            let admin_level = boundary.admin_level;
            let (sw, ne) = boundary.geometry.sw_ne();
            let bbox = JSONBBox { sw, ne };
            JSONBoundary {
                name,
                admin_level,
                bbox,
            }
        }).collect();



        // todo do not block but spawn n threads and upload in parallel
        block_on(async move {
            client
                .index("boundaries")
                .add_documents(&boundaries, None)
                .await
                .unwrap()
        });

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct JSONStreet {
    id: i64,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    boundary: Option<String>,
    length: f64,
    loc: (f64, f64),
}

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
    #[serde(rename = "type")]
    osm_type: &'static str,
    #[serde(rename = "_geo")]
    geo_info: MeiliCoordinate,
}

fn get_address_part(object: &Object, address_part: &str) -> Option<String> {
    if object.tags.contains_key(address_part) {
        Some(object.tags.get(address_part).unwrap().to_string())
    } else {
        None
    }
}

impl Output for Vec<Object> {
    fn write_json_lines(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        for object in self.iter() {
            let json = to_string(object)?;
            writeln!(writer, "{}", json)?;
        }
        Ok(())
    }

    fn write_geojson(&self, _writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        unimplemented!();
    }

    fn write_meili(&self) -> Result<(), Box<dyn Error>> {
        let client = Client::new("http://localhost:7700", Some(MEILI_API_KEY));
        let objects: Vec<MeiliObject> = self.iter().filter(|object| {
            match &object.geo_info {
                GeoInfo::Point {..} => {
                    true
                },
                GeoInfo::Shape {centroid, ..} => {
                    centroid.is_some()
                }
            }
        }).map(|object| {
            let geo_info = match &object.geo_info {
                GeoInfo::Point { lon, lat } => {
                    MeiliCoordinate { lon: *lon, lat: *lat }
                }
                GeoInfo::Shape { centroid, bounds, coordinates } => {
                    let centroid = centroid.as_ref().unwrap();
                    MeiliCoordinate { lon: centroid.lon, lat: centroid.lat }
                }
            };

            MeiliObject {
                id: object.id,
                country: get_address_part(object, "addr:country"),
                city: get_address_part(object, "addr:city"),
                postcode: get_address_part(object, "addr:postcode"),
                street: get_address_part(object, "addr:street"),
                house_number: get_address_part(object, "addr:housenumber"),
                osm_type: object.osm_type,
                geo_info,
            }
        }).collect();


        let searchable_attributes = [
            "street",
            "houseNumber",
            "postcode",
            "city",
            "country"
        ];
        // todo do not block but spawn n threads and upload in parallel
        // batches can only transfer a max of 100MB, so we need to split by that at least
        block_on(async move {
            client
                .index("addresses")
                .add_documents(&objects, None)
                .await
                .unwrap();

            // todo this needs to run after the above is finished
            client
                .index("addresses")
                .set_searchable_attributes(&searchable_attributes)
                .await
                .unwrap();

            client
                .index("addresses")
                .set_filterable_attributes(&["_geo"])
                .await
                .unwrap()
        });

        Ok(())
    }
}

impl Output for Vec<Street> {
    fn write_json_lines(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        for street in self.iter() {
            let id = street.id();
            let loc = street.middle().ok_or("could not calculate middle")?;
            let name = street.name.clone();
            let boundary = street.boundary.clone();
            let length = street.length();
            let json_street = JSONStreet {
                id,
                name,
                boundary,
                length,
                loc,
            };
            let json = to_string(&json_street)?;
            writeln!(writer, "{}", json)?;
        }
        Ok(())
    }

    fn write_geojson(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        let features = self
            .iter()
            .filter_map(|street| {
                let geometries: Vec<_> = street
                    .segments
                    .iter()
                    .filter(|segment| segment.geometry.len() >= 2)
                    .map(|segment| segment.geometry.clone())
                    .collect();
                if geometries.is_empty() {
                    return None;
                }
                let coordinates = geometries.iter().map(|g| g.into()).collect();
                let geometry = Geometry::MultiLineString { coordinates };
                let r = random::<u8>();
                let g = random::<u8>();
                let b = random::<u8>();
                let random_color = format!("#{:02X}{:02X}{:02X}", r, g, b);
                let mut properties: HashMap<String, String> = HashMap::new();
                properties.insert("name".into(), street.name.clone());
                properties.insert("stroke".into(), random_color);
                if let Some(name) = &street.boundary {
                    properties.insert("boundary".into(), name.clone());
                }
                let entity = Entity::Feature {
                    geometry,
                    properties,
                };
                Some(entity)
            })
            .collect();

        let feature_collection = Entity::FeatureCollection { features };
        let string = to_string(&feature_collection)?;
        writeln!(writer, "{}", string)?;
        Ok(())
    }

    fn write_meili(&self) -> Result<(), Box<dyn Error>> {
        let client = Client::new("http://localhost:7700", Some(MEILI_API_KEY));
        let streets: Result<Vec<JSONStreet>, Box<dyn Error>> = self.iter().map(|street| {
            let id = street.id();
            let loc = street.middle().ok_or("could not calculate middle")?;
            let name = street.name.clone();
            let boundary = street.boundary.clone();
            let length = street.length();
            Ok(JSONStreet {
                id,
                name,
                boundary,
                length,
                loc,
            })
        }).collect();

        if streets.is_ok() {
            // todo do not block but spawn n threads and upload in parallel
            block_on(async move {
                client
                    .index("streets")
                    .add_documents(&streets.unwrap(), None) // todo write more specific stuff
                    .await
                    .unwrap()
            });
        }

        Ok(())
    }
}
