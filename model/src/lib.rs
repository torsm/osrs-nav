use std::fmt::{Display, Formatter};

pub use multimap::MultiMap;
use num_traits::One;
use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::definitions::{EdgeDefinition, RequirementDefinition};

pub mod definitions;
pub mod constants;
pub mod util;

pub struct NavGrid {
    pub vertices: Vec<Vertex>,
    pub edges: MultiMap<u32, Edge>,
    pub teleports: Vec<Edge>,
}

impl NavGrid {
    pub fn new() -> NavGrid {
        NavGrid {
            vertices: vec![Vertex::default(); (WIDTH * HEIGHT * PLANES) as usize],
            edges: MultiMap::new(),
            teleports: Vec::new()
        }
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter_all().flat_map(|(_, v)| v).chain(self.teleports.iter())
    }

    pub fn iter_edges_mut(&mut self) -> impl Iterator<Item = &mut Edge> {
        self.edges.iter_all_mut().flat_map(|(_, v)| v).chain(self.teleports.iter_mut())
    }
}

#[derive(Default, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Vertex {
    pub flags: u8,
    pub extra_edges_and_group: u8, // surely rust will soon support bit fields
}

impl Vertex {
    pub fn has_extra_edges(&self) -> bool {
        self.extra_edges_and_group & 1 == 1
    }

    pub fn set_extra_edges(&mut self, extra_edges: bool) {
        self.extra_edges_and_group = extra_edges as u8 | self.extra_edges_and_group & 0xFE;
    }

    pub fn get_group(&self) -> u8 {
        self.extra_edges_and_group >> 1
    }

    pub fn set_group(&mut self, group: u8) {
        self.extra_edges_and_group = group << 1 | self.extra_edges_and_group & 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub destination: Coordinate,
    #[serde(default = "u32::one")]
    pub cost: u32,
    pub definition: EdgeDefinition,
    #[serde(default)]
    pub requirements: Vec<RequirementDefinition>,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: u16,
    pub y: u16,
    pub plane: u8,
}

impl Coordinate {
    pub fn from_index(index: u32) -> Coordinate {
        Coordinate {
            x: (index % WIDTH) as u16,
            y: ((index % (WIDTH * HEIGHT)) / WIDTH) as u16,
            plane: (index / (WIDTH * HEIGHT)) as u8,
        }
    }

    pub fn from_id(id: u32) -> Coordinate {
        Coordinate {
            x: ((id >> 14) & 0x3FFF) as u16,
            y: (id & 0x3FFF) as u16,
            plane: (id >> 28) as u8,
        }
    }

    pub fn from_map_square(i: u8, j: u8, x: u8, y: u8, plane: u8) -> Coordinate {
        Coordinate {
            x: i as u16 * REGION_SIZE as u16 + x as u16,
            y: j as u16 * REGION_SIZE as u16 + y as u16,
            plane
        }
    }

    pub fn index(&self) -> u32 {
        self.plane as u32 * WIDTH * HEIGHT + self.y as u32 * WIDTH + self.x as u32
    }

    pub fn id(&self) -> u32 {
        (self.plane as u32 & 0xF) << 28 | (self.x as u32 & 0x3FFF) << 14 | (self.y as u32 & 0x3FFF)
    }

    pub fn validate(&self) -> bool {
        self.x < WIDTH as u16 && self.y < HEIGHT as u16 && self.plane < PLANES as u8
    }

    pub fn derive(&self, dx: i16, dy: i16, dplane: i8) -> Coordinate {
        Coordinate {
            x: (self.x as i16 + dx) as u16,
            y: (self.y as i16 + dy) as u16,
            plane: (self.plane as i8 + dplane) as u8,
        }
    }

    pub fn derive_mut(&mut self, dx: i16, dy: i16, dplane: i8) {
        self.x = (self.x as i16 + dx) as u16;
        self.y = (self.y as i16 + dy) as u16;
        self.plane = (self.plane as i8 + dplane) as u8;
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.plane)
    }
}
