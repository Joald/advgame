use std::collections::HashMap;

pub struct Stage {
    pub name: String,
    pub neighbors: Vec<(usize, String)>,
    pub content: String,
    pub starting: bool,
    pub current_option: usize
}

impl Stage {
    pub fn new() -> Stage {
        Stage {
            name: String::new(),
            neighbors: Vec::new(),
            content: String::new(),
            starting: false,
            current_option: 1
        }
    }
    pub fn print(&self) {
        println!("Stage \"{}\"{}:\n{}", self.name, if self.starting { " INIT" } else { "" }, self.content);
        for neighbor in &self.neighbors {
            println!("    {}. {}", neighbor.0, neighbor.1)
        }
        println!()
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.content.is_empty() && self.neighbors.is_empty()
    }
}

impl Clone for Stage {
    fn clone(&self) -> Self {
        if self.is_empty() {
            Stage::new()
        } else {
            panic!("Attempting to clone an existing stage!")
        }
    }
}

pub struct GameState {
    pub name: String,
    pub stats: HashMap<String, usize>,
    pub stages: Vec<Stage>,
    pub current_stage: Option<usize>,
    pub exit_stage: Option<usize>
}

// invariant: stage graph and content are always the same size
impl GameState {
    pub fn new() -> GameState {
        GameState {
            name: "".to_string(),
            stats: HashMap::new(),
            stages: Vec::new(),
            current_stage: None,
            exit_stage: None
        }
    }
    pub fn print(&self) {
        print_game_state(self)
    }
}

pub fn print_game_state(state: &GameState) {
    println!("Here's what we got so far:\nName: {}\nStats:", state.name);
    for (name, val) in &state.stats {
        println!("{}: {}", name, val);
    }
    println!("Stages:");
    for stage in &state.stages {
        if !stage.is_empty() {stage.print();}
    }
}