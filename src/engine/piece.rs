use cgmath::{EuclideanSpace, Zero};

use super::{Color, Coordinate, Matrix, Offset};

#[derive(Debug)]
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

            if Matrix::valid_coord(new_coord) {
                *coord = new_coord
            } else {
                return None;
            }
        }

        Some(coords)
    }

    fn rotator(&self) -> impl Fn(Offset) -> Offset + '_ {
        // // 这样写是为了能让返回的闭包不带任何引用
        // // 把 `rotation` copy 到栈上, 摆脱对 self 的依赖
        // let rotation = self.rotation;
        // // 把栈上的这个 `rotation` move 到闭包里
        // // 所有的这一切都是为了避免：use after free 的问题
        // move |cell| cell * rotation

        |cell| match self.kind {
            Kind::O => cell,
            _ => {
                // 原点定在左下角，先绕原点做旋转，再根据情况做偏移
                cell * self.rotation
                    + (self.rotation.intrinsic_offset() * (self.kind.grid_size() - 1))
            }
        }
    }

    fn positioner(&self) -> impl Fn(Offset) -> Offset {
        let position = self.position;
        move |cell| cell + position
    }

    pub fn moved_by(&self, offset: Offset) -> Self {
        Self {
            position: self.position + offset,
            ..*self
        }
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
            Kind::O => &[(1, 1), (1, 2), (2, 2), (2, 1)],
            Kind::I => &[(0, 2), (1, 2), (2, 2), (3, 2)],
            Kind::T => &[(0, 1), (1, 1), (1, 2), (2, 1)],
            Kind::L => &[(0, 1), (1, 1), (2, 1), (2, 2)],
            Kind::J => &[(0, 1), (0, 2), (1, 1), (2, 1)],
            Kind::S => &[(0, 1), (1, 1), (1, 2), (2, 2)],
            Kind::Z => &[(0, 2), (1, 2), (1, 1), (2, 1)],
        }
        .map(Offset::from)
    }

    // rotate 后偏移的大小是根据 grid size 来计算的
    // offset = grid_size - 1
    fn grid_size(&self) -> isize {
        // 除了 I-Tetrimino 是 4 x 4的
        // 其余都是 3 x 3的
        match self {
            Self::I => 4,
            _ => 3,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Kind::O => Color::Yellow,
            Kind::I => Color::Cyan,
            Kind::T => Color::Purple,
            Kind::L => Color::Orange,
            Kind::J => Color::Blue,
            Kind::S => Color::Green,
            Kind::Z => Color::Red,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotation {
    N,
    S,
    E,
    W,
}

impl Rotation {
    // 这里确定各个 rotation 后需要偏移的方向
    // 比如，x轴 向增大的方向（1），y轴 向减小的方向（-1）
    // 在某个轴上不移动，那个轴就是 0
    fn intrinsic_offset(&self) -> Offset {
        match self {
            Rotation::N => Offset::zero(),
            Rotation::S => Offset::new(1, 1),
            Rotation::E => Offset::new(0, 1),
            Rotation::W => Offset::new(1, 0),
        }
    }
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
        // rotate west
        let s = Piece {
            kind: Kind::S,
            position: Offset::new(5, 6),
            rotation: Rotation::W,
        };

        assert_eq!(
            s.cells(),
            // (1, 0), (1, 1), (0, 1), (0, 2)
            Some([(6, 6), (6, 7), (5, 7), (5, 8)].map(Coordinate::from))
        );

        // rotate south
        let s = Piece {
            kind: Kind::S,
            position: Offset::new(5, 6),
            rotation: Rotation::S,
        };

        assert_eq!(
            s.cells(),
            // (2, 1), (1, 1), (1, 0), (0, 0)
            Some([(7, 7), (6, 7), (6, 6), (5, 6)].map(Coordinate::from))
        );
    }
}
