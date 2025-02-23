//! Pipeline state creation helpers
//!
//! Implementation of various pipeline state creation methods

use super::config::*;
use ash::vk;

/// Create vertex input state from configuration
pub fn create_vertex_input_state(config: &VertexConfig) -> vk::PipelineVertexInputStateCreateInfo {
    let binding_descriptions: Vec<_> = config
        .bindings
        .iter()
        .map(|binding| {
            vk::VertexInputBindingDescription::builder()
                .binding(binding.binding)
                .stride(binding.stride)
                .input_rate(match binding.input_rate {
                    VertexInputRate::Vertex => vk::VertexInputRate::VERTEX,
                    VertexInputRate::Instance(_) => vk::VertexInputRate::INSTANCE,
                })
                .build()
        })
        .collect();

    let attribute_descriptions: Vec<_> = config
        .attributes
        .iter()
        .map(|attr| {
            vk::VertexInputAttributeDescription::builder()
                .location(attr.location)
                .binding(attr.binding)
                .format(attr.format)
                .offset(attr.offset)
                .build()
        })
        .collect();

    vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions)
        .build()
}

/// Create rasterization state from configuration
pub fn create_rasterization_state(
    config: &RasterizationConfig,
) -> vk::PipelineRasterizationStateCreateInfo {
    let mut builder = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(config.polygon_mode)
        .cull_mode(config.cull_mode)
        .front_face(config.front_face)
        .line_width(config.line_width);

    if let Some((constant, clamp, slope)) = config.depth_bias {
        builder = builder
            .depth_bias_enable(true)
            .depth_bias_constant_factor(constant)
            .depth_bias_clamp(clamp)
            .depth_bias_slope_factor(slope);
    }

    builder.build()
}

/// Create multisample state from configuration
pub fn create_multisample_state(
    config: &MultisampleConfig,
) -> vk::PipelineMultisampleStateCreateInfo {
    let mut builder = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(config.samples)
        .sample_shading_enable(config.sample_shading.is_some());

    if let Some(min_sample_shading) = config.sample_shading {
        builder = builder.min_sample_shading(min_sample_shading);
    }

    if let Some(mask) = config.sample_mask {
        builder = builder.sample_mask(&[mask]);
    }

    builder.build()
}

/// Create depth stencil state from configuration
pub fn create_depth_stencil_state(config: &DepthConfig) -> vk::PipelineDepthStencilStateCreateInfo {
    let mut builder = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(config.test_enable)
        .depth_write_enable(config.write_enable)
        .depth_compare_op(config.compare_op);

    if let Some((min, max)) = config.bounds_test {
        builder = builder
            .depth_bounds_test_enable(true)
            .min_depth_bounds(min)
            .max_depth_bounds(max);
    }

    builder.build()
}

/// Create color blend state from configuration
pub fn create_color_blend_state(mode: &BlendMode) -> vk::PipelineColorBlendStateCreateInfo {
    let attachment = match mode {
        BlendMode::None => vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build(),

        BlendMode::Alpha => vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build(),

        BlendMode::Add => vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build(),

        BlendMode::Multiply => vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::DST_COLOR)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::DST_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build(),

        BlendMode::Custom {
            src_color,
            dst_color,
            color_op,
            src_alpha,
            dst_alpha,
            alpha_op,
        } => vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(*src_color)
            .dst_color_blend_factor(*dst_color)
            .color_blend_op(*color_op)
            .src_alpha_blend_factor(*src_alpha)
            .dst_alpha_blend_factor(*dst_alpha)
            .alpha_blend_op(*alpha_op)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build(),
    };

    vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&[attachment])
        .build()
}

/// Create dynamic state from configuration
pub fn create_dynamic_state(state: &DynamicState) -> vk::PipelineDynamicStateCreateInfo {
    vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&state.states)
        .build()
}
