const std = @import("std");
const debug = std.debug;

const RawState = [256][4]u64;
const Index = u8;

pub const State = enum {
    Lit,
    Dark,
};

pub const Board = struct {
    size: Index,
    state: RawState,

    pub fn init(size: Index) Board {
        debug.assert(size > 1);

        var board = Board{
            .size = size,
            .state = undefined, // Initialize the state array as undefined
        };

        // initialize the state within `size`
        StateOps.initialize(&board.state, size);

        return board;
    }

    pub fn cell(self: *const Board, row_index: Index, cell_index: Index) State {
        return StateOps.cell(&self.state, row_index, cell_index);
    }

    pub fn flip(self: *Board, row_index: Index, cell_index: Index) void {
        StateOps.flip(&self.state, row_index, cell_index);
    }

    pub fn print(self: *const Board) void {
        for (0..self.size) |row_idx| {
            for (0..self.size) |col_idx| {
                const cell_state = StateOps.cell(&self.state, @intCast(row_idx), @intCast(col_idx));
                switch (cell_state) {
                    State.Lit => debug.print("▣", .{}),
                    State.Dark => debug.print("▢", .{}),
                }
            }
            debug.print("\n", .{});
        }
    }
};

const StateOps = struct {
    fn initialize(data: *RawState, size: Index) void {
        const size_odd = size % 2;

        for (0..size) |row_idx| {
            data[row_idx] = [4]u64{ 0, 0, 0, 0 };
            const row_odd = row_idx % 2;
            const max_cells_to_lit = size / 2 + size_odd * row_odd;
            for (0..max_cells_to_lit) |col_idx| {
                const part_idx = col_idx / 64;
                const bit_idx: u6 = @intCast(col_idx - part_idx * 64);
                const cell_value = @as(u64, 1) << bit_idx;
                data[row_idx][part_idx] += cell_value;
            }
        }
    }

    fn cell(data: *const RawState, row_index: Index, cell_index: Index) State {
        const row = data[row_index];
        const part_index = cell_index / 64;
        const bit_index: u6 = @intCast(cell_index - part_index * 64);

        const part = row[part_index];
        const cell_v = @as(u64, 1) << bit_index;
        return if (part & cell_v > 0) State.Lit else State.Dark;
    }

    fn flip(data: *RawState, row_index: Index, cell_index: Index) void {
        const row = &data[row_index];
        const part_index = cell_index / 64;
        const bit_index: u6 = @intCast(cell_index - part_index * 64);

        const cell_v = @as(u64, 1) << bit_index;
        row[part_index] ^= cell_v;
    }
};
