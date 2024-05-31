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
/// Game object.
pub struct WasmGame {
    game: Game,
}

#[wasm_bindgen]
impl WasmGame {
    /// Create a new game given viewport size (pixels), the board size and starting time.
    pub fn new(board_size: u8, viewport_x: u64, viewport_y: u64, start_time_ms: u64) -> Self {
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
        self.game.tick(time_ms)
    }
}
