use ncurses::*;
use game_state::Stage;

pub struct Console {
    row_count: i32,
    col_count: i32,
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

    pub fn print_center_offset(&self, msg: &str, offset: i32) {
        mvprintw(self.row_count / 2 + offset, (self.col_count - msg.len() as i32) / 2, &msg);
        refresh();
    }

    pub fn print_center(&self, msg: &str) { self.print_center_offset(msg, 0) }

    pub fn print_top_offset(&self, msg: &str, offset: i32) {
        mvprintw(offset, (self.col_count - msg.len() as i32) / 2, &msg);
        refresh();
    }
    pub fn print_top(&self, msg: &str) { self.print_top_offset(msg, 0); }

    pub fn print_stage(&self, stage: &Stage){//}, game: &GameState) {
        attr_on(A_BOLD());
        self.print_top(&stage.name);
        attr_off(A_BOLD());

        // TODO: text wrapping
        self.print_top_offset(&stage.content, 3);

        // TODO: option text wrapping
        let default = (0usize, "".to_string());
        let max_width = stage.neighbors.iter().max_by(
            |(_, text1), (_, text2)| { text1.len().cmp(&text2.len()) }
        ).unwrap_or(&default).to_owned().1.len() + 4; // to make room for 2 chars of selection
        for (i, (_, text)) in stage.neighbors.iter().enumerate() {
            let row = if i + 1 == stage.current_option { "- " } else { "  " }.to_string() +
                &(i + 1).to_string() + ". " + text;
            mvprintw(5 + 2 * i as i32, (self.col_count - max_width as i32) / 2, &row);
        }
    }
    pub fn get_ch(&self) -> Option<i32> {
        let ret = getch();
        if ret == 'q' as i32 {
            clear();
            self.print_center("Thank you for playing!");
            self.print_center_offset("Press any key to exit.", 1);
            getch();
            None
        } else {
            Some(ret)
        }
    }

    pub fn get_action(&self) -> Action {
        let ret = getch();
        if 0 <= ret && ret < 256 {
            let ret = std::char::from_u32(ret as u32).expect("Apparently [0;256) is not the correct range for chars.");
            match ret {
                'q' | 'Q' => Action::Quit,
                '\n' | 'y' | 'Y' => Action::Confirm,
                '0' ... '9' => Action::Number(((ret as u8) - ('0' as u8)) as usize),
                'W' | 'w' => Action::Up,
                'S' | 's' => Action::Down,
                _ => Action::Unimplemented
            }
        } else {
            match ret {
                KEY_ENTER => Action::Confirm,
                KEY_BACKSPACE => Action::Cancel,
                KEY_UP => Action::Up,
                KEY_DOWN => Action::Down,
                _ => Action::Unimplemented
            }
        }
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
    }
}

pub enum Action {
    Confirm,
    Quit,
    Cancel,
    Unimplemented,
    Up,
    Down,
    Number(usize)
}