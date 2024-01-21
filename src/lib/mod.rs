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
///
/// # Example
///
/// ```
/// use std::fs::File;
/// use osm_pbf2json::objects;
/// use osm_pbf2json::filter::{Condition, Group};
///
/// let file = File::open("./tests/data/alexanderplatz.pbf").unwrap();
/// let cond_1 = Condition::new("surface", Some("cobblestone"));
/// let cond_2 = Condition::new("highway", None);
/// let group = Group { conditions: vec![cond_1, cond_2] };
/// let cobblestone_ways = objects(file, Some(&vec![group]), false).unwrap();
/// assert_eq!(cobblestone_ways.len(), 4);
/// ```
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

#[cfg(test)]
mod get_coordinates {
    use super::*;
    use osmpbfreader::objects::{Node, NodeId, OsmId, OsmObj, Ref, Relation, RelationId, Tags, Way, WayId};
    use std::collections::BTreeMap;

    fn create_node(id: NodeId, lng: i32, lat: i32) -> Node {
        let tags = Tags::new();
        let decimicro_lat = lat * 10_000_000;
        let decimicro_lon = lng * 10_000_000;
        Node {
            id,
            tags,
            decimicro_lat,
            decimicro_lon,
        }
    }

    fn create_relation(id: RelationId, refs: Vec<Ref>) -> Relation {
        let tags = Tags::new();
        Relation { id, tags, refs }
    }

    fn create_way(id: WayId, nodes: Vec<NodeId>) -> Way {
        let tags = Tags::new();
        Way { id, tags, nodes }
    }

    fn add_nodes(
        coordinates: Vec<(i32, i32)>,
        obj_map: &mut BTreeMap<OsmId, OsmObj>,
        node_ids: &mut Vec<NodeId>,
    ) {
        for (i, (lng, lat)) in coordinates.iter().enumerate() {
            let node_id = NodeId((i as i64) + 1);
            let node = create_node(node_id, *lng, *lat);
            obj_map.insert(node_id.into(), node.into());
            node_ids.push(node_id);
        }
    }

    fn create_refs(ids: Vec<OsmId>) -> Vec<Ref> {
        ids.into_iter()
            .map(|id| Ref {
                member: id,
                role: "something".into(),
            })
            .collect()
    }

    #[test]
    fn relation_without_refs() {
        let obj_map = BTreeMap::new();
        let id = RelationId(42);
        let rel = create_relation(id, vec![]);
        let coordinates = rel.get_coordinates(&obj_map, &mut vec![]);
        assert_eq!(coordinates.len(), 0);
    }

    #[test]
    fn relation_with_one_way() {
        let coordinates = vec![(9, 50), (9, 51), (10, 51)];

        // 1     2
        //
        //
        // 0

        let mut obj_map = BTreeMap::new();
        let mut node_ids = vec![];
        add_nodes(coordinates, &mut obj_map, &mut node_ids);

        let way_id = WayId(42);
        let way = create_way(way_id, node_ids);
        obj_map.insert(way_id.into(), way.into());

        let refs = create_refs(vec![way_id.into()]);
        let id = RelationId(43);
        let rel = create_relation(id, refs);

        // we expect a closed triangle

        let coordinates = rel.get_coordinates(&obj_map, &mut vec![]);
        assert_eq!(
            coordinates,
            vec![(9., 50.), (9., 51.), (10., 51.), (9., 50.)]
        );
    }

    #[test]
    fn relation_with_one_node() {
        let node_id = NodeId(41);
        let node = create_node(node_id, 5, 49);
        let mut obj_map = BTreeMap::new();
        obj_map.insert(node_id.into(), node.into());
        let id = RelationId(42);
        let refs = create_refs(vec![node_id.into()]);
        let rel = create_relation(id, refs);
        let coordinates = rel.get_coordinates(&obj_map, &mut vec![]);
        assert_eq!(coordinates, vec![(5., 49.)]);
    }

    #[test]
    fn relation_with_multiple_nodes() {
        let coordinates = vec![(6, 52), (6, 50), (8, 50), (8, 52), (7, 51)];

        // Node 4 is located in the middle of a grid
        // and should hence be ignored.
        //
        // 0     3
        //
        //    4
        //
        // 1     2

        let mut obj_map = BTreeMap::new();
        let mut node_ids = vec![];
        add_nodes(coordinates, &mut obj_map, &mut node_ids);

        let id = RelationId(42);
        let refs = create_refs(node_ids.into_iter().map(NodeId::into).collect());
        let rel = create_relation(id, refs);
        let coordinates = rel.get_coordinates(&obj_map, &mut vec![]);

        // We expect a simplified closed rectangle.
        //
        // 3-----2
        // |     |
        // |     |
        // |     |
        // 0/4---1

        assert_eq!(
            coordinates,
            vec![(6., 50.), (8., 50.), (8., 52.), (6., 52.), (6., 50.)]
        );
    }

    #[test]
    fn nested_relations() {
        let coordinates = vec![(6, 52), (6, 50)];
        let mut obj_map = BTreeMap::new();
        let mut node_ids = vec![];
        add_nodes(coordinates, &mut obj_map, &mut node_ids);

        let child_id = RelationId(42);
        let refs = create_refs(node_ids.into_iter().map(NodeId::into).collect());
        let child_rel = create_relation(child_id, refs);
        obj_map.insert(child_id.into(), child_rel.into());

        let node_id = NodeId(43);
        let node = create_node(node_id, 8, 52);
        obj_map.insert(node_id.into(), node.into());

        let parent_id = RelationId(44);
        let refs = create_refs(vec![child_id.into(), node_id.into()]);
        let parent_rel = create_relation(parent_id, refs);

        let coordinates = parent_rel.get_coordinates(&obj_map, &mut vec![]);

        assert_eq!(
            coordinates,
            vec![(6., 50.), (8., 52.), (6., 52.), (6., 50.)]
        );
    }

    #[test]
    fn nested_relations_with_cycle() {
        let mut obj_map = BTreeMap::new();
        let rel_id_1 = RelationId(42);
        let rel_id_2 = RelationId(44);
        let refs = create_refs(vec![rel_id_2.into()]);
        let rel_1 = create_relation(rel_id_1, refs);
        obj_map.insert(rel_id_1.into(), rel_1.into());

        let node_id = NodeId(43);
        let node = create_node(node_id, 8, 52);
        obj_map.insert(node_id.into(), node.into());

        let refs = create_refs(vec![rel_id_1.into(), node_id.into()]);
        let rel_2 = create_relation(rel_id_2, refs);

        let coordinates = rel_2.get_coordinates(&obj_map, &mut vec![]);

        assert_eq!(coordinates, vec![(8., 52.)]);
    }
}
