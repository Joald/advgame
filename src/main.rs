//#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate ncurses;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use parsing::parse_game;
use playback::play_game;

use crate::parsing::print_format;

#[macro_use]
mod debug;
mod misc;
mod playback;
mod game_components;
mod game_state;
mod parsing;
mod console;

fn main() {
    // TODO: replace this with a real arg parser
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} filename", args[0]);
        std::process::exit(1);
    }
    const FORMAT_ARG: &str = "--format";
    const FORMAT_REGEN_ARG: &str = "-regen";

    if args[1] == "--help" {
        println!("\
        Welcome to Text Adventure Parser 3000!\n\
        To play a game, run:\n\
        {name} game-file.agf\n\
        To display game file format, run:\n\
        {name} {format}\n\
        To also force the regeneration, run:\n\
        {name} {format}{regen}\n\

        Copyright © 2018 Jacek Olczyk",
                 name = args[0],
                 format = FORMAT_ARG,
                 regen = FORMAT_REGEN_ARG);
        std::process::exit(0);
    }

    if args[1] == FORMAT_ARG {
        print_format(false);
        std::process::exit(0);
    }
    if args[1] == FORMAT_ARG.to_string() + FORMAT_REGEN_ARG {
        print_format(true);
        std::process::exit(0);
    }

    let init_state = match parse_game(&args[1][..]) {
        Ok(st) => st,
        Err(err) => {
            println!("Error parsing game: {}", err);
            return;
        }
    };

    play_game(init_state);
}
