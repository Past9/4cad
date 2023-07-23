mod attachment;
mod framebuffer;
mod pipeline;
mod program;
mod render_pass;
mod shaders;
mod subpass;

use attachment::*;
use framebuffer::*;
use pipeline::*;
use program::*;
use render_pass::*;
use shaders::*;
use subpass::*;

pub struct Vulkan {}
impl Vulkan {
    pub fn execute(program: &crate::spec::Program) {
        let program = program::Program::build(program);

        // TODO execute built program
    }
}

trait IsMultisampled {
    fn is_multisampled(&self) -> bool;
}

impl IsMultisampled for vulkano::image::SampleCount {
    fn is_multisampled(&self) -> bool {
        match self {
            vulkano::image::SampleCount::Sample1 => true,
            _ => false,
        }
    }
}
