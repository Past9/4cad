use std::{env, path::Path};

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub(crate) vertex_shader: Option<String>,
    pub(crate) fragment_shader: Option<String>,
    pub(crate) cull_front: bool,
    pub(crate) cull_back: bool,
    pub(crate) active: bool,
}
impl Pipeline {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            cull_front: false,
            cull_back: false,
            active: true,
        }
    }

    pub fn vertex_shader(&mut self, path: &str) -> &mut Self {
        self.vertex_shader = Some(make_absolute_path(path.into()));
        self
    }

    pub fn fragment_shader(&mut self, path: &str) -> &mut Self {
        self.fragment_shader = Some(make_absolute_path(path));
        self
    }

    pub fn cull_front(&mut self, cull: bool) -> &mut Self {
        self.cull_front = cull;
        self
    }

    pub fn cull_back(&mut self, cull: bool) -> &mut Self {
        self.cull_back = cull;
        self
    }

    pub fn active(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }
}

fn make_absolute_path(rel_path: &str) -> String {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let root_path = Path::new(&root).to_string_lossy();
    format!("{root_path}/{rel_path}")
}
