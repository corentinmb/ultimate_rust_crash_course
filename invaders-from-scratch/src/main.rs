use cargo::frame::{new_frame, Drawable};
use cargo::player::Player;
use cargo::render;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, ExecutableCommand};
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
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
    let mut instant = Instant::now();
    // Game loop
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

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

        // Draw & render
        player.draw(&mut curr_frame);
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
