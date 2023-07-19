use crate::RenderPass;
use crate::Surface;

#[derive(Debug)]
pub struct Program<'a> {
    pub(crate) name: String,
    pub(crate) shader_clip_distance: u32,
    pub(crate) surface: Surface<'a>,
    pub(crate) render_passes: Vec<RenderPass>,
}
impl<'a> Program<'a> {
    pub fn new(surface: Surface<'a>) -> Self {
        Self {
            name: "Brimstone App".into(),
            surface,
            render_passes: vec![],
            shader_clip_distance: 1,
        }
    }

    pub fn on_window(window: &'a winit::window::Window) -> Self {
        Self::new(Surface::on_window(window))
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn render_pass(
        &mut self,
        build_render_pass: fn(render_pass: &mut RenderPass),
    ) -> &mut Self {
        let mut render_pass = RenderPass::new();
        build_render_pass(&mut render_pass);
        self.render_passes.push(render_pass);
        self
    }

    pub fn shader_clip_distance(&mut self, distance: u32) -> &mut Self {
        self.shader_clip_distance = distance;
        self
    }
}
