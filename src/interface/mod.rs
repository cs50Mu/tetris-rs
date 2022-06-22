use crate::engine::Color as SemanticColor;
use crate::engine::{Engine, Matrix, MoveKind};
use cgmath::{Point2, Vector2};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

pub struct Interface {
    engine: Engine,
}

const INIT_SIZE: Vector2<u32> = Vector2::new(1024, 1024);
const BACKGROUND_COLOR: Color = Color::RGB(0x10, 0x10, 0x18);
const MATRIX_COLOR: Color = Color::RGB(0x80, 0x75, 0xbf);
const WINDOW_TITLE: &str = "Tetris";

// when drawing with the SDL2, the (0, 0) coordinates are at the top-left of a window,
// not at the bottom-left. The same goes for all shapes.

impl Interface {
    pub fn run(mut engine: Engine) {

        let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
        let video_subsystem = sdl_context.video().expect("Failed to acquire display");

        let window = video_subsystem
            .window(WINDOW_TITLE, INIT_SIZE.x, INIT_SIZE.y)
            .position_centered()
            .resizable()
            .build()
            .expect("Failed to create window");

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .expect("Failed to get render canvas");

        let mut event_pump = sdl_context.event_pump().expect("Failed to get event loop");
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::Right => engine.move_cursor(MoveKind::Right).unwrap(),
                        Keycode::Left => engine.move_cursor(MoveKind::Left).unwrap(),
                        // hard_drop
                        Keycode::Space => engine.hard_drop(),
                        // rotate
                        Keycode::Up => {
                            engine.rotate_clockwise();
                            dbg!(engine.cursor);
                        }
                        // soft drop
                        Keycode::Down => {}
                        _ => {}
                    },
                    _ => {}
                }
            }

            draw(&mut canvas, &engine);
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

fn draw(canvas: &mut Canvas<Window>, engine: &Engine) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    let ui_square = canvas.viewport();
    let matrix = {
        let mut middle_section = ui_square;
        middle_section.set_width(middle_section.width() / 2);
        middle_section.center_on(ui_square.center());

        let mut matrix = middle_section;
        matrix.resize(
            (matrix.width() as f32 * (7.0 / 8.0)) as _,
            (matrix.height() as f32 * (7.0 / 8.0)) as _,
        );
        matrix.center_on(middle_section.center());

        matrix
    };
    let width_ui_quarter = ui_square.width() / 4;
    let height_ui_quarter = ui_square.height() / 4;
    let up_next = {
        let mut outer = ui_square;
        outer.resize(width_ui_quarter, height_ui_quarter);
        outer.offset(3 * width_ui_quarter as i32, 0);

        let mut inner = outer;
        inner.resize(outer.width() * 3 / 4, outer.height() * 3 / 4);
        inner.center_on(outer.center());

        inner
    };
    let next_queue = {
        let mut outer = ui_square;
        outer.resize(width_ui_quarter, 3 * height_ui_quarter);
        outer.offset(3 * width_ui_quarter as i32, height_ui_quarter as i32);

        let mut inner = outer;
        inner.resize(outer.width() * 3 / 4, outer.height() * 3 / 4);
        inner.center_on(outer.center());

        inner
    };
    let hold = {
        let mut outer = ui_square;
        outer.resize(width_ui_quarter, height_ui_quarter);

        let mut inner = outer;
        inner.resize(outer.width() * 3 / 4, outer.height() * 3 / 4);
        inner.center_on(outer.center());

        inner
    };
    let score = {
        let mut outer = ui_square;
        outer.resize(width_ui_quarter, 3 * height_ui_quarter);
        outer.offset(0, height_ui_quarter as i32);

        let mut inner = outer;
        inner.resize(outer.width() * 7 / 8, outer.height() * 3 / 4);
        inner.center_on(outer.center());

        inner
    };
    canvas.set_draw_color(MATRIX_COLOR);
    // canvas.draw_rect(ui_square).unwrap();
    canvas.fill_rect(matrix).unwrap();
    canvas.fill_rect(up_next).unwrap();
    canvas.fill_rect(next_queue).unwrap();
    canvas.fill_rect(hold).unwrap();
    canvas.fill_rect(score).unwrap();

    let mut cell_draw_ctx = CellDrawCtx {
        // 原点在左下角
        origin: matrix.bottom_left(),
        dims: matrix.size().into(),
        canvas,
    };
    // matrix 上已存在的 cell
    for (coord, cell_color) in engine.cells() {
        cell_draw_ctx.draw_cell(*cell_color, coord);
    }
    // cursor 处的 piece
    if let Some((cursor_cells, color)) = engine.cursor_info() {
        for coord in cursor_cells {
            cell_draw_ctx.draw_cell(Some(color), coord);
        }
    }
    canvas.present();
}

struct CellDrawCtx<'a> {
    origin: Point,
    dims: Vector2<u32>,
    canvas: &'a mut Canvas<Window>,
}

impl CellDrawCtx<'_> {
    fn draw_cell(&mut self, cell_color: Option<SemanticColor>, coord: Point2<usize>) {
        if let Some(cell_color) = cell_color {
            let matrix_width = self.dims.x;
            let matrix_height = self.dims.y;
            let coord = coord.cast::<i32>().unwrap();
            let this_x = (coord.x + 0) * matrix_width as i32 / Matrix::WIDTH as i32;
            let next_x = (coord.x + 1) * matrix_width as i32 / Matrix::WIDTH as i32;
            // y 轴需要额外偏移一个 matrix_height
            let this_y = (coord.y + 1) * matrix_height as i32 / Matrix::HEIGHT as i32;
            // 因为我们想要的坐标系是，原点在左下角，y 轴从下往上递增
            // 但实际 sdl2 的坐标系是，原点在左上角，y 轴是从上往下递增
            // 所以这里的 next_y 的坐标应该是比 this_y 要小
            let next_y = (coord.y + 0) * matrix_height as i32 / Matrix::HEIGHT as i32;
            let cell_rect = Rect::new(
                self.origin.x + this_x,
                self.origin.y - this_y,
                (next_x - this_x) as u32,
                (this_y - next_y) as u32,
            );

            self.canvas.set_draw_color(cell_color.screen_color());
            // canvas.draw_rect(cell_rect).unwrap();
            self.canvas.fill_rect(cell_rect).unwrap();
        }
    }
}

trait ScreenColor {
    fn screen_color(&self) -> SdlColor;
}

impl ScreenColor for SemanticColor {
    fn screen_color(&self) -> SdlColor {
        match self {
            SemanticColor::Yellow => SdlColor::RGB(0xed, 0xd4, 0x00),
            SemanticColor::Cyan => SdlColor::RGB(0x72, 0x9f, 0xcf),
            SemanticColor::Purple => SdlColor::RGB(0x75, 0x50, 0x7b),
            SemanticColor::Orange => SdlColor::RGB(0xf5, 0x79, 0x00),
            SemanticColor::Blue => SdlColor::RGB(0x34, 0x65, 0xa4),
            SemanticColor::Green => SdlColor::RGB(0x73, 0xd2, 0x16),
            SemanticColor::Red => SdlColor::RGB(0xef, 0x29, 0x29),
        }
    }
}
