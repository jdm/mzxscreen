use libmzx::{Counters, World, load_world, render};
use libmzx::board::{enter_board, update_board};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;
use std::process::exit;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const CHAR_BYTES: usize = 14;
const CHAR_WIDTH: usize = 8;
const BYTES_PER_PIXEL: usize = 4;
const BUFFER_SIZE: usize = WIDTH * HEIGHT * CHAR_WIDTH * CHAR_BYTES * BYTES_PER_PIXEL;

struct Framebuffer([u8; BUFFER_SIZE]);

impl libmzx::Renderer for Framebuffer {
    fn put_pixel(
        &mut self,
        x: usize,
        y: usize,
        r: u8,
        g: u8,
        b: u8,
    ) {
        let stride = WIDTH * CHAR_WIDTH;
        let start = y * stride + x;
        let pixels = &mut self.0[start * BYTES_PER_PIXEL..(start + 1) * BYTES_PER_PIXEL];
        pixels.copy_from_slice(&[r, g, b, 255]);
    }

    fn clear(&mut self) {
        for p in &mut self.0 {
            *p = 0;
        }
    }
}

struct DummyAudio;
impl libmzx::audio::AudioEngine for DummyAudio {
    fn mod_fade_in(&self, _file_path: &str) {}
    fn load_module(&self, _file_path: &str) {}
    fn end_module(&self) {}
    fn mod_fade_out(&self) {}
    fn set_mod_order(&self, _order: i32) {}
}

fn run(img_path: &Path, world_path: &Path, board_id: Option<usize>) {
    let world_data = match File::open(&world_path) {
        Ok(mut file) => {
            let mut v = vec![];
            file.read_to_end(&mut v).unwrap();
            v
        }
        Err(e) => {
            println!("Error opening {} ({})", world_path.display(), e);
            exit(1)
        }
    };

    let mut world = match load_world(&world_data) {
        Ok(world) => world,
        Err(e) => {
            println!("Error reading {} ({:?})", world_path.display(), e);
            exit(1)
        }
    };

    let world_path = Path::new(&world_path).parent().unwrap();
    let (board_id, is_title_screen) = match board_id {
        Some(0) | None => (0, true),
        Some(id) => (id, false),
    };

    let audio = DummyAudio;

    let player_pos = world.boards[board_id].player_pos;
    enter_board(
        &mut world.state,
        &audio,
        &mut world.boards[board_id],
        player_pos,
        &mut world.all_robots
    );

    let mut counters = Counters::new();
    let boards: Vec<_> = world.boards.iter().map(|b| b.title.clone()).collect();

    let _ = update_board(
        &mut world.state,
        &audio,
        None,
        &world_path,
        &mut counters,
        &boards,
        &mut world.boards[board_id],
        board_id,
        &mut world.all_robots
    );

    let mut canvas = Framebuffer([0; BUFFER_SIZE]);
    render_game(&world, board_id, &mut canvas, is_title_screen);

    let file = File::create(img_path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, WIDTH as u32 * 8, HEIGHT as u32 * 14);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&canvas.0).unwrap();
}

fn render_game(
    world: &World,
    board_id: usize,
    canvas: &mut Framebuffer,
    is_title_screen: bool,
) {
    let robots_start = world.boards[board_id].robot_range.0;
    let robots_end = robots_start + world.boards[board_id].robot_range.1;
    let robots = &world.all_robots[robots_start..robots_end];
    render(
        &world.state,
        (
            world.boards[board_id].upper_left_viewport,
            world.boards[board_id].viewport_size,
        ),
        world.boards[board_id].scroll_offset,
        &world.boards[board_id],
        robots,
        canvas,
        is_title_screen,
    );
}

fn main() {
    env_logger::init();
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: cargo run /path/to/img.png /path/to/world.mzx [board id]")
    } else {
        run(Path::new(&args[1]), Path::new(&args[2]), args.get(3).and_then(|a| a.parse().ok()));
    }
}
