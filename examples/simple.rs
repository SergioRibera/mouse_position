use mouse_position::{Mouse, MouseExt};

fn main() {
    let mut mouse = Mouse::default();

    loop {
        let (x, y) = match mouse.get_pos() {
            Ok((x, y)) => (x, y),
            Err(e) => {
                println!("{e:?}");
                continue;
            }
        };

        println!("x: {x}, y: {y}");

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
