use cargo::frame::{new_frame, Drawable};
use cargo::invaders::Invader;
use cargo::player::Player;
use cargo::render;
use cargo::shot::Shot;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, ExecutableCommand};
use rand::{thread_rng, Rng};
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
use std::thread::current;
use std::time::{Duration, Instant};
use std::{io, thread};

pub type InvadersResult = Result<(), Box<dyn Error>>;

fn main() -> InvadersResult {
    let mut audio = Audio::new();
    let mut stdout = io::stdout();

    let sounds = ["explode", "lose", "move", "pew", "startup", "win"];

    for sound in sounds.iter() {
        let path = format!("sounds/{}.wav", sound);
        audio.add(sound, &path);
    }
    audio.play("startup");

    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();
    let mut invaders = Vec::new();
    let mut instant = Instant::now();
    // Game loop
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
        let mut rng = thread_rng();
        if rng.gen_bool(1.0 / 1000.0) {
            invaders.push(Invader::new());
        }

        while event::poll(Duration::default())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    KeyCode::Left => {
                        player.move_left();
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Right => {
                        player.move_right();
                    }
                    _ => {}
                }
            }
        }

        // Update the timers
        player.update(delta);
        invaders
            .iter_mut()
            .for_each(|mut invader| invader.update(delta));

        // Draw & render
        invaders
            .iter_mut()
            .find(|invader| invader.y == player.y - 1 && invader.x == player.x)
            .map(|invader| {
                invader.exploding = true;
                player.exploding = true;
            });

        invaders
            .iter_mut()
            .find(|invader| {
                player
                    .shots
                    .iter()
                    .any(|shot| shot.y - 1 == invader.y && shot.x == invader.x)
            })
            .map(|invader| {
                player
                    .shots
                    .iter_mut()
                    .find(|shot| shot.y - 1 == invader.y)
                    .unwrap()
                    .exploding = true;
                invader.exploding = true;
            });

        player.draw(&mut curr_frame);
        invaders
            .iter()
            .for_each(|invader| invader.draw(&mut curr_frame));
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
