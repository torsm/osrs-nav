#[macro_use] extern crate rocket;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use expect_exit::ExpectedWithError;
use rocket::{Build, Rocket, State};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket_prometheus::PrometheusMetrics;
use serde::{Deserialize, Serialize};

use model::{Coordinate, NavGrid};
use model::definitions::{GameState, RequirementDefinition};
use pathfinder::Step;

#[derive(Parser)]
struct Options {
    /// Path to NavGrid file
    #[clap(short, long)]
    navgrid: PathBuf,
}

#[derive(Deserialize)]
struct Request {
    start: Coordinate,
    end: Coordinate,
    game_state: GameState,
}

#[derive(Clone, Default, Serialize)]
struct TrackedVarps {
    varps: HashSet<u32>,
    varbits: HashSet<u32>,
}

#[post("/", data = "<request>")]
fn handle_path_request(request: Json<Request>, nav_grid: &State<NavGrid>) -> Result<Json<Option<Vec<Step>>>, BadRequest<&str>> {
    if !request.start.validate() || !request.end.validate() {
        println!("[Path] {} -> {} invalid coordinates", request.start, request.end);
        Err(BadRequest(Some("Coordinate out of bounds")))
    } else {
        let begin = Instant::now();
        let (visited, mem_usage, path) = pathfinder::dijkstra(&nav_grid, &request.start, &request.end, &request.game_state);
        let duration = Instant::now() - begin;
        println!("[Path] {} -> {} in {:.2}ms, {}Kb, {} visited", request.start, request.end, duration.as_secs_f64() * 1000f64, mem_usage / 1024, visited);
        Ok(Json(path))
    }
}

#[get("/")]
fn handle_varps_request(tracked_varps: &State<TrackedVarps>) -> Json<TrackedVarps> {
    Json(tracked_varps.inner().clone())
}

#[launch]
fn rocket() -> Rocket<Build> {
    let options = Options::parse();
    let nav_grid = load_nav_grid(&options.navgrid).or_exit_e_("Error loading NavGrid");
    let mut tracked_varps = TrackedVarps::default();
    nav_grid.iter_edges().flat_map(|e| &e.requirements).for_each(|r| {
        match r {
            RequirementDefinition::Varp { index, .. } => tracked_varps.varps.insert(*index),
            RequirementDefinition::Varbit { index, .. } => tracked_varps.varbits.insert(*index),
            _ => false
        };
    });
    let prometheus = PrometheusMetrics::new();
    rocket::build()
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
        .mount("/path", routes![handle_path_request])
        .mount("/varps", routes![handle_varps_request])
        .manage(nav_grid)
        .manage(tracked_varps)
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
