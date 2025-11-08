use enigo::{
    Enigo, Settings,
    Mouse,
    Button,
    Direction,
    Coordinate,
};

pub fn handle_move(dx: i16, dy: i16) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    let factor = 1.5;
    enigo.move_mouse(
        (dx as f32 * factor) as i32,
        (dy as f32 * factor) as i32,
        Coordinate::Rel,
    ).unwrap();
}

pub fn handle_left_click() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.button(Button::Left, Direction::Click).unwrap();
}

pub fn handle_right_click() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.button(Button::Right, Direction::Click).unwrap();
}

pub fn handle_scroll(dy: i16) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    let amount = -(dy as i32 / 3); 

    enigo.scroll(amount, enigo::Axis::Vertical).unwrap();
}
