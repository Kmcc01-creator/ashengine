use glam::Vec3;

pub struct Lighting {
    pub ambient_color: Vec3,
    pub ambient_intensity: f32,
    pub directional_lights: Vec<DirectionalLight>,
}

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

pub struct PointLight {
    // Placeholder for future use
}

pub struct SpotLight {
    // Placeholder for future use
}
