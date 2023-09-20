pub struct Quad<T> {
    lu: QuadTree<T>,
    ru: QuadTree<T>,
    ll: QuadTree<T>,
    rl: QuadTree<T>,
}

pub enum QuadTree<T> {
    Quad(Box<Quad<T>>),
    Leaf(Box<T>),
}
impl<T> QuadTree<T> {
    //
}
