use crate::board::{self, Board};

pub type Coordinate = f32;
pub type Timestamp = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: Coordinate,
    pub y: Coordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Movement {
    angle: u16, // from 0 (right) to 359 clockwise
    speed: u8,
}

impl Movement {
    fn apply(&self, time_diff: f32, position: &mut Position) {
        let position_diff = self.speed as f32 / 64.0 * time_diff / 4.0;

        let angle = self.angle as f32;
        let mut y_component = (angle % 90.0) / 90.0;
        let mut x_component = 1.0 - y_component;

        let quadrant = angle / 90.0;
        if quadrant >= 1.0 && quadrant < 2.0 {
            x_component = -x_component;
        }
        if quadrant >= 2.0 && quadrant < 3.0 {
            x_component = -x_component;
            y_component = -y_component;
        }
        if quadrant >= 3.0 {
            y_component = -y_component;
        }

        position.x += position_diff * x_component;
        position.y += position_diff * y_component;
    }

    fn bounce(&mut self) {
        // TODO [ToDr] angle variations.
        self.angle = (self.angle + 180) % 360;
        // self.speed += 1;
    }
}

#[derive(Debug)]
pub struct Game {
    board: Board,
    viewport_size: Position,
    time: Timestamp,
    cell_size: Position,
    lit_ball: (Position, Movement),
    dark_ball: (Position, Movement),
}

impl Game {
    pub fn new(
        board: Board,
        start_time: Timestamp,
        viewport_size: Position,
    ) -> Self {
        let half_view = Position {
            x: viewport_size.x / 2.0,
            y: viewport_size.y / 2.0,
        };
        let y = half_view.y;

        let init_pos_dark = Position { x: half_view.x / 2.0 + half_view.x, y};
        let init_pos_lit = Position { x: half_view.x / 2.0, y};

        let movement_dark = Movement {
            angle: 180,
            speed: 1,
        };

        let movement_lit = Movement {
            angle: 0,
            speed: 1,
        };

        let cell_size = Position {
            x: viewport_size.x / board.size() as Coordinate,
            y: viewport_size.y / board.size() as Coordinate,
        };

        assert!(cell_size.x > 1.0, "The viewport size is too small to draw a cell");
        assert!(cell_size.y > 1.0, "The viewport size is too small to draw a cell");

        Self {
            board,
            time: start_time,
            viewport_size,
            cell_size,
            lit_ball: (init_pos_lit, movement_lit),
            dark_ball: (init_pos_dark, movement_dark),
        }
    }
}

pub fn game_loop(game: &mut Game, time: Timestamp) {
    assert!(time > game.time, "The time did not change!");
    let time_diff = (time - game.time) as f32;
    
    for (obj, kind) in [
        (&mut game.lit_ball, board::State::Lit),
        (&mut game.dark_ball, board::State::Dark)
    ] {
        // 1. move objects
        let (position, movement) = obj;
        movement.apply(time_diff, position);

        // 2. check collisions:
        //  2.2. With boundaries
        //      2.2.1 bounce balls
        Collisions::boundaries(position, movement, &game.viewport_size);
        //  2.1. With board items: 
        //      2.1.1. flip board elements
        //      2.1.2. bounce balls
        Collisions::board(position, movement, &game.cell_size, &mut game.board, kind);
    }
}

struct Collisions;

impl Collisions {
    fn boundaries(
        position: &mut Position,
        movement: &mut Movement,
        viewport_size: &Position,
    ) {
        let mut should_bounce = false;
        // check collisions with the environment.
        if position.x < 0.0 {
            position.x = 0.0;
            should_bounce = true;
        }
        if position.x >= viewport_size.x {
            position.x = viewport_size.x - 1.0;
            should_bounce = true;
        }
        if position.y < 0.0 {
            position.y = 0.0;
            should_bounce = true;
        }
        if position.y >= viewport_size.y {
            position.y = viewport_size.y - 1.0;
            should_bounce = true;
        }

        // bounce only once!
        if should_bounce {
            movement.bounce();
        }
    }

    fn board(
        position: &mut Position,
        movement: &mut Movement,
        cell_size: &Position,
        board: &mut Board,
        kind: board::State,
    ) {
        let cell_x = (position.x / cell_size.x).floor() as board::Index;
        let cell_y = (position.y / cell_size.y).floor() as board::Index;
    
        let at_kind = board.cell(cell_y, cell_x);
        if kind != at_kind {
            // flip the cell
            board.flip(cell_y, cell_x);
            // now bounce the ball depending on the approach angle.
            // TODO [ToDr] calculate angle
            movement.bounce();
        }
    }
}
