use clap::{Parser, Subcommand};
use icy_engine::{SaveOptions, Buffer};
use icy_engine_egui::animations::Animator;
use std::{fs, path::PathBuf, time::Duration};

#[derive(Parser)]
pub struct Cli {
    #[arg(
        help = "If true modern terminal output (UTF8) is used.",
        long,
        default_value_t = false
    )]
    utf8: bool,
    
    #[arg(help = "File to play/show.", required = true)]
    path: Option<PathBuf>,


    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Plays the animation (default)")]
    Play,

    #[command(about = "Show a specific frame of the animation")]
    ShowFrame { frame: usize },
}

fn main() {
    let args = Cli::parse();

    if let Some(path) = args.path {
        let parent = Some(path.parent().unwrap().to_path_buf());

        let Some(ext) = path.extension() else {
            println!("Error: File extension not found.");
            return;  
        };

        let ext = ext.to_string_lossy().to_ascii_lowercase();

        match ext.as_str() {
            "icyanim" => {
                match fs::read_to_string(path) {
                    Ok(txt) => {
                        let animator = Animator::run(&parent, &txt);
                        if let Ok(animator) = animator {
                            let animator = animator.lock().unwrap();
                            let mut opt: SaveOptions = SaveOptions::default();
                            if args.utf8 {
                                opt.modern_terminal_output = true;
                            }
                            match args.command.unwrap_or(Commands::Play) {
                                Commands::Play => {
                                    for (buffer, _, delay) in animator.frames.iter() {
                                        show_buffer(buffer, args.utf8);
                                        std::thread::sleep(Duration::from_millis(*delay as u64));
                                    }
                                }
                                Commands::ShowFrame { frame } => {
                                    show_buffer(&animator.frames[frame].0, args.utf8);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error opening file: {e}");
                    }
                }
            }
            _ => {
                let buffer = Buffer::load_buffer(&path, true);
                if let Ok(buffer) = &buffer {
                    show_buffer(buffer, args.utf8);
                }
            }
        }

    }
}

fn show_buffer(buffer: &Buffer, use_utf8: bool) {
    let mut opt: SaveOptions = SaveOptions::default();
    if use_utf8 {
        opt.modern_terminal_output = true;
    }

    let mut bytes = buffer.to_bytes("ans", &opt).unwrap();
    // print!("\x1BP0;1;0!z");
    print!("\x1b[2J\x1b[0;0H\x1b[0; D");
    if use_utf8 {
        // remove BOM
        bytes.drain(0..3);
        print!("{}", String::from_utf8(bytes).unwrap());
    } else {
        for b in bytes {
            print!("{}", b as char);
        }
    }
    // print!("\x1b[0*z");
}