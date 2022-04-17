use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use model::{Coordinate, Edge, NavGrid};
use model::constants::*;
use model::definitions::{EdgeDefinition, GameState};
use model::util::RegionCache;

#[derive(Debug, Deserialize, Serialize)]
pub enum Step {
    Edge(EdgeDefinition),
    Step(Coordinate),
}

#[derive(Clone, Copy)]
struct DijkstraCacheState<'a> {
    cost: u32,
    prev: u32,
    edge: Option<&'a Edge>
}

pub struct BucketRingBuffer<T> {
    buckets: Vec<Vec<T>>,
    cursor: usize,
}

impl<T: Clone> BucketRingBuffer<T> {
    pub fn new(max_cost: u32) -> BucketRingBuffer<T> {
        BucketRingBuffer {
            buckets: vec![Vec::new(); max_cost as usize + 1],
            cursor: 0
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.buckets.iter_mut().for_each(Vec::clear);
    }

    fn increment(&mut self) {
        self.cursor += 1;
        if self.cursor == self.buckets.len() {
            self.cursor = 0;
        }
    }

    fn next_bin(&mut self) -> Option<usize> {
        let len = self.buckets.len();
        for i in 0..len {
            let mut index = self.cursor + i;
            if index >= len {
                index -= len;
            }
            if !self.buckets[index].is_empty() {
                return Some(index);
            }
        }
        None
    }

    fn push(&mut self, cost: u32, state: T) {
        let len = self.buckets.len();
        let mut index = cost as usize + self.cursor;
        if index >= len {
            index -= len;
        }
        self.buckets[index].push(state);
    }
}

pub fn dijkstra(nav_grid: &NavGrid, start: &Coordinate, end: &Coordinate, game_state: &GameState) -> (usize, usize, Option<Vec<Step>>) {
    let start_index = start.index();
    let end_index = end.index();
    let target_group = nav_grid.vertices[end_index as usize].get_group();
    let max_cost = nav_grid.iter_edges().map(|edge| edge.cost).max().unwrap();
    let mut queue = BucketRingBuffer::new(max_cost); //TODO borrow from pool instead to prevent allocations?
    let mut cache = RegionCache::new(DijkstraCacheState { cost: u32::MAX, prev: u32::MAX, edge: None });
    let mut count = 0;
    if nav_grid.vertices[start_index as usize].get_group() == target_group {
        cache.get_mut(start_index).cost = 0;
        queue.push(0, (0, start_index));
    }
    for teleport in &nav_grid.teleports {
        if teleport.requirements.iter().all(|req| req.is_met(game_state)) {
            let index = teleport.destination.index();
            if nav_grid.vertices[index as usize].get_group() == target_group {
                let dest = cache.get_mut(index);
                if teleport.cost < dest.cost {
                    dest.cost = teleport.cost;
                    dest.prev = start_index;
                    dest.edge = Some(teleport);
                    queue.push(teleport.cost, (dest.cost, index));
                }
            }
        }
    }

    while let Some(current) = queue.next_bin() {
        while let Some((cost, mut index)) = queue.buckets[current].pop() {
            count += 1;
            if index == end_index {
                let mut path = Vec::new();
                while index != start_index {
                    let state = cache.get_mut(index);
                    if let Some(edge) = state.edge {
                        path.push(Step::Edge(edge.definition.clone()));
                    } else {
                        path.push(Step::Step(Coordinate::from_index(index)));
                    }
                    index = state.prev;
                }
                path.reverse();
                return (count, cache.mem_usage(), Some(path));
            }
            let v = &nav_grid.vertices[index as usize];
            for (flag, dx, dy) in &DIRECTIONS {
                if (v.flags & flag) != 0 {
                    let adj_index = index + (WIDTH * *dy as u32) + *dx as u32;
                    let adj = cache.get_mut(adj_index);
                    if cost + 1 < adj.cost {
                        adj.cost = cost + 1;
                        adj.prev = index;
                        adj.edge = None;
                        queue.push(1, (adj.cost, adj_index));
                    }
                }
            }
            if v.has_extra_edges() {
                for edge in nav_grid.edges.get_vec(&index).unwrap() {
                    if edge.requirements.iter().all(|req| req.is_met(game_state)) {
                        let adj = cache.get_mut(edge.destination.index());
                        if cost + edge.cost < adj.cost {
                            adj.cost = cost + edge.cost;
                            adj.prev = index;
                            adj.edge = Some(edge);
                            queue.push(edge.cost, (adj.cost, edge.destination.index()));
                        }
                    }
                }
            }
        }
        queue.increment();
    }

    (count, cache.mem_usage(), None)
}

pub fn flood<F>(nav_grid: &NavGrid, start: &Coordinate, mut visit_vertex: F) where F: FnMut(u32) -> bool {
    let mut queue = VecDeque::new();
    let mut cache = RegionCache::new(false);
    queue.push_back(start.index());
    *cache.get_mut(start.index()) = true;
    while let Some(index) = queue.pop_front() {
        let v = &nav_grid.vertices[index as usize];
        if !visit_vertex(index) {
            continue;
        }
        for (flag, dx, dy) in &DIRECTIONS {
            if (v.flags & flag) != 0 {
                let adj_index = index + (WIDTH * *dy as u32) + *dx as u32;
                let visited = cache.get_mut(adj_index);
                if !*visited {
                    queue.push_back(adj_index);
                    *visited = true;
                }
            }
        }
        if v.has_extra_edges() {
            for edge in nav_grid.edges.get_vec(&index).unwrap() {
                let visited = cache.get_mut(edge.destination.index());
                if !*visited {
                    queue.push_back(edge.destination.index());
                    *visited = true;
                }
            }
        }
    }
}
