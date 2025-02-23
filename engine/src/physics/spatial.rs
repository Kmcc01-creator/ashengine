use glam::Vec3;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const CELL_SIZE: f32 = 10.0;
const LOAD_FACTOR_THRESHOLD: f32 = 0.75;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GridCell {
    x: i32,
    y: i32,
    z: i32,
}

// Spatial hash table with dynamic resizing
pub struct SpatialHash {
    cell_size: f32,
    grid: HashMap<GridCell, Vec<usize>>,
    object_cells: Vec<Vec<GridCell>>, // Track which cells each object is in
    total_objects: usize,
    cells_used: usize,
}

impl SpatialHash {
    pub fn new() -> Self {
        Self {
            cell_size: CELL_SIZE,
            grid: HashMap::new(),
            object_cells: Vec::new(),
            total_objects: 0,
            cells_used: 0,
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.object_cells.clear();
        self.total_objects = 0;
        self.cells_used = 0;
    }

    fn position_to_cell(&self, position: Vec3) -> GridCell {
        GridCell {
            x: (position.x / self.cell_size).floor() as i32,
            y: (position.y / self.cell_size).floor() as i32,
            z: (position.z / self.cell_size).floor() as i32,
        }
    }

    fn calculate_load_factor(&self) -> f32 {
        if self.grid.capacity() == 0 {
            return 0.0;
        }
        self.cells_used as f32 / self.grid.capacity() as f32
    }

    fn resize(&mut self) {
        if self.calculate_load_factor() > LOAD_FACTOR_THRESHOLD {
            self.cell_size *= 2.0; // Double cell size to reduce number of cells
        } else if self.calculate_load_factor() < LOAD_FACTOR_THRESHOLD / 4.0 {
            self.cell_size /= 2.0; // Halve cell size to increase spatial resolution
        } else {
            return;
        }

        // Rebuild grid with new cell size
        let old_grid = std::mem::take(&mut self.grid);
        self.grid = HashMap::with_capacity(old_grid.capacity());
        self.cells_used = 0;

        for (_, objects) in old_grid {
            for &obj_idx in &objects {
                if obj_idx < self.object_cells.len() {
                    for cell in &self.object_cells[obj_idx] {
                        self.insert_to_cell(*cell, obj_idx);
                    }
                }
            }
        }
    }

    fn insert_to_cell(&mut self, cell: GridCell, object_index: usize) {
        if !self.grid.contains_key(&cell) {
            self.cells_used += 1;
        }
        self.grid.entry(cell).or_default().push(object_index);
    }

    pub fn insert(&mut self, object_index: usize, min: Vec3, max: Vec3) {
        // Ensure object_cells vector is large enough
        if object_index >= self.object_cells.len() {
            self.object_cells.resize_with(object_index + 1, Vec::new);
        }

        let min_cell = self.position_to_cell(min);
        let max_cell = self.position_to_cell(max);

        // Clear previous cells for this object
        self.object_cells[object_index].clear();

        // Insert into all overlapping cells
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = GridCell { x, y, z };
                    self.insert_to_cell(cell, object_index);
                    self.object_cells[object_index].push(cell);
                }
            }
        }

        self.total_objects += 1;
        self.resize();
    }

    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<usize> {
        let cells_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.position_to_cell(center);
        let mut result = std::collections::HashSet::new();

        for x in -cells_radius..=cells_radius {
            for y in -cells_radius..=cells_radius {
                for z in -cells_radius..=cells_radius {
                    let cell = GridCell {
                        x: center_cell.x + x,
                        y: center_cell.y + y,
                        z: center_cell.z + z,
                    };
                    if let Some(objects) = self.grid.get(&cell) {
                        result.extend(objects);
                    }
                }
            }
        }

        result.into_iter().collect()
    }

    pub fn get_potential_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = std::collections::HashSet::new();

        for objects in self.grid.values() {
            for i in 0..objects.len() {
                for j in (i + 1)..objects.len() {
                    let obj1 = objects[i];
                    let obj2 = objects[j];
                    if obj1 < obj2 {
                        pairs.insert((obj1, obj2));
                    } else {
                        pairs.insert((obj2, obj1));
                    }
                }
            }
        }

        pairs.into_iter().collect()
    }
}

// Parallel collision detection using rayon
pub struct ParallelBroadPhase {
    spatial_hash: SpatialHash,
}

impl ParallelBroadPhase {
    pub fn new() -> Self {
        Self {
            spatial_hash: SpatialHash::new(),
        }
    }

    pub fn update(&mut self, positions: &[(Vec3, Vec3)]) -> Vec<(usize, usize)> {
        use rayon::prelude::*;

        self.spatial_hash.clear();

        // Insert objects in parallel
        positions
            .par_iter()
            .enumerate()
            .for_each(|(i, (min, max))| {
                self.spatial_hash.insert(i, *min, *max);
            });

        // Get potential pairs
        let pairs = self.spatial_hash.get_potential_pairs();

        // Filter pairs in parallel
        pairs
            .into_par_iter()
            .filter(|&(i, j)| {
                // Additional filtering can be added here
                // For example, checking if objects are in the same island
                true
            })
            .collect()
    }
}

// Morton encoding for better cache coherency
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MortonCode(u64);

impl MortonCode {
    fn new(x: i32, y: i32, z: i32) -> Self {
        let x = Self::expand_bits(x as u64);
        let y = Self::expand_bits(y as u64);
        let z = Self::expand_bits(z as u64);
        MortonCode(x | (y << 1) | (z << 2))
    }

    fn expand_bits(mut v: u64) -> u64 {
        v = (v | (v << 16)) & 0x0000FFFF0000FFFF;
        v = (v | (v << 8)) & 0x00FF00FF00FF00FF;
        v = (v | (v << 4)) & 0x0F0F0F0F0F0F0F0F;
        v = (v | (v << 2)) & 0x3333333333333333;
        v = (v | (v << 1)) & 0x5555555555555555;
        v
    }
}

// Cache-efficient spatial hash using Morton codes
pub struct CacheFriendlySpatialHash {
    cell_size: f32,
    grid: HashMap<MortonCode, Vec<usize>>,
}

impl CacheFriendlySpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    fn position_to_morton(&self, position: Vec3) -> MortonCode {
        let x = (position.x / self.cell_size).floor() as i32;
        let y = (position.y / self.cell_size).floor() as i32;
        let z = (position.z / self.cell_size).floor() as i32;
        MortonCode::new(x, y, z)
    }

    pub fn insert(&mut self, object_index: usize, min: Vec3, max: Vec3) {
        let min_code = self.position_to_morton(min);
        let max_code = self.position_to_morton(max);

        // Insert into cells (simplified for example)
        self.grid.entry(min_code).or_default().push(object_index);
        if min_code != max_code {
            self.grid.entry(max_code).or_default().push(object_index);
        }
    }
}
