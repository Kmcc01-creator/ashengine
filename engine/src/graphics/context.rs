use ash::{vk, Device, Entry, Instance};
use ash_window;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CString;
use std::sync::Arc;
use winit::window::Window;

use crate::error::{Result, VulkanError};

pub struct Context {
    _entry: Entry,
    instance: Arc<Instance>,
    device: Arc<Device>,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    surface_loader: Arc<ash::extensions::khr::Surface>,
    swapchain_loader: Arc<ash::extensions::khr::Swapchain>,
    queue_family_index: u32,
    graphics_queue: vk::Queue,
}

impl Context {
    pub fn new(window: Option<&Window>) -> Result<Self> {
        let entry = unsafe { Entry::load()? };

        // Create instance
        let app_name = CString::new("AshEngine")?;
        let engine_name = CString::new("AshEngine")?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_2);

        let mut instance_extensions = vec![ash::extensions::khr::Surface::name().as_ptr()];

        if let Some(window) = window {
            #[cfg(target_os = "windows")]
            {
                instance_extensions.push(ash::extensions::khr::Win32Surface::name().as_ptr());
            }
            #[cfg(target_os = "linux")]
            {
                instance_extensions.push(ash::extensions::khr::XlibSurface::name().as_ptr());
            }
            #[cfg(target_os = "macos")]
            {
                instance_extensions.push(ash::extensions::khr::MetalSurface::name().as_ptr());
            }
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extensions);

        let instance = unsafe {
            Arc::new(
                entry
                    .create_instance(&create_info, None)
                    .map_err(|e| VulkanError::InstanceCreation(e.to_string()))?,
            )
        };

        // Create surface if window is provided
        let (surface, surface_loader) = if let Some(window) = window {
            let surface_loader = Arc::new(ash::extensions::khr::Surface::new(&entry, &instance));
            let display_handle = window.raw_display_handle();
            let window_handle = window.raw_window_handle();

            let surface = unsafe {
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .map_err(|e| VulkanError::SurfaceCreation(e.to_string()))?
            };
            (surface, surface_loader)
        } else {
            (
                vk::SurfaceKHR::null(),
                Arc::new(ash::extensions::khr::Surface::new(&entry, &instance)),
            )
        };

        // Select physical device
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| VulkanError::DeviceCreation(e.to_string()))?
        };

        let physical_device = physical_devices
            .into_iter()
            .find(|&device| {
                let props = unsafe { instance.get_physical_device_properties(device) };
                props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                    || props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
            })
            .ok_or(VulkanError::NoSuitableGpu)?;

        // Find queue family
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let queue_family_index = queue_families
            .iter()
            .enumerate()
            .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(i, _)| i as u32)
            .ok_or(VulkanError::NoSuitableGpu)?;

        // Create logical device
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0]);

        let mut device_extensions = vec![ash::extensions::khr::Swapchain::name().as_ptr()];
        device_extensions.push(ash::extensions::khr::ShaderNonSemanticInfo::name().as_ptr());

        // Enable the DebugPrintf feature
        let mut shader_non_semantic_info_features =
            vk::PhysicalDeviceShaderNonSemanticInfoFeaturesKHR::builder()
                .shader_debug_printf(true)
                .build();

        let mut device_features = vk::PhysicalDeviceFeatures2::builder()
            .features(vk::PhysicalDeviceFeatures::default())
            .push_next(&mut shader_non_semantic_info_features) // Chain the feature struct
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_features(&device_features.features) // Pass features, not the struct itself
            .enabled_extension_names(&device_extensions)
            .push_next(&mut device_features) // Chain the features struct for 1.1+ compatibility
            .build();

        let device = unsafe {
            Arc::new(
                instance
                    .create_device(physical_device, &device_create_info, None)
                    .map_err(|e| VulkanError::DeviceCreation(e.to_string()))?,
            )
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let swapchain_loader = Arc::new(ash::extensions::khr::Swapchain::new(&instance, &device));

        Ok(Self {
            _entry: entry,
            instance,
            device,
            physical_device,
            surface,
            surface_loader,
            swapchain_loader,
            queue_family_index,
            graphics_queue,
        })
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    pub fn surface_loader(&self) -> Arc<ash::extensions::khr::Surface> {
        self.surface_loader.clone()
    }

    pub fn swapchain_loader(&self) -> Arc<ash::extensions::khr::Swapchain> {
        self.swapchain_loader.clone()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            if self.surface != vk::SurfaceKHR::null() {
                self.surface_loader.destroy_surface(self.surface, None);
            }
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
