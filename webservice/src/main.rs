use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use clap::Parser;
use expect_exit::ExpectedWithError;

use model::NavGrid;

#[derive(Parser)]
struct Options {
    /// NavGrid file
    #[clap(short, long)]
    navgrid: PathBuf,
}

fn main() {
    let options = Options::parse();
    let nav_grid = load_nav_grid(&options.navgrid).or_exit_e_("Error loading NavGrid");

    //TODO stuff
}

fn load_nav_grid(path: impl AsRef<Path>) -> Result<NavGrid, ciborium::de::Error<std::io::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut nav_grid = NavGrid::new();
    for vertex in &mut nav_grid.vertices {
        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        vertex.flags = buf[0];
        vertex.extra_edges_and_group = buf[1];
    }
    nav_grid.edges = ciborium::de::from_reader(&mut reader)?;
    nav_grid.teleports = ciborium::de::from_reader(&mut reader)?;
    Ok(nav_grid)
}
