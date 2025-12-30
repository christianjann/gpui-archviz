use crate::types::*;
use std::collections::{HashMap, VecDeque};

pub struct Grid {
    pub cell_size: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub width: usize,
    pub height: usize,
    pub obstacles: Vec<Vec<bool>>,
}

impl Grid {
    pub fn new(nodes: &[Node], _canvas_width: f64, _canvas_height: f64, cell_size: f64) -> Self {
        let min_x = nodes
            .iter()
            .map(|n| n.position.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = nodes
            .iter()
            .map(|n| n.position.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = nodes
            .iter()
            .map(|n| n.position.x + n.size.width)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = nodes
            .iter()
            .map(|n| n.position.y + n.size.height)
            .fold(f64::NEG_INFINITY, f64::max);
        let canvas_width = (max_x - min_x + 100.0).max(400.0);
        let canvas_height = (max_y - min_y + 100.0).max(300.0);
        let width = (canvas_width / cell_size).ceil() as usize + 1;
        let height = (canvas_height / cell_size).ceil() as usize + 1;
        let mut obstacles = vec![vec![false; width]; height];

        for node in nodes {
            let x1 = ((node.position.x - 7.0 - min_x) / cell_size).floor() as usize;
            let y1 = ((node.position.y - 7.0 - min_y) / cell_size).floor() as usize;
            let x2 =
                ((node.position.x + node.size.width + 7.0 - min_x) / cell_size).ceil() as usize;
            let y2 =
                ((node.position.y + node.size.height + 7.0 - min_y) / cell_size).ceil() as usize;

            for y in y1..y2.min(height) {
                for x in x1..x2.min(width) {
                    obstacles[y][x] = true;
                }
            }
        }

        // Block port areas
        for node in nodes {
            for port in &node.ports {
                let port_x1 =
                    (((node.position.x + port.position.x - min_x) / cell_size).floor() as i32 - 2)
                        .max(0) as usize;
                let port_y1 =
                    (((node.position.y + port.position.y - min_y) / cell_size).floor() as i32 - 2)
                        .max(0) as usize;
                let port_x2 = (((node.position.x + port.position.x + port.size.width - min_x)
                    / cell_size)
                    .ceil() as i32
                    + 2)
                .min((width - 1) as i32) as usize
                    + 1;
                let port_y2 = (((node.position.y + port.position.y + port.size.height - min_y)
                    / cell_size)
                    .ceil() as i32
                    + 2)
                .min((height - 1) as i32) as usize
                    + 1;

                for y in port_y1..port_y2.min(height) {
                    for x in port_x1..port_x2.min(width) {
                        obstacles[y][x] = true;
                    }
                }
            }
        }

        Grid {
            cell_size,
            origin_x: min_x,
            origin_y: min_y,
            width,
            height,
            obstacles,
        }
    }

    pub fn pos_to_grid(&self, pos: &Position) -> (usize, usize) {
        let x = (((pos.x - self.origin_x) / self.cell_size).round() as usize).min(self.width - 1);
        let y = (((pos.y - self.origin_y) / self.cell_size).round() as usize).min(self.height - 1);
        (x, y)
    }

    pub fn grid_to_pos(&self, x: usize, y: usize) -> Position {
        Position {
            x: self.origin_x + x as f64 * self.cell_size,
            y: self.origin_y + y as f64 * self.cell_size,
        }
    }

    pub fn find_path(&self, start: Position, end: Position) -> Vec<Position> {
        let start_grid = self.pos_to_grid(&start);
        let end_grid = self.pos_to_grid(&end);

        if start_grid == end_grid {
            return vec![start, end];
        }

        // BFS for orthogonal path
        let mut queue = VecDeque::new();
        queue.push_back(start_grid);

        let mut came_from = HashMap::new();
        came_from.insert(start_grid, None);

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == end_grid {
                found = true;
                break;
            }

            for neighbor in self.neighbors(current) {
                came_from.entry(neighbor).or_insert_with(|| {
                    queue.push_back(neighbor);
                    Some(current)
                });
            }
        }

        if !found {
            return vec![start, end];
        }

        // Reconstruct path
        let mut path = vec![end_grid];
        let mut current = end_grid;
        while let Some(prev) = came_from[&current] {
            path.push(prev);
            current = prev;
        }
        path.reverse();

        path.into_iter()
            .map(|(x, y)| self.grid_to_pos(x, y))
            .collect()
    }

    pub fn neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = pos;
        let mut neighbors = vec![];
        if x > 0 && !self.obstacles[y][x - 1] {
            neighbors.push((x - 1, y));
        }
        if x < self.width - 1 && !self.obstacles[y][x + 1] {
            neighbors.push((x + 1, y));
        }
        if y > 0 && !self.obstacles[y - 1][x] {
            neighbors.push((x, y - 1));
        }
        if y < self.height - 1 && !self.obstacles[y + 1][x] {
            neighbors.push((x, y + 1));
        }
        neighbors
    }
}

pub fn nearest_grid(pos: &Position, origin_x: f64, origin_y: f64, cell_size: f64) -> Position {
    let gx = ((pos.x - origin_x) / cell_size).round();
    let gy = ((pos.y - origin_y) / cell_size).round();
    Position {
        x: origin_x + gx * cell_size,
        y: origin_y + gy * cell_size,
    }
}
