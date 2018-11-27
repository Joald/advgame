use std::collections::HashMap;
use console::Action;
use std::fmt;
use std::iter::Iterator;
use std::cell::RefCell;

type StatValue = i32;

pub trait Conditional { fn get_condition(&self) -> &Condition; }

#[derive(Serialize, Deserialize)]
pub struct Statistic {
    pub id: usize,
    pub name: String,
    #[serde(rename = "default_value")]
    pub value: StatValue,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Always,
    IfStatHigher { stat_id: usize, higher_than: StatValue },
    IfStatLower { stat_id: usize, lower_than: StatValue },
    IfStatExact { stat_id: usize, value: StatValue },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    NoEffect,
    SetStatHigher { stat_id: usize, to_add: StatValue },
    SetStatLower { stat_id: usize, to_subtract: StatValue },
    SetStatExact { stat_id: usize, new_value: StatValue },
}

#[derive(Serialize, Deserialize)]
pub struct StageOption {
    pub target_stage: usize,
    pub text: Vec<String>,
    #[serde(default = "Condition::always")]
    pub condition: Condition,
    #[serde(default = "Effect::no_effect")]
    pub effect: Effect,
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
    stages: Vec<RefCell<Stage>>,
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
    pub fn get_name(&self) -> &str { &self.name }

    pub fn post_process(mut self) -> Result<GameState, String> {
        // Check if all stages except the last one have at least one option.
        {
            let it =
                self.stages.iter().filter(|stage| {
                    stage.borrow().options.len() == 0
                });
            let mut found_last = false;
            for stage in it {
                if stage.borrow().index != self.exit_stage {
                    return Err(format!("Stage nr. {} has no options and isn't the final stage!", stage.borrow().index));
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
            dprintln!("Stage {} becomes stage {}!", stage.borrow().index, i);
            mapper.insert(stage.borrow().index, i);
        }

        let mut stat_mapper = HashMap::new();
        for (i, stat) in self.stats.iter().enumerate() {
            dprintln!("Stat {} becomes stat {}!", stat.id, i);
            stat_mapper.insert(stat.id, i);
        }

        for stage in self.stages.iter_mut() {
            let mut stage_index;
            let mut stage_name;
            {
                stage_index = stage.borrow().index;
                stage_name = stage.borrow().name.clone();
            }
            {
                let mapping = |x: usize| {
                    match mapper.get(&x) {
                        Some(&ind) => Ok(ind),
                        None => Err(format!(
                            "Entry \"{}\" in stage {}, \"{}\" points to an inexistent stage.",
                            x, stage_index, stage_name
                        ))
                    }
                };
                let stat_mapping = |x: usize| {
                    match stat_mapper.get(&x) {
                        Some(&ind) => Ok(ind),
                        None => Err(format!(
                            "Entry \"{}\" in stage {}, \"{}\" points to an inexistent stat.",
                            x, stage_index, stage_name
                        ))
                    }
                };

                for option in stage.borrow_mut().options.iter_mut() {
                    if option.text.is_empty() {
                        return Err(format!("No text provided for option in stage {}", stage_index));
                    }

                    option.target_stage = mapping(option.target_stage)?;
                    match option.condition {
                        Condition::Always => {}
                        Condition::IfStatExact { ref mut stat_id, value: _ } |
                        Condition::IfStatHigher { ref mut stat_id, higher_than: _ } |
                        Condition::IfStatLower { ref mut stat_id, lower_than: _ } =>
                            *stat_id = stat_mapping(*stat_id)?,
                    }
                    match option.effect {
                        Effect::NoEffect => {}
                        Effect::SetStatLower {ref mut stat_id, to_subtract: _} |
                        Effect::SetStatHigher {ref mut stat_id, to_add: _} |
                        Effect::SetStatExact {ref mut stat_id, new_value: _} =>
                            *stat_id = stat_mapping(*stat_id)?

                    }
                }
            }
            stage.borrow_mut().index = *mapper.get(&stage_index).expect(
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
        self.enter_current_stage();
        Ok(self)
    }

    fn enter_current_stage(&mut self) {
        { self.get_current_stage().borrow_mut().current_option = 0; }
        self.get_current_stage().replace(
            self.get_current_stage_into_inner()
                .change_option(Direction::Down, self)
        );
        dprintln!("Entered stage {}!", self.get_current_stage().borrow().index);
    }

    pub fn change_to_stage_index(mut self, stage: usize) -> Self {
        dprintln!("Changing stage from {} to {}", self.current_stage, stage);
        if self.current_stage != stage {
            //GameState::get_current_stage_mut(state).leave();
            self.current_stage = stage;
            self.enter_current_stage();
        }
        self
    }

    pub fn get_current_stage(&self) -> &RefCell<Stage> {
        &self.stages[self.current_stage]
    }

    pub fn get_current_stage_into_inner(&self) -> Stage {
        self.stages[self.current_stage].replace(Stage::dummy())
    }

    #[allow(unused)]
    pub fn replace_current_stage(&self, stage: Stage) -> Stage {
        self.stages[self.current_stage].replace(stage)
    }

    pub fn replace_current_stage_with<F: FnOnce(Stage) -> Stage>(&self, f: F) {
        let x = self.stages[self.current_stage].replace(Stage::dummy());
        self.stages[self.current_stage].replace(f(x));
    }

    pub fn handle_action(self, action: &Action) -> GameState {
        let mut stage_change: Option<StageOption> = None;
        let mut finish = self.finished;
        let mut number_press: Option<usize> = None;
        self.replace_current_stage_with(|stage: Stage| {
            match action {
                Action::Up => stage.change_option(Direction::Up, &self),
                Action::Down => stage.change_option(Direction::Down, &self),
                Action::Confirm => {
                    {
                        let index = stage.get_current_option();
                        if stage.options.len() == 0 {
                            dprintln!("Reached the final stage!");
                            finish = true;
                        } else if index.is_none() {
                            dprintln!("Invalid option selection. Check changes of current option.");
                        } else {
                            dprintln!("Changing to current option {} that points to stage {}",
                                stage.current_option, index.unwrap().target_stage
                            );
                            stage_change = Some(index.unwrap().clone())
                        }
                    }
                    stage
                }
                Action::Number(num) => {
                    if stage.has_option(*num) {
                        number_press = Some(*num);
                        dprintln!("Stage may be changed due to {} being pressed.", *num)
                    }
                    stage
                }
                _ => { stage } //yes rust, these are all the options I want
            }
        });
        let mut temp = self;
        if stage_change.is_some() {
            let op = stage_change.unwrap();
            temp = temp.apply_effect(op.effect.clone()).change_to_stage_index(op.target_stage)
        }
        if number_press.is_some() {
            let mut x = None;
            {
                let borrow = &temp.get_current_stage().borrow();
                let mut it = temp.visible_options(borrow);
                let num = it.nth(number_press.unwrap() - 1);
                if num.is_some() {
                    x = Some(num.unwrap().target_stage);
                }
            }
            if x.is_some() {
                temp = temp.change_to_stage_index(x.unwrap())
            }
        }
        temp.finished = finish;
        temp
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn visible_options<'a>(&'a self, stage: &'a Stage) -> impl std::iter::Iterator<Item=&'a StageOption> {
        stage.options.iter()
            .filter(move |option| {
                self.is_filled(*option)
            })
    }

    pub fn is_filled<T>(&self, conditional: &T) -> bool where T: Conditional {
        match conditional.get_condition() {
            Condition::Always => true,
            Condition::IfStatHigher { stat_id, higher_than } =>
                self.stats[*stat_id].value > *higher_than,
            Condition::IfStatLower { stat_id, lower_than } =>
                self.stats[*stat_id].value < *lower_than,
            Condition::IfStatExact { stat_id, value } =>
                self.stats[*stat_id].value == *value,
        }
    }

    pub fn is_index_visible(&self, stage: &Stage, index: usize) -> bool {
        let mut it = stage.options.iter();
        let x = match it.nth(index) {
            Some(option) => option,
            None => return false
        };
        self.is_filled(x)
    }

    fn apply_effect(mut self, effect: Effect) -> Self {
        match effect {
            Effect::NoEffect => {}
            Effect::SetStatExact { stat_id, new_value } => {
                self.stats[stat_id].value = new_value
            }
            Effect::SetStatHigher { stat_id, to_add } => {
                self.stats[stat_id].value += to_add
            }
            Effect::SetStatLower { stat_id, to_subtract } => {
                self.stats[stat_id].value -= to_subtract
            }
        }
        self
    }
}

pub enum Direction {
    Up,
    Down,
}

impl Stage {
    pub fn dummy() -> Self {
        Stage {
            index: 0,
            name: String::new(),
            text: Vec::new(),
            options: Vec::new(),
            current_option: 0,
        }
    }

    pub fn change_option(self, dir: Direction, game: &GameState) -> Self {
        let old = self.current_option;
        if game.visible_options(&self).count() == 0 {
            dprintln!("Cannot change option due to lack of options.");
            return self;
        }
        let mut stage = self;
        match dir {
            Direction::Up => {
                loop {
                    stage.current_option -= 1;
                    if stage.current_option == 0 {
                        stage.current_option = stage.options.len();
                    }
                    if game.is_index_visible(&stage, stage.current_option - 1) {
                        break;
                    }
                }
            }
            Direction::Down => {
                loop {
                    stage.current_option += 1;
                    if stage.current_option == stage.options.len() + 1 {
                        stage.current_option = 1;
                    }
                    if game.is_index_visible(&stage, stage.current_option - 1) {
                        break;
                    }
                }
            }
        }
        dprintln!("Moving arrow from {} to {}", old, stage.current_option);
        return stage;
    }

    fn has_option(&self, option_nr: usize) -> bool {
        !self.options.is_empty() && 0 < option_nr && option_nr <= self.options.len()
    }

    fn get_current_option(&self) -> Option<&StageOption> {
        if !self.has_option(self.current_option) {
            None
        } else {
            Some(&self.options[self.current_option - 1])
        }
    }

    #[allow(unused)]
    fn get_option(&self, index: usize) -> Option<&StageOption> {
        if !self.has_option(index) {
            None
        } else {
            Some(&self.options[index - 1])
        }
    }
}

impl Conditional for StageOption {
    fn get_condition(&self) -> &Condition {
        &self.condition
    }
}

impl Condition { fn always() -> Condition { Condition::Always } }

impl Effect { fn no_effect() -> Effect { Effect::NoEffect } }

impl Copy for Condition {}
impl Clone for Condition {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Effect {}
impl Clone for Effect {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for StageOption {
    fn clone(&self) -> Self {
        StageOption {
            target_stage: self.target_stage,
            text: Vec::new(), // for now cloning is only for temporary info
            condition: self.condition,
            effect: self.effect,
        }
    }
}