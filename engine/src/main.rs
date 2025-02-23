use std::path::PathBuf;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ashengine::{
    context::Context,
    error::{Result, VulkanError},
    render_pass::RenderPass,
    renderer::Renderer,
    shader::ShaderSet,
    swapchain::Swapchain,
    text::{FontAtlas, TextLayout},
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() -> Result<()> {
    pretty_env_logger::init();

    log::info!("Starting AshEngine...");

    // Create window
    log::info!("Creating window...");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Text Engine Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .map_err(|e| VulkanError::WindowError(e.to_string()))?;

    // Initialize Vulkan
    log::info!("Initializing Vulkan context...");
    let context = Arc::new(Context::new(Some(&window))?);
    let device = context.device();
    log::info!("Vulkan context initialized successfully");

    // Set up shader paths
    log::info!("Setting up shader paths...");
    let shader_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let vert_path = shader_dir.join("text.vert.spv");
    let frag_path = shader_dir.join("text.frag.spv");

    log::info!("Creating shader set with:");
    log::info!("Vertex shader: {:?}", vert_path);
    log::info!("Fragment shader: {:?}", frag_path);

    // Create shader set
    let shader_set = ShaderSet::new(device.clone(), &vert_path, &frag_path)?;
    let descriptor_set_layouts = Vec::new(); // No descriptor sets needed for text rendering yet

    // Create renderer
    log::info!("Creating renderer...");
    let mut renderer = Renderer::new(
        device.clone(),
        context.graphics_queue(),
        context.queue_family_index(),
        context.physical_device(),
        context.instance(),
        context.surface_loader(),
        context.surface(),
        shader_set,
        &descriptor_set_layouts,
    )?;

    // Create swapchain
    let swapchain = Swapchain::new(
        context.clone(),
        context.surface(),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )?;

    // Create render pass
    let render_pass = RenderPass::new(
        device.clone(),
        swapchain.surface_format(),
        swapchain.image_views(),
        swapchain.extent(),
    )?;

    // Initialize renderer with swapchain and render pass
    log::info!("Initializing swapchain...");
    renderer.initialize_swapchain(swapchain, render_pass)?;
    log::info!("Renderer created successfully");

    // Initialize text rendering components
    log::info!("Initializing text rendering components...");
    let _font_atlas = FontAtlas::new(context.clone(), 512, 512)?;
    let _text_layout = TextLayout::new();
    log::info!("Text rendering components initialized");

    // Main event loop
    log::info!("Entering main event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                log::info!("Window resized to: {}x{}", new_size.width, new_size.height);
                if let Err(e) = renderer.handle_resize([new_size.width, new_size.height]) {
                    log::error!("Failed to handle resize: {}", e);
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Err(e) = render_frame(&mut renderer) {
                    log::error!("Failed to render frame: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    })
}

fn render_frame(renderer: &mut Renderer) -> Result<()> {
    // Begin frame
    log::trace!("Beginning frame");
    renderer.begin_frame()?;

    // Submit frame
    log::trace!("Ending frame");
    renderer.end_frame()?;
    Ok(())
}
