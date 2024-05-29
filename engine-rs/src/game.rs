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
        let position_diff = self.speed as f32 / 64.0 * time_diff / 40.0;

        let angle = self.angle as f32;
        let a_component = (self.angle % 90) as f32 / 90.0;
        let b_component = 1.0 - a_component;

        let quadrant = angle / 90.0;
        let (x_component, y_component) = match quadrant {
            q if q < 1.0 => (b_component, a_component),
            q if q >= 1.0 && q < 2.0 => (-a_component, b_component),
            q if q >= 2.0 && q < 3.0 => (-a_component, -b_component),
            _ => (a_component, -b_component),
        };
        
        println!("Angle: {}, x_component: {}, y_component: {}", angle, x_component, y_component);
        position.x += position_diff * x_component;
        position.y += position_diff * y_component;
    }

    fn bounce(&mut self, collision_type: CollisionType) {
        let before_angle = self.angle;
        let speed_factor = self.speed as u16 * 3;
        self.angle = match collision_type {
            CollisionType::Horizontal => (540 - self.angle + speed_factor) % 360,
            CollisionType::Vertical => (360 - self.angle + speed_factor) % 360,
        };

        println!("Bounce! {} -> {}", before_angle, self.angle);
    }
}

#[derive(Debug)]
pub struct Game {
    board: Board,
    viewport_size: Position,
    time: Timestamp,
    cell_size: Position,
    ball_radius: Coordinate,
    lit_ball: (Position, Movement),
    dark_ball: (Position, Movement),
}

impl Game {

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn lit_ball(&self) -> &Position {
        &self.lit_ball.0
    }

    pub fn dark_ball(&self) -> &Position {
        &self.dark_ball.0
    }

    pub fn cell_size(&self) -> &Position {
        &self.cell_size
    }

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
            angle: 225,
            speed: 1,
        };

        let movement_lit = Movement {
            angle: 45,
            speed: 1,
        };

        let cell_size = Position {
            x: viewport_size.x / board.size() as Coordinate,
            y: viewport_size.y / board.size() as Coordinate,
        };

        assert!(cell_size.x > 1.0, "The viewport size is too small to draw a cell");
        assert!(cell_size.y > 1.0, "The viewport size is too small to draw a cell");

        let ball_radius = (cell_size.x + cell_size.y) / 4.0;

        Self {
            board,
            time: start_time,
            viewport_size,
            cell_size,
            ball_radius,
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
        Collisions::boundaries(position, movement, game.ball_radius, &game.viewport_size);
        //  2.1. With board items: 
        //      2.1.1. flip board elements
        //      2.1.2. bounce balls
        Collisions::board(position, movement, game.ball_radius, &game.cell_size, &mut game.board, kind);
    }
}

struct Collisions;

impl Collisions {
    fn boundaries(
        position: &mut Position,
        movement: &mut Movement,
        ball_radius: Coordinate,
        viewport_size: &Position,
    ) {
        let mut collision_type = None;
        // check collisions with the environment.
        if position.x < ball_radius {
            position.x = ball_radius;
            collision_type = Some(CollisionType::Horizontal);
        }
        if position.x >= viewport_size.x - ball_radius {
            position.x = viewport_size.x - ball_radius - 1.0;
            collision_type = Some(CollisionType::Horizontal);
        }
        if position.y < ball_radius {
            position.y = ball_radius;
            collision_type = Some(CollisionType::Vertical);
        }
        if position.y >= viewport_size.y - ball_radius {
            position.y = viewport_size.y - ball_radius - 1.0;
            collision_type = Some(CollisionType::Vertical);
        }

        // bounce only once!
        if let Some(collision_type) = collision_type {
            movement.bounce(collision_type);
        }
    }

    fn board(
        position: &mut Position,
        movement: &mut Movement,
        ball_radius: Coordinate,
        cell_size: &Position,
        board: &mut Board,
        kind: board::State,
    ) {
        let mut collision_type = None;
        for box_x in [position.x + ball_radius, position.x - ball_radius] {
            for box_y in [position.y + ball_radius, position.y - ball_radius] {
                let cell_x = (box_x / cell_size.x).floor() as board::Index;
                let cell_y = (box_y / cell_size.y).floor() as board::Index;

                let at_kind = board.cell(cell_y, cell_x);
                if kind != at_kind {
                    // flip the cell
                    board.flip(cell_y, cell_x);
                    let cell_pos_x  = cell_x as f32 * cell_size.x;
                    let cell_pos_y = cell_y as f32 * cell_size.y;
                    collision_type = if (cell_pos_x - position.x).abs() < (cell_pos_y - position.y).abs() {
                        Some(CollisionType::Horizontal)
                    } else {
                        Some(CollisionType::Vertical)
                    };
                }
            }
        }
        if let Some(collision_type) = collision_type {
            movement.bounce(collision_type);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollisionType {
    Horizontal,
    Vertical,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_horizontal_bounce_angle_correctly() {
        let values = vec![
            (0, 180),
            (15, 165),
            (30, 150),
            (65, 115),
            (89, 91),
            (90, 90), //edge case?
            (270, 270), //edge case?
            (105, 75),
        ];

        for (angle, expected) in values {
            let mut mov = Movement {
                angle,
                speed: 0,
            };
            // when
            mov.bounce(CollisionType::Horizontal);

            // then
            assert_eq!(mov.angle, expected);
        }
    }

    #[test]
    fn should_calculate_vertical_bounce_angle_correctly() {
        let values = vec![
            (90, 270),
            (0, 0), //edge case?
            (180, 180), //edge case?
        ];

        for (angle, expected) in values {
            let mut mov = Movement {
                angle,
                speed: 0,
            };
            // when
            mov.bounce(CollisionType::Vertical);

            // then
            assert_eq!(mov.angle, expected);
        }
    }

}
