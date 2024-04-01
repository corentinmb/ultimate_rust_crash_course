use std::time::Duration;

use rand::{thread_rng, Rng};
use rusty_time::Timer;

use crate::frame::{Drawable, Frame};
use crate::{NUM_COLS, NUM_ROWS};

pub struct Invader {
    pub x: usize,
    pub y: usize,
    pub exploding: bool,
    timer: Timer,
}

impl Invader {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            x: rng.gen_range(0..NUM_COLS),
            y: 0,
            exploding: false,
            timer: Timer::new(Duration::from_millis(1000)),
        }
    }

    pub fn move_down(&mut self) {
        if self.y < NUM_ROWS - 1 {
            self.y += 1;
        }
    }

    pub fn explode(&mut self) {
        self.exploding = true;
        self.timer = Timer::new(Duration::from_millis(250));
    }

    pub fn update(&mut self, delta: Duration) {
        self.timer.tick(delta);

        if self.timer.finished() && !self.exploding {
            self.move_down();
            self.timer.reset();
        }
    }

    pub fn dead(&self) -> bool {
        (self.timer.finished() && self.exploding) || (self.y == NUM_ROWS)
    }
}

impl Drawable for Invader {
    fn draw(&self, frame: &mut Frame) {
        frame[self.x][self.y] = if self.exploding { "*" } else { "V" };
    }
}
