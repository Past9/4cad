use crate::{section, transpose};

#[derive(Debug)]
pub struct EwTree<T: Clone> {
    e: Box<BspTree<T>>,
    w: Box<BspTree<T>>,
}
impl<T: Clone> EwTree<T> {
    pub fn new(e: T, w: T) -> Self {
        Self {
            e: Box::new(BspTree::Cell(e)),
            w: Box::new(BspTree::Cell(w)),
        }
    }

    pub fn node(e: T, w: T) -> BspTree<T> {
        BspTree::EW(Self::new(e, w))
    }
}

#[derive(Debug)]
pub struct NsTree<T: Clone> {
    n: Box<BspTree<T>>,
    s: Box<BspTree<T>>,
}
impl<T: Clone> NsTree<T> {
    pub fn new(n: T, s: T) -> Self {
        Self {
            n: Box::new(BspTree::Cell(n)),
            s: Box::new(BspTree::Cell(s)),
        }
    }

    pub fn node(n: T, s: T) -> BspTree<T> {
        BspTree::NS(Self::new(n, s))
    }
}

#[derive(Debug)]
pub enum BspTree<T: Clone> {
    EW(EwTree<T>),
    NS(NsTree<T>),
    Cell(T),
}
impl<T: Clone> BspTree<T> {
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
                Self::NS(NsTree {
                    n: Box::new(Self::from_grid(section(&cells, ..1, ..split))),
                    s: Box::new(Self::from_grid(section(&cells, ..1, split..))),
                })
            }
            (_, 1) => {
                let split = len_u / 2;
                Self::EW(EwTree {
                    e: Box::new(Self::from_grid(section(&cells, split.., ..1))),
                    w: Box::new(Self::from_grid(section(&cells, ..split, ..1))),
                })
            }
            (_, _) => {
                let split_u = len_u / 2;
                let split_v = len_u / 2;
                Self::EW(EwTree {
                    e: Box::new(Self::NS(NsTree {
                        n: Box::new(Self::from_grid(section(&cells, split_u.., ..split_v))),
                        s: Box::new(Self::from_grid(section(&cells, split_u.., split_v..))),
                    })),
                    w: Box::new(Self::NS(NsTree {
                        n: Box::new(Self::from_grid(section(&cells, ..split_u, ..split_v))),
                        s: Box::new(Self::from_grid(section(&cells, ..split_u, split_v..))),
                    })),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BspTree;

    #[test]
    fn builds_bsp_from_uv_patches() {
        println!("{:#?}", BspTree::from_grid(vec![vec![1],]));
        println!("{:#?}", BspTree::from_grid(vec![vec![1, 2],]));
        println!("{:#?}", BspTree::from_grid(vec![vec![1, 2, 3],]));
        println!("{:#?}", BspTree::from_grid(vec![vec![1, 2, 3, 4],]));

        println!("{:#?}", BspTree::from_grid(vec![vec![1],]));
        println!("{:#?}", BspTree::from_grid(vec![vec![1], vec![2],]));
        println!(
            "{:#?}",
            BspTree::from_grid(vec![vec![1], vec![2], vec![3],])
        );
        println!(
            "{:#?}",
            BspTree::from_grid(vec![vec![1], vec![2], vec![3], vec![4]])
        );

        let grid = vec![
            vec![1, 2, 3],    //
            vec![4, 5, 6],    //
            vec![7, 8, 9],    //
            vec![10, 11, 12], //
        ];

        let tree = BspTree::from_grid(grid);

        println!("{:#?}", tree);
    }

    #[test]
    fn test() {
        let ints: Vec<Vec<i32>> = vec![
            vec![0, 1, 2], //
            vec![3, 4, 5], //
            vec![6, 7, 8], //
        ];

        println!("{:#?}", &ints[..2][0]);
    }
}
