#![allow(dead_code)]
use engine::{Engine, Matrix, Color, piece::Kind as PieceKind};
use interface::Interface;

mod engine;
mod interface;

fn main() {
    // let engine = Engine::new();
    let mut matrix = Matrix::blank();

    matrix[(1,1).into()] = Some(Color::Green);
    let mut engine = Engine::with_matrix(matrix);
    engine.debug_test_cursor(PieceKind::T, (5,5).into());

    Interface::run(engine);
}
