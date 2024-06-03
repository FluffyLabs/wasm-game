//! Game physics.
//!
//! The components here are responsible for moving objects
//! over the board. In particular we track:
//! 1. The "lit" ball.
//! 2. The "dark" ball.
//!
//! Since collisions and physics are tightly coupled with
//! how objects are going to be rendered, this part of the
//! code is aware of the sizes of objects within the coordinate space.
use crate::board::{self, Board};

/// Space coordinate type.
///
/// (0, 0) is as a top-left corner of the space.
/// Physics is calculated using floating point operations,
/// but obviously the rendering needs to project these points
/// into solid pixels, however obviously the rendering resolution
/// might be higher / lower than the physics resolution.
pub type Coordinate = f32;

/// Timestamp type (milliseconds).
pub type Timestamp = u64;

/// Position or dimensions of some object on the screen within the coordinate space.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// `x` coordinate of the position.
    pub x: Coordinate,
    /// `y` coordinate of the position.
    pub y: Coordinate,
}

const INITIAL_SPEED: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Movement {
    /// Movement angle from 0 (right) to 359 clockwise.
    angle: u16,
    /// Speed of movement (see also [`INITIAL_SPEED`]).
    ///
    /// The speed should be roughly between `INITIAL_SPEED` and `2 * INITIAL_SPEED`.
    speed: u8,
}

impl Movement {
    /// Apply the movement to given position.
    ///
    /// The method will alter the next position the object is at.
    /// Note we do not take boundaries or other objects into account here,
    /// so the new position might be out of bounds.
    fn apply(&self, time_diff_ms: f32, position: &mut Position) {
        let position_diff = (self.speed as f32 / INITIAL_SPEED as f32) * time_diff_ms / 2.0;

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

        position.x += position_diff * x_component;
        position.y += position_diff * y_component;
    }

    /// Reflect the movement, after the object has hit some obstacle.
    ///
    /// The rebound angle is matching the approach angle, however
    /// there is slight (deterministic) skew based on the speed of the object.
    fn bounce(&mut self, collision_type: CollisionType) {
        let speed_factor = self.speed as u16 * 3 / INITIAL_SPEED as u16;
        self.angle = match collision_type {
            CollisionType::Horizontal => (540 - self.angle + speed_factor) % 360,
            CollisionType::Vertical => (360 - self.angle + speed_factor) % 360,
        };
        self.speed = (self.speed + 1).min(2 * INITIAL_SPEED);
    }
}

/// Main game object encapsulating all parts of the game.
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
    /// View the board state.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Get the position of the lit ball.
    pub fn lit_ball(&self) -> &Position {
        &self.lit_ball.0
    }

    /// Get the position of the dark ball.
    pub fn dark_ball(&self) -> &Position {
        &self.dark_ball.0
    }

    /// Get the board cell size in coordinate space.
    pub fn cell_size(&self) -> &Position {
        &self.cell_size
    }

    /// Get the ball radius.
    pub fn ball_radius(&self) -> Coordinate {
        self.ball_radius
    }

    /// Create new game object.
    ///
    /// Given coordinate space dimensions (viewport size), the underlying
    /// board and the starting time in milliseconds.
    pub fn new(board: Board, start_time_ms: Timestamp, viewport_size: Position) -> Self {
        let half_view = Position {
            x: viewport_size.x / 2.0,
            y: viewport_size.y / 2.0,
        };
        let y = half_view.y;

        let init_pos_dark = Position {
            x: half_view.x / 2.0 + half_view.x,
            y,
        };
        let init_pos_lit = Position {
            x: half_view.x / 2.0,
            y,
        };

        let movement_dark = Movement {
            angle: 220,
            speed: INITIAL_SPEED,
        };

        let movement_lit = Movement {
            angle: 40,
            speed: INITIAL_SPEED,
        };

        let cell_size = Position {
            x: viewport_size.x / board.size() as Coordinate,
            y: viewport_size.y / board.size() as Coordinate,
        };

        assert!(
            cell_size.x > 1.0,
            "The viewport size is too small to draw a cell"
        );
        assert!(
            cell_size.y > 1.0,
            "The viewport size is too small to draw a cell"
        );

        let ball_radius = (cell_size.x + cell_size.y) / 4.0;

        Self {
            board,
            time: start_time_ms,
            viewport_size,
            cell_size,
            ball_radius,
            lit_ball: (init_pos_lit, movement_lit),
            dark_ball: (init_pos_dark, movement_dark),
        }
    }

    /// Recalculate objects positions and check collisions.
    pub fn tick(&mut self, time_ms: Timestamp) {
        assert!(time_ms > self.time, "The time did not change!");
        let time_diff_ms = (time_ms - self.time) as f32;
        self.time = time_ms;

        for (obj, kind) in [
            (&mut self.lit_ball, board::State::Lit),
            (&mut self.dark_ball, board::State::Dark),
        ] {
            // 1. move objects
            let (position, movement) = obj;
            movement.apply(time_diff_ms, position);

            // 2. check collisions:
            //  2.2. With boundaries
            //      2.2.1 bounce balls
            Collisions::boundaries(position, movement, self.ball_radius, &self.viewport_size);
            //  2.1. With board items:
            //      2.1.1. flip board elements
            //      2.1.2. bounce balls
            Collisions::board(
                position,
                movement,
                self.ball_radius,
                &self.cell_size,
                &mut self.board,
                kind,
            );
        }
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
                    // check if it's actually colliding
                    let cell_center_x = (cell_x as f32 + 0.5) * cell_size.x;
                    let cell_center_y = (cell_y as f32 + 0.5) * cell_size.y;

                    let distance_x = (cell_center_x - position.x).abs();
                    let distance_y = (cell_center_y - position.y).abs();

                    let distance_sq = distance_x * distance_x + distance_y * distance_y;
                    let cell_size_avg = (cell_size.x + cell_size.y) / 4.0;
                    let max_distance = cell_size_avg * 0.95 + ball_radius;
                    let max_distance_sq = max_distance * max_distance;

                    if distance_sq < max_distance_sq {
                        // flip the cell
                        board.flip(cell_y, cell_x);
                        collision_type = if (cell_center_x - position.x).abs()
                            < (cell_center_y - position.y).abs()
                        {
                            Some(CollisionType::Vertical)
                        } else {
                            Some(CollisionType::Horizontal)
                        };
                    }
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
            (90, 90),   //edge case?
            (270, 270), //edge case?
            (105, 75),
        ];

        for (angle, expected) in values {
            let mut mov = Movement { angle, speed: 0 };
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
            (0, 0),     //edge case?
            (180, 180), //edge case?,
            (120, 240),
            (280, 80),
        ];

        for (angle, expected) in values {
            let mut mov = Movement { angle, speed: 0 };
            // when
            mov.bounce(CollisionType::Vertical);

            // then
            assert_eq!(mov.angle, expected);
        }
    }

    #[test]
    fn should_not_find_collisions() {
        let kind = board::State::Lit;
        let ball_radius = 10f32;

        let cell_size = Position { x: 10.0, y: 10.0 };
        let mut position = Position { x: 10.0, y: 15.0 };

        let mut movement = Movement {
            angle: 90,
            speed: 1,
        };
        let mut board = Board::new(5);

        // we are only touching Lit board cells,
        // so there should be no collisions.
        // However bounding rectangle is touching cells at (2,0) and (2,2)
        // hence we are testing if these collisions are omitted.
        Collisions::board(
            &mut position,
            &mut movement,
            ball_radius,
            &cell_size,
            &mut board,
            kind,
        );

        // no change
        assert_eq!(movement.angle, 90);
        assert_eq!(movement.speed, 1);
    }
}
