use shaderc;
use std::{collections::HashMap, fs::File, io::Read, marker::PhantomData, sync::Arc};

use vulkano::{
    device::Device,
    shader::{EntryPoint, ShaderModule},
};

pub(super) struct ShaderCache {
    shaders: HashMap<(String, ShaderUsage), Arc<ShaderModule>>,
}
impl ShaderCache {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    pub fn get_shader(
        &mut self,
        device: Arc<Device>,
        path: &str,
        usage: ShaderUsage,
    ) -> Arc<ShaderModule> {
        self.shaders
            .entry((path.into(), usage))
            .or_insert_with(|| {
                let mut f = File::open(path).expect(&format!("Cannot find file {}", path));
                let mut source: String = "".into();
                f.read_to_string(&mut source).unwrap();

                println!(
                    "Compiling {} shader {}",
                    usage.to_string().to_ascii_lowercase(),
                    path
                );

                let compiler = shaderc::Compiler::new().unwrap();
                let bin = compiler
                    .compile_into_spirv(&source, shaderc::ShaderKind::Vertex, path, "main", None)
                    .unwrap();

                let module = unsafe {
                    ShaderModule::from_bytes(device.clone(), bin.as_binary_u8()).unwrap()
                };
                module
            })
            .clone()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum ShaderUsage {
    Vertex,
    Fragment,
}
impl ToString for ShaderUsage {
    fn to_string(&self) -> String {
        match self {
            ShaderUsage::Vertex => "Vertex",
            ShaderUsage::Fragment => "Fragment",
        }
        .to_string()
    }
}
impl From<ShaderUsage> for shaderc::ShaderKind {
    fn from(usage: ShaderUsage) -> Self {
        match usage {
            ShaderUsage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderUsage::Fragment => shaderc::ShaderKind::Fragment,
        }
    }
}
