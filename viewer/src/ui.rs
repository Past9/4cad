use cgmath::vec3;
use components::scene::SceneViewer;
use eframe::egui::{self, Id};
use egui_winit_vulkano::Gui;
use render::{camera::CameraAngle, scene::Scene};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{event::Event, event_loop::ControlFlow};

pub trait Window {
    fn draw(&mut self, gui: &mut Gui);

    fn on_close(&mut self) -> bool {
        return true;
    }

    fn on_event(
        &mut self,
        _event: &Event<()>,
        _control_flow: &mut ControlFlow,
        _renderer: &mut VulkanoWindowRenderer,
        _gui: &mut Gui,
    ) {
        // Do nothing
    }
}

pub struct Ui {
    scene_viewer: SceneViewer,
    show_vectors: bool,
    show_points: bool,
    show_edges: bool,
    show_surfaces: bool,
}
impl Ui {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene_viewer: SceneViewer::new(
                CameraAngle::Front.get_rotation(),
                vec3(0.0, 0.0, 0.0),
                true,
                true,
                true,
                scene,
            ),
            show_vectors: true,
            show_points: true,
            show_edges: true,
            show_surfaces: true,
        }
    }
}
impl Window for Ui {
    fn draw(&mut self, gui: &mut Gui) {
        gui.immediate_ui(|gui| {
            let ctx = &gui.egui_ctx;

            egui::SidePanel::new(egui::panel::Side::Left, Id::new("side-panel")).show(ctx, |ui| {
                ui.set_width(150.0);
                ui.vertical_centered(|ui| ui.heading("View options"));
                ui.separator();

                if ui
                    .checkbox(&mut self.show_surfaces, "Show surfaces")
                    .changed()
                {
                    self.scene_viewer.set_show_surfaces(self.show_surfaces);
                };

                if ui.checkbox(&mut self.show_edges, "Show edges").changed() {
                    self.scene_viewer.set_show_edges(self.show_edges);
                };

                if ui.checkbox(&mut self.show_points, "Show points").changed() {
                    self.scene_viewer.set_show_points(self.show_points);
                };

                if ui
                    .checkbox(&mut self.show_vectors, "Show vectors")
                    .changed()
                {
                    self.scene_viewer.set_show_vectors(self.show_vectors);
                };

                ui.separator();

                if ui.button("Reset camera").clicked() {
                    self.scene_viewer.reset_camera();
                };
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                self.scene_viewer.show(ui);
            });
        });
    }
}
