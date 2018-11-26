use std::fs::File;
use std::io::{Read, Error as IOError};
use game_state::GameState;
use std::io::ErrorKind;

pub fn open_json_file(filename: &str) -> Result<String, IOError> {
    const EXTENSION: &str = ".json";
    if !filename.ends_with(EXTENSION) {
        return Err(IOError::new(ErrorKind::InvalidInput, "Not a JSON file!"));
    }
    let path = std::path::Path::new(filename);
    let mut ret = String::new();
    let file_size = File::open(path)?.read_to_string(&mut ret)?;
    ret = ret[..file_size].to_string();
    Ok(ret)
}

pub fn parse_game(filename: &str) -> Result<GameState, String> {
    let file = match open_json_file(filename) {
        Ok(f) => f, Err(err) => return Err(err.to_string())
    };
    match serde_json::from_str::<GameState>(&file) {
        Ok(st) => st.post_process(),
        Err(err) => Err(format!("Error while parsing the JSON file, line {}:{}\n{}\n",
            err.line(), err.column(), err
        ))
    }
}


pub const FORMAT: &str = "\
struct Statistic {\
    id: usize
    name: String,\
    default_value: i32\
}\
\
struct StageOption {\
    target_stage: usize,\
    text: String[]\
}\
\
struct Stage {\
    index: usize,\
    name: String,\
    text: String[],\
    options: StageOption[]\
}\
\
struct GameState {\
    name: String,\
    stats: Statistic[],\
    stages: Stage[],\
    entry_stage: usize,\
    exit_stage: usize\
}";