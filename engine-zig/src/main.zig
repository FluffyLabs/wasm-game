const std = @import("std");

extern fn print(a: [*c]const u8) void;

export fn mul(a: i32, b: i32) i32 {
    return a * b;
}

pub export fn main() void {
    print("Hello Debug!");
}
