use crate::game_state::GameState;
use crate::console::*;

pub fn main_loop(cls: &Console, mut state: GameState) {
    dprintln!("Starting main loop!");
    let mut action = Action::Unimplemented;
    while match action { Action::Quit => false, _ => true } {
        cls.clear();
        cls.print_stage(&state.get_current_stage().borrow(), &state);
        action = cls.get_action();
        state = state.handle_action(&action);
        if state.is_finished() {
            break;
        }
    }
    cls.clear();
    cls.print_center("Thank you for playing!");
    cls.print_center_offset("Press any key to exit the application.", 1);
    cls.get_ch();
}

pub fn play_game(state: GameState) {
    let cls = Console::new();
    dprintln!("Welcome to the advgame debug mode!");
    cls.print_center(&format!("You have successfully loaded \"{}\"!", state.get_name()));
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
