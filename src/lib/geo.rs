use geo::prelude::*;
use geo_types::{Coord, Point};
use serde::{Deserialize, Serialize};

const EQ_PRECISION: f64 = 1.0e-5;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

impl From<(f64, f64)> for Location {
    fn from(tuple: (f64, f64)) -> Location {
        Location {
            lon: tuple.0,
            lat: tuple.1,
        }
    }
}

impl PartialEq<Location> for Location {
    fn eq(&self, other: &Self) -> bool {
        let self_point = Point::new(self.lon, self.lat);
        let other_point = Point::new(other.lon, other.lat);
        let distance = self_point.euclidean_distance(&other_point);
        distance < EQ_PRECISION
    }
}

impl PartialEq<Bounds> for Bounds {
    fn eq(&self, other: &Self) -> bool {
        let (self_ne, self_sw) = self.into();
        let (other_ne, other_sw) = other.into();
        self_ne == other_ne && self_sw == other_sw
    }
}

pub trait Length {
    fn length(&self) -> f64;
}

impl From<&(f64, f64)> for Location {
    fn from(coordinates: &(f64, f64)) -> Self {
        Location {
            lon: coordinates.0,
            lat: coordinates.1,
        }
    }
}

impl From<Location> for [f64; 2] {
    fn from(loc: Location) -> Self {
        [loc.lon, loc.lat]
    }
}

impl From<Point<f64>> for Location {
    fn from(point: Point<f64>) -> Self {
        Location {
            lat: point.y(),
            lon: point.x(),
        }
    }
}

impl From<Coord<f64>> for Location {
    fn from(coordinate: Coord<f64>) -> Self {
        Location {
            lat: coordinate.y,
            lon: coordinate.x,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bounds {
    e: f64,
    n: f64,
    s: f64,
    w: f64,
}

impl From<&Bounds> for (Location, Location) {
    fn from(bounds: &Bounds) -> Self {
        let ne = Location {
            lon: bounds.n,
            lat: bounds.e,
        };
        let sw = Location {
            lon: bounds.s,
            lat: bounds.w,
        };

        (ne, sw)
    }
}
