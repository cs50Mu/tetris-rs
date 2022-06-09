use std::char::CharTryFromError;

use cgmath::EuclideanSpace;

use super::{Coordinate, Matrix, Offset};

pub(super) struct Piece {
    pub kind: Kind,
    pub position: Offset,
    pub rotation: Rotation,
}

impl Piece {
    const CELL_COUNT: usize = 4;

    // rotate 后再叠加 piece 原来的坐标
    // 可以得到 piece 现在的新坐标
    pub fn cells(&self) -> Option<[Coordinate; Self::CELL_COUNT]> {
        let offsets = self.kind.cells().map(self.rotator()).map(self.positioner());
        let mut coords = [Coordinate::origin(); Self::CELL_COUNT];

        // for (Offset { x, y }, coord) in offsets.into_iter().zip(&mut coords) {
        //     let new_coord = match (x.try_into(), y.try_into()) {
        //         // Offset --> Coordinate 是否能转成功
        //         // 即 isize --> usize 的转换
        //         (Ok(x), Ok(y)) => Coordinate { x, y },
        //         _ => return None,
        //     };

        //     if Matrix::in_bounds(new_coord) {
        //         *coord = new_coord
        //     } else {
        //         return None;
        //     }
        // }

        // 更好的写法
        for (offset, coord) in offsets.into_iter().zip(&mut coords) {
            let positive_offset = offset.cast::<usize>()?;
            let new_coord = Coordinate::from_vec(positive_offset);

            if Matrix::in_bounds(new_coord) {
                *coord = new_coord
            } else {
                return None;
            }
        }

        Some(coords)
    }

    fn rotator(&self) -> impl Fn(Offset) -> Offset {
        // 这样写是为了能让返回的闭包不带任何引用
        // 把 `rotation` copy 到栈上, 摆脱对 self 的依赖
        let rotation = self.rotation;
        // 把栈上的这个 `rotation` move 到闭包里
        // 所有的这一切都是为了避免：use after free 的问题
        move |cell| cell * rotation
    }

    fn positioner(&self) -> impl Fn(Offset) -> Offset {
        let position = self.position;
        move |cell| cell + position
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    O,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl Kind {
    pub const ALL: [Self; 7] = [
        Self::O,
        Self::I,
        Self::T,
        Self::L,
        Self::J,
        Self::S,
        Self::Z,
    ];

    fn cells(&self) -> [Offset; Piece::CELL_COUNT] {
        match self {
            Kind::O => &[(0, 0), (0, 1), (1, 0), (1, 1)],
            Kind::I => &[(-1, 0), (0, 0), (1, 0), (2, 0)],
            Kind::T => &[(-1, 0), (0, 0), (1, 0), (0, 1)],
            Kind::L => &[(-1, 0), (0, 0), (1, 0), (1, 1)],
            Kind::J => &[(-1, 1), (-1, 0), (0, 0), (1, 0)],
            Kind::S => &[(-1, 0), (0, 0), (0, 1), (1, 1)],
            Kind::Z => &[(-1, 1), (0, 1), (0, 0), (1, 0)],
        }
        .map(Offset::from)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotation {
    N,
    S,
    E,
    W,
}

impl std::ops::Mul<Rotation> for Offset {
    type Output = Self;

    fn mul(self, rotation: Rotation) -> Self::Output {
        match rotation {
            Rotation::N => self,
            Rotation::S => Offset::new(-self.x, -self.y),
            Rotation::E => Offset::new(self.y, -self.x),
            Rotation::W => Offset::new(-self.y, self.x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn s_piece_position() {
        let s = Piece {
            kind: Kind::S,
            position: Offset::new(5, 6),
            rotation: Rotation::W,
        };

        assert_eq!(
            s.cells(),
            Some([(5, 5), (5, 6), (4, 6), (4, 7)].map(Coordinate::from))
        );
    }
}
