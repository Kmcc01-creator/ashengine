use glam::Vec3;
use rayon::prelude::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::physics::{
    constraints::Constraint,
    physics::{PhysicsObject, PhysicsWorld},
};

const MIN_ISLAND_SIZE: usize = 4; // Minimum objects per island for parallel processing
const MAX_THREAD_ISLANDS: usize = 8; // Maximum islands per thread

pub struct ConstraintSolver {
    pub relaxation: f32,
    pub error_tolerance: f32,
    thread_pool: rayon::ThreadPool,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            relaxation: 0.2,
            error_tolerance: 1e-6,
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .build()
                .unwrap(),
        }
    }

    pub fn solve_constraints(&self, world: &mut PhysicsWorld, delta_time: f32) {
        let islands = world.island_solver.build_islands(world);

        // Group small islands together for better thread utilization
        let grouped_islands = self.group_islands(&islands);

        // Process island groups in parallel
        let results: Vec<_> = self.thread_pool.install(|| {
            grouped_islands
                .par_iter()
                .map(|island_group| {
                    let mut local_world = PhysicsWorld {
                        objects: island_group
                            .iter()
                            .flat_map(|island| {
                                island.objects.iter().map(|&idx| world.objects[idx].clone())
                            })
                            .collect(),
                        gravity: world.gravity,
                        constraints: island_group
                            .iter()
                            .flat_map(|island| {
                                island
                                    .constraints
                                    .iter()
                                    .map(|&idx| world.constraints[idx].clone_box())
                            })
                            .collect(),
                        num_iterations: world.num_iterations,
                        substeps: 1,
                        broad_phase: world.broad_phase.clone(),
                        constraint_solver: ConstraintSolver::new(),
                        island_solver: IslandSolver::new(),
                    };

                    self.solve_island_group(&mut local_world, delta_time);
                    local_world.objects
                })
                .collect()
        });

        // Merge results back
        self.merge_results(world, results, &islands);
    }

    fn group_islands<'a>(&self, islands: &'a [Island]) -> Vec<Vec<&'a Island>> {
        let mut grouped = Vec::new();
        let mut current_group = Vec::new();
        let mut current_size = 0;

        for island in islands {
            if island.objects.len() >= MIN_ISLAND_SIZE {
                // Large islands get their own group
                grouped.push(vec![island]);
            } else {
                current_group.push(island);
                current_size += island.objects.len();

                if current_size >= MIN_ISLAND_SIZE || current_group.len() >= MAX_THREAD_ISLANDS {
                    grouped.push(std::mem::take(&mut current_group));
                    current_size = 0;
                }
            }
        }

        if !current_group.is_empty() {
            grouped.push(current_group);
        }

        grouped
    }

    fn solve_island_group(&self, world: &mut PhysicsWorld, delta_time: f32) {
        // Process constraints in parallel within each island group
        let object_chunks = world.objects.chunks_mut(MAX_THREAD_ISLANDS);
        let constraint_chunks = world.constraints.chunks(MAX_THREAD_ISLANDS);

        for _ in 0..world.num_iterations {
            // Solve position constraints
            constraint_chunks
                .zip(object_chunks)
                .par_bridge()
                .for_each(|(constraints, objects)| {
                    for constraint in constraints {
                        constraint.project(objects);
                    }
                });

            // Apply position corrections with SIMD when possible
            self.apply_position_corrections(world, delta_time);
        }
    }

    fn merge_results(
        &self,
        world: &mut PhysicsWorld,
        results: Vec<Vec<RefCell<PhysicsObject>>>,
        islands: &[Island],
    ) {
        let mut offset = 0;
        for (result, island) in results.iter().zip(islands) {
            for (local_idx, &world_idx) in island.objects.iter().enumerate() {
                let mut world_obj = world.objects[world_idx].borrow_mut();
                let local_obj = result[offset + local_idx].borrow();
                *world_obj = local_obj.clone();
            }
            offset += island.objects.len();
        }
    }

    fn apply_position_corrections(&self, world: &mut PhysicsWorld, delta_time: f32) {
        world.objects.par_iter_mut().for_each(|object| {
            let mut obj = object.borrow_mut();
            match &mut *obj {
                PhysicsObject::RigidBody {
                    position,
                    velocity,
                    acceleration,
                    orientation,
                    angular_velocity,
                    ..
                } => {
                    // Update position
                    let position_correction = *velocity * delta_time;
                    *position += position_correction * self.relaxation;

                    // Update orientation
                    let angle = angular_velocity.length() * delta_time;
                    if angle != 0.0 {
                        let axis = *angular_velocity / angle;
                        let rotation = glam::Quat::from_axis_angle(axis, angle);
                        *orientation = rotation * *orientation;
                        orientation.normalize();
                    }

                    *acceleration = Vec3::ZERO;
                }
                PhysicsObject::DeformableBody {
                    positions,
                    prev_positions,
                    velocities,
                    masses,
                    ..
                } => {
                    // Use SIMD operations when available
                    #[cfg(target_feature = "simd")]
                    {
                        use std::simd::*;
                        for i in (0..positions.len()).step_by(4) {
                            if i + 4 <= positions.len() {
                                let pos = Float32x4::from_slice(&[
                                    positions[i].x,
                                    positions[i + 1].x,
                                    positions[i + 2].x,
                                    positions[i + 3].x,
                                ]);
                                let prev = Float32x4::from_slice(&[
                                    prev_positions[i].x,
                                    prev_positions[i + 1].x,
                                    prev_positions[i + 2].x,
                                    prev_positions[i + 3].x,
                                ]);
                                let vel = (pos - prev) / Float32x4::splat(delta_time);
                                vel.copy_to_slice(&mut [
                                    velocities[i].x,
                                    velocities[i + 1].x,
                                    velocities[i + 2].x,
                                    velocities[i + 3].x,
                                ]);
                            }
                        }
                    }

                    #[cfg(not(target_feature = "simd"))]
                    {
                        for i in 0..positions.len() {
                            velocities[i] = (positions[i] - prev_positions[i]) / delta_time;
                        }
                    }
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct Island {
    pub objects: Vec<usize>,
    pub constraints: Vec<usize>,
}

pub struct IslandSolver {
    islands: Vec<Island>,
    visited: Vec<bool>,
    island_connections: Vec<Vec<usize>>,
}

impl IslandSolver {
    pub fn new() -> Self {
        Self {
            islands: Vec::new(),
            visited: Vec::new(),
            island_connections: Vec::new(),
        }
    }

    pub fn build_islands(&mut self, world: &PhysicsWorld) -> Vec<Island> {
        self.islands.clear();
        self.visited = vec![false; world.objects.len()];
        self.island_connections = vec![Vec::new(); world.objects.len()];

        // Build connection graph
        for (i, constraint) in world.constraints.iter().enumerate() {
            for connected in self.get_connected_objects(constraint, world) {
                self.island_connections[connected].push(i);
            }
        }

        // Find connected components using parallel DFS
        let mut island_indices = Arc::new(Mutex::new(Vec::new()));

        (0..world.objects.len()).into_par_iter().for_each(|i| {
            if !self.visited[i] {
                let mut island = Island {
                    objects: Vec::new(),
                    constraints: Vec::new(),
                };
                self.parallel_dfs(i, world, &mut island);

                if !island.objects.is_empty() {
                    island_indices.lock().unwrap().push(island);
                }
            }
        });

        std::mem::take(&mut *island_indices.lock().unwrap())
    }

    fn parallel_dfs(&mut self, object_index: usize, world: &PhysicsWorld, island: &mut Island) {
        if self.visited[object_index] {
            return;
        }

        self.visited[object_index] = true;
        island.objects.push(object_index);

        // Add all constraints connected to this object
        for &constraint_index in &self.island_connections[object_index] {
            island.constraints.push(constraint_index);

            // Recursively visit connected objects
            for connected in self.get_connected_objects(&world.constraints[constraint_index], world)
            {
                if !self.visited[connected] {
                    self.parallel_dfs(connected, world, island);
                }
            }
        }
    }

    fn get_connected_objects(
        &self,
        constraint: &Box<dyn Constraint>,
        _world: &PhysicsWorld,
    ) -> Vec<usize> {
        // This is a placeholder - implement based on your constraint system
        Vec::new()
    }
}
