const std = @import("std");

const Board = @import("Board.zig").Board;
const Game = @import("Game.zig");
const Position = Game.Position;

pub const GameObjects = struct {
    lit_ball_x: u32,
    lit_ball_y: u32,
    dark_ball_x: u32,
    dark_ball_y: u32,
    cell_size_x: u32,
    cell_size_y: u32,
    ball_radius: u32,
};

pub const WasmGame = struct {
    game: Game.Game,

    pub fn new(board_size: u8, viewport_x: u32, viewport_y: u32, start_time_ms: u64) WasmGame {
        // game init
        const board = Board.init(board_size);
        const viewport_size = Position{
            .x = @floatFromInt(viewport_x),
            .y = @floatFromInt(viewport_y),
        };
        const game = Game.Game.new(board, start_time_ms, viewport_size);

        return WasmGame{ .game = game };
    }

    pub fn tick(self: *WasmGame, time_ms: u64) void {
        self.game.tick(time_ms);
    }

    pub fn board_state_ptr(self: *const WasmGame) *const [256][4]u64 {
        return &self.game.board.state;
    }

    pub fn fill_game_objects(self: *const WasmGame, go: *GameObjects) void {
        const lit_ball = self.game.lit_ball;
        const dark_ball = self.game.dark_ball;
        const cell_size = self.game.cell_size;
        const ball_radius = self.game.ball_radius;

        go.lit_ball_x = @intFromFloat(lit_ball.position.x);
        go.lit_ball_y = @intFromFloat(lit_ball.position.y);
        go.dark_ball_x = @intFromFloat(dark_ball.position.x);
        go.dark_ball_y = @intFromFloat(dark_ball.position.y);
        go.cell_size_x = @intFromFloat(cell_size.x);
        go.cell_size_y = @intFromFloat(cell_size.y);
        go.ball_radius = @intFromFloat(ball_radius);
    }
};

pub export fn wasm_game_fill(game: *WasmGame, objects: *GameObjects) void {
    game.fill_game_objects(objects);
}

pub export fn wasm_game_board_ptr(
    game: *WasmGame,
) *const [256][4]u64 {
    return game.board_state_ptr();
}

pub export fn wasm_game_tick(
    game: *WasmGame,
    time_ms: u64,
) void {
    game.tick(time_ms);
}

pub export fn wasm_game_new(
    board_size: u8,
    viewport_x: u32,
    viewport_y: u32,
    start_time_ms: u64,
) *const WasmGame {
    return &WasmGame.new(board_size, viewport_x, viewport_y, start_time_ms);
}
pub export fn new_game_objects() *GameObjects {
    return std.heap.page_allocator.create(GameObjects) catch unreachable;
}

pub fn main() u8 {
    var wasm_game = WasmGame.new(16, 512, 512, 0);
    const game_objects = new_game_objects();
    wasm_game.game.board.flip(12, 12);
    wasm_game.fill_game_objects(game_objects);
    //wasm_game.game.board.print();

    wasm_game.tick(1);
    //wasm_game.game.board.print();

    //std.debug.print("{}", .{game_objects});

    return 0;
}
