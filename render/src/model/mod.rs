use crate::Rgba;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::memory::allocator::MemoryAllocator;

mod edge;
mod material;
mod point;
mod surface;

pub use edge::*;
pub use material::*;
pub use point::*;
pub use surface::*;

pub struct GeometryBuffers {
    pub opaque_materials: Option<Arc<CpuAccessibleBuffer<[Std140OpaqueMaterial]>>>,
    pub opaque_surface_vertices: Option<Arc<CpuAccessibleBuffer<[BufferedSurfaceVertex]>>>,
    pub opaque_surface_indices: Option<Arc<CpuAccessibleBuffer<[u32]>>>,

    pub translucent_materials: Option<Arc<CpuAccessibleBuffer<[Std140TranslucentMaterial]>>>,
    pub translucent_surface_vertices: Option<Arc<CpuAccessibleBuffer<[BufferedSurfaceVertex]>>>,
    pub translucent_surface_indices: Option<Arc<CpuAccessibleBuffer<[u32]>>>,

    pub edge_vertices: Option<Arc<CpuAccessibleBuffer<[BufferedEdgeVertex]>>>,
    pub edge_indices: Option<Arc<CpuAccessibleBuffer<[u32]>>>,

    pub point_vertices: Option<Arc<CpuAccessibleBuffer<[BufferedPointVertex]>>>,
}

#[derive(Debug)]
pub struct Geometry {
    models: Vec<Model>,
    materials: MaterialSet,
}
impl Geometry {
    pub fn new() -> Self {
        Self {
            models: vec![],
            materials: MaterialSet::new(),
        }
    }

    pub fn insert_material(&mut self, reflect: Rgba, roughness: f32) -> MaterialId {
        self.materials.insert(reflect, roughness)
    }

    pub fn insert_model(&mut self, model: Model) {
        self.models.push(model)
    }

    pub fn build_buffers(&self, allocator: &(impl MemoryAllocator + ?Sized)) -> GeometryBuffers {
        let (opaque_surface_vertices, opaque_surface_indices) = Self::buffer_surfaces(
            allocator,
            self.models
                .iter()
                .flat_map(|model| model.surfaces.iter())
                .filter(|surface| surface.is_opaque()),
        );

        let (translucent_surface_vertices, translucent_surface_indices) = Self::buffer_surfaces(
            allocator,
            self.models
                .iter()
                .flat_map(|model| model.surfaces.iter())
                .filter(|surface| surface.is_translucent()),
        );

        let (edge_vertices, edge_indices) = self.buffer_edges(allocator);

        let point_vertices = self.buffer_points(allocator);

        GeometryBuffers {
            opaque_materials: self.materials.buffer_opaque(allocator),
            opaque_surface_vertices,
            opaque_surface_indices,

            translucent_materials: self.materials.buffer_translucent(allocator),
            translucent_surface_vertices,
            translucent_surface_indices,

            edge_vertices,
            edge_indices,

            point_vertices,
        }
    }

    fn buffer_surfaces<'a>(
        allocator: &(impl MemoryAllocator + ?Sized),
        surfaces: impl Iterator<Item = &'a ModelSurface>,
    ) -> (
        Option<Arc<CpuAccessibleBuffer<[BufferedSurfaceVertex]>>>,
        Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    ) {
        let mut vertices: Vec<BufferedSurfaceVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut index_offset = 0;
        for surface in surfaces {
            vertices.extend(
                surface
                    .vertices()
                    .iter()
                    .map(|vert| BufferedSurfaceVertex::new(vert, surface.material_id())),
            );

            indices.extend(surface.indices().iter().map(|i| i + index_offset));
            index_offset += surface.vertices().len() as u32;
        }

        if vertices.len() > 0 && indices.len() > 0 {
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                vertices,
            )
            .unwrap();

            let index_buffer = CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage {
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                indices,
            )
            .unwrap();

            (Some(vertex_buffer), Some(index_buffer))
        } else {
            (None, None)
        }
    }

    pub fn buffer_edges(
        &self,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> (
        Option<Arc<CpuAccessibleBuffer<[BufferedEdgeVertex]>>>,
        Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    ) {
        let mut vertices: Vec<BufferedEdgeVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut index = 0;
        for model in self.models.iter() {
            for edge in model.edges.iter() {
                let color = edge.color();
                for vertex in edge.vertices().iter() {
                    vertices.push(BufferedEdgeVertex::new(vertex, color));

                    indices.push(index);
                    index += 1;
                }
                indices.push(u32::MAX);
            }
        }

        if vertices.len() > 0 && indices.len() > 0 {
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                vertices,
            )
            .unwrap();

            let index_buffer = CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage {
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                indices,
            )
            .unwrap();

            (Some(vertex_buffer), Some(index_buffer))
        } else {
            (None, None)
        }
    }

    pub fn buffer_points(
        &self,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Option<Arc<CpuAccessibleBuffer<[BufferedPointVertex]>>> {
        let mut vertices: Vec<BufferedPointVertex> = Vec::new();

        for model in self.models.iter() {
            for point in model.points.iter() {
                vertices.push(BufferedPointVertex::new(point));
            }
        }

        if vertices.len() > 0 {
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                vertices,
            )
            .unwrap();

            Some(vertex_buffer)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    surfaces: Vec<ModelSurface>,
    edges: Vec<ModelEdge>,
    points: Vec<ModelPoint>,
}
impl Model {
    pub fn empty() -> Self {
        Self {
            surfaces: vec![],
            edges: vec![],
            points: vec![],
        }
    }

    pub fn surface(self, surface: ModelSurface) -> Self {
        let Self {
            mut surfaces,
            edges,
            points,
        } = self;

        surfaces.push(surface);

        Self {
            surfaces,
            edges,
            points,
        }
    }

    pub fn surfaces(self, new_surfaces: Vec<ModelSurface>) -> Self {
        let Self {
            mut surfaces,
            edges,
            points,
        } = self;

        surfaces.extend(new_surfaces);

        Self {
            surfaces,
            edges,
            points,
        }
    }

    pub fn edge(self, edge: ModelEdge) -> Self {
        let Self {
            surfaces,
            mut edges,
            points,
        } = self;

        edges.push(edge);

        Self {
            surfaces,
            edges,
            points,
        }
    }

    pub fn edges(self, new_edges: Vec<ModelEdge>) -> Self {
        let Self {
            surfaces,
            mut edges,
            points,
        } = self;

        edges.extend(new_edges);

        Self {
            surfaces,
            edges,
            points,
        }
    }

    pub fn point(self, point: ModelPoint) -> Self {
        let Self {
            surfaces,
            edges,
            mut points,
        } = self;

        points.push(point);

        Self {
            surfaces,
            edges,
            points,
        }
    }

    pub fn points(self, new_points: Vec<ModelPoint>) -> Self {
        let Self {
            surfaces,
            edges,
            mut points,
        } = self;

        points.extend(new_points);

        Self {
            surfaces,
            edges,
            points,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModelObjectId(u32);
impl From<u32> for ModelObjectId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
