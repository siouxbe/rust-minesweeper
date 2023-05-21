use piston_window as pw;

#[derive(Default)]
pub struct Input {
    pub cursor_pos: CursorPosition,
    pub draw_size: DrawSize,
    pub mouse_down: MouseDown,
}

#[derive(Clone, Default, Copy)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for CursorPosition {
    fn from(pos: [f64; 2]) -> Self {
        Self {
            x: pos[0],
            y: pos[1],
        }
    }
}

impl From<CursorPosition> for [f64; 2] {
    fn from(pos: CursorPosition) -> Self {
        [pos.x, pos.y]
    }
}

#[derive(Copy, Clone, Default)]
pub struct DrawSize {
    pub width: u32,
    pub height: u32,
}

impl From<[u32; 2]> for DrawSize {
    fn from(s: [u32; 2]) -> Self {
        Self {
            width: s[0],
            height: s[1],
        }
    }
}

#[derive(Default)]
pub struct MouseDown {
    pub left: bool,
    pub right: bool,
}

impl MouseDown {
    fn left_or_right<F>(state: pw::ButtonState, when_down: F, button: &mut bool, other_button: bool)
    where
        F: FnOnce(),
    {
        match state {
            pw::ButtonState::Press => {
                let mouse_free = !*button && !other_button;
                if mouse_free {
                    *button = true;
                    when_down()
                }
            }
            pw::ButtonState::Release => {
                *button = false;
            }
        }
    }

    pub fn left<F>(&mut self, state: pw::ButtonState, when_down: F)
    where
        F: FnOnce(),
    {
        Self::left_or_right(state, when_down, &mut self.left, self.right)
    }

    pub fn right<F>(&mut self, state: pw::ButtonState, when_down: F)
    where
        F: FnOnce(),
    {
        Self::left_or_right(state, when_down, &mut self.right, self.left)
    }
}
