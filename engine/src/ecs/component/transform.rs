//! Transform component for spatial information
//!
//! Provides position, rotation, and scale data for entities

use super::Component;
use glam::{Mat4, Quat, Vec3};

/// Component for entity spatial transformation
#[derive(Debug, Clone)]
pub struct TransformComponent {
    /// Position in 3D space
    pub position: Vec3,
    /// Rotation as a quaternion
    pub rotation: Quat,
    /// Scale in 3D space
    pub scale: Vec3,
    /// Cached transformation matrix
    cached_matrix: Mat4,
    /// Whether the cache needs updating
    dirty: bool,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            cached_matrix: Mat4::IDENTITY,
            dirty: true,
        }
    }
}

impl TransformComponent {
    /// Create a new transform component
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            cached_matrix: Mat4::IDENTITY,
            dirty: true,
        }
    }

    /// Get the transformation matrix
    pub fn matrix(&mut self) -> Mat4 {
        if self.dirty {
            self.update_matrix();
        }
        self.cached_matrix
    }

    /// Set the position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    /// Set the rotation
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.dirty = true;
    }

    /// Set the scale
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.dirty = true;
    }

    /// Update the cached transformation matrix
    fn update_matrix(&mut self) {
        self.cached_matrix =
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        self.dirty = false;
    }
}

impl Component for TransformComponent {}
