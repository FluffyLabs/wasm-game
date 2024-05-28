use engine_rs::{board::Board, game::{game_loop, Game, Position}};


fn main() {
    let board = Board::new(16);
    let viewport_size = Position { x: 160.0, y: 160.0 };
    let start_time = std::time::Instant::now();
    let mut game = Game::new(board, 0, viewport_size);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let new_time = std::time::Instant::now().duration_since(start_time).as_millis();
        game_loop(&mut game, new_time as u64);

        println!("{:?}", game);
    }
}
