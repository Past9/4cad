use crate::submatrix;

#[derive(Debug, Clone, PartialEq)]
pub enum BspTree<T: Clone> {
    EW {
        e: Box<BspTree<T>>,
        w: Box<BspTree<T>>,
    },
    NS {
        n: Box<BspTree<T>>,
        s: Box<BspTree<T>>,
    },
    Cell(T),
}
impl<T: Clone> BspTree<T> {
    pub fn visit<F, R>(&self, visitor: &mut F)
    where
        F: FnMut(&T) -> R,
    {
        match self {
            BspTree::EW { e, w } => {
                e.visit(visitor);
                w.visit(visitor);
            }
            BspTree::NS { n, s } => {
                n.visit(visitor);
                s.visit(visitor);
            }
            BspTree::Cell(cell) => {
                visitor(cell);
            }
        }
    }

    pub fn from_grid(cells: Vec<Vec<T>>) -> Self {
        let len_u = cells.len();

        if len_u == 0 {
            panic!("U length is zero");
        }

        let len_v = cells[0].len();

        if len_v == 0 {
            panic!("V length is zero");
        }

        match (len_u, len_v) {
            (0, _) | (_, 0) => panic!("No cells"),

            (1, 1) => Self::Cell(cells[0][0].clone()),
            (1, _) => {
                let split = len_v / 2;
                Self::EW {
                    w: Box::new(Self::from_grid(submatrix(&cells, ..1, ..split))),
                    e: Box::new(Self::from_grid(submatrix(&cells, ..1, split..))),
                }
            }
            (_, 1) => {
                let split = len_u / 2;
                Self::NS {
                    s: Box::new(Self::from_grid(submatrix(&cells, split.., ..1))),
                    n: Box::new(Self::from_grid(submatrix(&cells, ..split, ..1))),
                }
            }
            (_, _) => {
                let split_u = len_u / 2;
                let split_v = len_u / 2;
                Self::NS {
                    s: Box::new(Self::EW {
                        w: Box::new(Self::from_grid(submatrix(&cells, split_u.., ..split_v))),
                        e: Box::new(Self::from_grid(submatrix(&cells, split_u.., split_v..))),
                    }),
                    n: Box::new(Self::EW {
                        w: Box::new(Self::from_grid(submatrix(&cells, ..split_u, ..split_v))),
                        e: Box::new(Self::from_grid(submatrix(&cells, ..split_u, split_v..))),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::BspTree;
    use BspTree::*;

    #[test]
    fn builds_from_row() {
        assert_eq!(Cell(1), BspTree::from_grid(vec![vec![1],]));
        assert_eq!(
            EW {
                e: Box::new(Cell(2)),
                w: Box::new(Cell(1))
            },
            BspTree::from_grid(vec![vec![1, 2]])
        );
        assert_eq!(
            EW {
                w: Box::new(Cell(1)),
                e: Box::new(EW {
                    e: Box::new(Cell(3)),
                    w: Box::new(Cell(2))
                })
            },
            BspTree::from_grid(vec![vec![1, 2, 3],])
        );
        assert_eq!(
            EW {
                e: Box::new(EW {
                    e: Box::new(Cell(4)),
                    w: Box::new(Cell(3))
                }),
                w: Box::new(EW {
                    e: Box::new(Cell(2)),
                    w: Box::new(Cell(1))
                })
            },
            BspTree::from_grid(vec![vec![1, 2, 3, 4],])
        );
    }

    #[test]
    fn builds_from_column() {
        assert_eq!(Cell(1), BspTree::from_grid(vec![vec![1],]));
        assert_eq!(
            NS {
                n: Box::new(Cell(1)),
                s: Box::new(Cell(2))
            },
            BspTree::from_grid(vec![vec![1], vec![2],])
        );
        assert_eq!(
            NS {
                n: Box::new(Cell(1)),
                s: Box::new(NS {
                    n: Box::new(Cell(2)),
                    s: Box::new(Cell(3))
                })
            },
            BspTree::from_grid(vec![vec![1], vec![2], vec![3],])
        );
        assert_eq!(
            NS {
                n: Box::new(NS {
                    n: Box::new(Cell(1)),
                    s: Box::new(Cell(2))
                }),
                s: Box::new(NS {
                    n: Box::new(Cell(3)),
                    s: Box::new(Cell(4))
                }),
            },
            BspTree::from_grid(vec![vec![1], vec![2], vec![3], vec![4]])
        );
    }

    #[test]
    fn builds_from_grid() {
        assert_eq!(
            NS {
                n: Box::new(EW {
                    e: Box::new(NS {
                        n: Box::new(Cell(3)),
                        s: Box::new(Cell(6))
                    }),
                    w: Box::new(NS {
                        n: Box::new(EW {
                            e: Box::new(Cell(2)),
                            w: Box::new(Cell(1)),
                        }),
                        s: Box::new(EW {
                            e: Box::new(Cell(5)),
                            w: Box::new(Cell(4)),
                        }),
                    }),
                }),
                s: Box::new(EW {
                    e: Box::new(NS {
                        n: Box::new(Cell(9)),
                        s: Box::new(Cell(12))
                    }),
                    w: Box::new(NS {
                        n: Box::new(EW {
                            e: Box::new(Cell(8)),
                            w: Box::new(Cell(7))
                        }),
                        s: Box::new(EW {
                            e: Box::new(Cell(11)),
                            w: Box::new(Cell(10))
                        }),
                    }),
                }),
            },
            BspTree::from_grid(vec![
                vec![1, 2, 3],    //
                vec![4, 5, 6],    //
                vec![7, 8, 9],    //
                vec![10, 11, 12], //
            ])
        );
    }
}
