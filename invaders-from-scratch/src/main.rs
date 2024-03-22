use crossterm::cursor::{Hide, Show};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{terminal, ExecutableCommand};
use rusty_audio::Audio;
use std::error::Error;
use std::io;
use std::io::Stdout;

type InvadersResult = Result<(), Box<dyn Error>>;

fn main() -> InvadersResult {
    let mut audio = Audio::new();
    let mut stdout = io::stdout();

    init_sounds(&mut audio);
    init_terminal(&mut stdout);
    cleanup(&mut audio, &mut stdout);
    Ok(())
}

fn cleanup(audio: &Audio, stdout: &mut Stdout) -> InvadersResult {
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn init_terminal(stdout: &mut Stdout) -> InvadersResult {
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    Ok(())
}

fn init_sounds(audio: &mut Audio) -> InvadersResult {
    let sounds = ["explode", "lose", "move", "pew", "startup", "win"];

    for sound in sounds.iter() {
        let path = format!("sounds/{}.wav", sound);
        audio.add(sound, &path);
    }
    audio.play("startup");
    Ok(())
}
