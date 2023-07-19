mod run;
mod spec;

pub use run::Vulkan;
pub use spec::*;

pub use vulkano;
pub use vulkano_shaders;

#[macro_export]
macro_rules! shader {
    ($content:tt) => {
        use crate::vulkano as vulkano;
        use crate::vulkano_shaders as vulkano_shaders;
        vulkano_shaders::shader! $content
    };
}
