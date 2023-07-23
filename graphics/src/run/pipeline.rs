use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::layout::DescriptorSetLayoutCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::layout::PushConstantRange;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::render_pass::RenderPassCreateInfo;
use vulkano::shader::DescriptorBindingRequirements;
use vulkano::shader::EntryPoint;

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
        shaders: &mut run::ShaderCache,
    ) -> Self {
        let mut descriptor_binding_requirements: HashMap<
            (u32, u32),
            DescriptorBindingRequirements,
        > = HashMap::new();

        if let Some(path) = spec.vertex_shader.as_ref() {
            let shader = shaders.get_shader(device.clone(), path);
            let entry_point = shader.entry_point("main").unwrap();
            for (loc, reqs) in entry_point.descriptor_binding_requirements() {
                match descriptor_binding_requirements.entry(loc) {
                    Entry::Occupied(entry) => {
                        //
                        entry.into_mut().merge(reqs).expect("Could not merge");
                    }
                    Entry::Vacant(entry) => {
                        //
                        entry.insert(reqs.clone());
                    }
                }
            }
        }

        let mut set_layout_create_infos = DescriptorSetLayoutCreateInfo::from_requirements(
            descriptor_binding_requirements
                .iter()
                .map(|(&loc, reqs)| (loc, reqs)),
        );

        let mut range_map = HashMap::new();
        if let Some(path) = spec.vertex_shader.as_ref() {
            let shader = shaders.get_shader(device.clone(), path);
            let entry_point = shader.entry_point("main").unwrap();
            if let Some(range) = entry_point.push_constant_requirements() {
                match range_map.entry((range.offset, range.size)) {
                    Entry::Vacant(entry) => {
                        //
                        entry.insert(range.stages);
                    }
                    Entry::Occupied(mut entry) => {
                        //
                        *entry.get_mut() = *entry.get() | range.stages;
                    }
                }
            }
        }

        let push_constant_ranges: Vec<_> = range_map
            .iter()
            .map(|((offset, size), stages)| PushConstantRange {
                stages: *stages,
                offset: *offset,
                size: *size,
            })
            .collect();

        let set_layouts = set_layout_create_infos
            .into_iter()
            .map(|desc| DescriptorSetLayout::new(device.clone(), desc))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts,
                push_constant_ranges,
                ..Default::default()
            },
        )
        .unwrap();

        TODO: Implement the logic in GraphicsPipelineBuilder::with_pipeline_layout(...)

        /*
        let vs = if let Some(path) = spec.vertex_shader.as_ref() {
            let shader = shaders.get_shader(device.clone(), path);
            let entry = shader.entry_point("main");
            Some(entry.clone())
        } else {
            None
        };

        let x = spec.vertex_shader.as_ref().map(|path| {
            shaders
                .get_shader(device.clone(), &path)
                .entry_point("main")
        });
         */
        //.and_then(|s| s.entry_point("main"));

        /*
            let stages: Vec<EntryPoint<'_>> = [
                spec.vertex_shader
                    .map(|path| {
                        shaders.get_shader(device.clone(), &path)
                        //.entry_point("main")
                    })
                    .and_then(|s| s.entry_point("main")),
                /*
                spec.fragment_shader.map(|path| {
                    shaders
                        .get_shader(device.clone(), &path)
                        .entry_point("main")
                }),
                */
            ]
            .into_iter()
            .filter_map(|s| s)
            //.flatten()
            .collect();
        */

        /*
        let vertex_shader = match spec.vertex_shader {
            Some(ref path) => shaders.get_shader(device.clone(), path).entry_point("main"),
            None => todo!(),
        };

        let vertex_shader = match spec.vertex_shader {
            Some(ref path) => shaders.get_shader(device.clone(), path),
            None => todo!(),
        };
         */

        Self { pipeline: todo!() }
    }
}
