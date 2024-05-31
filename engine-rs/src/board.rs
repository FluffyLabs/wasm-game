//! Game board definitions.
//!
//! The game board is indepdent from the flying balls,
//! it is only responsible for maintaing the board state,
//! i.e. which cells are lit or dark.

use std::fmt::{self, Write};

type RawState = [RawRow; 256];
type RawRow = [u64; 4];
const ROW_PART_SIZE: usize = 64;

/// Row/Column indexing type.
pub type Index = u8;

/// High level cell/ball state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// The cell/ball is lit.
    Lit,
    /// The cell/ball is dark.
    Dark,
}

/// A representation of the game board.
///
/// The game board is a square filled with cells that can be either
/// "Lit" or "Dark".
///
/// Within the dimensions of the board there are two balls that move
/// in a continous manner (independently from the cells).
/// One ball is "Lit", second one "Dark".
///
/// When the ball hits a cell that is of the opposite kind,
/// it bounces of it and flips the cell state ("Lit" <> "Dark").
pub struct Board {
    /// Size of the board - both width and height (number of cells).
    size: Index,

    /// A bit vector representing every cell on the board.
    ///
    /// The cell state can be either "Lit" or "Dark" represented by `1` and `0`
    /// correspondigly.
    ///
    /// The cells in a single row are represented by consecutive bits (left to right),
    /// rows are concatenated from top to bottom into the state.
    state: RawState,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        StateOps::debug(f, self.size, &self.state)?;
        f.debug_struct("Board").field("size", &self.size).finish()
    }
}

impl Board {
    /// Create and initialize a new board of given size.
    pub fn new(size: Index) -> Self {
        assert!(size > 1, "The board is too small");
        let state = StateOps::initial(size);

        Self { size, state }
    }

    /// Return the size of the board.
    pub fn size(&self) -> Index {
        self.size
    }

    /// Inspect a single raw of the game board.
    pub fn row(&self, row_index: Index) -> Row {
        assert!(row_index < self.size, "The row index is beyond board size.");
        let row = StateOps::row(&self.state, row_index);
        Row { board: self, row }
    }

    /// Inspect a single cell state at given row and column index.
    pub fn cell(&self, row_index: Index, col_index: Index) -> State {
        self.row(row_index).cell(col_index)
    }

    /// Flip the cell state at given row and column index.
    pub fn flip(&mut self, row_index: Index, col_index: Index) {
        assert!(row_index < self.size, "The row index is beyond board size.");
        assert!(
            col_index < self.size,
            "The column index is beyond board size."
        );
        let row = StateOps::row_mut(&mut self.state, row_index);
        StateOps::flip(row, col_index);
    }
}

/// A view of a single row of the game board.
#[derive(Debug)]
pub struct Row<'a> {
    board: &'a Board,
    row: &'a RawRow,
}

impl<'a> Row<'a> {
    /// Inspect the state of a single cell within the row.
    ///
    /// Panics in case the column index is greater than the size of the game board.
    pub fn cell(&self, col_index: Index) -> State {
        assert!(
            col_index < self.board.size,
            "The column index is beyond board size."
        );
        StateOps::cell(self.row, col_index)
    }
}

struct StateOps;

impl StateOps {
    pub fn initial(size: Index) -> RawState {
        let mut state = [[0u64; 4]; 256];
        let is_size_odd = size % 2 > 0;
        for row_index in 0..size {
            let row = &mut state[row_index as usize];
            let max_cell_to_lit = if is_size_odd && row_index % 2 > 0 {
                size / 2 + 1
            } else {
                size / 2
            };
            for col_index in 0..max_cell_to_lit {
                let (part_index, bit_index) = Self::part_and_bit_index(col_index);
                let cell = 1 << bit_index;
                row[part_index] += cell;
            }
        }
        state
    }

    pub fn row(state: &RawState, row_index: Index) -> &RawRow {
        let row_state = &state[row_index as usize];
        row_state
    }

    pub fn row_mut(state: &mut RawState, row_index: Index) -> &mut RawRow {
        let row_state = &mut state[row_index as usize];
        row_state
    }

    pub fn cell(row: &RawRow, col_index: Index) -> State {
        let (part_index, bit_index) = Self::part_and_bit_index(col_index);
        let cell = 1 << bit_index;
        if row[part_index] & cell > 0 {
            State::Lit
        } else {
            State::Dark
        }
    }

    fn debug(f: &mut fmt::Formatter, size: Index, state: &RawState) -> fmt::Result {
        for row_index in 0..size {
            let row = state[row_index as usize];
            let (max_part_index, max_bit) = Self::part_and_bit_index(size);
            for part_index in 0..=max_part_index {
                let mut part = row[part_index];
                let max = if part_index < max_part_index {
                    ROW_PART_SIZE
                } else {
                    max_bit
                };
                for _ in 0..max {
                    f.write_char(if part & 0b1 > 0 { '▣' } else { '▢' })?;
                    part >>= 1;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }

    fn flip(row: &mut RawRow, col_index: Index) {
        let (part_index, bit_index) = Self::part_and_bit_index(col_index);
        let cell = 1 << bit_index;
        let part = &mut row[part_index];
        *part = *part ^ cell;
    }

    fn part_and_bit_index(col_index: Index) -> (usize, usize) {
        let col_index = col_index as usize;
        let part_index = col_index / ROW_PART_SIZE;
        let bit_index = ROW_PART_SIZE.min(col_index - part_index * 64);
        (part_index, bit_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_set_initial_state() {
        for size in [4, 8, 9, 16, 64, 128, 255] {
            let board = Board::new(size);
            for i in 0..size {
                let r0 = board.row(i);
                for j in 0..size {
                    let state = if j < size / 2 {
                        State::Lit
                    // in case of odd size, every odd row has one more cell lit.
                    } else if j == size / 2 && i % 2 == 1 && size % 2 == 1 {
                        State::Lit
                    } else {
                        State::Dark
                    };
                    assert_eq!(r0.cell(j), state);
                }
            }
        }
    }

    #[test]
    fn should_flip_the_state_at_location() {
        let mut board = Board::new(4);
        let cell = board.row(3).cell(2);
        assert_eq!(cell, State::Dark);

        // when
        board.flip(3, 2);

        // then
        let cell = board.row(3).cell(2);
        assert_eq!(cell, State::Lit);
    }

    #[test]
    fn should_debug_board_properly() {
        let board = Board::new(5);

        let view = format!("\n{:?}", board);

        assert_eq!(
            view,
            r#"
▣▣▢▢▢
▣▣▣▢▢
▣▣▢▢▢
▣▣▣▢▢
▣▣▢▢▢
Board { size: 5 }"#
        );
    }
}
