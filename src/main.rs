extern crate ncurses;
mod playback;
mod game_state;
mod parsing;
mod console;
use parsing::parse_game;
use playback::play_game;

use parsing::GRAMMAR;



fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} filename", args[0]);
        std::process::exit(1);
    }

    if args[1] == "--help" {
        println!("\
        Welcome to Text Adventure Parser 3000!\n\
        To play a game, run:\n\
        {name} game-file.agf\n\
        To display game file format, run:\n\
        {name} --format\n\
        Copyright © 2018 Jacek Olczyk", name = args[0]);
        std::process::exit(0);
    }

    if args[1] == "--format" {
        println!("{}", GRAMMAR);
        std::process::exit(0);
    }

    let mut init_state = parse_game(&args[1][..]);

    init_state.print();
    play_game(&mut init_state);
}
