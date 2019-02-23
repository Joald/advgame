use crate::game_state::GameState;

type StatValue = i32;

pub trait Conditional { fn get_condition(&self) -> &Condition; }

#[derive(Serialize, Deserialize, Debug)]
pub struct Statistic {
    pub id: usize,
    pub name: String,
    #[serde(rename = "default_value")]
    pub value: StatValue,
}

pub type ItemSlot = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ItemEffect {
    NoEffect,
    Consumable { on_consume: Effect },
    Equippable { slot: ItemSlot, when_equipped: Effect },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub effect: ItemEffect,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Always,
    IfStatHigher { stat_id: usize, higher_than: StatValue },
    IfStatLower { stat_id: usize, lower_than: StatValue },
    IfStatExact { stat_id: usize, value: StatValue },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    NoEffect,
    SetStatHigher { stat_id: usize, to_add: StatValue },
    SetStatLower { stat_id: usize, to_subtract: StatValue },
    SetStatExact { stat_id: usize, new_value: StatValue },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StageOption {
    pub target_stage: usize,
    pub text: Vec<String>,
    #[serde(default = "Condition::always")]
    pub condition: Condition,
    #[serde(default = "Effect::no_effect")]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stage {
    pub index: usize,
    pub name: String,
    pub text: Vec<String>,
    pub options: Vec<StageOption>,
    #[serde(skip)]
    pub current_option: usize,
}


pub enum Direction {
    Up,
    Down,
}

impl Stage {
    #[allow(unused)]
    pub fn new() -> Self {
        Stage {
            index: 0,
            name: String::new(),
            text: Vec::new(),
            options: Vec::new(),
            current_option: 0,
        }
    }

    pub fn change_option(&mut self, dir: Direction, game: &GameState) {
        let old = self.current_option;
        if game.visible_options(self).count() == 0 {
            dprintln!("Cannot change option due to lack of options.");
            return;
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
        return;
    }

    pub fn has_option(&self, option_nr: usize) -> bool {
        !self.options.is_empty() && 0 < option_nr && option_nr <= self.options.len()
    }

    pub fn get_current_option(&self) -> Option<&StageOption> {
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


impl Clone for Stage {
    fn clone(&self) -> Self {
        Stage {
            index: self.index,
            name: self.name.clone(),
            text: self.text.clone(),
            options: self.options.clone(),
            current_option: self.current_option,
        }
    }
}

impl Conditional for StageOption {
    fn get_condition(&self) -> &Condition {
        &self.condition
    }
}

impl Condition { fn always() -> Condition { Condition::Always } }

impl Effect {
    fn no_effect() -> Effect { Effect::NoEffect }
    pub fn change_stat_id(&self, new_id: usize) -> Option<Self> {
        let mut x = self.clone();
        match x {
            Effect::NoEffect => None,
            Effect::SetStatLower { ref mut stat_id, to_subtract: _ } |
            Effect::SetStatHigher { ref mut stat_id, to_add: _ } |
            Effect::SetStatExact { ref mut stat_id, new_value: _ } => {
                *stat_id = new_id;
                Some(x)
            }
        }
    }
    pub fn map_state_id<F: FnOnce(usize) -> Result<usize, String>>(&self, mapping: F) -> Result<Self, String> {
        match self {
            Effect::NoEffect => Ok(self.clone()),
            Effect::SetStatLower { stat_id, to_subtract: _ } |
            Effect::SetStatHigher { stat_id, to_add: _ } |
            Effect::SetStatExact { stat_id, new_value: _ } => {
                let new_id = mapping(*stat_id)?;
                self.change_stat_id(new_id).ok_or(format!("Invalid stat id {} in effect!", stat_id))
            }
        }
    }
}

impl ItemEffect {
    pub fn map_state_id<F: FnOnce(usize) -> Result<usize, String>>(&self, mapping: F) -> Result<Self, String> {
        let mut copy = self.clone();
        match copy {
            ItemEffect::NoEffect => Ok(copy),
            ItemEffect::Consumable { ref mut on_consume } => {
                *on_consume = on_consume.map_state_id(mapping)?;
                Ok(copy)
            }
            ItemEffect::Equippable { slot: _, ref mut when_equipped } => {
                *when_equipped = when_equipped.map_state_id(mapping)?;
                Ok(copy)
            }
        }
    }
}


impl Copy for Condition {}
impl Clone for Condition {
    fn clone(&self) -> Self {
        *self
    }
}


impl Clone for ItemEffect {
    fn clone(&self) -> Self {
        match self {
            ItemEffect::Equippable { slot, when_equipped } =>
                ItemEffect::Equippable { slot: slot.clone(), when_equipped: *when_equipped },
            ItemEffect::Consumable { on_consume } =>
                ItemEffect::Consumable { on_consume: *on_consume },
            ItemEffect::NoEffect => ItemEffect::NoEffect
        }
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
            text: self.text.clone(),
            condition: self.condition,
            effect: self.effect,
        }
    }
}