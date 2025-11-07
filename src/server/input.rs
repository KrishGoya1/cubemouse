use enigo::{Enigo, Settings, Mouse, Coordinate};

pub fn handle_move(dx: i16, dy: i16) {
    
    let mut enigo = Enigo::new(&Settings::default())
        .expect("Failed to initialize Enigo");

    let factor = 1.5;
    let rel_x = (dx as f32 * factor) as i32;
    let rel_y = (dy as f32 * factor) as i32;

    // Use move_mouse with relative coordinate
    enigo.move_mouse(rel_x, rel_y, Coordinate::Rel)
        .expect("Failed to move mouse");
}
