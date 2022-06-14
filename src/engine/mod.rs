use std::ops::{Index, IndexMut};

use self::piece::{Kind as PieceKind, Piece};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

mod piece;

type Coordinate = cgmath::Point2<usize>;
type Offset = cgmath::Vector2<isize>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MoveKind {
    Left,
    Right,
}

impl MoveKind {
    fn offset(&self) -> Offset {
        match self {
            MoveKind::Left => Offset::new(-1, 0),
            MoveKind::Right => Offset::new(1, 0),
        }
    }
}

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
        debug_assert!(
            self.matrix.is_placeable(&cursor),
            "Tried to place cursor in an unplaceable location: {:?}",
            cursor
        );
        for coord in cursor.cells().unwrap() {
            self.matrix[coord] = Some(cursor.kind.color());
        }
    }

    fn move_cursor(&mut self, kind: MoveKind) -> Result<(), ()> {
        if let Some(cursor) = self.cursor.as_mut() {
            let new_cursor = cursor.moved_by(kind.offset());
            if self.matrix.is_clipping(&new_cursor) {
                Err(())
            } else {
                self.cursor = Some(new_cursor);
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn tick_down(&mut self) {
        self.cursor = Some(self.ticked_down_cursor().unwrap());
    }

    pub fn cusor_has_hit_bottom(&self) -> bool {
        self.cursor.is_some() && self.ticked_down_cursor().is_none()
    }

    fn ticked_down_cursor(&self) -> Option<Piece> {
        if let Some(cursor) = &self.cursor {
            let new_cursor = cursor.moved_by(Offset::new(0, -1));
            (!self.matrix.is_clipping(&new_cursor)).then(|| new_cursor)
        } else {
            None
        }
    }

    fn hard_drop(&mut self) {
        while let Some(new_cursor) = self.ticked_down_cursor() {
            self.cursor = Some(new_cursor);
        }
        self.place_cursor();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Yellow,
    Cyan,
    Purple,
    Orange,
    Blue,
    Green,
    Red,
}

struct Matrix([Option<Color>; Self::SIZE]);

impl Matrix {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 20;
    const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    // 还可以这样来定义参数。。
    fn on_matrix(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH && y < Self::HEIGHT
    }

    fn valid_coord(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH
    }

    fn blank() -> Self {
        Self([None; Self::SIZE])
    }

    fn indexing(Coordinate { x, y }: Coordinate) -> usize {
        y * Self::WIDTH + x
    }

    fn is_placeable(&self, piece: &Piece) -> bool {
        if let Some(cells) = piece.cells() {
            cells
                .into_iter()
                .all(|coord| Matrix::on_matrix(coord) && self[coord].is_none())
        } else {
            false
        }
    }

    fn is_clipping(&self, piece: &Piece) -> bool {
        if let Some(cells) = piece.cells() {
            cells
                .into_iter()
                .any(|coord| !Matrix::on_matrix(coord) || self[coord].is_some())
        } else {
            true
        }
    }
}

impl Index<Coordinate> for Matrix {
    type Output = Option<Color>;

    fn index(&self, coord: Coordinate) -> &Self::Output {
        debug_assert!(Self::on_matrix(coord));
        &self.0[Self::indexing(coord)]
    }
}

impl IndexMut<Coordinate> for Matrix {
    fn index_mut(&mut self, coord: Coordinate) -> &mut Self::Output {
        debug_assert!(Self::on_matrix(coord));
        &mut self.0[Self::indexing(coord)]
    }
}
