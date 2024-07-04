import { WasmGame } from "engine-rs-js";
import { memory } from "engine-rs-js/engine_rs_js_bg";

const size = 480;
const time = () => BigInt(Date.now());

const $canvas = document.getElementById('canvas');
$canvas.setAttribute("width", size);
$canvas.setAttribute("height", size);

let previous_time = time() - BigInt(1);
const board_size = 16;
const game = WasmGame.new(board_size, size, size, previous_time);

const cell_size = size / board_size;
const LIT_BALL = "rgb(200, 200, 200)";
const LIT_CELL = "rgb(50,50,50)";
const DARK_BALL = "rgb(70, 70, 70)";
const DARK_CELL = "rgb(180, 180, 180)";

const render = () => {
  const ctx = $canvas.getContext('2d');
  const gameObjects = game.game_objects();

  renderCells(ctx);
  renderBalls(ctx, gameObjects);
};

const renderBalls = (ctx, g) => {
  const drawBall = (x, y, radius) => {
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2, false);
    ctx.fill();
  };

  ctx.fillStyle = LIT_BALL;
  drawBall(g.lit_ball_x, g.lit_ball_y, g.ball_radius);

  ctx.fillStyle = DARK_BALL;
  drawBall(g.dark_ball_x, g.dark_ball_y, g.ball_radius);
};

const renderCells = (ctx) => {
  const cellsPtr = game.board_state_ptr();
  const cells = new BigUint64Array(memory.buffer, cellsPtr, 256 * 4);

  const drawCell = (row, col) => {
    ctx.fillRect(col * cell_size, row * cell_size, cell_size, cell_size);
  };
  ctx.fillStyle = LIT_CELL;
  forEveryCell(cells, (row, col, isSet) => {
    if (isSet) {
      drawCell(row, col);
    }
  });

  ctx.fillStyle = DARK_CELL;
  forEveryCell(cells, (row, col, isSet) => {
    if (!isSet) {
      drawCell(row, col);
    }
  });
};

const forEveryCell = (cells, callback) => {
  for (let row = 0; row < board_size; ++row) {
    const row_parts = cells.slice(row * 4);
    for (let part_idx = 0; part_idx < (board_size + 63) / 64; ++part_idx) {
      let part = row_parts[part_idx];
      const col_max = Math.min(board_size, (part_idx + 1) * 64);
      for (let col = part_idx * 64; col < col_max; ++col) {
        const isSet = 0x1n & part; 
        part = part >> 1n;
        
        callback(row, col, isSet);
      }
    }
  }
};

const tick = () => {
  const current_time = time();
  if (previous_time < current_time) {
    game.tick(current_time);
    previous_time = current_time;
  }
};

const tickAndRender = () => {
  tick();
  render();
  requestAnimationFrame(tickAndRender);
};

render();
requestAnimationFrame(tickAndRender);
