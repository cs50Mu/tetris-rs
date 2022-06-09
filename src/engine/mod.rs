use std::ops::{Index, IndexMut};

use self::piece::{Kind as PieceKind, Piece};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

mod piece;

type Coordinate = cgmath::Point2<usize>;
type Offset = cgmath::Vector2<isize>;

pub struct Engine {
    matrix: Matrix,
    bag: Vec<PieceKind>,
    rng: ThreadRng,
    cursor: Option<Piece>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            matrix: Matrix::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            cursor: None,
        }
    }

    fn refill_bag(&mut self) {
        debug_assert!(self.bag.is_empty());
        // put all pieces in bag
        self.bag.extend_from_slice(&PieceKind::ALL);
        // shuffle the bag
        self.bag.shuffle(&mut self.rng)
    }

    fn place_cursor(&mut self) {
        let cursor = self
            .cursor
            .take()
            .expect("called place_cursor without cursor");
        for coord in cursor.cells().expect("cursor is out of bounds") {
            // 这样就得到一个可变引用了
            let cell = &mut self.matrix[coord];
            // cell should not be occupied yet (false)
            debug_assert!(!*cell);
            *cell = true;
        }
    }
}

struct Matrix([bool; Self::SIZE]);

impl Matrix {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 20;
    const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    // 还可以这样来定义参数。。
    fn in_bounds(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH && y < Self::HEIGHT
    }

    fn blank() -> Self {
        Self([false; Self::SIZE])
    }

    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * Self::WIDTH + x
    }
}

impl Index<Coordinate> for Matrix {
    type Output = bool;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        debug_assert!(Self::in_bounds(coord));
        &self.0[Self::indexing(coord)]
    }
}

impl IndexMut<Coordinate> for Matrix {
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        debug_assert!(Self::in_bounds(coord));
        &mut self.0[Self::indexing(coord)]
    }
}
