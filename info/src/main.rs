use libmzx::load_world;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

fn main() {
    let world_path = std::env::args().nth(1).unwrap();
    let world_path = Path::new(&world_path);
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

    let _world = match load_world(&world_data) {
        Ok(world) => world,
        Err(e) => {
            println!("Error reading {} ({:?})", world_path.display(), e);
            exit(1)
        }
    };
}
