use ash::vk;
use std::sync::Arc;

use crate::{
    error::{Result, VulkanError},
    graphics::context::Context,
};

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    extent: vk::Extent2D,
    format: vk::Format,
    context: Arc<Context>,
}

impl Swapchain {
    pub fn new(
        context: Arc<Context>,
        surface: vk::SurfaceKHR,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let surface_capabilities = unsafe {
            context
                .surface_loader()
                .get_physical_device_surface_capabilities(context.physical_device(), surface)
                .map_err(|e| {
                    VulkanError::General(format!("Failed to get surface capabilities: {}", e))
                })?
        };

        let extent = vk::Extent2D {
            width: width.clamp(
                surface_capabilities.min_image_extent.width,
                surface_capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                surface_capabilities.min_image_extent.height,
                surface_capabilities.max_image_extent.height,
            ),
        };

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0 {
            desired_image_count = desired_image_count.min(surface_capabilities.max_image_count);
        }

        let surface_formats = unsafe {
            context
                .surface_loader()
                .get_physical_device_surface_formats(context.physical_device(), surface)
                .map_err(|e| {
                    VulkanError::General(format!("Failed to get surface formats: {}", e))
                })?
        };
        let surface_format = surface_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&surface_formats[0]);

        let present_modes = unsafe {
            context
                .surface_loader()
                .get_physical_device_surface_present_modes(context.physical_device(), surface)
                .map_err(|e| VulkanError::General(format!("Failed to get present modes: {}", e)))?
        };
        let present_mode = present_modes
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(*present_mode)
            .clipped(true);

        let swapchain = unsafe {
            context
                .swapchain_loader()
                .create_swapchain(&swapchain_create_info, None)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        let images = unsafe {
            context
                .swapchain_loader()
                .get_swapchain_images(swapchain)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        let mut image_views = Vec::with_capacity(images.len());
        for image in &images {
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = unsafe {
                context
                    .device()
                    .create_image_view(&image_view_create_info, None)
                    .map_err(|e| VulkanError::ImageViewCreation(e.to_string()))?
            };

            image_views.push(image_view);
        }

        Ok(Self {
            swapchain,
            images,
            image_views,
            extent,
            format: surface_format.format,
            context,
        })
    }

    pub fn recreate(&mut self, width: u32, height: u32, surface: vk::SurfaceKHR) -> Result<()> {
        // Get new surface capabilities
        let surface_capabilities = unsafe {
            self.context
                .surface_loader()
                .get_physical_device_surface_capabilities(self.context.physical_device(), surface)
                .map_err(|e| {
                    VulkanError::General(format!("Failed to get surface capabilities: {}", e))
                })?
        };

        let extent = vk::Extent2D {
            width: width.clamp(
                surface_capabilities.min_image_extent.width,
                surface_capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                surface_capabilities.min_image_extent.height,
                surface_capabilities.max_image_extent.height,
            ),
        };

        // Create new swapchain
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(self.images.len() as u32)
            .image_format(self.format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(self.swapchain);

        let new_swapchain = unsafe {
            self.context
                .swapchain_loader()
                .create_swapchain(&swapchain_create_info, None)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Clean up old resources
        unsafe {
            for &image_view in &self.image_views {
                self.context.device().destroy_image_view(image_view, None);
            }
            self.context
                .swapchain_loader()
                .destroy_swapchain(self.swapchain, None);
        }

        // Get new images
        let images = unsafe {
            self.context
                .swapchain_loader()
                .get_swapchain_images(new_swapchain)
                .map_err(|e| VulkanError::SwapchainCreation(e.to_string()))?
        };

        // Create new image views
        let mut image_views = Vec::with_capacity(images.len());
        for image in &images {
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = unsafe {
                self.context
                    .device()
                    .create_image_view(&image_view_create_info, None)
                    .map_err(|e| VulkanError::ImageViewCreation(e.to_string()))?
            };

            image_views.push(image_view);
        }

        // Update state
        self.swapchain = new_swapchain;
        self.images = images;
        self.image_views = image_views;
        self.extent = extent;

        Ok(())
    }

    pub fn acquire_next_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<(u32, bool)> {
        let result = unsafe {
            self.context
                .swapchain_loader()
                .acquire_next_image(self.swapchain, u64::MAX, semaphore, fence)
                .map_err(|e| match e {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => VulkanError::SwapchainOutOfDate,
                    _ => VulkanError::General(format!("Failed to acquire next image: {}", e)),
                })?
        };

        Ok(result)
    }

    pub fn present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.context
                .swapchain_loader()
                .queue_present(queue, &present_info)
                .map_err(|e| match e {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => VulkanError::SwapchainOutOfDate,
                    _ => VulkanError::General(format!("Failed to present queue: {}", e)),
                })
        }
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn surface_format(&self) -> vk::Format {
        self.format
    }

    pub fn image_views(&self) -> &[vk::ImageView] {
        &self.image_views
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in &self.image_views {
                self.context.device().destroy_image_view(image_view, None);
            }
            self.context
                .swapchain_loader()
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
