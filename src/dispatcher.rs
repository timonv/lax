use self::DispatchType::{ChangeCurrentChannel, OutgoingMessage, IncomingMessage};
use std::collections::HashMap;
use std::sync::mpsc;

enum DispatchType {
    ChangeCurrentChannel,
    OutgoingMessage,
    IncomingMessage
}

struct Dispatcher {
    // Internally store as string for faster access
    listeners: HashMap<&'static str, Vec<mpsc::Sender<String>>>,
    broadcasters: HashMap<&'static str, Vec<mpsc::Receiver<String>>>
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher { listeners: HashMap::new(), broadcasters: HashMap::new() }
    }

    pub fn register_broadcaster(&mut self, dispatch_type: DispatchType, broadcaster: mpsc::Receiver<String>) {
        let key = type_to_str(dispatch_type);
        let mut other = match self.broadcasters.get_mut(key) {
            Some(values) => *values,
            None => vec![]
        };
        other.push(broadcaster);
        self.broadcasters.insert(key, other);
    }

    fn num_broadcasters(&self, dispatch_type: DispatchType) -> usize {
        match self.broadcasters.get(type_to_str(dispatch_type)) {
            Some(vals) => vals.len(),
            None => 0
        }
    }
}

fn type_to_str(dispatch_type: DispatchType) -> &'static str {
   match dispatch_type {
       OutgoingMessage => "OutgoingMessage",
       ChangeCurrentChannel => "ChangeCurrentChannel",
       IncomingMessage => "IncomingMessage"
   }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use super::{ Dispatcher };
    use super::DispatchType::{OutgoingMessage};

    #[test]
    fn test_register_broadcaster() {
        // (lst_tx, lst_rx) = mpsc::channel();
        let (ev_tx, ev_rx) = mpsc::channel();

        let dispatcher = Dispatcher::new();
        assert_eq!(dispatcher.num_broadcasters(OutgoingMessage), 0);
        dispatcher.register_broadcaster(OutgoingMessage, ev_rx);
        assert_eq!(dispatcher.num_broadcasters(OutgoingMessage), 1);
    }
}
