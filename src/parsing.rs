use std::fs::File;
use std::io::{Error as IOError, Read};
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use crate::game_state::GameState;

pub fn open_json_file(filename: &str) -> Result<String, IOError> {
    const EXTENSION: &str = ".json";
    if !filename.ends_with(EXTENSION) {
        return Err(IOError::new(ErrorKind::InvalidInput, format!("\"{}\" is not a JSON file!", filename)));
    }
    let path = Path::new(filename);
    let mut ret = String::new();
    let file_size = File::open(path)?.read_to_string(&mut ret)?;
    ret = ret[..file_size].to_string();
    Ok(ret)
}

pub fn parse_game(filename: &str) -> Result<GameState, String> {
    let file = match open_json_file(filename) {
        Ok(f) => f,
        Err(err) => return Err(err.to_string())
    };
    match serde_json::from_str::<GameState>(&file) {
        Ok(st) => st.post_process(),
        Err(err) => Err(format!("Error while parsing the JSON file, line {}:{}\n{}\n",
                                err.line(), err.column(), err
        ))
    }
}

pub fn print_format(force_regen: bool) {
    const FORMAT_FILE: &str = "format.txt";
    let path = Path::new(FORMAT_FILE);
    if force_regen || !path.exists() {
        let command = "python3 create_format.py";
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", command])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("failed to execute process")
        };
    }
    let mut buf = String::new();
    File::open(path)
        .and_then(|mut file| file.read_to_string(&mut buf))
        .and_then(|size| Ok(buf[..size].to_string()))
        .and_then(|format_str| Ok(print!("{}", format_str)))
        .expect("There was an error. Sorry!")
}