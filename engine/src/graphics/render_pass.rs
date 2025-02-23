use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::sync::Arc;

pub struct RenderPass {
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    device: Arc<Device>,
    extent: vk::Extent2D,
}

impl RenderPass {
    pub fn new(
        device: Arc<Device>,
        format: vk::Format,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Self> {
        log::debug!("Creating render pass with format: {:?}", format);

        // Color attachment description
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        // Subpass configuration
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .build();

        // Update the subpass dependencies
        let dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        ];

        log::debug!(
            "Creating render pass with {} dependencies",
            dependencies.len()
        );
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .map_err(|e| VulkanError::RenderPassCreation(e.to_string()))?
        };
        log::debug!("Render pass created successfully");

        // Create framebuffers
        log::debug!(
            "Creating framebuffers for {} image views",
            image_views.len()
        );
        let framebuffers = Self::create_framebuffers(&device, render_pass, image_views, extent)?;
        log::debug!("Created {} framebuffers", framebuffers.len());

        Ok(Self {
            render_pass,
            framebuffers,
            device,
            extent,
        })
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
        extent: vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>> {
        log::debug!(
            "Creating framebuffers with extent: {}x{}",
            extent.width,
            extent.height
        );

        image_views
            .iter()
            .enumerate()
            .map(|(i, &image_view)| {
                let attachments = [image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    let framebuffer = device
                        .create_framebuffer(&framebuffer_info, None)
                        .map_err(|e| VulkanError::FramebufferCreation(e.to_string()))?;
                    log::debug!(
                        "Created framebuffer {} with dimensions: {}x{}",
                        i,
                        extent.width,
                        extent.height
                    );
                    Ok(framebuffer)
                }
            })
            .collect()
    }

    pub fn begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer_index: usize,
        extent: vk::Extent2D,
        clear_color: [f32; 4],
    ) {
        log::debug!(
            "Beginning render pass with framebuffer {} and extent: {}x{}",
            framebuffer_index,
            extent.width,
            extent.height
        );

        // Verify that framebuffer index is valid
        if framebuffer_index >= self.framebuffers.len() {
            log::error!("Invalid framebuffer index: {}", framebuffer_index);
            return;
        }

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[framebuffer_index])
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            log::debug!("Starting render pass command");
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            log::debug!("Render pass begin completed successfully");
        }
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        &self.framebuffers
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            log::debug!("Cleaning up render pass resources");
            for (i, &framebuffer) in self.framebuffers.iter().enumerate() {
                log::debug!("Destroying framebuffer {}", i);
                self.device.destroy_framebuffer(framebuffer, None);
            }
            log::debug!("Destroying render pass");
            self.device.destroy_render_pass(self.render_pass, None);
            log::debug!("Render pass cleanup complete");
        }
    }
}
