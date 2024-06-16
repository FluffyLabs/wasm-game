const Board = @import("Board.zig").Board;

pub export fn main() u8 {
    var board = Board.init(16);
    board.flip(12, 12);
    board.print();
    return 0;
}
