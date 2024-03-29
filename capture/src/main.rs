use libmzx::board::{enter_board, run_board_update, LabelAction};
use libmzx::{load_world, render, Counters, World};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Read};
use std::path::Path;
use std::process::exit;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const CHAR_BYTES: usize = 14;
const CHAR_WIDTH: usize = 8;
const BYTES_PER_PIXEL: usize = 3;
const BUFFER_SIZE: usize = WIDTH * HEIGHT * CHAR_WIDTH * CHAR_BYTES * BYTES_PER_PIXEL;

#[derive(Clone, PartialEq)]
struct Framebuffer([u8; BUFFER_SIZE]);

impl libmzx::Renderer for Framebuffer {
    fn put_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let stride = WIDTH * CHAR_WIDTH;
        let start = y * stride + x;
        let pixels = &mut self.0[start * BYTES_PER_PIXEL..(start + 1) * BYTES_PER_PIXEL];
        pixels.copy_from_slice(&[r, g, b]);
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

fn run(img_path: &Path, data_path: &Path, world_path: &Path, board_id: Option<usize>) {
    let data = fs::read_to_string(&data_path).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&data).unwrap();
    v.as_object_mut().unwrap().insert(
        "world".to_string(),
        world_path.file_name().unwrap().to_str().unwrap().into(),
    );

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
    let mut board_id = match board_id {
        None => loop {
            let id = random_number::random!(0..world.boards.len());
            let board = &world.boards[id].0;
            if board.width != 0 && board.height != 0 {
                break id;
            }
        },
        Some(id) => id,
    };

    let audio = DummyAudio;

    let world_title: String = world.boards[0].0.title.iter().map(|&c| c as char).collect();

    let (ref mut board, ref mut robots) = world.boards[board_id];
    let board_title: String = board
        .title
        .iter()
        .map(|&c| c as char)
        .collect();
    println!("Capturing board {}: {}", board_id, board_title,);

    let player_pos = board.player_pos;
    enter_board(
        &mut world.state,
        &audio,
        board,
        player_pos,
        robots,
        &mut world.global_robot,
        LabelAction::RunJustLoadedAndJustEntered,
        false,
    );

    v.as_object_mut()
        .unwrap()
        .insert("board".to_string(), board_title.into());

    let title = v
        .as_object()
        .unwrap()
        .get("title")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    if [
        "BKZX",
        "Day of Zeux",
        "MZX Olympics",
        "Week of Zeux",
        "Weekend of Zeux",
    ]
    .iter()
    .any(|t| title.contains(t))
    {
        let obj = v.as_object_mut().unwrap();
        obj.insert("title".to_string(), world_title.into());
        obj.insert("author".to_string(), title.into());
    }

    let mut counters = Counters::new();
    let boards: Vec<_> = world.boards.iter().map(|(b, _)| b.title.clone()).collect();

    let mut canvas = Framebuffer([0; BUFFER_SIZE]);
    const TIMEOUT: usize = 250;
    const MAX_DELAY: usize = 15;
    let mut delay = MAX_DELAY;
    let mut last_frame = canvas.clone();
    let mut cycles = 0;
    let is_title_screen = board_id == 0 && world.boards.len() > 1;
    loop {
        cycles += 1;
        let _ = run_board_update(
            &mut world,
            &audio,
            &world_path,
            &mut counters,
            &boards,
            &mut board_id,
            None,
        );

        render_game(&world, board_id, &mut canvas, is_title_screen);

        if cycles == TIMEOUT {
            println!("Heuristics gave up after {} cycles.", TIMEOUT);
            break;
        }

        let pixels = &canvas.0[0..BYTES_PER_PIXEL];
        if canvas.0.chunks(BYTES_PER_PIXEL).any(|p| p != pixels) {
            // First frame is not a uniform colour, let's take it.
            if false && cycles == 1 {
                break;
            }
            if last_frame != canvas {
                // Wait for some cycles to look for a stable image;
                delay = MAX_DELAY;
            } else if delay == 0 {
                // We've had a stable image for a number of cycles, take it.
                break;
            } else {
                // This image is stable, so it's still a candidate.
                delay -= 1;
            }
        }
        last_frame = canvas.clone();
    }

    println!("Ran {} cycles before non-uniform frame.", cycles);

    let file = File::create(img_path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, WIDTH as u32 * 8, HEIGHT as u32 * 14);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&canvas.0).unwrap();

    fs::write(data_path, &serde_json::to_string(&v).unwrap()).unwrap();
}

fn render_game(world: &World, board_id: usize, canvas: &mut Framebuffer, is_title_screen: bool) {
    let board = &world.boards[board_id].0;
    render(
        &world.state,
        (
            board.upper_left_viewport,
            board.viewport_size,
        ),
        board.scroll_offset,
        &board,
        &world.boards[board_id].1,
        canvas,
        is_title_screen,
    );
}

fn main() {
    env_logger::init();
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!(
            "Usage: cargo run /path/to/img.png /path/to/data.json /path/to/world.mzx [board id]"
        )
    } else {
        run(
            Path::new(&args[1]),
            Path::new(&args[2]),
            Path::new(&args[3]),
            args.get(4).and_then(|a| a.parse().ok()),
        );
    }
}
