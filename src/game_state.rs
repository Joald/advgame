use std::collections::HashMap;
use console::Action;
use std::cell::UnsafeCell;
use std::fmt;


#[derive(Serialize, Deserialize)]
pub struct Statistic {
    pub name: String,
    pub default_value: i32,
}

#[derive(Serialize, Deserialize)]
pub struct StageOption {
    pub target_stage: usize,
    pub text: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Stage {
    pub index: usize,
    pub name: String,
    pub text: Vec<String>,
    pub options: Vec<StageOption>,
    #[serde(skip)]
    pub current_option: usize,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    name: String,
    stats: Vec<Statistic>,
    stages: Vec<Stage>,
    #[serde(rename = "entry_stage")]
    current_stage: usize,
    exit_stage: usize,
    #[serde(skip)]
    finished: bool,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Name: \"{}\"\nTODO rest", self.name)
    }
}

impl GameState {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn post_process(mut self) -> Result<GameState, String> {
        // Check if all stages except the last one have at least one option.
        {
            let it =
                self.stages.iter().filter(|stage| { stage.options.len() == 0 });
            let mut found_last = false;
            for stage in it {
                if stage.index != self.exit_stage {
                    return Err(format!("Stage nr. {} has no options and isn't the final stage!", stage.index));
                }
                found_last = true;
            }
            if !found_last {
                return Err("Exit stage can't contain options!".to_string());
            }
        }

        // Map all stage numbers to array indices.
        let mut mapper = HashMap::new();
        for (i, stage) in self.stages.iter().enumerate() {
            mapper.insert(stage.index, i);
        }
        for stage in self.stages.iter_mut() {
            for option in stage.options.iter_mut() {
                if option.text.is_empty() {
                    return Err(format!("No text provided for option in stage {}", stage.index));
                }
                option.target_stage = match mapper.get(&option.target_stage) {
                    Some(&ind) => ind,
                    None => return Err(format!(
                        "Option \"{}\" in stage {}, \"{}\" points to an inexistent stage {}.",
                        option.text[0], stage.index, stage.text[0], option.target_stage
                    ))
                }
            }
            stage.index = *mapper.get(&stage.index).expect(
                "Post processing of data failed. It's a bug on our side. Sorry!"
            ); // panic because this should never happen regardless of input data
        }
        self.current_stage = *match mapper.get(&self.current_stage) {
            Some(st) => st,
            None => return Err("Entry stage is invalid!".to_string())
        };
        self.exit_stage = *match mapper.get(&self.exit_stage) {
            Some(st) => st,
            None => return Err("Exit stage is invalid!".to_string())
        };

        // Make sure we start in the correct stage
        self.stages[self.current_stage].enter();
        Ok(self)
    }

    pub unsafe fn change_to_stage_index(state: &UnsafeCell<GameState>, stage: usize) {
        let from = &mut (*state.get()).current_stage;
        dprintln!("Changing from {} to {}", *from, stage);
        if *from != stage {
            GameState::get_current_stage_mut(state).leave();
            *from = stage;
            GameState::get_current_stage_mut(state).enter();
        }
    }

    pub unsafe fn get_current_stage_mut(state: &UnsafeCell<GameState>) -> &mut Stage {
        &mut (*state.get()).stages[(*state.get()).current_stage]
    }

    pub fn get_current_stage(&self) -> &Stage {
        &self.stages[self.current_stage]
    }

    pub unsafe fn handle_action(self, action: &Action) -> GameState {
        let state = UnsafeCell::new(self);

        { // new scope so state borrows end before end of fn
            let stage = GameState::get_current_stage_mut(&state);
            match action {
                Action::Up => stage.change_option(Direction::Up),
                Action::Down => stage.change_option(Direction::Down),
                Action::Confirm => {
                    if stage.options.len() == 0 {}
                    let index = stage.get_current_option_target();
                    if index.is_none() {
                        dprintln!("Invalid option selection. Check changes of current option.");
                        (*state.get()).finished = true
                    } else {
                        GameState::change_to_stage_index(&state, index.unwrap());
                    }
                }
                Action::Number(num) =>
                    if stage.has_option(num.to_owned()) {
                        GameState::change_to_stage_index(&state, num.to_owned());
                    }
                _ => {} //yes rust, these are all the options I want
            }
        };
        state.into_inner()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

pub enum Direction {
    Up,
    Down,
}

impl Stage {
    pub fn change_option(&mut self, dir: Direction) {
        let old = self.current_option;
        match dir {
            Direction::Up => {
                self.current_option -= 1;
                if self.current_option == 0 {
                    self.current_option = self.options.len();
                }
            }
            Direction::Down => {
                self.current_option += 1;
                if self.current_option == self.options.len() + 1 {
                    self.current_option = 1;
                }
            }
        }
        dprintln!("Moving arrow from {} to {}", old, self.current_option);
    }

    pub fn has_option(&self, option_nr: usize) -> bool {
        !self.options.is_empty() && 0 < option_nr && option_nr <= self.options.len()
    }

    pub fn get_option_target(&self, option_nr: usize) -> Option<usize> {
        if self.has_option(option_nr) {
            Some(self.options[option_nr - 1].target_stage)
        } else {
            None
        }
    }

    pub fn get_current_option_target(&self) -> Option<usize> {
        if self.options.is_empty() {
            None
        } else {
            self.get_option_target(self.current_option)
        }
    }

    pub fn enter(&mut self) {
        self.current_option = 1;
    }

    #[allow(unused)]
    pub fn leave(&mut self) {
        // left for future
    }
}