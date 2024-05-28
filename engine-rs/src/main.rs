use std::fmt;

type Index = u8;
type RawState = [RawRow; 256];
type RawRow = [u64; 4];
const ROW_PART_SIZE: usize = 64;

struct StateOps;

impl StateOps {
    pub fn initial(size: Index) -> RawState {
        let mut state = [[0u64; 4]; 256];
        for row_index in 0..size {
            let row = &mut state[row_index as usize];
            for cell_index in 0..size / 2 {
                let (part_index, bit_index) = Self::part_and_bit_index(cell_index);
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

    pub fn cell(row: &RawRow, cell_index: Index) -> State {
        let (part_index, bit_index) = Self::part_and_bit_index(cell_index);
        let cell = 1 << bit_index;
        if row[part_index] & cell > 0 {
            State::Lit
        } else {
            State::Dark
        }
    }

    fn debug(f: &mut fmt::Formatter, size: Index, state: &RawState) -> fmt::Result {
        for row_index in 0..size as usize {
            let row = state[row_index];
            let (max_part_index, max_bit) = Self::part_and_bit_index(size);
            for part_index in 0..=max_part_index {
                let mut part = row[part_index];
                let max = if part_index < max_part_index { ROW_PART_SIZE } else { max_bit };
                for _ in 0..max {
                    write!(f, "{:b}", part & 0b1)?;
                    part >>= 1;
                }
            }
        }
        writeln!(f, "")?;
        Ok(())
    }

    fn flip(row: &mut RawRow, cell_index: Index) {
        let (part_index, bit_index) = Self::part_and_bit_index(cell_index);
        let part = &mut row[part_index];
        let cell = 1 << bit_index;
        *part = *part ^ cell;
    }

    fn part_and_bit_index(cell_index: Index) -> (usize, usize) {
        let cell_index = cell_index as usize;
        let part_index = cell_index / ROW_PART_SIZE;
        let bit_index = ROW_PART_SIZE.min(cell_index - part_index * 64);
        (part_index, bit_index)
    }
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
    pub size: Index,

    /// A bit vector representing every cell on the board.
    ///
    /// The cell state can be either "Lit" or "Dark" represented by `1` and `0`
    /// correspondigly.
    ///
    /// The cells in a single row are represented by consecutive bits (left to right),
    /// rows are concatenated from top to bottom into the state.
    pub state: RawState,
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
        assert!(size > 1);
        let state = StateOps::initial(size);

        Board { size, state }
    }

    /// Inspect a single raw of the game board.
    pub fn row(&self, row_index: Index) -> Row {
        assert!(row_index < self.size);
        let row = StateOps::row(&self.state, row_index);
        Row { board: self, row }
    }

    pub fn flip(&mut self, row_index: Index, cell_index: Index) {
        assert!(row_index < self.size);
        assert!(cell_index < self.size);
        let row = StateOps::row_mut(&mut self.state, row_index);
        StateOps::flip(row, cell_index);
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
    /// Panics in case the cell index is greater than the size of the game board.
    pub fn cell(&self, cell_index: Index) -> State {
        assert!(cell_index < self.board.size);
        StateOps::cell(self.row, cell_index)
    }
}

/// High level cell/ball state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// The cell/ball is lit.
    Lit,
    /// The cell/ball is dark.
    Dark,
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_set_initial_state() {
        for size in [4, 8, 16, 64, 128, 255] {
            let board = Board::new(size);
            for i in 0..size {
                let r0 = board.row(i);
                for j in 0..size {
                    let state = if j < size / 2 {
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
}
