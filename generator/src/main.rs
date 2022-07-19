extern crate core;

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use expect_exit::{Expected, ExpectedWithError};
use flate2::Compression;
use flate2::write::GzEncoder;
use rs3cache::cli::Config;
use rs3cache::definitions::location_configs::LocationConfig;
use rs3cache::definitions::mapsquares::MapSquares;
use serde::{Deserialize, Serialize};

use generator::NavGenerator;
use model::{Coordinate, Edge, NavGrid};
use model::definitions::RequirementDefinition;
use model::util::RegionCache;

use crate::generator::GeneratorConfig;

mod generator;

#[derive(Parser)]
struct Options {
    /// Directory containing cache files
    #[clap(short, long)]
    cache: PathBuf,
    /// JSON file containing XTEA keys for the selected cache
    #[clap(short, long)]
    xteas: PathBuf,
    /// File that the generated NavGrid is serialized into
    #[clap(short, long)]
    output: PathBuf,
    /// YAML file with custom edges
    #[clap(long)]
    edges: Option<PathBuf>,
    /// YAML file with generator configuration
    #[clap(long)]
    config: Option<PathBuf>,
}

fn main() {
    let options = Options::parse();

    let config = if let Some(config_file) = &options.config {
        let file = File::open(config_file).or_exit_e_("Error opening config file");
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).or_exit_e_("Error parsing config file")
    } else {
        GeneratorConfig::default()
    };

    let mut nav_grid = {
        println!("Processing cache...");
        let mut generator = NavGenerator::new(config);
        let cache_config = Config {
            input: options.cache,
            xteas: options.xteas,
            output: PathBuf::new(),
            render: vec![],
            dump: vec![],
            assert_coherence: false,
        };
        let loc_configs = LocationConfig::dump_all(&cache_config).or_exit_e_("Error loading location definitions");
        let map_squares = MapSquares::new(&cache_config).or_exit_e_("Error loading map squares");
        map_squares.into_iter().for_each(|sq| {
            let map_square = sq.or_exit_e_("Error deserializing map square");
            generator.process_map_square(&map_square, &loc_configs);
        });
        println!("Transforming flags...");
        generator.transform_flags();
        generator.nav_grid
    };

    println!("Processing custom edges...");
    if let Some(edges_file) = &options.edges {
        let file = File::open(edges_file).or_exit_e_("Error opening edges file");
        load_custom_edges(&mut nav_grid, file).or_exit_e_("Error loading custom edges");
    }

    println!("Postprocessing...");
    for index in nav_grid.edges.keys() {
        nav_grid.vertices[*index as usize].set_extra_edges(true);
    }
    create_groups(&mut nav_grid);
    nav_grid.iter_edges_mut().flat_map(|e| e.requirements.iter_mut()).for_each(|r| {
        if let RequirementDefinition::Skill { skill, .. } = r {
            *skill = skill.to_uppercase();
        }
    });

    println!("Exporting nav...");
    std::fs::create_dir_all(&options.output.parent().or_exit_("Invalid output path")).or_exit_e_("Error creating output directory");
    let nav_file = File::create(&options.output).or_exit_e_("Error creating output file");
    let encoder = GzEncoder::new(nav_file, Compression::default());
    let mut writer = BufWriter::new(encoder);
    for vertex in &nav_grid.vertices {
        let bytes = [vertex.flags, vertex.extra_edges_and_group];
        writer.write(&bytes).or_exit_e_("Error serializing vertices");
    }
    ciborium::ser::into_writer(&nav_grid.edges, &mut writer).or_exit_e_("Error serializing edges");
    ciborium::ser::into_writer(&nav_grid.teleports, &mut writer).or_exit_e_("Error serializing teleports");

    println!("Complete");
}

fn create_groups(nav_grid: &mut NavGrid) {
    let mut cache = RegionCache::new(false);
    let mut groups = Vec::new();
    for index in 0..nav_grid.vertices.len() {
        let vertex = &mut nav_grid.vertices[index];
        if vertex.flags == 0 {
            continue;
        }
        vertex.set_group(1);
        if *cache.get_mut(index as u32) {
            continue;
        }
        let c = Coordinate::from_index(index as u32);
        // Only start floods from within the surface area
        if let (1152..=3903, 2496..=4159, 0) = (c.x, c.y, c.plane) {
            let mut reachable = Vec::new();
            pathfinder::flood(nav_grid, &c, |i| {
                let visited = cache.get_mut(i);
                if *visited {
                    false
                } else {
                    reachable.push(i);
                    *cache.get_mut(i) = true;
                    true
                }
            });
            groups.push(reachable);
        }
    }
    groups.sort_by(|a, b| b.len().cmp(&a.len()));
    for (index, group) in groups.iter().take(126).enumerate() {
        let group_id = index as u8 + 2;
        for index in group {
            nav_grid.vertices[*index as usize].set_group(group_id);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CustomEdges {
    #[serde(default)]
    edges: Vec<CustomEdge>,
    #[serde(default)]
    teleports: Vec<Edge>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomEdge {
    pub source: Coordinate,
    #[serde(default)]
    pub bidirectional: bool,
    #[serde(flatten)]
    pub edge: Edge,
}

fn load_custom_edges(nav_grid: &mut NavGrid, file: File) -> serde_yaml::Result<()> {
    let reader = BufReader::new(file);
    let mut edges: CustomEdges = serde_yaml::from_reader(reader)?;
    while let Some(edge) = edges.edges.pop() {
        if edge.bidirectional {
            let mut edge2 = edge.edge.clone();
            let dest2 = edge2.destination;
            edge2.destination = edge.source.clone();
            nav_grid.edges.insert(dest2.index(), edge2);
        }
        nav_grid.edges.insert(edge.source.index(), edge.edge);
    }
    nav_grid.teleports.append(&mut edges.teleports);
    Ok(())
}
