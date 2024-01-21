pub mod osm {
    use super::super::geo::{Bounds, Location};
    use osmpbfreader::objects::Tags;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum GeoInfo {
        Point {
            lon: f64,
            lat: f64,
        },
        Shape {
            centroid: Option<Location>,
            bounds: Option<Bounds>,
            #[serde(skip_serializing_if = "Option::is_none")]
            coordinates: Option<Vec<(f64, f64)>>,
        },
    }

    #[derive(Serialize, Deserialize)]
    pub struct Object {
        pub id: i64,
        #[serde(rename = "type")]
        pub osm_type: &'static str,
        pub tags: Tags,
        #[serde(flatten)]
        pub geo_info: GeoInfo,
    }
}
