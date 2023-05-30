use std::sync::Arc;

use bytemuck::Pod;
use bytemuck::Zeroable;
use cgmath::prelude::*;
use cgmath::Matrix4;
use cgmath::Quaternion;
use cgmath::Rad;
use cgmath::Rotation3;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::impl_vertex;

use crate::Mat4;
use crate::Point3;
use crate::Quat;
use crate::Vec3;
use crate::Vec3Utils;

use super::Line;
use super::Mesh;
use super::Point;
use super::Triangle;

pub struct Model {
    pub triangles: Vec<Triangle>,
    pub lines: Vec<Line>,
    pub points: Vec<Point>,
}
impl Mesh for Model {
    fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    fn lines(&self) -> &[Line] {
        &self.lines
    }

    fn points(&self) -> &[Point] {
        &self.points
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct SurfaceVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
impl_vertex!(SurfaceVertex, position, normal);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct EdgeVertex {
    pub position: [f32; 3],
    pub expand: [f32; 3],
}
impl_vertex!(EdgeVertex, position, expand);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct PointVertex {
    pub position: [f32; 3],
    pub expand: [f32; 3],
}
impl_vertex!(PointVertex, position, expand);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Normal {
    normal: [f32; 3],
}
impl From<Vec3> for Normal {
    fn from(v: Vec3) -> Self {
        Self {
            normal: [v.x as f32, v.y as f32, v.z as f32],
        }
    }
}
impl_vertex!(Normal, normal);

pub struct SurfaceVertexBuffers {
    pub vertices: Arc<CpuAccessibleBuffer<[SurfaceVertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u16]>>,
}

pub struct EdgeVertexBuffers {
    pub vertices: Arc<CpuAccessibleBuffer<[EdgeVertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u16]>>,
}

pub struct PointVertexBuffers {
    pub vertices: Arc<CpuAccessibleBuffer<[PointVertex]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u16]>>,
}

pub struct BufferedModel {
    pub surface: Option<SurfaceVertexBuffers>,
    pub edges: Option<EdgeVertexBuffers>,
    pub points: Option<PointVertexBuffers>,

    position: Point3,
    scale: (f32, f32, f32),
    rotation: Quaternion<f32>,
}
impl BufferedModel {
    pub fn from_mesh<M: Mesh>(device: Arc<Device>, mesh: M) -> Self {
        let mut surface_vertices: Vec<SurfaceVertex> = Vec::new();
        let mut surface_indices: Vec<u16> = Vec::new();

        for (i, triangle) in mesh.triangles().iter().enumerate() {
            let v_a = SurfaceVertex {
                position: triangle.vertex_a.to_f32_array(),
                normal: triangle.normal_a.to_f32_array(),
            };
            let v_b = SurfaceVertex {
                position: triangle.vertex_b.to_f32_array(),
                normal: triangle.normal_b.to_f32_array(),
            };
            let v_c = SurfaceVertex {
                position: triangle.vertex_c.to_f32_array(),
                normal: triangle.normal_c.to_f32_array(),
            };

            surface_vertices.push(v_a);
            surface_vertices.push(v_b);
            surface_vertices.push(v_c);

            let index = i * 3;

            surface_indices.push(index as u16);
            surface_indices.push((index + 1) as u16);
            surface_indices.push((index + 2) as u16);
        }

        let mut edge_vertices: Vec<EdgeVertex> = Vec::new();
        let mut edge_indices: Vec<u16> = Vec::new();
        for (i, line) in mesh.lines().iter().enumerate() {
            edge_vertices.push(EdgeVertex {
                position: line.vertex_a.to_f32_array(),
                expand: line.expand_a.to_f32_array(),
            });
            edge_vertices.push(EdgeVertex {
                position: line.vertex_b.to_f32_array(),
                expand: line.expand_b.to_f32_array(),
            });

            let index = i * 2;

            edge_indices.push(index as u16);
            edge_indices.push((index + 1) as u16);
        }

        let mut point_vertices: Vec<PointVertex> = Vec::new();
        let mut point_indices: Vec<u16> = Vec::new();
        for (i, point) in mesh.points().iter().enumerate() {
            let single = true;

            if !single {
                point_vertices.push(PointVertex {
                    position: point.vertex.to_f32_array(),
                    expand: point.expand.to_f32_array(),
                });

                let index = i * 2;

                point_indices.push(index as u16);
            } else {
                point_vertices.push(PointVertex {
                    position: point.vertex.to_f32_array(),
                    expand: point.expand.to_f32_array(),
                });
                point_vertices.push(PointVertex {
                    position: point.vertex.to_f32_array(),
                    expand: point.expand.to_f32_array(),
                });

                let index = i * 2;

                point_indices.push(index as u16);
                point_indices.push((index + 1) as u16);
            }
        }

        Self {
            surface: match (surface_vertices.len(), surface_indices.len()) {
                (0, 0) | (0, _) | (_, 0) => None,
                _ => Some(SurfaceVertexBuffers {
                    vertices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        surface_vertices,
                    )
                    .unwrap(),
                    indices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        surface_indices,
                    )
                    .unwrap(),
                }),
            },

            edges: match (edge_vertices.len(), edge_indices.len()) {
                (0, 0) | (0, _) | (_, 0) => None,
                _ => Some(EdgeVertexBuffers {
                    vertices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        edge_vertices,
                    )
                    .unwrap(),
                    indices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        edge_indices,
                    )
                    .unwrap(),
                }),
            },

            points: match (point_vertices.len(), point_indices.len()) {
                (0, 0) | (0, _) | (_, 0) => None,
                _ => Some(PointVertexBuffers {
                    vertices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        point_vertices,
                    )
                    .unwrap(),
                    indices: CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        point_indices,
                    )
                    .unwrap(),
                }),
            },

            position: Point3::new(0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            rotation: Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Rad(0.0)),
        }
    }

    pub fn position(&mut self, position: Point3) -> &mut Self {
        self.position = position;
        self
    }

    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        self.position += translation;
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.scale.0 *= x;
        self.scale.1 *= y;
        self.scale.2 *= z;
        self
    }

    pub fn rotate(&mut self, axis: Vec3, angle: Rad<f32>) -> &mut Self {
        self.rotation = Quat::from_axis_angle(axis.normalize(), angle) * self.rotation;
        self
    }

    pub fn get_transform_matrix(&self) -> Mat4 {
        Mat4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.0, self.scale.1, self.scale.2)
            * Matrix4::from_translation((self.position.x, self.position.y, self.position.z).into())
    }
}
