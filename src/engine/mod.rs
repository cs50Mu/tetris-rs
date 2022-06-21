use std::ops::{Index, IndexMut};

use self::piece::{Kind as PieceKind, Piece};
use cgmath::EuclideanSpace;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod piece;

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

    pub fn with_matrix(matrix: Matrix) -> Self {
        Self {
            matrix,
            ..Self::new()
        }
    }

    pub fn debug_test_cursor(&mut self, kind: PieceKind, position: Offset) {
        let piece = Piece {
            kind,
            rotation: piece::Rotation::N,
            position,
        };
        self.cursor = Some(piece);
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

    pub fn cells(&self) -> CellIter<'_> {
        CellIter {
            position: Coordinate::origin(),
            cells: self.matrix.0.iter(),
        }
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

pub struct Matrix([Option<Color>; Self::SIZE]);

impl Matrix {
    pub const WIDTH: usize = 10;
    pub const HEIGHT: usize = 20;
    const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    // 还可以这样来定义参数。。
    fn on_matrix(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH && y < Self::HEIGHT
    }

    fn valid_coord(Coordinate { x, y }: Coordinate) -> bool {
        x < Self::WIDTH
    }

    pub fn blank() -> Self {
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

pub struct CellIter<'matrix> {
    position: Coordinate,
    cells: ::std::slice::Iter<'matrix, Option<Color>>,
}

impl<'matrix> Iterator for CellIter<'matrix> {
    type Item = (Coordinate, &'matrix Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cell) = self.cells.next() {
            let coord = self.position;
            self.position.grid_incd();
            Some((coord, cell))
        } else {
            None
        }
    }
}

pub trait GridIncrement: Sized {
    type Width;
    const WIDTH: Self::Width;

    fn grid_inc(mut self) -> Self {
        self.grid_incd();
        self
    }

    fn grid_incd(&mut self);
}

impl GridIncrement for Coordinate {
    type Width = usize;
    const WIDTH: Self::Width = Matrix::WIDTH;

    fn grid_incd(&mut self) {
        self.x += 1;
        self.x %= Self::WIDTH;
        if self.x == 0 {
            self.y += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cell_iter() {
        let mut matrix = Matrix::blank();
        matrix[Coordinate::new(2, 0)] = Some(Color::Blue);
        matrix[Coordinate::new(3, 1)] = Some(Color::Green);

        let mut cell_iter = CellIter {
            position: Coordinate::origin(),
            cells: matrix.0.iter(),
        };

        // 这里使用引用可以让这个 iter 被使用多次
        // 因为 take 接受 self
        // https://stackoverflow.com/questions/31374051/why-does-iteratortake-while-take-ownership-of-the-iterator
        // let first_five = (&mut cell_iter).take(5).collect::<Vec<_>>();
        let first_five = cell_iter.by_ref().take(5).collect::<Vec<_>>();
        assert_eq!(
            first_five,
            [
                (Coordinate::new(0, 0), &None),
                (Coordinate::new(1, 0), &None),
                (Coordinate::new(2, 0), &Some(Color::Blue)),
                (Coordinate::new(3, 0), &None),
                (Coordinate::new(4, 0), &None),
            ]
        );

        let green_item = (&mut cell_iter).skip(8).next();
        assert_eq!(
            green_item,
            Some((Coordinate::new(3, 1), &Some(Color::Green)))
        );

        assert!(cell_iter.all(|(_, content)| content.is_none()));
    }
}
