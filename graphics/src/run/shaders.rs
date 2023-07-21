use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

use vulkano::{
    device::Device,
    shader::{EntryPoint, ShaderModule},
};

pub(super) struct ShaderCache {
    shaders: HashMap<String, Arc<ShaderModule>>,
}
impl ShaderCache {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    fn get_shader(&mut self, device: Arc<Device>, path: &str) -> Arc<ShaderModule> {
        self.shaders
            .entry(path.into())
            .or_insert_with(|| {
                let mut f = File::open(path).expect(&format!("Canot find file {}", path));
                let mut v = vec![];
                f.read_to_end(&mut v).unwrap();
                let module = unsafe { ShaderModule::from_bytes(device.clone(), &v).unwrap() };
                module
            })
            .clone()
    }
}
