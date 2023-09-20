use crate::SurfaceBezierComponent;

struct EwTree {
    e: Box<BspSurface>,
    w: Box<BspSurface>,
}
impl EwTree {
    pub fn new(e: SurfaceBezierComponent, w: SurfaceBezierComponent) -> Self {
        Self {
            e: Box::new(BspSurface::Patch(e)),
            w: Box::new(BspSurface::Patch(w)),
        }
    }

    pub fn node(e: SurfaceBezierComponent, w: SurfaceBezierComponent) -> BspSurface {
        BspSurface::EW(Self::new(e, w))
    }
}

struct NsTree {
    n: Box<BspSurface>,
    s: Box<BspSurface>,
}
impl NsTree {
    pub fn new(n: SurfaceBezierComponent, s: SurfaceBezierComponent) -> Self {
        Self {
            n: Box::new(BspSurface::Patch(n)),
            s: Box::new(BspSurface::Patch(s)),
        }
    }

    pub fn node(n: SurfaceBezierComponent, s: SurfaceBezierComponent) -> BspSurface {
        BspSurface::NS(Self::new(n, s))
    }
}

pub enum BspSurface {
    EW(EwTree),
    NS(NsTree),
    Patch(SurfaceBezierComponent),
}
impl BspSurface {
    pub fn from_uv_patches(patches: &[&[SurfaceBezierComponent]]) -> Self {
        let len_u = patches.len();
        let len_v = patches[0].len();

        match (len_u, len_v) {
            (0, _) | (_, 0) => panic!("No patches"),

            (1, 1) => Self::Patch(patches[0][0].clone()),
            //(1, 2) => NsTree::node(patches[0][0].clone(), patches[0][1].clone()),
            (1, _) => {
                let split = len_v / 2;
                let n = Self::from_uv_patches(&[&patches[0][..split]]);
                let s = Self::from_uv_patches(&[&patches[0][split..]]);
                Self::NS(NsTree {
                    n: Box::new(n),
                    s: Box::new(s),
                })
            }
            //(2, 1) => EwTree::node(patches[0][0].clone(), patches[1][0].clone()),
            /*
            (2, 2) => Self::EW(EwTree {
                e: Box::new(NsTree::node(patches[0][0].clone(), patches[0][1].clone())),
                w: Box::new(NsTree::node(patches[1][0].clone(), patches[1][1].clone())),
            }),
            (2, _) => Self::EW(EwTree {
                e: Box::new(NsTree::node(patches[0][0].clone(), patches[0][1].clone())),
                w: {
                    let split = len_v / 2;
                    let n = Self::from_uv_patches(&[&patches[1][..split]]);
                    let s = Self::from_uv_patches(&[&patches[1][split..]]);
                    Box::new(Self::NS(NsTree {
                        n: Box::new(n),
                        s: Box::new(s),
                    }))
                },
            }),
             */
            (_, 1) => {
                let split = len_u / 2;
                let e = Self::from_uv_patches(&[&patches[..split][0]]);
                let w = Self::from_uv_patches(&[&patches[split..][0]]);
                Self::EW(EwTree {
                    e: Box::new(e),
                    w: Box::new(w),
                })
            }
            /*
            (_, 2) => Self::NS(NsTree {
                n: Box::new(EwTree::node(patches[0][0].clone(), patches[1][0].clone())),
                s: {
                    let split = len_u / 1;
                    let e = Self::from_uv_patches(&[&patches[..split][1]]);
                    let w = Self::from_uv_patches(&[&patches[split..][1]]);
                    Box::new(Self::EW(EwTree {
                        e: Box::new(e),
                        w: Box::new(w),
                    }))
                },
            }),
             */
            (_, _) => {
                let split_u = len_u / 2;
                let split_v = len_u / 2;
                let ne = Self::from_uv_patches(&patches[..split_u][..split_v]);
                let nw = Self::from_uv_patches(&patches[split_u..][..split_v]);
                let se = Self::from_uv_patches(&patches[..split_u][split_v..]);
                let sw = Self::from_uv_patches(&patches[split_u..][split_v..]);
                Self::EW(EwTree {
                    e: Box::new(Self::NS(NsTree {
                        n: Box::new(ne),
                        s: Box::new(se),
                    })),
                    w: Box::new(Self::NS(NsTree {
                        n: Box::new(nw),
                        s: Box::new(sw),
                    })),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn builds_bsp_from_uv_patches() {}
}
