use engine_rs::board::State;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::video::Window;
use sdl2::{pixels::Color, render::Canvas};
use std::time;

use engine_rs::{
    board::Board,
    game::{game_loop, Game, Position},
};

fn main() -> Result<(), String> {
    // sdl init
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let size = 640;

    let window = video_subsystem
        .window("Arkanoid Example", size, size)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // game init
    let board = Board::new(16);
    let viewport_size = Position {
        x: size as f32,
        y: size as f32,
    };
    let start_time = time::Instant::now();
    let mut game = Game::new(board, 0, viewport_size);

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        std::thread::sleep(time::Duration::from_millis(50));
        // process events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        // process the game
        let new_time = time::Instant::now().duration_since(start_time).as_millis();
        game_loop(&mut game, new_time as u64);

        render(&mut canvas, &game)?;
    }

    Ok(())
}

fn render(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    game: &Game,
) -> Result<(), String> {
    canvas.clear();

    let lit_color = Color::RGB(80, 250, 60);
    let dark_color = Color::RGB(175, 238, 238);

    let board = game.board();
    let board_size = board.size();
    let cell_size = game.cell_size();
    for row in 0..board_size {
        for col in 0..board_size {
            let kind = board.cell(col, row);
            canvas.set_draw_color(match kind {
                State::Dark => Color::RGB(152, 251, 152),
                State::Lit => Color::RGB(135, 206, 250),
            });
            canvas.fill_rect(Rect::new(
                row as i32 * cell_size.x as i32,
                col as i32 * cell_size.y as i32,
                cell_size.x as u32,
                cell_size.y as u32,
            ))?;
        }
    }

    let ball_radius = ((cell_size.x + cell_size.y) / 4.0) as i32;

    canvas.set_draw_color(dark_color);
    let b = game.lit_ball();
    draw_filled_circle(canvas, b.x as i32, b.y as i32, ball_radius)?;

    canvas.set_draw_color(lit_color);
    let b = game.dark_ball();
    draw_filled_circle(canvas, b.x as i32, b.y as i32, ball_radius)?;

    canvas.present();
    Ok(())
}

fn draw_filled_circle(
    canvas: &mut Canvas<Window>,
    x0: i32,
    y0: i32,
    radius: i32,
) -> Result<(), String> {
    let mut x = radius;
    let mut y = 0;
    let mut radius_error = 1 - x;

    while x >= y {
        draw_line(canvas, x0 - x, y0 + y, x0 + x, y0 + y)?;
        draw_line(canvas, x0 - y, y0 + x, x0 + y, y0 + x)?;
        draw_line(canvas, x0 - x, y0 - y, x0 + x, y0 - y)?;
        draw_line(canvas, x0 - y, y0 - x, x0 + y, y0 - x)?;

        y += 1;
        if radius_error < 0 {
            radius_error += 2 * y + 1;
        } else {
            x -= 1;
            radius_error += 2 * (y - x + 1);
        }
    }

    Ok(())
}

fn draw_line(
    canvas: &mut Canvas<Window>,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> Result<(), String> {
    canvas.draw_line(Point::new(x1, y1), Point::new(x2, y2))?;
    Ok(())
}
