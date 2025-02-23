use ash::vk;
use memoffset::offset_of;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
    pub element_id: u32,
}

impl TextVertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<TextVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        [
            // Position
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(TextVertex, position) as u32)
                .build(),
            // Texture Coordinates
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(TextVertex, tex_coord) as u32)
                .build(),
            // Color
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(TextVertex, color) as u32)
                .build(),
            // Element ID
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(3)
                .format(vk::Format::R32_UINT)
                .offset(offset_of!(TextVertex, element_id) as u32)
                .build(),
        ]
    }
}
