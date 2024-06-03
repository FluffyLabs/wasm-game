#![warn(missing_docs)]

//! JS bindings for the `engine-rs` crate.
//!
//! The crate is compiled into WASM and tiny JS interface is exposed
//! to calcualte the game physics and return objects to render on the JS side.

use engine_rs::{
    board::Board,
    game::{Game, Position},
};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
/// Positions and sizes of game objects.
pub struct GameObjects {
    /// Lit ball X coordinate.
    pub lit_ball_x: u32,
    /// Lit ball Y coordinate.
    pub lit_ball_y: u32,
    /// Dark ball X coordinate.
    pub dark_ball_x: u32,
    /// Dark ball Y coordinate.
    pub dark_ball_y: u32,
    /// Cell width.
    pub cell_size_x: u32,
    /// Cell height.
    pub cell_size_y: u32,
    /// Radius of the ball
    pub ball_radius: u32,
}

#[wasm_bindgen]
/// Game object.
pub struct WasmGame {
    game: Game,
}

#[wasm_bindgen]
impl WasmGame {
    /// Create a new game given viewport size (pixels), the board size and starting time.
    pub fn new(board_size: u8, viewport_x: u32, viewport_y: u32, start_time_ms: u64) -> Self {
        // game init
        let board = Board::new(board_size);
        let viewport_size = Position {
            x: viewport_x as _,
            y: viewport_y as _,
        };
        let game = Game::new(board, start_time_ms, viewport_size);

        Self { game }
    }

    /// Recalculate objects positions and game physics.
    pub fn tick(&mut self, time_ms: u64) {
        let _timer = Timer::new("Game::tick");
        self.game.tick(time_ms)
    }

    /// Export the board state.
    pub fn board_state_ptr(&self) -> *const [[u64; 4]; 256] {
        let _timer = Timer::new("Game::state");
        self.game.board().raw_state()
    }

    /// Export game objects positions.
    pub fn game_objects(&self) -> GameObjects {
        let lit_ball = self.game.lit_ball();
        let dark_ball = self.game.dark_ball();
        let ball_radius = self.game.ball_radius();
        let cell_size = self.game.cell_size();

        GameObjects {
            lit_ball_x: lit_ball.x as _,
            lit_ball_y: lit_ball.y as _,
            dark_ball_x: dark_ball.x as _,
            dark_ball_y: dark_ball.y as _,
            cell_size_x: cell_size.x as _,
            cell_size_y: cell_size.y as _,
            ball_radius: ball_radius as _,
        }
    }
}

struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}
