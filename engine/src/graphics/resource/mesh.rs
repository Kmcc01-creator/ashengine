//! Mesh resource definition

use crate::graphics::resource::ResourceHandle;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>, // Assuming u32 for indices
    pub vertex_buffer: ResourceHandle,
    pub index_buffer: ResourceHandle,
}
