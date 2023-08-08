mod run;
mod spec;

pub use run::Vulkan;
pub use spec::*;

//pub use vulkano;
//pub use vulkano::pipeline::graphics::vertex_input::Vertex;
//pub use vulkano_shaders;

use vulkano::*;

pub use vulkano::pipeline::graphics::vertex_input::Vertex;

#[macro_export]
macro_rules! shader {
    ($content:tt) => {
        use crate::vulkano;
        use crate::vulkano_shaders;
        vulkano_shaders::shader! $content
    };
}
