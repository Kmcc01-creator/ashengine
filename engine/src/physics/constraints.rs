use crate::physics::{collision::CollisionManifold, physics::PhysicsObject};
use glam::{Quat, Vec3, Vec4};
use std::cell::RefCell;

pub trait Constraint: Send + Sync {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>);
    fn clone_box(&self) -> Box<dyn Constraint>;
    fn is_collision_constraint(&self) -> bool {
        false
    }
}

pub struct DistanceConstraint {
    object1_index: usize,
    object2_index: usize,
    rest_distance: f32,
}

impl DistanceConstraint {
    pub fn new(object1_index: usize, object2_index: usize, rest_distance: f32) -> Self {
        DistanceConstraint {
            object1_index,
            object2_index,
            rest_distance,
        }
    }
}

impl Clone for DistanceConstraint {
    fn clone(&self) -> Self {
        DistanceConstraint {
            object1_index: self.object1_index,
            object2_index: self.object2_index,
            rest_distance: self.rest_distance,
        }
    }
}

impl Constraint for DistanceConstraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>) {
        let obj1 = objects[self.object1_index].borrow();
        let obj2 = objects[self.object2_index].borrow();

        match (&*obj1, &*obj2) {
            (
                PhysicsObject::RigidBody {
                    position: p1,
                    mass: m1,
                    ..
                },
                PhysicsObject::RigidBody {
                    position: p2,
                    mass: m2,
                    ..
                },
            ) => {
                let delta = *p2 - *p1;
                let distance = delta.length();
                if distance == 0.0 {
                    return;
                }

                let correction = delta * ((distance - self.rest_distance) / distance);
                let total_mass = m1 + m2;
                if total_mass == 0.0 {
                    return;
                }

                drop(obj1);
                drop(obj2);

                let mut obj1_mut = objects[self.object1_index].borrow_mut();
                let mut obj2_mut = objects[self.object2_index].borrow_mut();

                if let PhysicsObject::RigidBody { position, .. } = &mut *obj1_mut {
                    *position += correction * (*m2 / total_mass);
                }
                if let PhysicsObject::RigidBody { position, .. } = &mut *obj2_mut {
                    *position -= correction * (*m1 / total_mass);
                }
            }
            _ => (), // Other cases handled elsewhere
        }
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}

pub struct VolumeConstraint {
    object_index: usize,
    stiffness: f32,
}

impl VolumeConstraint {
    pub fn new(object_index: usize, stiffness: f32) -> Self {
        VolumeConstraint {
            object_index,
            stiffness: stiffness.clamp(0.0, 1.0),
        }
    }
}

impl Clone for VolumeConstraint {
    fn clone(&self) -> Self {
        VolumeConstraint {
            object_index: self.object_index,
            stiffness: self.stiffness,
        }
    }
}

impl Constraint for VolumeConstraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>) {
        let mut object = objects[self.object_index].borrow_mut();

        if let PhysicsObject::DeformableBody {
            positions,
            masses,
            volumes,
            tetrahedra,
            rest_volume,
            ..
        } = &mut *object
        {
            let mut total_volume = 0.0;

            // Calculate current volume and gradients
            for (i, tet) in tetrahedra.iter().enumerate() {
                let p0 = positions[tet[0]];
                let p1 = positions[tet[1]];
                let p2 = positions[tet[2]];
                let p3 = positions[tet[3]];

                let v1 = p1 - p0;
                let v2 = p2 - p0;
                let v3 = p3 - p0;

                volumes[i] = v1.cross(v2).dot(v3) / 6.0;
                total_volume += volumes[i];
            }

            let volume_error = total_volume - *rest_volume;
            if volume_error.abs() < 1e-6 {
                return;
            }

            // Apply volume correction
            for tet in tetrahedra.iter() {
                let p0 = positions[tet[0]];
                let p1 = positions[tet[1]];
                let p2 = positions[tet[2]];
                let p3 = positions[tet[3]];

                // Calculate volume gradients
                let grad0 = (p1 - p2).cross(p3 - p2) / 6.0;
                let grad1 = (p2 - p0).cross(p3 - p0) / 6.0;
                let grad2 = (p3 - p0).cross(p1 - p0) / 6.0;
                let grad3 = (p1 - p0).cross(p2 - p0) / 6.0;

                let w0 = 1.0 / masses[tet[0]];
                let w1 = 1.0 / masses[tet[1]];
                let w2 = 1.0 / masses[tet[2]];
                let w3 = 1.0 / masses[tet[3]];

                let sum_weights = w0 + w1 + w2 + w3;
                if sum_weights == 0.0 {
                    continue;
                }

                let lambda = -volume_error
                    / (grad0.length_squared() * w0
                        + grad1.length_squared() * w1
                        + grad2.length_squared() * w2
                        + grad3.length_squared() * w3);

                // Apply position corrections
                positions[tet[0]] += grad0 * lambda * w0 * self.stiffness;
                positions[tet[1]] += grad1 * lambda * w1 * self.stiffness;
                positions[tet[2]] += grad2 * lambda * w2 * self.stiffness;
                positions[tet[3]] += grad3 * lambda * w3 * self.stiffness;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}

pub struct CollisionConstraint {
    object1_index: usize,
    object2_index: usize,
    manifold: Option<CollisionManifold>,
    restitution: f32,
    friction: f32,
}

impl CollisionConstraint {
    pub fn new(object1_index: usize, object2_index: usize) -> Self {
        CollisionConstraint {
            object1_index,
            object2_index,
            manifold: None,
            restitution: 0.5,
            friction: 0.3,
        }
    }
}

impl Clone for CollisionConstraint {
    fn clone(&self) -> Self {
        CollisionConstraint {
            object1_index: self.object1_index,
            object2_index: self.object2_index,
            manifold: self.manifold.clone(),
            restitution: self.restitution,
            friction: self.friction,
        }
    }
}

impl Constraint for CollisionConstraint {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>) {
        let mut obj1 = objects[self.object1_index].borrow_mut();
        let mut obj2 = objects[self.object2_index].borrow_mut();

        if let Some(ref manifold) = self.manifold {
            match (&mut *obj1, &mut *obj2) {
                (
                    PhysicsObject::RigidBody {
                        position: p1,
                        velocity: v1,
                        angular_velocity: w1,
                        orientation: o1,
                        mass: m1,
                        inertia_tensor: i1,
                        ..
                    },
                    PhysicsObject::RigidBody {
                        position: p2,
                        velocity: v2,
                        angular_velocity: w2,
                        orientation: o2,
                        mass: m2,
                        inertia_tensor: i2,
                        ..
                    },
                ) => {
                    for contact_point in &manifold.contact_points {
                        let r1 = *contact_point - *p1;
                        let r2 = *contact_point - *p2;

                        // Calculate relative velocity at contact point
                        let v1_at_p = *v1 + w1.cross(r1);
                        let v2_at_p = *v2 + w2.cross(r2);
                        let rel_vel = v2_at_p - v1_at_p;

                        let vel_along_normal = rel_vel.dot(manifold.normal);

                        // Only resolve if objects are moving toward each other
                        if vel_along_normal < 0.0 {
                            // Calculate inverse mass and inertia
                            let w1 = if *m1 == 0.0 { 0.0 } else { 1.0 / m1 };
                            let w2 = if *m2 == 0.0 { 0.0 } else { 1.0 / m2 };

                            let i1_world = o1.mul_vec3(*i1);
                            let i2_world = o2.mul_vec3(*i2);

                            // Calculate angular factors
                            let angular1 = (i1_world * r1.cross(manifold.normal)).cross(r1);
                            let angular2 = (i2_world * r2.cross(manifold.normal)).cross(r2);

                            let angular_factor = angular1.dot(manifold.normal) * w1
                                + angular2.dot(manifold.normal) * w2;

                            // Calculate impulse
                            let j = -(1.0 + self.restitution) * vel_along_normal
                                / (w1 + w2 + angular_factor);

                            let impulse = manifold.normal * j;

                            // Apply linear impulse
                            *v1 -= impulse * w1;
                            *v2 += impulse * w2;

                            // Apply angular impulse
                            *w1 -= i1_world * r1.cross(impulse);
                            *w2 += i2_world * r2.cross(impulse);

                            // Friction
                            let tangent =
                                (rel_vel - manifold.normal * vel_along_normal).normalize_or_zero();
                            if tangent != Vec3::ZERO {
                                let friction_impulse = -tangent * j * self.friction;

                                *v1 -= friction_impulse * w1;
                                *v2 += friction_impulse * w2;

                                *w1 -= i1_world * r1.cross(friction_impulse);
                                *w2 += i2_world * r2.cross(friction_impulse);
                            }

                            // Positional correction
                            let percent = 0.2;
                            let correction = manifold.normal * (manifold.penetration * percent);
                            *p1 -= correction * w1;
                            *p2 += correction * w2;
                        }
                    }
                }
                _ => (), // Other cases handled in base implementation
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }

    fn is_collision_constraint(&self) -> bool {
        true
    }
}
