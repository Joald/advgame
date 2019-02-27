use std::fmt;
use std::sync::Mutex;

use ncurses::*;

use crate::debug::DEBUG;
use crate::game_components::Stage;
use crate::game_state::GameState;
use crate::misc;

pub struct Console {
    row_count: i32,
    col_count: i32,
}
lazy_static! {
    pub static ref DEBUG_LOG: Mutex<String> = Mutex::new(String::new());
}

impl Console {
    pub fn new() -> Console {
        initscr();
        let mut x = 0;
        let mut y = 0;
        getmaxyx(stdscr(), &mut y, &mut x);
        raw();
        noecho();
        keypad(stdscr(), true);
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        return Console {
            row_count: y,
            col_count: x,
        };
    }

    pub fn print_center_offset(&self, msg: &str, offset: i32) -> &Console {
        mvprintw(self.row_count / 2 + offset, (self.col_count - msg.len() as i32) / 2, &msg);
        refresh();
        &self
    }

    pub fn print_center(&self, msg: &str) -> &Console { self.print_center_offset(msg, 0) }

    pub fn print_top_offset(&self, msg: &str, offset: i32) {
        mvprintw(offset, (self.col_count - msg.len() as i32) / 2, &msg);
        refresh();
    }

    #[allow(unused)]
    pub fn print_top(&self, msg: &str) { self.print_top_offset(msg, 0); }

    pub fn left_align(&self, width: i32) -> i32 {
        (self.col_count - width) / 2
    }

    pub fn print_stage(&self, stage: &Stage, game: &GameState) {
        // Stage name is in bold.
        dprintln!("Printing stage {:?}", stage);
        attr_on(A_BOLD());
        self.print_top_offset(&stage.name, 1);
        attr_off(A_BOLD());

        let mut current_line_nr = 4;
        let max_width = misc::max_str_len(&stage.text);
        for line in stage.text.iter() {
            let line = game.parse_format_text(line);
            mvprintw(current_line_nr, self.left_align(max_width as i32), &line);
            current_line_nr += 1;
        }
        current_line_nr += 2;


        // To print options evenly, we need the maximum line width.
        // As we can have lots of options, we need to figure out how many digits
        // the option number will have. Then, to center the text we pad the max line width
        // on both sides with the number length, two more characters for the dot and space
        // after the number and four for the selection arrow.
        const ARROW: &str = "--> ";
        const DOT_AND_SPACE: usize = 2;
        let max_number_width = game.visible_options(stage).count().to_string().len();
        let cont_offset = max_number_width + DOT_AND_SPACE + ARROW.len();
        let max_width = game.visible_options(stage)
            .map(|option| { misc::max_str_len(&option.text) })
            .max()
            .unwrap_or(0)
            + 2 * cont_offset;

        let mut display_index = 0;
        for (internal_index, option) in stage.options.iter().enumerate() {
            if !game.is_filled(option) {
                continue;
            }
            display_index += 1;
            let arrow = if internal_index == stage.current_option - 1 {
                ARROW.to_string()
            } else { " ".repeat(ARROW.len()) };
            let row_text = if option.text.len() == 0 { "" } else { &option.text[0] };
            let row = game.parse_format_text(&format!("{}{}. {}", arrow, display_index, row_text));

            dprintln!("Printing option {}, first row text is {}", display_index, row);
            mvprintw(current_line_nr, self.left_align(max_width as i32), &row);
            for line in option.text[1..].iter() {
                let line = " ".repeat(cont_offset) + line;
                current_line_nr += 1;
                mvprintw(current_line_nr, self.left_align(max_width as i32), &line);
            }
            current_line_nr += 2;
        }
    }

    pub fn get_ch(&self) -> Option<i32> {
        Some(getch())
        // maybe some stuff needs to be handled in the future?
    }

    fn interpret_ch(&self, ch: i32) -> Action {
        if 0 <= ch && ch < 256 {
            let ch = std::char::from_u32(ch as u32).expect("Apparently [0;256) is not the correct range for chars.");
            dprintln!("Detected character '{}' pressed!", ch);
            match ch {
                'q' | 'Q' => Action::Quit,
                '\n' | 'y' | 'Y' => Action::Confirm,
                '0'...'9' => Action::Number(((ch as u8) - ('0' as u8)) as usize),
                'W' | 'w' => Action::Up,
                'S' | 's' => Action::Down,
                'N' | 'n' => Action::Cancel,
                _ => Action::Unimplemented
            }
        } else {
            match ch {
                KEY_ENTER => Action::Confirm,
                KEY_BACKSPACE => Action::Cancel,
                KEY_UP => Action::Up,
                KEY_DOWN => Action::Down,
                KEY_END if DEBUG => Action::Debug,
                _ => Action::Unimplemented
            }
        }
    }

    pub fn get_action(&self) -> Action {
        let c = match self.get_ch() {
            Some(c) => c,
            None => return Action::Quit
        };
        self.interpret_ch(c)
    }

    pub fn clear(&self) {
        clear();
        let msg = "| y/enter to confirm, n to decline, q to exit |";
        mvprintw(self.row_count - 1, (self.col_count - msg.len() as i32) / 2, &msg);
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        endwin();
        if DEBUG {
            eprintln!("{}", DEBUG_LOG.lock().unwrap());
            DEBUG_LOG.lock().unwrap().clear()
        }
    }
}

pub enum Action {
    Confirm,
    Quit,
    Cancel,
    Unimplemented,
    Up,
    Down,
    Number(usize),
    Debug,
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Action::Number(num) = self {
            write!(f, "Action::Number({})", num)
        } else {
            write!(f, "Action::{}", match self {
                Action::Confirm => "Confirm",
                Action::Quit => "Quit",
                Action::Cancel => "Cancel",
                Action::Unimplemented => "Unimplemented",
                Action::Up => "Up",
                Action::Down => "Down",
                Action::Debug => "Debug",
                _ => "This will never be printed."
            })
        }
    }
}