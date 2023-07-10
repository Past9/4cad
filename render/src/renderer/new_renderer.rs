use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer,
    image::{view::ImageView, AttachmentImage},
};

use crate::{
    lights::LightBuffers,
    model::{
        BufferedEdgeVertex, BufferedPointVertex, BufferedSurfaceVertex, Std140OpaqueMaterial,
        Std140TranslucentMaterial,
    },
};

use super::{surface_vs::PushConstants, SurfaceMode};

pub struct SubpassBuildParams {
    pub surface_mode: SurfaceMode,
}

pub struct SubpassRunParams<'a> {
    pub opaque_surface_push_constants: PushConstants,
    pub opaque_surface_vertices: &'a Option<Subbuffer<[BufferedSurfaceVertex]>>,
    pub opaque_surface_indices: &'a Option<Subbuffer<[u32]>>,
    pub opaque_surface_materials: &'a Option<Subbuffer<[Std140OpaqueMaterial]>>,

    pub translucent_surface_push_constants: PushConstants,
    pub translucent_surface_vertices: &'a Option<Subbuffer<[BufferedSurfaceVertex]>>,
    pub translucent_surface_indices: &'a Option<Subbuffer<[u32]>>,
    pub translucent_surface_materials: &'a Option<Subbuffer<[Std140TranslucentMaterial]>>,

    pub edge_vertices: &'a Option<Subbuffer<[BufferedEdgeVertex]>>,
    pub edge_indices: &'a Option<Subbuffer<[u32]>>,

    pub point_vertices: &'a Option<Subbuffer<[BufferedPointVertex]>>,

    pub light_buffers: &'a LightBuffers,

    pub show_surfaces: bool,
    pub show_edges: bool,
    pub show_points: bool,

    pub depth_image: Arc<ImageView<AttachmentImage>>,
}

pub struct NewRenderer {}
impl NewRenderer {
    pub fn new() -> Self {
        todo!()
    }
}
