use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    render,player::{self, Player}, invaders::Invaders,
};
use rusty_audio::Audio;
use std::{error::Error, io, sync::mpsc, thread, time::{Duration, Instant}};

fn  main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");

    audio.add("move", "move.wav");

    audio.add("pew", "pew.wav");

    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");

    audio.play("lose");
    audio.play("win");
    //terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide);

    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame:Frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => panic!("Unexpected value"),
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player =Player::new();
    let mut instant = Instant::now();
    let mut invaders= Invaders::new();
    'gameloop: loop {
        let mut curr_frame = new_frame();

        let delta = instant.elapsed();
        instant=Instant::now();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    KeyCode::Char(' ')| KeyCode::Enter=>{
                        if player.shoot(){
                            audio.play("pew");
                        }
                    }
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    _ => {},
                    
                }
            }
        }
        
        player.update(delta);
        if invaders.update(delta){
            audio.play("move");
        }
        if player.detect_hits(&mut invaders){
            audio.play("explode");
        }


        let drawables: Vec<&dyn Drawable>= vec![&player,&invaders];

        for drawables in drawables{
            drawables.draw(&mut curr_frame)
        }
        // ingnoring the error until the channel is setup
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
        //win or lose?

        if invaders.all_killed(){
            audio.play("win");
            break 'gameloop;

        }
        if invaders.reached_bottom(){
            audio.play("lose");
            break 'gameloop;

        }
    }
    drop(render_tx);
    render_handle.join();


    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
