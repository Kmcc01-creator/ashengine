use glam::{Quat, Vec3, Vec4};
use std::cell::RefCell;

use crate::physics::{
    collision::{detect_collision, BoundingVolume, CollisionManifold},
    constraints::Constraint,
    solver::{ConstraintSolver, IslandSolver},
    spatial::ParallelBroadPhase,
};

#[derive(Clone)]
pub enum PhysicsObject {
    RigidBody {
        position: Vec3,
        velocity: Vec3,
        acceleration: Vec3,
        orientation: Quat,
        angular_velocity: Vec3,
        angular_acceleration: Vec3,
        mass: f32,
        inertia_tensor: Vec3,
        bounding_box: Vec4,
    },
    DeformableBody {
        positions: Vec<Vec3>,
        prev_positions: Vec<Vec3>,
        velocities: Vec<Vec3>,
        masses: Vec<f32>,
        rest_volume: f32,
        volumes: Vec<f32>,
        tetrahedra: Vec<[usize; 4]>,
        bounding_box: Vec4,
    },
}

pub struct PhysicsWorld {
    pub objects: Vec<RefCell<PhysicsObject>>,
    pub gravity: Vec3,
    pub constraints: Vec<Box<dyn Constraint>>,
    pub num_iterations: usize,
    pub substeps: usize,
    pub broad_phase: ParallelBroadPhase,
    pub constraint_solver: ConstraintSolver,
    pub island_solver: IslandSolver,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        PhysicsWorld {
            objects: Vec::new(),
            gravity,
            constraints: Vec::new(),
            num_iterations: 10,
            substeps: 1,
            broad_phase: ParallelBroadPhase::new(),
            constraint_solver: ConstraintSolver::new(),
            island_solver: IslandSolver::new(),
        }
    }

    pub fn add_object(&mut self, object: PhysicsObject) {
        self.objects.push(RefCell::new(object));
    }

    pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) {
        self.constraints.push(constraint);
    }

    pub fn update(&mut self, delta_time: f32) {
        let sub_delta_time = delta_time / self.substeps as f32;
        for _ in 0..self.substeps {
            self.sub_update(sub_delta_time);
        }
    }

    fn sub_update(&mut self, delta_time: f32) {
        // Phase 1: Position update and external forces
        self.parallel_update_positions(delta_time);

        // Phase 2: Parallel broad-phase collision detection
        let aabb_pairs = self.gather_aabb_pairs();
        let potential_collisions = self.broad_phase.update(&aabb_pairs);

        // Phase 3: Parallel narrow-phase collision detection
        let collision_constraints = self.parallel_collision_detection(&potential_collisions);
        self.constraints.extend(collision_constraints);

        // Phase 4: Parallel constraint solving with islands
        self.constraint_solver.solve_constraints(self, delta_time);

        // Phase 5: Parallel velocity update
        self.parallel_update_velocities(delta_time);

        // Clean up temporary collision constraints
        self.constraints.retain(|c| !c.is_collision_constraint());
    }

    fn parallel_update_positions(&self, delta_time: f32) {
        use rayon::prelude::*;

        self.objects.par_iter().for_each(|obj| {
            let mut obj = obj.borrow_mut();
            match &mut *obj {
                PhysicsObject::RigidBody {
                    position,
                    velocity,
                    acceleration,
                    orientation,
                    angular_velocity,
                    angular_acceleration,
                    ..
                } => {
                    *velocity += self.gravity * delta_time;
                    *velocity += *acceleration * delta_time;
                    *position += *velocity * delta_time;

                    let angle = angular_velocity.length() * delta_time;
                    if angle != 0.0 {
                        let axis = *angular_velocity / angle;
                        let rotation = Quat::from_axis_angle(axis, angle);
                        *orientation = rotation * *orientation;
                        orientation.normalize();
                    }

                    *angular_velocity += *angular_acceleration * delta_time;
                    *acceleration = Vec3::ZERO;
                    *angular_acceleration = Vec3::ZERO;
                }
                PhysicsObject::DeformableBody {
                    positions,
                    prev_positions,
                    velocities,
                    ..
                } => {
                    positions
                        .par_iter_mut()
                        .zip(prev_positions.par_iter_mut())
                        .zip(velocities.par_iter_mut())
                        .for_each(|((pos, prev), vel)| {
                            *prev = *pos;
                            *vel += self.gravity * delta_time;
                            *pos += *vel * delta_time;
                        });
                }
            }
        });
    }

    fn gather_aabb_pairs(&self) -> Vec<(Vec3, Vec3)> {
        self.objects
            .par_iter()
            .map(|obj| {
                let obj = obj.borrow();
                match &*obj {
                    PhysicsObject::RigidBody {
                        position,
                        bounding_box,
                        orientation,
                        ..
                    } => {
                        let corners = [
                            orientation.mul_vec3(Vec3::new(1.0, 1.0, 1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(-1.0, 1.0, 1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(1.0, -1.0, 1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(1.0, 1.0, -1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(-1.0, -1.0, 1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(-1.0, 1.0, -1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(1.0, -1.0, -1.0) * bounding_box.w),
                            orientation.mul_vec3(Vec3::new(-1.0, -1.0, -1.0) * bounding_box.w),
                        ];

                        let min = corners
                            .iter()
                            .fold(Vec3::splat(f32::INFINITY), |acc, &v| acc.min(v + *position));
                        let max = corners
                            .iter()
                            .fold(Vec3::splat(f32::NEG_INFINITY), |acc, &v| {
                                acc.max(v + *position)
                            });
                        (min, max)
                    }
                    PhysicsObject::DeformableBody {
                        positions,
                        bounding_box,
                        ..
                    } => {
                        let center = positions.iter().sum::<Vec3>() / positions.len() as f32;
                        (
                            center - Vec3::splat(bounding_box.w),
                            center + Vec3::splat(bounding_box.w),
                        )
                    }
                }
            })
            .collect()
    }

    fn parallel_collision_detection(
        &self,
        potential_collisions: &[(usize, usize)],
    ) -> Vec<Box<dyn Constraint>> {
        use rayon::prelude::*;

        potential_collisions
            .par_iter()
            .filter_map(|&(i, j)| {
                let obj1 = self.objects[i].borrow();
                let obj2 = self.objects[j].borrow();

                detect_collision(&*obj1, &*obj2).map(|_| {
                    Box::new(crate::physics::constraints::CollisionConstraint::new(i, j))
                        as Box<dyn Constraint>
                })
            })
            .collect()
    }

    fn parallel_update_velocities(&self, delta_time: f32) {
        use rayon::prelude::*;

        self.objects.par_iter().for_each(|obj| {
            let mut obj = obj.borrow_mut();
            if let PhysicsObject::DeformableBody {
                positions,
                prev_positions,
                velocities,
                ..
            } = &mut *obj
            {
                positions
                    .par_iter()
                    .zip(prev_positions.par_iter())
                    .zip(velocities.par_iter_mut())
                    .for_each(|((pos, prev), vel)| {
                        *vel = (*pos - *prev) / delta_time;
                    });
            }
        });
    }
}

// Implement Clone for PhysicsWorld to support island-based solving
impl Clone for PhysicsWorld {
    fn clone(&self) -> Self {
        PhysicsWorld {
            objects: self.objects.clone(),
            gravity: self.gravity,
            constraints: self.constraints.iter().map(|c| c.clone_box()).collect(),
            num_iterations: self.num_iterations,
            substeps: self.substeps,
            broad_phase: ParallelBroadPhase::new(),
            constraint_solver: ConstraintSolver::new(),
            island_solver: IslandSolver::new(),
        }
    }
}
