const std = @import("std");
const Board = @import("./Board.zig").Board;

const debug = std.debug;

pub const Coordinate = f32;
pub const Timestamp = u64;

pub const Position = struct {
    x: Coordinate,
    y: Coordinate,
};

pub const Movement = struct {
    angle: u16,
    speed: u8,

    pub fn apply(self: *const Movement, time_diff_ms: f32, position: *Position) void {
        const position_diff = (@as(f32, self.speed) / @as(f32, INITIAL_SPEED)) * time_diff_ms / 2.0;

        const angle = @as(f32, self.angle);
        const a_component = @as(f32, self.angle % 90) / 90.0;
        const b_component = 1.0 - a_component;

        const quadrant = angle / 90.0;
        const x_y = if (quadrant < 1.0) {
            .{ b_component, a_component };
        } else if (quadrant < 2.0) {
            .{ -a_component, b_component };
        } else if (quadrant < 3.0) {
            .{ -a_component, -b_component };
        } else {
            .{ a_component, -b_component };
        };
        const x_component = x_y[0];
        const y_component = x_y[1];

        position.x += position_diff * x_component;
        position.y += position_diff * y_component;
    }

    pub fn bounce(self: *Movement, collision_type: CollisionType) void {
        const speed_factor = @as(u16, self.speed) * 3 / @as(u16, INITIAL_SPEED);
        self.angle = switch (collision_type) {
            CollisionType.Horizontal => (540 - self.angle + speed_factor) % 360,
            CollisionType.Vertical => (360 - self.angle + speed_factor) % 360,
        };
        self.speed = (self.speed + 1).min(2 * INITIAL_SPEED);
    }
};

pub const CollisionType = enum {
    Horizontal,
    Vertical,
};

const INITIAL_SPEED: u8 = 100;

pub const Game = struct {
    board: Board,
    viewport_size: Position,
    time: Timestamp,
    cell_size: Position,
    ball_radius: Coordinate,
    lit_ball: struct { position: Position, movement: Movement },
    dark_ball: struct { position: Position, movement: Movement },

    pub fn new(board: Board, start_time_ms: Timestamp, viewport_size: Position) Game {
        const half_view = Position{
            .x = viewport_size.x / 2.0,
            .y = viewport_size.y / 2.0,
        };
        const init_pos_dark = Position{
            .x = half_view.x / 2.0 + half_view.x,
            .y = half_view.y,
        };
        const init_pos_lit = Position{
            .x = half_view.x / 2.0,
            .y = half_view.y,
        };
        const movement_dark = Movement{
            .angle = 220,
            .speed = INITIAL_SPEED,
        };
        const movement_lit = Movement{
            .angle = 40,
            .speed = INITIAL_SPEED,
        };
        const cell_size = Position{
            .x = viewport_size.x / @as(f32, @floatFromInt(board.size)),
            .y = viewport_size.y / @as(f32, @floatFromInt(board.size)),
        };
        debug.assert(cell_size.x > 1);
        debug.assert(cell_size.y > 1);

        const ball_radius = (cell_size.x + cell_size.y) / 4.0;

        return Game{
            .board = board,
            .time = start_time_ms,
            .viewport_size = viewport_size,
            .cell_size = cell_size,
            .ball_radius = ball_radius,
            .lit_ball = .{ .position = init_pos_lit, .movement = movement_lit },
            .dark_ball = .{ .position = init_pos_dark, .movement = movement_dark },
        };
    }

    pub fn tick(self: *Game, time_ms: Timestamp) void {
        self.time = time_ms;
    }
};
