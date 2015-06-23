use std::thread;
use std::sync::{mpsc,Arc, Mutex};
use dispatcher::{self, DispatchType, Subscribe, SubscribeHandle, DispatchMessage, Broadcast, BroadcastHandle};
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

    // Rather have ncurses call a callback
    // instead of a receiver to cut the dependency
    // but couldn't get it to work
    pub fn init(&mut self, onInput: Box<Fn(String)>, print_rx: mpsc::Receiver<String>) {
        self.draw_prompt();
        let mut input = String::new();
        loop {
            match  print_rx.try_recv() {
                Ok(message) => self.print_message(message),
                _ => ()
            }

            let ch = wgetch(self.input);
            if ch == ERR { continue };
            if ch != '\n' as i32 && ch != '\r' as i32 {
                // TODO This might send wrong keys
                unsafe { input.as_mut_vec().push(ch as u8); }
                mvwprintw(self.input, 1, 3, &input);
                wrefresh(self.input);
            } else {
                onInput(input);
                input = String::new();
                self.draw_prompt();
            }
        }
        // Doesn't work
        // mvwin(self.messages,1,1); // Padding for messages
    }

    pub fn print_message(&self, mut string: String) {
        string = string + "\n";
        wprintw(self.messages, &string);
        wrefresh(self.messages);
    }

    // fn prompt_input(&self) -> String {
    //     wgetch(self.input)
    // }

    fn draw_prompt(&self) {
        wclear(self.input);
        let top = 0 as chtype;
        let bottom = ' ' as chtype;
        let empty = ' ' as chtype;
        wborder(self.input, empty,empty,top,bottom,empty,empty,empty,empty);
        mvwprintw(self.input, 1, 1, "> ");
        wmove(self.input, 1, 3); // Move physical cursor to prompt start
        wrefresh(self.input);
    }

}

impl Drop for View {
    fn drop(&mut self) {
        endwin();
    }
}

fn input_to_message(string: String) -> DispatchMessage {
    DispatchMessage {
        dispatch_type: DispatchType::UserInput,
        payload: string
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
    nodelay(win, true);
    // box_(win, 0, 0);
    wrefresh(win);
    win
}
