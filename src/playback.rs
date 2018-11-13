use game_state::GameState;
use console::*;
use game_state::Stage;

pub fn main_loop(cls: &Console, state: &mut GameState) {
    let mut action = Action::Unimplemented;
    while match action { Action::Quit => false, _ => true } {
        cls.clear();
        let nr = state.current_stage.unwrap();
        let stage = &mut state.stages[nr];

        cls.print_stage(stage);
        action = cls.get_action();
        match action { // TODO refactor changing into methods.
            Action::Up => {
                stage.current_option -= 1;
                if stage.current_option == 0 {
                    stage.current_option = stage.neighbors.len();
                }
            }
            Action::Down => {
                stage.current_option += 1;
                if stage.current_option == stage.neighbors.len() + 1 {
                    stage.current_option = 1;
                }
            }
            Action::Confirm => {
                stage.current_option = 1;
                state.current_stage = Some(stage.neighbors[stage.current_option - 1].0);
            }
            Action::Number(num) =>
                if !stage.neighbors.is_empty() && 0 < num && num <= stage.neighbors.len() {
                    stage.current_option = 1;
                    state.current_stage = Some(stage.neighbors[num - 1].0);
                }
            _ => {}//shut
        }
    }
    cls.clear();
    cls.print_center("Thank you for playing!");
    cls.print_center_offset("Press any key to exit the application.", 1);
    cls.get_ch();
}

pub fn play_game(state: &mut GameState) {
    let cls = Console::new();

    cls.print_center(&format!("You have successfully parsed {}!", state.name));
    cls.print_center_offset("Play now? [y]/n", 1);

    let mut response_guard: Option<bool> = None;
    while response_guard.is_none() {
        match cls.get_action() {
            Action::Confirm => response_guard = Some(true),
            Action::Cancel => response_guard = Some(false),
            _ => {}
        }
    }
    if let Some(true) = response_guard {
        cls.print_center_offset("Press any key to start game!", 2);
        cls.get_ch();
        main_loop(&cls, state);
    } else {
        cls.print_center_offset("Too bad!", 2);
        cls.get_ch();
    }
}
