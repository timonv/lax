use ncurses::*;

pub struct View {
    max_x: i32,
    max_y: i32,
    messages: WINDOW,
    input: WINDOW
}

impl View {
    pub fn new() -> View {
        initscr();

        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        let messages = init_messages(max_y, max_x);
        let input = init_input(max_y, max_x);

        View {
            max_x: max_x,
            max_y: max_y,
            messages: messages,
            input: input
        }
    }

    pub fn init(&mut self) {
        self.draw_prompt();
        // Doesn't work
        // mvwin(self.messages,1,1); // Padding for messages
    }

    pub fn print_message(&self, string: &str) {
        let mut string = string.to_string();
        string = string + "\n";
        wprintw(self.messages, &string);
        wrefresh(self.messages);
    }

    fn prompt_input(&self) -> String {
        let mut string = String::new();
        wgetstr(self.input, &mut string);
        string
    }

    fn draw_prompt(&self) {
        wclear(self.input);
        let top = 0 as chtype;
        let bottom = ' ' as chtype;
        let empty = ' ' as chtype;
        wborder(self.input, empty,empty,top,bottom,empty,empty,empty,empty);
        mvwprintw(self.input, 1, 1, ">> ");
        wrefresh(self.input);
    }
}

impl Drop for View {
    fn drop(&mut self) {
        endwin();
    }
}

fn init_messages(max_y: i32, max_x: i32) -> WINDOW {
    let win = newwin(max_y - 2, max_x, 0, 0);
    let top = ' ' as chtype;
    let bottom = ' ' as chtype;
    let empty = ' ' as chtype;
    wborder(win, empty,empty,top,bottom,empty,empty,empty,empty);
    // box_(win, 0, 0);
    scrollok(win, true);
    wrefresh(win);
    win
}

fn init_input(max_y: i32, max_x: i32) -> WINDOW {
    let win = newwin(2, max_x, max_y - 2, 0);
    // box_(win, 0, 0);
    wrefresh(win);
    win
}
