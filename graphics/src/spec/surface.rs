#[derive(Debug)]
pub enum Surface<'a> {
    Window {
        winit_window: &'a winit::window::Window,
    },
}
impl<'a> Surface<'a> {
    pub fn on_window(window: &'a winit::window::Window) -> Self {
        Self::Window {
            winit_window: window,
        }
    }

    pub fn renderable_size(&self) -> (u32, u32) {
        match self {
            Surface::Window { winit_window } => {
                let inner_size = winit_window.inner_size();
                (inner_size.width, inner_size.height)
            }
        }
    }
}
