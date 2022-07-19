use std::collections::{BTreeMap, HashSet};

use expect_exit::Expected;
use rs3cache::definitions::location_configs::LocationConfig;
use rs3cache::definitions::locations::Location;
use rs3cache::definitions::mapsquares::MapSquare;
use rs3cache::definitions::tiles::TileArray;
use serde::Deserialize;

use model::{Coordinate, Edge, NavGrid};
use model::constants::*;
use model::definitions::{EdgeDefinition, Regex};
use model::util::RegionCache;

#[derive(Default, Deserialize)]
pub struct GeneratorConfig {
    excluded_location_ids: HashSet<u32>,
}

pub struct NavGenerator {
    pub collision_flags: RegionCache<u32>,
    pub nav_grid: NavGrid,
    pub config: GeneratorConfig,
}

impl NavGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        NavGenerator {
            collision_flags: RegionCache::default(),
            nav_grid: NavGrid::new(),
            config,
        }
    }
}

impl NavGenerator {
    pub fn process_map_square(&mut self, map_square: &MapSquare, loc_configs: &BTreeMap<u32, LocationConfig>) {
        if let Ok(tiles) = map_square.get_tiles() {
            self.process_tiles(&map_square, tiles);
            if let Ok(locations) = map_square.get_locations() {
                self.process_locations(locations, loc_configs, tiles);
            }
        }
    }

    fn process_tiles(&mut self, sq: &MapSquare, tiles: &TileArray) {
        for ((plane, x, y), tile) in tiles.indexed_iter() {
            let mut c = Coordinate::from_map_square(sq.i(), sq.j(), x as u8, y as u8, plane as u8);
            if tile.settings.unwrap_or(0) & 1 == 1 {
                if tiles[[1, x, y]].settings.unwrap_or(0) & 2 == 2 {
                    if c.plane == 0 {
                        continue;
                    }
                    c.derive_mut(0, 0, -1);
                }
                self.set_flag(&c, BLOCK_MOVEMENT_FLOOR);
            }
        }
    }

    fn process_locations(&mut self, locations: &Vec<Location>, configs: &BTreeMap<u32, LocationConfig>, tiles: &TileArray) {
        for loc in locations {
            let config = configs.get(&loc.id).or_exit(|| format!("Missing LocationConfig {}", loc.id));
            let mut c = Coordinate::from_map_square(loc.i, loc.j, loc.x, loc.y, loc.plane.inner());

            if tiles[[1, loc.x as usize, loc.y as usize]].settings.unwrap_or(0) & 2 == 2 {
                if c.plane == 0 {
                    continue;
                }
                c.derive_mut(0, 0, -1);
            }

            match loc.r#type {
                0..=3 => if config.interact_type.unwrap_or(2) != 0 {
                    self.add_wall(&c, loc.r#type, loc.rotation, config.break_line_of_sight.unwrap_or(true));
                }
                22 => if config.interact_type.unwrap_or(2) == 1 {
                    self.set_flag(&c, BLOCK_MOVEMENT_FLOOR_DECORATION);
                },
                t if t >= 9 => if config.interact_type.unwrap_or(2) != 0 {
                    self.add_location(&c, config.dim_x.unwrap_or(1), config.dim_y.unwrap_or(1), loc.rotation, config.break_line_of_sight.unwrap_or(true));
                }
                _ => {}
            }

            self.process_special_location(loc, &c, config);
        }
    }

    fn process_special_location(&mut self, loc: &Location, adjusted_c: &Coordinate, config: &LocationConfig) {
        if self.config.excluded_location_ids.contains(&loc.id) {
            return;
        }
        if let 0..=3 = loc.r#type {
            if let (Some(name), Some(actions)) = (&config.name, &config.actions) {
                if (name == "Door" || name == "Gate" || name == "Large door") && actions.contains(&Some("Open".to_string())) {
                    self.add_door(adjusted_c, loc.id, loc.rotation);
                }
            }
        }
    }

    /// Refer to original [implementation](https://github.com/open-osrs/runelite/blob/master/runescape-client/src/main/java/CollisionMap.java)
    fn add_location(&mut self, c: &Coordinate, mut width: u8, mut height: u8, rotation: u8, solid: bool) {
        if rotation == 1 || rotation == 3 {
            std::mem::swap(&mut width, &mut height);
        }
        let mut flag = BLOCK_MOVEMENT_OBJECT;
        if solid {
            flag |= BLOCK_LINE_OF_SIGHT_FULL;
        }
        for ix in 0..width as i16 {
            for iy in 0..height as i16 {
                self.set_flag(&c.derive(ix, iy, 0), flag);
            }
        }
    }

    /// Refer to original [implementation](https://github.com/open-osrs/runelite/blob/master/runescape-client/src/main/java/CollisionMap.java)
    fn add_wall(&mut self, c: &Coordinate, r#type: u8, rotation: u8, solid: bool) {
        let solid_mask = if solid { u32::MAX } else { 0u32 };
        match r#type {
            0 => {
                match rotation {
                    0 => {
                        self.set_flag(c, BLOCK_MOVEMENT_WEST | (BLOCK_LINE_OF_SIGHT_WEST & solid_mask));
                        self.set_flag(&c.derive(-1, 0, 0), BLOCK_MOVEMENT_EAST | (BLOCK_LINE_OF_SIGHT_EAST & solid_mask));
                    }
                    1 => {
                        self.set_flag(c, BLOCK_MOVEMENT_NORTH | (BLOCK_LINE_OF_SIGHT_NORTH & solid_mask));
                        self.set_flag(&c.derive(0, 1, 0), BLOCK_MOVEMENT_SOUTH | (BLOCK_LINE_OF_SIGHT_SOUTH & solid_mask));
                    }
                    2 => {
                        self.set_flag(c, BLOCK_MOVEMENT_EAST | (BLOCK_LINE_OF_SIGHT_EAST & solid_mask));
                        self.set_flag(&c.derive(1, 0, 0), BLOCK_MOVEMENT_WEST | (BLOCK_LINE_OF_SIGHT_WEST & solid_mask));
                    }
                    3 => {
                        self.set_flag(c, BLOCK_MOVEMENT_SOUTH | (BLOCK_LINE_OF_SIGHT_SOUTH & solid_mask));
                        self.set_flag(&c.derive(0, -1, 0), BLOCK_MOVEMENT_NORTH | (BLOCK_LINE_OF_SIGHT_NORTH & solid_mask));
                    }
                    _ => {}
                }
            }
            1 | 3 => {
                match rotation {
                    0 => {
                        self.set_flag(c, BLOCK_MOVEMENT_NORTH_WEST | (BLOCK_LINE_OF_SIGHT_NORTH_WEST & solid_mask));
                        self.set_flag(&c.derive(-1, 1, 0), BLOCK_MOVEMENT_SOUTH_EAST | (BLOCK_LINE_OF_SIGHT_SOUTH_EAST & solid_mask));
                    }
                    1 => {
                        self.set_flag(c, BLOCK_MOVEMENT_NORTH_EAST | (BLOCK_LINE_OF_SIGHT_NORTH_EAST & solid_mask));
                        self.set_flag(&c.derive(1, 1, 0), BLOCK_MOVEMENT_SOUTH_WEST | (BLOCK_LINE_OF_SIGHT_SOUTH_WEST & solid_mask));
                    }
                    2 => {
                        self.set_flag(c, BLOCK_MOVEMENT_SOUTH_EAST | (BLOCK_LINE_OF_SIGHT_SOUTH_EAST & solid_mask));
                        self.set_flag(&c.derive(1, -1, 0), BLOCK_MOVEMENT_NORTH_WEST | (BLOCK_LINE_OF_SIGHT_NORTH_WEST & solid_mask));
                    }
                    3 => {
                        self.set_flag(c, BLOCK_MOVEMENT_SOUTH_WEST | (BLOCK_LINE_OF_SIGHT_SOUTH_WEST & solid_mask));
                        self.set_flag(&c.derive(-1, -1, 0), BLOCK_MOVEMENT_NORTH_EAST | (BLOCK_LINE_OF_SIGHT_NORTH_EAST & solid_mask));
                    }
                    _ => {}
                }
            }
            2 => {
                match rotation {
                    0 => {
                        self.set_flag(c, BLOCK_MOVEMENT_NORTH | BLOCK_MOVEMENT_WEST | ((BLOCK_LINE_OF_SIGHT_NORTH | BLOCK_LINE_OF_SIGHT_WEST) & solid_mask));
                        self.set_flag(&c.derive(-1, 0, 0), BLOCK_MOVEMENT_EAST | (BLOCK_LINE_OF_SIGHT_EAST & solid_mask));
                        self.set_flag(&c.derive(0, 1, 0), BLOCK_MOVEMENT_SOUTH | (BLOCK_LINE_OF_SIGHT_SOUTH & solid_mask));
                    }
                    1 => {
                        self.set_flag(c, BLOCK_MOVEMENT_NORTH | BLOCK_MOVEMENT_EAST | ((BLOCK_LINE_OF_SIGHT_NORTH | BLOCK_LINE_OF_SIGHT_EAST) & solid_mask));
                        self.set_flag(&c.derive(0, 1, 0), BLOCK_MOVEMENT_SOUTH | (BLOCK_LINE_OF_SIGHT_SOUTH & solid_mask));
                        self.set_flag(&c.derive(1, 0, 0), BLOCK_MOVEMENT_WEST | (BLOCK_LINE_OF_SIGHT_WEST & solid_mask));
                    }
                    2 => {
                        self.set_flag(c, BLOCK_MOVEMENT_SOUTH | BLOCK_MOVEMENT_EAST | ((BLOCK_LINE_OF_SIGHT_SOUTH | BLOCK_LINE_OF_SIGHT_EAST) & solid_mask));
                        self.set_flag(&c.derive(1, 0, 0), BLOCK_MOVEMENT_WEST | (BLOCK_LINE_OF_SIGHT_WEST & solid_mask));
                        self.set_flag(&c.derive(0, -1, 0), BLOCK_MOVEMENT_NORTH | (BLOCK_LINE_OF_SIGHT_NORTH & solid_mask));
                    }
                    3 => {
                        self.set_flag(c, BLOCK_MOVEMENT_SOUTH | BLOCK_MOVEMENT_WEST | ((BLOCK_LINE_OF_SIGHT_SOUTH | BLOCK_LINE_OF_SIGHT_WEST) & solid_mask));
                        self.set_flag(&c.derive(0, -1, 0), BLOCK_MOVEMENT_NORTH | (BLOCK_LINE_OF_SIGHT_NORTH & solid_mask));
                        self.set_flag(&c.derive(-1, 0, 0), BLOCK_MOVEMENT_EAST | (BLOCK_LINE_OF_SIGHT_EAST & solid_mask));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn add_door(&mut self, c: &Coordinate, id: u32, rotation: u8) {
        let (dx, dy) = match rotation {
            0 => (-1, 0),
            1 => (0, 1),
            2 => (1, 0),
            3 => (0, -1),
            _ => (0, 0)
        };
        let c2 = c.derive(dx, dy, 0);
        let def = EdgeDefinition::Door {
            id,
            position: c.clone(),
            action: Regex::new("^Open$").expect("Invalid regex"),
        };
        self.nav_grid.edges.insert(c.index(), Edge {
            destination: c2,
            cost: 2,
            definition: def.clone(),
            requirements: vec![],
        });
        self.nav_grid.edges.insert(c2.index(), Edge {
            destination: c.clone(),
            cost: 2,
            definition: def,
            requirements: vec![],
        });
    }

    fn can_travel_in_direction(&self, c: &Coordinate, dx: i16, dy: i16) -> bool {
        let dest = c.derive(dx, dy, 0);
        if !dest.validate() {
            return false;
        }
        let mut x_flags = BLOCK_MOVEMENT_FULL;
        let mut y_flags = BLOCK_MOVEMENT_FULL;
        let mut xy_flags = BLOCK_MOVEMENT_FULL;

        if dx < 0 {
            x_flags |= BLOCK_MOVEMENT_EAST;
        } else if dx > 0 {
            x_flags |= BLOCK_MOVEMENT_WEST;
        }
        if dy < 0 {
            y_flags |= BLOCK_MOVEMENT_NORTH;
        } else if dy > 0 {
            y_flags |= BLOCK_MOVEMENT_SOUTH;
        }
        if dx < 0 && dy < 0 {
            xy_flags |= BLOCK_MOVEMENT_NORTH_EAST;
        } else if dx < 0 && dy > 0 {
            xy_flags |= BLOCK_MOVEMENT_SOUTH_EAST;
        } else if dx > 0 && dy < 0 {
            xy_flags |= BLOCK_MOVEMENT_NORTH_WEST;
        } else if dx > 0 && dy > 0 {
            xy_flags |= BLOCK_MOVEMENT_SOUTH_WEST;
        }
        let dest_flags = self.get_flag(&dest).unwrap_or(u32::MAX);
        if dx != 0 {
            if dest_flags & x_flags != 0 {
                return false;
            }
        }
        if dy != 0 {
            if dest_flags & y_flags != 0 {
                return false;
            }
        }
        if dx != 0 && dy != 0 {
            if dest_flags & xy_flags != 0 {
                return false;
            }
            if self.get_flag(&dest.derive(0, -dy, 0)).unwrap_or(u32::MAX) & x_flags != 0 {
                return false;
            }
            if self.get_flag(&dest.derive(-dx, 0, 0)).unwrap_or(u32::MAX) & y_flags != 0 {
                return false;
            }
        }
        true
    }

    pub fn transform_flags(&mut self) {
        for index in 0..self.nav_grid.vertices.len() {
            let c = Coordinate::from_index(index as u32);
            if self.get_flag(&c).unwrap_or(u32::MAX) & BLOCK_MOVEMENT_FULL > 0 {
                continue;
            }
            let mut flags = 0;
            for (flag, dx, dy) in &DIRECTIONS {
                if self.can_travel_in_direction(&c, *dx as i16, *dy as i16) {
                    flags |= flag;
                }
            }
            self.nav_grid.vertices[index].flags = flags;
        }
    }

    fn get_flag(&self, c: &Coordinate) -> Option<u32> {
        let flag = self.collision_flags.get(c.index())?;
        Some(*flag)
    }

    fn set_flag(&mut self, c: &Coordinate, flag: u32) {
        *self.collision_flags.get_mut(c.index()) |= flag;
    }
}

const BLOCK_MOVEMENT_NORTH_WEST: u32 = 0x1;
const BLOCK_MOVEMENT_NORTH: u32 = 0x2;
const BLOCK_MOVEMENT_NORTH_EAST: u32 = 0x4;
const BLOCK_MOVEMENT_EAST: u32 = 0x8;
const BLOCK_MOVEMENT_SOUTH_EAST: u32 = 0x10;
const BLOCK_MOVEMENT_SOUTH: u32 = 0x20;
const BLOCK_MOVEMENT_SOUTH_WEST: u32 = 0x40;
const BLOCK_MOVEMENT_WEST: u32 = 0x80;
const BLOCK_MOVEMENT_OBJECT: u32 = 0x100;
const BLOCK_LINE_OF_SIGHT_NORTH_WEST: u32 = 0x200;
const BLOCK_LINE_OF_SIGHT_NORTH: u32 = 0x400;
const BLOCK_LINE_OF_SIGHT_NORTH_EAST: u32 = 0x800;
const BLOCK_LINE_OF_SIGHT_EAST: u32 = 0x1000;
const BLOCK_LINE_OF_SIGHT_SOUTH_EAST: u32 = 0x2000;
const BLOCK_LINE_OF_SIGHT_SOUTH: u32 = 0x4000;
const BLOCK_LINE_OF_SIGHT_SOUTH_WEST: u32 = 0x8000;
const BLOCK_LINE_OF_SIGHT_WEST: u32 = 0x10000;
const BLOCK_LINE_OF_SIGHT_FULL: u32 = 0x20000;
const BLOCK_MOVEMENT_FLOOR_DECORATION: u32 = 0x40000;
const BLOCK_MOVEMENT_FLOOR: u32 = 0x200000;
const BLOCK_MOVEMENT_FULL: u32 = BLOCK_MOVEMENT_FLOOR | BLOCK_MOVEMENT_FLOOR_DECORATION | BLOCK_MOVEMENT_OBJECT;
