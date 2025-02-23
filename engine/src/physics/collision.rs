use crate::physics::physics::PhysicsObject;
use glam::{Quat, Vec3, Vec4};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct CollisionManifold {
    pub normal: Vec3,
    pub penetration: f32,
    pub contact_points: Vec<Vec3>,
}

#[derive(Debug)]
pub struct BoundingVolume {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub orientation: Quat,
}

impl BoundingVolume {
    pub fn from_aabb(position: Vec3, size: Vec4, orientation: Quat) -> Self {
        Self {
            center: position,
            half_extents: Vec3::splat(size.w),
            orientation,
        }
    }

    pub fn intersects(&self, other: &BoundingVolume) -> bool {
        // Transform to A's local space
        let rel_center = other.center - self.center;
        let rel_orientation = self.orientation.conjugate() * other.orientation;

        // Rotation matrices
        let ra = Mat3::from_quat(self.orientation);
        let rb = Mat3::from_quat(other.orientation);

        // Compute rotation matrix from b to a
        let r = ra.transpose() * rb;
        let abs_r = r.abs();

        // Vector from center a to center b in a's frame
        let t = ra.transpose() * rel_center;

        // Test axes L = A0, A1, A2
        for i in 0..3 {
            let ra = self.half_extents[i];
            let rb = other.half_extents.dot(abs_r.row(i));
            if (t[i]).abs() > ra + rb {
                return false;
            }
        }

        // Test axes L = B0, B1, B2
        for i in 0..3 {
            let ra = self.half_extents.dot(abs_r.col(i));
            let rb = other.half_extents[i];
            let t_axis = t.dot(r.col(i));
            if t_axis.abs() > ra + rb {
                return false;
            }
        }

        true
    }
}

#[derive(Copy, Clone)]
struct Mat3 {
    cols: [Vec3; 3],
}

impl Mat3 {
    fn from_quat(q: Quat) -> Self {
        let x2 = q.x * 2.0;
        let y2 = q.y * 2.0;
        let z2 = q.z * 2.0;
        let xx = q.x * x2;
        let xy = q.x * y2;
        let xz = q.x * z2;
        let yy = q.y * y2;
        let yz = q.y * z2;
        let zz = q.z * z2;
        let wx = q.w * x2;
        let wy = q.w * y2;
        let wz = q.w * z2;

        Mat3 {
            cols: [
                Vec3::new(1.0 - (yy + zz), xy + wz, xz - wy),
                Vec3::new(xy - wz, 1.0 - (xx + zz), yz + wx),
                Vec3::new(xz + wy, yz - wx, 1.0 - (xx + yy)),
            ],
        }
    }

    fn transpose(&self) -> Self {
        Mat3 {
            cols: [
                Vec3::new(self.cols[0].x, self.cols[1].x, self.cols[2].x),
                Vec3::new(self.cols[0].y, self.cols[1].y, self.cols[2].y),
                Vec3::new(self.cols[0].z, self.cols[1].z, self.cols[2].z),
            ],
        }
    }

    fn row(&self, i: usize) -> Vec3 {
        Vec3::new(self.cols[0][i], self.cols[1][i], self.cols[2][i])
    }

    fn col(&self, i: usize) -> Vec3 {
        self.cols[i]
    }

    fn abs(&self) -> Self {
        Mat3 {
            cols: [self.cols[0].abs(), self.cols[1].abs(), self.cols[2].abs()],
        }
    }
}

pub fn detect_collision(obj1: &PhysicsObject, obj2: &PhysicsObject) -> Option<CollisionManifold> {
    match (obj1, obj2) {
        (
            PhysicsObject::RigidBody {
                position: p1,
                orientation: o1,
                bounding_box: bb1,
                ..
            },
            PhysicsObject::RigidBody {
                position: p2,
                orientation: o2,
                bounding_box: bb2,
                ..
            },
        ) => detect_rigid_rigid_collision(p1, o1, bb1, p2, o2, bb2),

        (
            PhysicsObject::DeformableBody {
                positions: p1s,
                tetrahedra: t1s,
                bounding_box: bb1,
                ..
            },
            PhysicsObject::DeformableBody {
                positions: p2s,
                tetrahedra: t2s,
                bounding_box: bb2,
                ..
            },
        ) => detect_soft_soft_collision(p1s, t1s, bb1, p2s, t2s, bb2),

        (
            PhysicsObject::RigidBody {
                position,
                orientation,
                bounding_box,
                ..
            },
            PhysicsObject::DeformableBody {
                positions,
                tetrahedra,
                ..
            },
        ) => {
            detect_rigid_soft_collision(position, orientation, bounding_box, positions, tetrahedra)
        }

        (
            PhysicsObject::DeformableBody {
                positions,
                tetrahedra,
                ..
            },
            PhysicsObject::RigidBody {
                position,
                orientation,
                bounding_box,
                ..
            },
        ) => {
            // Swap normal direction for proper collision response
            detect_rigid_soft_collision(position, orientation, bounding_box, positions, tetrahedra)
                .map(|mut manifold| {
                    manifold.normal = -manifold.normal;
                    manifold
                })
        }
    }
}

fn detect_rigid_rigid_collision(
    p1: &Vec3,
    o1: &Quat,
    bb1: &Vec4,
    p2: &Vec3,
    o2: &Quat,
    bb2: &Vec4,
) -> Option<CollisionManifold> {
    let bv1 = BoundingVolume::from_aabb(*p1, *bb1, *o1);
    let bv2 = BoundingVolume::from_aabb(*p2, *bb2, *o2);

    if !bv1.intersects(&bv2) {
        return None;
    }

    // Generate contact points using SAT (Separating Axis Theorem)
    let rel_pos = *p2 - *p1;
    let r1 = Mat3::from_quat(*o1);
    let r2 = Mat3::from_quat(*o2);

    // Find axis of least penetration
    let mut min_penetration = f32::INFINITY;
    let mut collision_normal = Vec3::ZERO;
    let axes = [
        r1.col(0),
        r1.col(1),
        r1.col(2),
        r2.col(0),
        r2.col(1),
        r2.col(2),
    ];

    for axis in &axes {
        let axis = axis.normalize();
        let p1_proj = project_box(*p1, bb1.w, *o1, axis);
        let p2_proj = project_box(*p2, bb2.w, *o2, axis);

        let dist = p2_proj.0 - p1_proj.1;
        if dist > 0.0 {
            return None;
        }

        let penetration = -dist;
        if penetration < min_penetration {
            min_penetration = penetration;
            collision_normal = axis;
        }
    }

    // Generate contact points
    let mut contact_points = Vec::new();
    let corners = generate_box_corners(*p1, bb1.w, *o1);

    for corner in corners {
        if point_in_box(corner, *p2, bb2.w, *o2) {
            contact_points.push(corner);
        }
    }

    let corners = generate_box_corners(*p2, bb2.w, *o2);
    for corner in corners {
        if point_in_box(corner, *p1, bb1.w, *o1) {
            contact_points.push(corner);
        }
    }

    if !contact_points.is_empty() {
        Some(CollisionManifold {
            normal: collision_normal,
            penetration: min_penetration,
            contact_points,
        })
    } else {
        None
    }
}

fn detect_soft_soft_collision(
    p1s: &[Vec3],
    t1s: &[[usize; 4]],
    bb1: &Vec4,
    p2s: &[Vec3],
    t2s: &[[usize; 4]],
    bb2: &Vec4,
) -> Option<CollisionManifold> {
    let mut contact_points = Vec::new();
    let mut total_normal = Vec3::ZERO;
    let mut max_penetration = 0.0;

    // Check each vertex of body 1 against each tetrahedron of body 2
    for p1 in p1s {
        for tet in t2s {
            let tet_points = [p2s[tet[0]], p2s[tet[1]], p2s[tet[2]], p2s[tet[3]]];

            if let Some((penetration, normal)) = point_in_tetrahedron(*p1, &tet_points) {
                contact_points.push(*p1);
                total_normal += normal;
                max_penetration = max_penetration.max(penetration);
            }
        }
    }

    // Check each vertex of body 2 against each tetrahedron of body 1
    for p2 in p2s {
        for tet in t1s {
            let tet_points = [p1s[tet[0]], p1s[tet[1]], p1s[tet[2]], p1s[tet[3]]];

            if let Some((penetration, normal)) = point_in_tetrahedron(*p2, &tet_points) {
                contact_points.push(*p2);
                total_normal += normal;
                max_penetration = max_penetration.max(penetration);
            }
        }
    }

    if !contact_points.is_empty() {
        Some(CollisionManifold {
            normal: total_normal.normalize(),
            penetration: max_penetration,
            contact_points,
        })
    } else {
        None
    }
}

fn detect_rigid_soft_collision(
    rigid_pos: &Vec3,
    rigid_ori: &Quat,
    rigid_bb: &Vec4,
    soft_positions: &[Vec3],
    soft_tetrahedra: &[[usize; 4]],
) -> Option<CollisionManifold> {
    let mut contact_points = Vec::new();
    let mut total_normal = Vec3::ZERO;
    let mut max_penetration = 0.0;

    // Check soft body vertices against rigid body
    for &pos in soft_positions {
        if let Some((penetration, normal)) =
            point_box_distance(pos, *rigid_pos, rigid_bb.w, *rigid_ori)
        {
            contact_points.push(pos);
            total_normal += normal;
            max_penetration = max_penetration.max(penetration);
        }
    }

    // Check rigid body corners against soft body tetrahedra
    let corners = generate_box_corners(*rigid_pos, rigid_bb.w, *rigid_ori);
    for corner in corners {
        for tet in soft_tetrahedra {
            let tet_points = [
                soft_positions[tet[0]],
                soft_positions[tet[1]],
                soft_positions[tet[2]],
                soft_positions[tet[3]],
            ];

            if let Some((penetration, normal)) = point_in_tetrahedron(corner, &tet_points) {
                contact_points.push(corner);
                total_normal += normal;
                max_penetration = max_penetration.max(penetration);
            }
        }
    }

    if !contact_points.is_empty() {
        Some(CollisionManifold {
            normal: total_normal.normalize(),
            penetration: max_penetration,
            contact_points,
        })
    } else {
        None
    }
}

fn project_box(pos: Vec3, half_size: f32, ori: Quat, axis: Vec3) -> (f32, f32) {
    let corners = generate_box_corners(pos, half_size, ori);
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;

    for corner in corners {
        let proj = corner.dot(axis);
        min = min.min(proj);
        max = max.max(proj);
    }

    (min, max)
}

fn generate_box_corners(pos: Vec3, half_size: f32, ori: Quat) -> [Vec3; 8] {
    let local_corners = [
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(-1.0, -1.0, -1.0),
    ];

    let mut corners = [Vec3::ZERO; 8];
    for (i, corner) in local_corners.iter().enumerate() {
        corners[i] = pos + ori.mul_vec3(*corner * half_size);
    }
    corners
}

fn point_in_box(point: Vec3, box_pos: Vec3, half_size: f32, box_ori: Quat) -> bool {
    let local_point = box_ori.conjugate().mul_vec3(point - box_pos);
    local_point.x.abs() <= half_size
        && local_point.y.abs() <= half_size
        && local_point.z.abs() <= half_size
}

fn point_box_distance(
    point: Vec3,
    box_pos: Vec3,
    half_size: f32,
    box_ori: Quat,
) -> Option<(f32, Vec3)> {
    let local_point = box_ori.conjugate().mul_vec3(point - box_pos);
    let closest = local_point.clamp(Vec3::splat(-half_size), Vec3::splat(half_size));

    if local_point == closest {
        return None;
    }

    let normal = (local_point - closest).normalize();
    let penetration = (local_point - closest).length();

    Some((penetration, box_ori.mul_vec3(normal)))
}

fn point_in_tetrahedron(point: Vec3, tet_points: &[Vec3; 4]) -> Option<(f32, Vec3)> {
    let (v0, v1, v2, v3) = (
        tet_points[1] - tet_points[0],
        tet_points[2] - tet_points[0],
        tet_points[3] - tet_points[0],
        point - tet_points[0],
    );

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d02 = v0.dot(v2);
    let d11 = v1.dot(v1);
    let d12 = v1.dot(v2);
    let d22 = v2.dot(v2);
    let d30 = v3.dot(v0);
    let d31 = v3.dot(v1);
    let d32 = v3.dot(v2);

    let denom = d00 * d11 * d22 + 2.0 * d01 * d12 * d02
        - d02 * d11 * d02
        - d01 * d01 * d22
        - d00 * d12 * d12;
    if denom == 0.0 {
        return None;
    }

    let inv_denom = 1.0 / denom;
    let u = (d11 * d22 * d30 + d02 * d12 * d31 + d01 * d32 * d12
        - d02 * d11 * d32
        - d01 * d31 * d22
        - d12 * d12 * d30)
        * inv_denom;
    let v = (d00 * d22 * d31 + d02 * d32 * d01 + d02 * d30 * d12
        - d02 * d02 * d31
        - d00 * d32 * d12
        - d02 * d30 * d01)
        * inv_denom;
    let w = (d00 * d11 * d32 + d01 * d31 * d02 + d01 * d30 * d12
        - d02 * d11 * d30
        - d00 * d31 * d12
        - d01 * d01 * d32)
        * inv_denom;

    if u < 0.0 || v < 0.0 || w < 0.0 || u + v + w > 1.0 {
        return None;
    }

    let normal = v1.cross(v2).normalize();
    let penetration = point.dot(normal) - tet_points[0].dot(normal);

    Some((penetration.abs(), -normal.normalize()))
}
