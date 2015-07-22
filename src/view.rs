use std::sync::mpsc;
use ncurses::*;
use view_data::ViewData;
use message::Message;
use channel::Channel;

const LINE_BREAK: i32 = '\n' as i32;
const BACKSPACE: i32 = 127; // ncurses KEY_BACKSPACE not working?

pub struct View {
    messages: WINDOW,
    input: WINDOW,
    unread: WINDOW,
    view_data: Option<ViewData>
}

impl View {
    pub fn new() -> View {
        initscr();

        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        let messages = init_messages(max_y, max_x);
        let input = init_input(max_y, max_x);
        let unread = init_unread(max_y, max_y);

        View {
            messages: messages,
            input: input,
            unread: unread,
            view_data: None
        }
    }

    // Rather have ncurses call a callback
    // instead of a receiver to cut the dependency
    // but couldn't get it to work
    pub fn init(&mut self, on_input: Box<Fn(String, String)>, view_data_rx: mpsc::Receiver<ViewData>) {
        self.draw_prompt();
        let mut buffer = String::new();
        loop {
            match view_data_rx.try_recv() {
                Ok(view_data) => {
                    self.view_data = Some(view_data);
                    self.naive_redraw();
                    self.draw_prompt();
                }
                _ => ()
            }

            match wgetch(self.input) {
                ERR => continue,

                LINE_BREAK  => {
                    on_input(buffer, self.current_channel().id.clone());
                    buffer = String::new();
                    wclear(self.input);
                    self.draw_prompt();
                },

                BACKSPACE => {
                    if buffer.len() > 0 {
                        buffer.pop();
                        // Just redraw the whole thing to avoid crap
                    }
                    // for some reason backspaces get rendered regardles
                    // of what this code does. So clear screen.
                    wclear(self.input);
                    self.draw_prompt();
                    mvwprintw(self.input, 1, (self.current_prompt().len() + 1) as i32, &buffer);
                    wrefresh(self.input);
                },

                ch @ _ => {
                    // TODO why unsafe?
                    unsafe { buffer.as_mut_vec().push(ch as u8); }
                    // Just redraw the whole thing to avoid crap
                    wclear(self.input);
                    self.draw_prompt();
                    mvwprintw(self.input, 1, (self.current_prompt().len() + 1) as i32, &buffer);
                    wrefresh(self.input);
                }
            }
        }
    }

    pub fn print_message(&self, message: &Message) {
        let string = format!("{}", message) + "\n";
        wprintw(self.messages, &string);
    }

    fn current_channel(&self) -> &Channel {
       &self.view_data
           .as_ref()
           .expect("Trying to get current channel without viewdata")
           .channel
    }

    fn draw_prompt(&self) {
        let top = 0 as chtype;
        let bottom = ' ' as chtype;
        let empty = ' ' as chtype;
        wborder(self.input, empty,empty,top,bottom,empty,empty,empty,empty);
        let promptname: &str = &self.current_prompt();
        mvwprintw(self.input, 1, 1, promptname);
        wmove(self.input, 1, (promptname.len() + 1) as i32); // Move physical cursor to prompt start
        wrefresh(self.input);
    }

    fn draw_unread(&self) {
        wclear(self.unread);
        let channels = &self.view_data.as_ref().unwrap().unread_channels;
        attron(A_BOLD());
        wmove(self.unread, 0, 1);
        for channel in channels {
            wprintw(self.unread, &format!("#{}", channel.name));
            wmove(self.unread, 0, (channel.name.len() + 2) as i32);
        }
        attroff(A_BOLD());
        wrefresh(self.unread);
    }

    fn current_prompt(&self) -> String {
        if self.view_data.is_none() { return "> ".to_string() }

        let name = &self.current_channel().name;
        format!("#{} > ", name)
    }

    // Naively redraws the whole UI by clearing all data
    fn naive_redraw(&self) {
        if self.view_data.is_none() { return };

        self.draw_unread();

        wclear(self.messages);

        let view_data = self.view_data.as_ref().unwrap();
        for message in view_data.messages.iter() {
            self.print_message(message)
        }

        wrefresh(self.messages);
    }

}

impl Drop for View {
    fn drop(&mut self) {
        endwin();
    }
}

fn init_messages(max_y: i32, max_x: i32) -> WINDOW {
    let win = newwin(max_y - 3, max_x, 0, 0);
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
    // nodelay(win, true);
    // box_(win, 0, 0);
    halfdelay(1); // Dirty fix for cpu cycles, needs better fix
    // cbreak()
    wrefresh(win);
    win
}

fn init_unread(max_y: i32, max_x: i32) -> WINDOW {
    let win = newwin(1, max_x, max_y - 3, 0);
    wrefresh(win);
    win
}


