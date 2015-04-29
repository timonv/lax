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
    subscribers: HashMap<&'static str, Vec<mpsc::Sender<String>>>,
    broadcasters: Vec<mpsc::Receiver<String>>
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher { subscribers: HashMap::new(), broadcasters: vec![] }
    }

    pub fn register_broadcaster(&mut self, broadcaster: &Broadcast) {
       self.broadcasters.push(broadcaster.broadcast());
    }

    pub fn register_subscriber(&mut self, subscriber: &Subscribe) {
       let sender = subscriber.subscribe();
       let type_key = type_to_str(subscriber.what_subscribe());
       let new = match self.subscribers.get_mut(type_key) {
          Some(others) => {
             others.push(sender);
             None
          },
          None => {
             Some(vec![sender])
          }
       };
       // Improve me. Can chain because double mut borrow not allowed
       new.and_then(|new_senders| self.subscribers.insert(type_key, new_senders));
    }

    fn num_broadcasters(&self) -> usize {
       self.broadcasters.len()
    }

    fn num_subscribers(&self, dispatch_type: DispatchType) -> usize {
       match self.subscribers.get(type_to_str(dispatch_type)) {
          Some(subscribers) => subscribers.len(),
          None => 0
       }
    }
}

// Convert to hashable for dispatchtype?
fn type_to_str(dispatch_type: DispatchType) -> &'static str {
   match dispatch_type {
       OutgoingMessage => "OutgoingMessage",
       ChangeCurrentChannel => "ChangeCurrentChannel",
       IncomingMessage => "IncomingMessage"
   }
}

trait Broadcast {
   fn broadcast(&self) -> mpsc::Receiver<String>;
}

trait Subscribe {
   fn subscribe(&self) -> mpsc::Sender<String>;
   fn what_subscribe(&self) -> DispatchType;
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use super::{ Dispatcher, Broadcast, Subscribe};
    use super::DispatchType::{self, OutgoingMessage};

    #[test]
    fn test_register_broadcaster() {
        let mut dispatcher = Dispatcher::new();
        let brd = TestBroadcaster;
        assert_eq!(dispatcher.num_broadcasters(), 0);
        dispatcher.register_broadcaster(&brd);
        assert_eq!(dispatcher.num_broadcasters(), 1);
    }

    #[test]
    fn test_register_subscriber() {
        let mut dispatcher = Dispatcher::new();
        let brd = TestSubscriber;
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 0);
        dispatcher.register_subscriber(&brd);
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 1);
    }

    struct TestBroadcaster;

    impl Broadcast for TestBroadcaster {
      fn broadcast(&self) -> mpsc::Receiver<String> {
         let (_, rx) = mpsc::channel::<String>();
         rx
      }
    }

    struct TestSubscriber;

    impl Subscribe for TestSubscriber {
       fn subscribe(&self) -> mpsc::Sender<String> {
          let(tx, _) = mpsc::channel::<String>();
          tx
       }

       fn what_subscribe(&self) -> DispatchType {
          OutgoingMessage
       }
    }
}
