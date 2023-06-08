use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use egui_winit_vulkano::{CallbackContext, RenderResources};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    descriptor_set::{
        self, allocator::StandardDescriptorSetAllocator, DescriptorSet, PersistentDescriptorSet,
        WriteDescriptorSet,
    },
    image::ImageViewAbstract,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, StateMode,
    },
};

pub struct EguiTransfer {
    pipeline: Arc<GraphicsPipeline>,
    quad: Arc<CpuAccessibleBuffer<[EguiVertex]>>,
    descriptor_set: Arc<PersistentDescriptorSet>,
}
impl EguiTransfer {
    pub fn new<'a>(resources: &RenderResources<'a>) -> Self {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            &resources.memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            [
                EguiVertex {
                    position: [-1.0, -1.0],
                },
                EguiVertex {
                    position: [-1.0, 1.0],
                },
                EguiVertex {
                    position: [1.0, -1.0],
                },
                EguiVertex {
                    position: [1.0, -1.0],
                },
                EguiVertex {
                    position: [-1.0, 1.0],
                },
                EguiVertex {
                    position: [1.0, 1.0],
                },
            ],
        )
        .unwrap();

        let pipeline = {
            let queue = resources.queue.clone();
            let subpass = resources.subpass.clone();

            let egui_vs =
                egui_vs::load(queue.device().clone()).expect("failed to create vertex shader");
            let egui_fs =
                egui_fs::load(queue.device().clone()).expect("failed to create fragment shader");

            GraphicsPipeline::start()
                .input_assembly_state(InputAssemblyState::new())
                .vertex_input_state(BuffersDefinition::new().vertex::<EguiVertex>())
                .vertex_shader(egui_vs.entry_point("main").unwrap(), ())
                .fragment_shader(egui_fs.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_dynamic(1))
                .rasterization_state(RasterizationState {
                    front_face: StateMode::Fixed(FrontFace::CounterClockwise),
                    cull_mode: StateMode::Fixed(CullMode::None),
                    ..Default::default()
                })
                .color_blend_state(
                    ColorBlendState::new(subpass.num_color_attachments()).blend_alpha(),
                )
                .render_pass(subpass.clone())
                .build(queue.device().clone())
                .unwrap()
        };

        // Create an empty descriptor set. On the first call to Self::transfer(),
        // this will be immediately replaced by one actually bound to the ImageView
        // that we'll be transferring.
        let descriptor_set = PersistentDescriptorSet::new(
            resources.descriptor_set_allocator,
            pipeline.layout().set_layouts().get(0).unwrap().clone(),
            [],
        )
        .unwrap();

        Self {
            pipeline,
            quad: vertex_buffer,
            descriptor_set,
        }
    }

    pub(crate) fn transfer(&mut self, view: Arc<dyn ImageViewAbstract>, ctx: &mut CallbackContext) {
        ctx.builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.descriptor_set(view, ctx.resources.descriptor_set_allocator),
            )
            .bind_vertex_buffers(0, self.quad.clone())
            .draw(self.quad.len() as u32, 1, 0, 0)
            .unwrap();
    }

    fn descriptor_set(
        &mut self,
        view: Arc<dyn ImageViewAbstract>,
        allocator: &StandardDescriptorSetAllocator,
    ) -> Arc<PersistentDescriptorSet> {
        // We need to ensure that our descriptor set is bound to an ImageView, and
        // that that ImageView points to the same data passed in to `view`. This may
        // not be the case if the renderer that generated `view` has rebuilt its
        // framebuffers (due to reposition or resizing, for example).
        let binding = self.descriptor_set.resources().binding(0).unwrap();
        let same_image = match binding {
            descriptor_set::DescriptorBindingResources::ImageView(elements) => {
                if let Some(element) = &elements.get(0) {
                    if let Some(element) = element.as_ref() {
                        element == &view
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        // If the reference is still good, return it.
        if same_image {
            return self.descriptor_set.clone();
        }

        // If the reference was bad or we haven't created a descriptor set yet, create a
        // new descriptor set with `view` bound to it.
        let ds = PersistentDescriptorSet::new(
            allocator,
            self.pipeline.layout().set_layouts().get(0).unwrap().clone(),
            [WriteDescriptorSet::image_view(0, view)],
        )
        .unwrap();

        // Cache the descriptor set
        self.descriptor_set = ds.clone();

        // Return it
        ds
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
struct EguiVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(EguiVertex, position);

mod egui_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod egui_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

// The final rendered scene.
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;

layout(location = 0) out vec4 f_color;

void main() {
    // Load the value at the current pixel.
    f_color.rgba = subpassLoad(u_diffuse).rgba;
}",
        types_meta: {
            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}
