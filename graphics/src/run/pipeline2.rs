use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::layout::DescriptorSetLayoutCreateInfo;
use vulkano::device::Device;
use vulkano::pipeline::graphics::GraphicsPipelineBuilder;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::layout::PushConstantRange;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::render_pass::RenderPassCreateInfo;
use vulkano::shader::DescriptorBindingRequirements;
use vulkano::shader::EntryPoint;
use vulkano::shader::ShaderModule;

use crate::run;
use crate::spec;

use super::framebuffer::Framebuffer;

pub(super) struct Pipeline {
    pipeline: Arc<GraphicsPipeline>,
}
impl Pipeline {
    pub fn build(
        spec: &spec::Pipeline,
        device: Arc<vulkano::device::Device>,
        cache: &mut run::ShaderCache,
        subpass: vulkano::render_pass::Subpass,
    ) -> Self {
        let mut pipeline = GraphicsPipeline::start();

        let vertex_shader: Arc<ShaderModule>;
        let pipeline = if let Some(path) = spec.vertex_shader.as_ref() {
            vertex_shader = cache.get_shader(device.clone(), path, run::ShaderUsage::Vertex);
            let entry_point = vertex_shader.entry_point("main").unwrap();
            pipeline.vertex_shader(entry_point, ())
        } else {
            pipeline
        };

        let fragment_shader: Arc<ShaderModule>;
        let pipeline = if let Some(path) = spec.fragment_shader.as_ref() {
            fragment_shader = cache.get_shader(device.clone(), path, run::ShaderUsage::Fragment);
            let entry_point = fragment_shader.entry_point("main").unwrap();
            pipeline.fragment_shader(entry_point, ())
        } else {
            pipeline
        };

        let pipeline = pipeline.render_pass(subpass).build(device.clone()).unwrap();

        Self { pipeline }
    }
}

/*
fn get_shader<'a>(
    path: &str,
    cache: &'a mut run::ShaderCache,
    device: Arc<Device>,
) -> (Arc<ShaderModule>, EntryPoint<'a>) {
    let shader = cache.get_shader(device.clone(), path);
    let entry_point = shader.entry_point("main").unwrap();
    (shader.clone(), entry_point)
}
 */
