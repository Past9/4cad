use cgmath::{Point3, Quaternion, Vector2, Vector3};
use eframe::epaint::{PaintCallbackInfo, Pos2};
use egui_winit_vulkano::{CallbackContext, RenderResources};
use vulkano::image::SampleCount;

use render::{renderer::Renderer, scene::Scene, PixelViewport};

use super::{egui_transfer::EguiTransfer, ColorId, SceneObject};

pub(super) struct GuiRenderer {
    scene: Option<Scene>,
    internal: Option<InternalGuiRenderer>,
}
impl GuiRenderer {
    pub(super) fn empty(scene: Scene) -> Self {
        Self {
            scene: Some(scene),
            internal: None,
        }
    }

    pub(super) fn camera_vec_to(&self, location: Point3<f32>) -> Option<Vector3<f32>> {
        if let Some(ref renderer) = self.internal {
            Some(renderer.camera_vec_to(location))
        } else {
            None
        }
    }

    pub(super) fn viewport_size_at_dist(&self, dist: f32) -> Option<Vector2<f32>> {
        if let Some(ref renderer) = self.internal {
            Some(renderer.viewport_size_at_dist(dist))
        } else {
            None
        }
    }

    pub(super) fn read_color_id(&mut self, pos: Pos2) -> Option<ColorId> {
        if let Some(ref mut renderer) = self.internal {
            renderer.read_color_id(pos)
        } else {
            None
        }
    }

    pub(super) fn hover_object(&mut self, id: Option<ColorId>) {
        if let Some(ref mut renderer) = self.internal {
            renderer.hover_object(id)
        }
    }

    pub(super) fn toggle_select_object(&mut self, id: Option<ColorId>, exclusive: bool) {
        if let Some(ref mut renderer) = self.internal {
            renderer.toggle_select_object(id, exclusive);
        }
    }

    pub(super) fn get_object(&mut self, id: Option<ColorId>) -> Option<&SceneObject> {
        if let Some(ref mut renderer) = self.internal {
            renderer.get_object(id)
        } else {
            None
        }
    }

    pub(super) fn deselect_all_objects(&mut self) {
        if let Some(ref mut renderer) = self.internal {
            renderer.deselect_all_objects();
        }
    }

    pub(super) fn render(&mut self, info: &PaintCallbackInfo, ctx: &mut CallbackContext) {
        self.require_internal_mut(&ctx.resources).render(info, ctx);
    }

    pub(super) fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        if let Some(ref mut internal) = self.internal {
            internal
                .scene_renderer
                .scene_mut()
                .orientation_mut()
                .set_rotation(rotation);
        }
    }

    pub(super) fn set_position(&mut self, offset: Vector3<f32>) {
        if let Some(ref mut internal) = self.internal {
            internal
                .scene_renderer
                .scene_mut()
                .orientation_mut()
                .set_offset(offset);
        }
    }

    fn require_internal<'a>(&mut self, resources: &RenderResources<'a>) -> &InternalGuiRenderer {
        self.internal
            .get_or_insert_with(|| InternalGuiRenderer::new(self.scene.take().unwrap(), resources))
    }

    fn require_internal_mut<'a>(
        &mut self,
        resources: &RenderResources<'a>,
    ) -> &mut InternalGuiRenderer {
        self.internal
            .get_or_insert_with(|| InternalGuiRenderer::new(self.scene.take().unwrap(), resources))
    }
}

struct InternalGuiRenderer {
    scene_renderer: Renderer,
    transfer: EguiTransfer,
}
impl InternalGuiRenderer {
    fn new<'a>(scene: Scene, resources: &RenderResources<'a>) -> Self {
        let renderer = Renderer::new(
            scene,
            SampleCount::Sample8,
            &resources.memory_allocator,
            resources.queue.clone(),
        );
        let transfer = EguiTransfer::new(resources);

        Self {
            scene_renderer: renderer,
            transfer,
        }
    }

    fn camera_vec_to(&self, location: Point3<f32>) -> Vector3<f32> {
        self.scene_renderer.camera_vec_to(location)
    }

    fn viewport_size_at_dist(&self, dist: f32) -> Vector2<f32> {
        self.scene_renderer.viewport_size_at_dist(dist)
    }

    fn render(&mut self, info: &PaintCallbackInfo, ctx: &mut CallbackContext) {
        let vpip = info.viewport_in_pixels();

        self.scene_renderer.render(
            &PixelViewport {
                left: vpip.left_px as u32,
                top: vpip.top_px as u32,
                width: vpip.width_px as u32,
                height: vpip.height_px as u32,
            },
            &ctx.resources.memory_allocator,
            &ctx.resources.command_buffer_allocator,
            &ctx.resources.descriptor_set_allocator,
            ctx.resources.queue.clone(),
        );
        self.transfer.transfer(self.scene_renderer.view(), ctx);
    }

    fn read_color_id(&mut self, _pos: Pos2) -> Option<ColorId> {
        // todo
        None
    }

    fn hover_object(&mut self, _id: Option<ColorId>) {
        // todo
    }

    fn toggle_select_object(&mut self, _id: Option<ColorId>, _exclusive: bool) {
        // todo
    }

    fn get_object(&mut self, _id: Option<ColorId>) -> Option<&SceneObject> {
        // todo
        None
    }

    fn deselect_all_objects(&mut self) {
        // todo
    }

    fn scene_mut(&mut self) -> &mut Scene {
        self.scene_renderer.scene_mut()
    }

    fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.scene_renderer
            .scene_mut()
            .orientation_mut()
            .set_rotation(rotation);
    }

    fn set_offset(&mut self, offset: Vector3<f32>) {
        self.scene_renderer
            .scene_mut()
            .orientation_mut()
            .set_offset(offset)
    }
}
