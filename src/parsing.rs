use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::error::Error;

use game_state::GameState;

pub fn open_agf_file(filename: &str) -> (File, String) {
    if &filename[filename.len() - 4..] != ".agf" {
        panic!("Not an .agf file!");
    }
    let path = std::path::Path::new(filename);
    let display = path.display();
    (match File::open(path) {
        Err(err) => panic!("couldn't open {}: {}", display, err.description()),
        Ok(file) => file
    }, display.to_string())
}

enum ParsingState {
    Entry,
    NameParsed,
    StatsParsing,
    StatsParsed,
    StagesParsing,
    StageParsing,
    StageTextParsing(usize),
    // stage index
    NeighboursParsing(usize),
    // stage index
    NeighbourTextParsing(usize, usize),
    // stage index, neighbor index
    End,
}

use game_state::print_game_state;
use game_state::Stage;


const STATS_BEGIN: &str = "STATS[[";
const STATS_END: &str = "]];";
const STAT_BEGIN: &str = "STAT[";
const STAT_END: &str = "]";
const STAGES_BEGIN: &str = "STAGES[[";
const STAGES_END: &str = "]]";
const STAGE_BEGIN: &str = "STAGE[";
const STAGE_END: &str = "],";
const BEGIN_STAGE_MARKER: &str = "BEGIN";
const END_STAGE_MARKER: &str = "END";

pub const GRAMMAR: &str = concat!(
    "This is the adventure game definition format.\n",
    "\\n stands for a newline, \"\" is an empty string.\n",
    "Leading and trailing whitespace in the line are skipped.\n",
    "All items ending with the '$' can be multiline.\n",
    "Text in [% these brackets %] is optional.\n",
    "All endlines are mandatory.\n",
    "<game>   ::= <game_name>\\nSTATS[[\\n<stats>\\n]];\\nSTAGES[[\\n<stages>\\n]];\n",
    "<stats>  ::= <stat>, <stats> | \"\"\n",
    "<stat>   ::= STAT[<name>: <default_val>] \n",
    "<stages> ::= STAGE[\\n<stage>\\n],\\n <stages> | <stage> \n",
    "<stage>  ::= [% BEGIN | END %]<stage_num>. <stage_name>: <stage_text>$\\n<neis>\n",
    "<neis>   ::= <nei>\\n <neis> | \"\"\n",
    "<nei>    ::= <nei_num> - <nei_text>$"
);

fn parse_stage_text(line: &str, game: &mut GameState, nr: usize) -> Result<ParsingState, &'static str> {
    if line.is_empty() {
        Ok(ParsingState::StageTextParsing(nr))
    } else if line.ends_with("$") {
        let line = &line[..line.len() - 1];
        game.stages[nr].content.push_str(" ");
        game.stages[nr].content.push_str(line);

        Ok(ParsingState::NeighboursParsing(nr))
    } else {
        game.stages[nr].content.push_str(" ");
        game.stages[nr].content.push_str(line);
        Ok(ParsingState::StageTextParsing(nr))
    }
}

fn parse_nei_text(line: &str, game: &mut GameState, nr: usize, nei_nr: usize) -> Result<ParsingState, &'static str> {
    let nei_name = &mut game.stages[nr].neighbors[nei_nr].1;
    *nei_name += " ";
    if line.is_empty() {
        Ok(ParsingState::NeighbourTextParsing(nr, nei_nr))
    } else if line.ends_with("$") {
        *nei_name += &line[..line.len() - 1];
        Ok(ParsingState::NeighboursParsing(nr))
    } else {
        *nei_name += line;
        Ok(ParsingState::NeighbourTextParsing(nr, nei_nr))
    }
}

fn parse_line(line: &str, state: ParsingState, game: &mut GameState) -> Result<ParsingState, &'static str> {
    match state {
        ParsingState::Entry => {
            game.name = line.to_string();
            Ok(ParsingState::NameParsed)
        }
        ParsingState::NameParsed => match line {
            STATS_BEGIN => Ok(ParsingState::StatsParsing),
            _ => Err("Game name must be followed by stats.")
        },
        ParsingState::StatsParsing => {
            if line == STATS_END {
                Ok(ParsingState::StatsParsed)
            } else if line.starts_with(STAT_BEGIN) && line.ends_with(STAT_END) {
                let mut stat = line[STAT_BEGIN.len()..line.len() - 1].split(": ");
                let name = match stat.next() {
                    Some(str) => str,
                    None => {
                        return Err("Stat is empty.");
                    }
                };
                let value = match stat.next() {
                    Some(val) => {
                        match val.parse::<usize>() {
                            Ok(i) => i,
                            Err(_) => return Err("Cannot convert stat value to 32-bit integer.")
                        }
                    }
                    None => return Err("No stat value provided.")
                };
                game.stats.insert(name.to_string(), value);
                Ok(ParsingState::StatsParsing)
            } else {
                Err("Incorrect stat format.")
            }
        }
        ParsingState::StatsParsed => match line {
            STAGES_BEGIN => Ok(ParsingState::StagesParsing),
            _ => Err("Stages specification not found")
        }
        ParsingState::StagesParsing => match line {
            STAGE_BEGIN => Ok(ParsingState::StageParsing),
            STAGES_END => Ok(ParsingState::End),
            _ => Err("Incorrect stage format.")
        }
        ParsingState::StageParsing => {
            let mut line = line.to_string();
            let mut change_begin = false;
            let mut change_end = false;
            if line.starts_with(BEGIN_STAGE_MARKER) {
                if game.current_stage.is_some() {
                    return Err("Found at least two stages marked with 'BEGIN'");
                }
                change_begin = true;
                line.drain(..BEGIN_STAGE_MARKER.len() + 1);
                println!("{}", line);
            }
            if line.starts_with(END_STAGE_MARKER) {
                if game.exit_stage.is_some() { // TODO: Add support for multiple END stages.
                    return Err("Found at least two stages marked with 'END'");
                }
                change_end = true;
                line.drain(..END_STAGE_MARKER.len() + 1);
            }
            let nr_end = match line.find('.') {
                Some(nr) => nr,
                None => return Err("Number of the stage must be followed by a dot (.).")
            };
            let nr = match line[..nr_end].parse::<usize>() {
                Ok(nr) => nr,
                Err(_) => return Err("Cannot parse stage number.")
            };
            let line = &line[nr_end + 2..];
            let name_end = match line.find(':') {
                Some(nr) => nr,
                None => return Err("Name of the stage must be followed by a colon (:).")
            };
            let name = &line[..name_end];

            if game.stages.len() <= nr {
                game.stages.resize(nr + 1, Stage::new());
            }

            {// new scope to end mutable borrow before parse_stage_text call
                let stage = &mut game.stages[nr];
                if !stage.is_empty() {
                    return Err("Found duplicate stage number.");
                }
                stage.content.clear();
                stage.name = name.to_string();
            }

            if change_begin {
                game.current_stage = Some(nr);
            }
            if change_end {
                game.exit_stage = Some(nr);
            }
            let line = if name_end + 2 < line.len() { &line[name_end + 2..] } else { "" };
            parse_stage_text(line, game, nr)
        }
        ParsingState::StageTextParsing(nr) => parse_stage_text(line, game, nr),
        ParsingState::NeighboursParsing(nr) => match line {
            STAGE_END => Ok(ParsingState::StagesParsing),
            _ => {
                let mut split = line.split(" - ");
                let nei_index = match split.next() {
                    Some(nei) => match nei.parse::<usize>() {
                        Ok(ind) => ind,
                        Err(_) => return Err("Cannot parse neighbor index as integer.")
                    },
                    None => {
                        return Err("Error while parsing stage option");
                    }
                };
                let nei_text = match split.next() {
                    Some(text) => text,
                    None => { return Err("Error while parsing stage option"); }
                };
                game.stages[nr].neighbors.push((nei_index, "".to_string()));
                let nei_count = game.stages[nr].neighbors.len() - 1;
                parse_nei_text(nei_text, game, nr, nei_count)
            }
        }
        ParsingState::NeighbourTextParsing(nr, nei_nr) => parse_nei_text(line, game, nr, nei_nr),
        _ => Err("Unimplemented")
    }
}

fn verify_state(parsing_state: ParsingState, state: GameState) -> Result<GameState, &'static str> {
    if state.exit_stage.is_none() {
        Err("Missing exit stage. Maybe you didn't add 'END' to the beginning of a stage?")
    } else if state.current_stage.is_none() {
        Err("Missing starting stage. Maybe you didn't add 'BEGIN' to the beginning of a stage?")
    } else if match parsing_state { ParsingState::End => false, _ => true} {
        Err("Parsing ended unexpectedly.")
    } else if state.stages.iter().enumerate().find(|(i, stage)| {
        !stage.is_empty() && stage.neighbors.len() == 0 && state.exit_stage.unwrap() != *i
    }).is_some() {
        Err("Found a dead end stage not marked with END.")
    } else {
        Ok(state)
    }
}

fn get_state<T>(reader: T) -> GameState where T: IntoIterator<Item=(usize, String)> {
    let mut state = ParsingState::Entry;
    let mut game_state = GameState::new();

    for (i, line) in reader {
        if line.is_empty() {
            continue;
        }
        state = match parse_line(&line, state, &mut game_state) {
            Ok(st) => st,
            Err(err) => {
                eprintln!("Cannot parse game at line {}: {}", i, err);
                eprintln!("Line is \"{}\"", line);
                print_game_state(&game_state);
                std::process::exit(1);
            }
        }
    }
    match verify_state(state, game_state) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Game parsed successfully, but the following integrity check failed: {}", err);
            std::process::exit(1)
        }
    }
}

pub fn parse_game(filename: &str) -> GameState {
    let file = open_agf_file(filename);
    let file = BufReader::new(file.0);

    // All lines are trimmed and numbered
    let gen = file.lines().enumerate().map(|(i, line)| match line {
        Err(err) => panic!("Error while reading line {}: {}!", i + 1, err.description()),
        Ok(line) => (i + 1, line.trim().to_string())
    });
    get_state(gen)
}