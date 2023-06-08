use std::sync::Arc;

use crevice::std140::AsStd140;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    memory::allocator::MemoryAllocator,
};

use crate::{Rgb, Rgba};

#[derive(Debug, Clone, Copy, PartialEq)]
enum MaterialKind {
    Opaque,
    Translucent,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialId(MaterialKind, u32);
impl MaterialId {
    pub fn index(&self) -> u32 {
        self.1
    }

    pub fn is_opaque(&self) -> bool {
        self.0 == MaterialKind::Opaque
    }

    pub fn is_translucent(&self) -> bool {
        self.0 == MaterialKind::Translucent
    }
}

#[derive(Debug)]
pub struct MaterialSet {
    opaque: Vec<OpaqueMaterial>,
    translucent: Vec<TranslucentMaterial>,
}
impl MaterialSet {
    pub fn new() -> Self {
        Self {
            opaque: vec![],
            translucent: vec![],
        }
    }

    pub fn insert(&mut self, reflect: Rgba, roughness: f32) -> MaterialId {
        if reflect.a() == 1.0 {
            let material = OpaqueMaterial {
                diffuse: reflect.rgb(),
                roughness,
            };

            let id = if let Some((i, _)) = self
                .opaque
                .iter()
                .enumerate()
                .find(|(_, mat)| **mat == material)
            {
                i
            } else {
                self.opaque.push(material);
                self.opaque.len() - 1
            };

            MaterialId(MaterialKind::Opaque, id as u32)
        } else {
            let material = TranslucentMaterial {
                diffuse: reflect,
                roughness,
            };

            let id = if let Some((i, _)) = self
                .translucent
                .iter()
                .enumerate()
                .find(|(_, mat)| **mat == material)
            {
                i
            } else {
                self.translucent.push(material);
                self.translucent.len() - 1
            };

            MaterialId(MaterialKind::Translucent, id as u32)
        }
    }

    pub fn buffer_opaque(
        &self,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Option<Arc<CpuAccessibleBuffer<[Std140OpaqueMaterial]>>> {
        if !self.opaque.is_empty() {
            Some(
                CpuAccessibleBuffer::from_iter(
                    allocator,
                    BufferUsage {
                        storage_buffer: true,
                        ..BufferUsage::empty()
                    },
                    false,
                    self.opaque.iter().map(|material| material.as_std140()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }

    pub fn buffer_translucent(
        &self,
        allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Option<Arc<CpuAccessibleBuffer<[Std140TranslucentMaterial]>>> {
        if !self.translucent.is_empty() {
            Some(
                CpuAccessibleBuffer::from_iter(
                    allocator,
                    BufferUsage {
                        storage_buffer: true,
                        ..BufferUsage::empty()
                    },
                    false,
                    self.translucent.iter().map(|material| material.as_std140()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }
}

#[derive(AsStd140, Clone, Debug, PartialEq)]
pub struct OpaqueMaterial {
    diffuse: Rgb,
    roughness: f32,
}
impl OpaqueMaterial {
    pub fn buffer(
        allocator: &(impl MemoryAllocator + ?Sized),
        materials: &[OpaqueMaterial],
    ) -> Option<Arc<CpuAccessibleBuffer<[Std140OpaqueMaterial]>>> {
        if !materials.is_empty() {
            Some(
                CpuAccessibleBuffer::from_iter(
                    allocator,
                    BufferUsage {
                        storage_buffer: true,
                        ..BufferUsage::empty()
                    },
                    false,
                    materials.iter().map(|material| material.as_std140()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }
}
impl Default for OpaqueMaterial {
    fn default() -> Self {
        Self {
            diffuse: Rgb::BLACK,
            roughness: 0.0,
        }
    }
}

#[derive(AsStd140, Clone, Debug, PartialEq)]
pub struct TranslucentMaterial {
    diffuse: Rgba,
    roughness: f32,
}
impl TranslucentMaterial {
    pub fn buffer(
        allocator: &(impl MemoryAllocator + ?Sized),
        materials: &[TranslucentMaterial],
    ) -> Option<Arc<CpuAccessibleBuffer<[Std140TranslucentMaterial]>>> {
        if !materials.is_empty() {
            Some(
                CpuAccessibleBuffer::from_iter(
                    allocator,
                    BufferUsage {
                        storage_buffer: true,
                        ..BufferUsage::empty()
                    },
                    false,
                    materials.iter().map(|material| material.as_std140()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }
}
impl Default for TranslucentMaterial {
    fn default() -> Self {
        Self {
            diffuse: Rgba::BLACK,
            roughness: 0.0,
        }
    }
}
