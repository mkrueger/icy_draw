use std::{path::PathBuf, fs, time::Duration};
use clap::Parser;
use icy_engine::SaveOptions;
use icy_engine_egui::animations::Animator;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(help = "icy animation file to play.", required=true)]
    path: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();

    if let Some(path) = args.path {
        let parent = Some(path.parent().unwrap().to_path_buf());

        match fs::read_to_string(path) {
            Ok(txt) => {
                let animator = Animator::run(&parent, &txt);
                if let Ok(animator) = animator {
                    let animator = animator.lock().unwrap();
                    let mut opt = SaveOptions::default();
                    opt.longer_terminal_output = true;

                    for (buffer, _, delay) in animator.frames.iter() {
                        if let Ok(bytes) = buffer.to_bytes("ans", &opt) {
                            // print!("\x1BP0;1;0!z");
                            print!("\x1b[2J\x1b[0;0H\x1b[0; D");
                            for b in bytes {
                                print!("{}", b as char);  
                            }
                            // print!("\x1b[0*z");
                            std::thread::sleep(Duration::from_millis(*delay as u64));
                        }
                    }
                }
            },
            Err(e) => {
                println!("Error opening file: {e}");
            }
        }   
    }
}