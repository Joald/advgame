use std::collections::HashMap;
use std::fmt;
use std::iter::FromIterator;
use std::iter::Iterator;

use crate::console::Action;
use crate::game_components::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub name: String,
    pub stats: Vec<Statistic>,
    pub stages: Vec<Stage>,
    pub item_slots: Vec<ItemSlot>,
    pub items: Vec<Item>,
    #[serde(rename = "entry_stage")]
    pub current_stage: usize,
    pub exit_stage: usize,
    #[serde(skip)]
    finished: bool,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Name: \"{}\"\nTODO rest", self.name)
    }
}

type ParseResult = Result<GameState, String>;

impl GameState {
    pub fn get_name(&self) -> &str { &self.name }

    pub fn check_dead_ends(self) -> ParseResult {
        let it =
            self.stages.iter().filter(|stage| {
                stage.options.len() == 0
            });
        let mut found_last = false;
        for stage in it {
            if stage.index != self.exit_stage {
                return Err(format!("Stage nr. {} has no options and isn't the final stage!", stage.index));
            }
            found_last = true;
        }
        if !found_last {
            Err("Exit stage can't contain options!".to_string())
        } else {
            Ok(self)
        }
    }

    pub fn map_stat_ids(mut self) -> ParseResult {
        dprintln!("map_stat_ids():    {:?}", self);
        let mapper: HashMap<usize, usize> = HashMap::from_iter(
            self.stats.iter().enumerate().map(|(i, stat)| {
                dprintln!("Stat {} becomes stat {}!", stat.id, i);
                (stat.id, i)
            })
        );
        self.stages.iter_mut().fold(Ok(0), |res, stage| {
            let stage_index = stage.index;
            dprintln!("Stage {}:", stage_index);
            dprintln!("Before:    {:?}", stage);
            let stage_name = stage.name.clone();
            let mapping = |x| mapper.get(&x).and_then(|x| Some(*x)).ok_or(format!(
                "Entry \"{}\" in stage {}, \"{}\" points to an inexistent stat.", x, stage_index, stage_name
            ));
            let rv = res.and_then(|state| stage.options.iter_mut().fold(Ok(state), |res, option| {
                match option.condition {
                    Condition::Always => {}
                    Condition::IfStatExact { ref mut stat_id, value: _ } |
                    Condition::IfStatHigher { ref mut stat_id, higher_than: _ } |
                    Condition::IfStatLower { ref mut stat_id, lower_than: _ } =>
                        *stat_id = mapping(*stat_id)?,
                };
                option.effect = option.effect.map_state_id(mapping)?;
                res
            }));
            dprintln!("After:     {:?}", stage);
            rv
        }).and(self.items.iter_mut().fold(Ok(0), |res, item| {
            item.effect = item.effect.map_state_id(
                |x| mapper.get(&x).and_then(|x| Some(*x)).ok_or(format!("Invalid state id in item {}.", item.name))
            )?;
            res
        })).and(Ok(self))
    }

    pub fn map_stage_ids(mut self) -> ParseResult {
        dprintln!("map_stage_ids():    {:?}", self);
        let mapper: HashMap<usize, usize> = HashMap::from_iter(
            self.stages.iter().enumerate().map(|(i, stage)| {
                dprintln!("Stage {} becomes stage {}!", stage.index, i);
                (stage.index, i)
            })
        );

        self.stages.iter_mut().fold(Ok(0), |res, stage| {
            let stage_index = stage.index;
            let stage_name = stage.name.clone();

            let mapping = |x: usize| {
                mapper.get(&x).ok_or(format!(
                    "Entry \"{}\" in stage {}, \"{}\" points to an inexistent stage.",
                    x, stage_index, stage_name
                )).and_then(|x| Ok(*x))
            };

            stage.options.iter_mut().fold(res, |res, option| {
                if option.text.is_empty() {
                    return Err(format!("No text provided for option in stage {}", stage_index));
                }
                option.target_stage = mapping(option.target_stage)?;
                res
            }).and(mapper.get(&stage_index).ok_or(
                "Post processing of data failed. It's a bug on our side. Sorry!".to_string()
            )).and_then(|mapped_index| {
                stage.index = *mapped_index;
                Ok(1)
            })
        }).and(
            mapper.get(&self.current_stage).ok_or("Entry stage is invalid!".to_string())
        ).and_then(|first_stage| {
            self.current_stage = *first_stage;
            mapper.get(&self.exit_stage).ok_or("Exit stage is invalid!".to_string())
        }).and_then(|stage| {
            self.exit_stage = *stage;
            Ok(self)
        })
    }

    pub fn map_item_ids(mut self) -> ParseResult {
        dprintln!("map_item_ids():    {:?}", self);
        let item_id_mapper: HashMap<usize, usize> = HashMap::from_iter(
            self.items.iter().enumerate().map(|(i, item)| {
                dprintln!("Item {} becomes item {}!", item.id, i);
                (item.id, i)
            })
        );
        let effect_mapper = |effect: &mut Effect| match effect {
            Effect::NoEffect | Effect::SetStatHigher { .. } |
            Effect::SetStatLower { .. } | Effect::SetStatExact { .. } => Ok(1),
            Effect::UseItem { ref mut item_id } =>
                item_id_mapper.get(item_id)
                    .ok_or(format!("Invalid item {} in a use_item effect.", item_id))
                    .and_then(|mapped_item_id| {
                        *item_id = *mapped_item_id;
                        Ok(1)
                    }),
        };
        self.items.iter_mut().fold(Ok(1), |res, item|
            res.and(item_id_mapper.get(&item.id)
                .ok_or(format!("Invalid item {}", item.id)))
                .and_then(|item_id| {
                    item.id = *item_id;
                    match item.effect {
                        ItemEffect::NoEffect => Ok(1),
                        ItemEffect::Consumable { ref mut on_consume } =>
                            effect_mapper(on_consume),
                        ItemEffect::Equippable { slot: _, ref mut when_equipped } =>
                            effect_mapper(when_equipped),
                    }
                }),
        ).and(Ok(self))
    }

    pub fn post_process(mut self) -> ParseResult {
        // Check if all stages except the last one have at least one option.
        self = self.check_dead_ends()?;

        // Map all IDs to array indices.
        self = self.map_stat_ids()?.map_stage_ids()?.map_item_ids()?;

        dprintln!("After map:    {:?}", self);
        // Make sure we start in the correct stage
        self.enter_current_stage();
        dprintln!("Finish of post process:    {:?}", self);
        Ok(self)
    }

    fn enter_current_stage(&mut self) {
        let mut stage = self.get_current_stage().clone();
        stage.current_option = 0;
        stage.change_option(Direction::Down, self);
        self.replace_current_stage(stage);
        dprintln!("Entered stage {}!", self.get_current_stage().index);
    }

    pub fn change_to_stage_index(&mut self, stage: usize) {
        dprintln!("Changing stage from {} to {}", self.current_stage, stage);
        if self.current_stage != stage {
            //GameState::get_current_stage_mut(state).leave();
            self.current_stage = stage;
            self.enter_current_stage();
        }
        dprintln!("Current stage is now {}: {:?}", self.current_stage, self.get_current_stage());
    }

    pub fn get_current_stage(&self) -> &Stage {
        &self.stages[self.current_stage]
    }

    #[allow(unused)]
    pub fn get_current_stage_mut(&mut self) -> &mut Stage {
        &mut self.stages[self.current_stage]
    }

    #[allow(unused)]
    pub fn replace_current_stage(&mut self, stage: Stage) {
        self.stages[self.current_stage] = stage
    }

    #[allow(unused)]
    pub fn replace_current_stage_with<F: FnOnce(Stage) -> Stage>(&mut self, f: F) {
        self.replace_current_stage(f(self.get_current_stage().clone()));
    }

    pub fn handle_action(mut self, action: &Action) -> Self {
        let mut stage_change: Option<StageOption> = None;
        let mut finish = self.finished;
        let mut stage = self.get_current_stage().clone();
        dprintln!("Handling {}...", action);
        match action {
            Action::Up => {
                stage.change_option(Direction::Up, &self);
                self.replace_current_stage(stage)
            }
            Action::Down => {
                stage.change_option(Direction::Down, &self);
                self.replace_current_stage(stage)
            }
            Action::Confirm => {
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
            Action::Number(num) => {
                if stage.has_option(*num) {
                    let stage = self.get_current_stage().clone();
                    let it = self.visible_options(&stage).nth(*num - 1);
                    it.and_then(|num| Some(num.target_stage)).and_then(|ind| {
                        Some(self.change_to_stage_index(ind))
                    });

                    dprintln!("Stage may be changed due to {} being pressed.", *num)
                }
            }
            _ => {} //yes rust, these are all the options I want
        };
        if stage_change.is_some() {
            let op = stage_change.unwrap();
            self.apply_effect(op.effect.clone());
            self.change_to_stage_index(op.target_stage);
        }
        self.finished = finish;
        self
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn visible_options<'a>(&'a self, stage: &'a Stage) -> impl std::iter::Iterator<Item=&'a StageOption> {
        stage.options.iter().filter(move |option| {
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

    fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::NoEffect | Effect::UseItem { item_id: _ } => {}
            Effect::SetStatExact { stat_id, new_value } =>
                self.stats[stat_id].value = new_value,
            Effect::SetStatHigher { stat_id, to_add } =>
                self.stats[stat_id].value += to_add,
            Effect::SetStatLower { stat_id, to_subtract } =>
                self.stats[stat_id].value -= to_subtract
        }
    }
}
