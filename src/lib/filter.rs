use std::str::FromStr;
use osm_io::osm::model::element::Element;
use osm_io::osm::model::tag::Tag;

#[derive(PartialEq, Debug, Clone)]
pub enum Condition {
    TagPresence(String),
    ValueMatch(String, String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Group {
    pub conditions: Vec<Condition>,
}

impl FromStr for Group {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_group(s))
    }
}

fn parse_condition(condition_str: &str) -> Condition {
    let split_str: Vec<&str> = condition_str.splitn(2, '~').collect();
    if split_str.len() < 2 {
        Condition::TagPresence(condition_str.into())
    } else {
        let key = split_str[0];
        let value = split_str[1];
        Condition::ValueMatch(key.into(), value.into())
    }
}


/// Parse an expression into a filter groups
///
/// Stating a key (`amenity`), will pick all entities which are tagged using that key.
/// To further narrow down the results, a specific value can be given using a `~` field
/// separator (`amenity~fountain`). To check the presence of multiple tags for the same
/// entity, statements can be combined using the `+` operator (`'amenity~fountain+tourism'`).
/// Finally, options can be specified by concatenating groups of statements with `,`
/// (`amenity~fountain+tourism,amenity~townhall`). If an entity matches the criteria of
/// either group it will be included in the output.
/// ```
fn parse_group(group_str: &str) -> Group {
    let conditions = group_str.split('+').map(parse_condition).collect();
    Group { conditions }
}

fn check_condition(tags: &Vec<Tag>, condition: &Condition) -> bool {
    match condition {
        Condition::TagPresence(key) => tags.iter().any(|tag| tag.k().contains(key)),
        Condition::ValueMatch(key, value) => tags.iter().any(|tag| tag.k() == key && tag.v() == value),
    }
}

fn check_group(tags: &Vec<Tag>, group: &Group) -> bool {
    group.conditions.iter().all(|c| check_condition(tags, c))
}

pub trait Filter {
    fn filter(&self, groups: &Vec<Group>) -> bool;
}

impl Filter for Element {
    fn filter(&self, groups: &Vec<Group>) -> bool {
        let borrow: &Vec<Tag> = &Vec::new();
        let tags: &Vec<Tag> = match self {
            Element::Node { node} => node.tags(),
            Element::Relation {relation} => relation.tags(),
            Element::Way {way} => way.tags(),
            Element::Sentinel => borrow,
        };
        groups.iter().any(|c| check_group(tags, c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Condition {
        pub fn new(tag: &str, value: Option<&str>) -> Self {
            if let Some(value) = value {
                return Condition::ValueMatch(tag.into(), value.into());
            }
            Condition::TagPresence(tag.into())
        }
    }


    fn parse(selector_str: &str) -> Vec<Group> {
        selector_str.split(',').map(parse_group).collect()
    }

    #[test]
    fn parse_single_group() {
        let condition = Condition::TagPresence("amenity".into());
        let conditions = vec![condition];
        let group = Group { conditions };

        assert_eq!(parse("amenity"), [group]);
    }

    #[test]
    fn parse_multiple_groups() {
        let condition_1 = Condition::TagPresence("amenity".into());
        let condition_2 = Condition::TagPresence("highway".into());
        let group_1 = Group {
            conditions: vec![condition_1],
        };
        let group_2 = Group {
            conditions: vec![condition_2],
        };

        assert_eq!(parse("amenity,highway"), [group_1, group_2]);
    }

    #[test]
    fn parse_multiple_conditions() {
        let condition_1 = Condition::TagPresence("amenity".into());
        let condition_2 = Condition::TagPresence("highway".into());
        let conditions = vec![condition_1, condition_2];
        let group = Group { conditions };

        assert_eq!(parse("amenity+highway"), vec![group]);
    }

    #[test]
    fn parse_value_match() {
        let condition = Condition::ValueMatch("amenity".into(), "theatre".into());
        let conditions = vec![condition];
        let group = Group { conditions };

        assert_eq!(parse("amenity~theatre"), vec![group]);
    }
}
