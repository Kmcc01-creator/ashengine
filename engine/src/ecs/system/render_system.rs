//! Unified render system for ECS integration with graphics backend
//!
//! Handles collection of renderable entities from ECS and interfaces directly
//! with the graphics system for efficient rendering.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::{
    ecs::{
        component::{RenderComponent, TransformComponent},
        System, SystemStage, World,
    },
    error::Result,
    graphics::{
        render::{PassType, RenderGraph},
        resource::{ResourceHandle, ResourceManager},
        Renderer,
    },
};

/// Batched render data for each pass type
#[derive(Default)]
struct PassBatches {
    /// Entities by material for optimal state changes
    render_batches: Vec<(
        ResourceHandle, // Material
        Vec<(TransformComponent, RenderComponent)>,
    )>,
    /// Dirty resources that need updating
    dirty_resources: HashSet<ResourceHandle>,
}

/// Unified render system that handles both ECS integration and graphics rendering
pub struct RenderSystem {
    renderer: Arc<Renderer>,
    resource_manager: Arc<ResourceManager>,
    render_graph: Arc<RenderGraph>,
    batches: HashMap<PassType, PassBatches>,
    frustum_culling: bool,
}

impl RenderSystem {
    /// Create a new render system
    pub fn new(
        renderer: Arc<Renderer>,
        resource_manager: Arc<ResourceManager>,
        render_graph: Arc<RenderGraph>,
    ) -> Self {
        Self {
            renderer,
            resource_manager,
            render_graph,
            batches: HashMap::new(),
            frustum_culling: true,
        }
    }

    /// Configure the system
    pub fn config() -> super::SystemConfig {
        super::SystemConfig {
            stage: SystemStage::Late, // Run after transform updates
            enabled: true,
            fixed_timestep: None,
        }
    }

    /// Enable/disable frustum culling
    pub fn set_frustum_culling(&mut self, enable: bool) {
        self.frustum_culling = enable;
    }

    /// Clear all batches
    fn clear_batches(&mut self) {
        self.batches.clear();
    }

    /// Add an entity to the appropriate render batch
    fn add_to_batch(
        batches: &mut Vec<(ResourceHandle, Vec<(TransformComponent, RenderComponent)>)>,
        material: Option<ResourceHandle>,
        transform: TransformComponent,
        renderer: RenderComponent,
    ) {
        let material_handle = material.unwrap_or_else(ResourceHandle::default);

        // Find existing batch or create new one
        if let Some(batch) = batches.iter_mut().find(|(mat, _)| *mat == material_handle) {
            batch.1.push((transform, renderer));
        } else {
            batches.push((material_handle, vec![(transform, renderer)]));
        }
    }

    /// Sort batches by material and render order
    fn sort_batches(&mut self) {
        for batches in self.batches.values_mut() {
            batches.render_batches.sort_by_key(|(material, entities)| {
                (
                    *material, // First by material for minimal state changes
                    entities.first().map(|(_, r)| r.sort_key()).unwrap_or(0), // Then by render order
                )
            });
        }
    }

    /// Update transform resources and record draw commands for a render pass
    fn process_pass_batches(&self, pass_type: PassType) -> Result<()> {
        if let Some(pass_batches) = self.batches.get(&pass_type) {
            // Begin the render pass
            self.render_graph.begin_pass(pass_type)?;

            // Process each material batch
            for (material, entities) in &pass_batches.render_batches {
                // Bind material resources
                if *material != ResourceHandle::default() {
                    self.renderer.bind_material(*material)?;
                }

                // Process each entity
                for (transform, renderer) in entities {
                    // Update transform if needed
                    if pass_batches
                        .dirty_resources
                        .contains(&renderer.transform_buffer())
                    {
                        if let Some(matrix) = transform.matrix() {
                            self.renderer.update_buffer(
                                renderer.transform_buffer(),
                                &matrix.to_cols_array(),
                                0,
                            )?;
                        }
                    }

                    // Draw the mesh
                    self.renderer.draw_mesh(renderer.mesh(), 1)?;
                }
            }

            // End the render pass
            self.render_graph.end_pass()?;
        }

        Ok(())
    }

    /// Collect and batch renderable entities
    fn collect_renderables(&mut self, world: &World) {
        for (_, (transform, renderer)) in world.query::<(&TransformComponent, &RenderComponent)>() {
            if !renderer.should_render() || (self.frustum_culling && !renderer.culling_enabled()) {
                continue;
            }

            // Get or create pass batches
            let pass_batches = self
                .batches
                .entry(renderer.pass_type())
                .or_insert_with(PassBatches::default);

            // Add to appropriate batch
            Self::add_to_batch(
                &mut pass_batches.render_batches,
                renderer.material(),
                transform.clone(),
                renderer.clone(),
            );

            // Mark transform buffer as dirty if needed
            if transform.is_dirty() {
                pass_batches
                    .dirty_resources
                    .insert(renderer.transform_buffer());
            }
        }
    }
}

impl System for RenderSystem {
    fn update(&mut self, world: &mut World) -> Result<()> {
        // Clear previous frame's batches
        self.clear_batches();

        // Collect renderable entities
        self.collect_renderables(world);

        // Sort batches for optimal rendering
        self.sort_batches();

        // Process each pass type in order
        for pass_type in &[
            PassType::Geometry,
            PassType::Lighting,
            PassType::PostProcess,
            PassType::UI,
        ] {
            self.process_pass_batches(*pass_type)?;
        }

        Ok(())
    }
}
