use eframe::egui::{self, Id};
use egui_winit_vulkano::Gui;
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

#[derive(Debug)]
pub struct Ui {
    show_points: bool,
    show_edges: bool,
    show_surfaces: bool,
}
impl Ui {
    pub fn new() -> Self {
        Self {
            show_points: true,
            show_edges: true,
            show_surfaces: true,
        }
    }
}
impl Window for Ui {
    fn draw(&mut self, gui: &mut Gui) {
        //println!("UI: {:?}", self);

        gui.immediate_ui(|gui| {
            let ctx = &gui.egui_ctx;

            egui::SidePanel::new(egui::panel::Side::Left, Id::new("side-panel")).show(ctx, |ui| {
                ui.set_width(150.0);
                ui.vertical_centered(|ui| ui.heading("View options"));
                ui.separator();

                if ui.checkbox(&mut self.show_points, "Show points").changed() {
                    println!("Show points: {}", self.show_points);
                };

                if ui.checkbox(&mut self.show_edges, "Show edges").changed() {
                    println!("Show edges: {}", self.show_edges);
                };

                if ui
                    .checkbox(&mut self.show_surfaces, "Show surfaces")
                    .changed()
                {
                    println!("Show surfaces: {}", self.show_surfaces);
                };

                ui.separator();

                if ui.button("Reset camera").clicked() {
                    println!("Reset camera");
                };
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                //self.workspace.show(ctx, ui, &mut self.messages);
            });
        });
    }
}
