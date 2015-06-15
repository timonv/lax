use self::DispatchType::{ChangeCurrentChannel, OutgoingMessage, RawIncomingMessage, UserInput};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// Aliases for easier refactoring

pub type SubscribeHandle = mpsc::Sender<DispatchMessage>;
pub type BroadcastHandle = mpsc::Receiver<DispatchMessage>;

#[derive(PartialEq, Debug, Clone)]
pub enum DispatchType {
    ChangeCurrentChannel,
    OutgoingMessage,
    RawIncomingMessage,
    UserInput
}

#[derive(Clone)]
pub struct DispatchMessage {
   pub dispatch_type: DispatchType,
   pub payload: String
}

pub struct Dispatcher {
    // I heard you like types
    subscribers: HashMap<&'static str, Vec<SubscribeHandle>>,
    broadcasters: Vec<Arc<Mutex<BroadcastHandle>>>
}

pub trait Broadcast {
   // fn broadcast(&self, dispatch_type: DispatchType, payload: String);
   fn broadcast_handle(&mut self) -> BroadcastHandle;
}

pub trait Subscribe {
   fn subscribe_handle(&self) -> SubscribeHandle;
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher { subscribers: HashMap::new(), broadcasters: vec![] }
    }

    pub fn register_broadcaster(&mut self, broadcaster: &mut Broadcast) {
       let handle = Arc::new(Mutex::new(broadcaster.broadcast_handle()));
       self.broadcasters.push(handle);
    }

    pub fn register_subscriber(&mut self, subscriber: &Subscribe, dispatch_type: DispatchType) {
       let sender = subscriber.subscribe_handle();
       let type_key = type_to_str(&dispatch_type);
       let new = match self.subscribers.get_mut(type_key) {
          Some(others) => {
             others.push(sender);
             None
          },
          None => {
             Some(vec![sender])
          }
       };
       // Improve me. Cant chain because double mut borrow not allowed
       new.and_then(|new_senders| self.subscribers.insert(type_key, new_senders));
    }

    pub fn start(&self) {
       // Assuming that broadcasters.clone() copies the vector, but increase ref count on els
       for broadcaster in self.broadcasters.clone() {
          let subscribers = self.subscribers.clone();
          thread::spawn(move || {
             loop {
                let message = broadcaster.lock().unwrap().recv().ok().expect("Couldn't receive message in broadcaster or channel hung up");
                match subscribers.get(type_to_str(&message.dispatch_type)) {
                  Some(ref subs) => { 
                      for sub in subs.iter() { sub.send(message.clone()).unwrap(); }
                  },
                  None => ()
                }

             }
          });
       }
    }

    fn num_broadcasters(&self) -> usize {
       self.broadcasters.len()
    }

    fn num_subscribers(&self, dispatch_type: DispatchType) -> usize {
       match self.subscribers.get(type_to_str(&dispatch_type)) {
          Some(subscribers) => subscribers.len(),
          None => 0
       }
    }
}

// Convert to hashable for dispatchtype?
fn type_to_str(dispatch_type: &DispatchType) -> &'static str {
   match *dispatch_type {
       OutgoingMessage => "OutgoingMessage",
       ChangeCurrentChannel => "ChangeCurrentChannel",
       RawIncomingMessage => "RawIncomingMessage",
       UserInput => "UserInput"
   }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use super::{ Dispatcher, Broadcast, Subscribe, DispatchMessage, SubscribeHandle, BroadcastHandle};
    use super::DispatchType::{self, OutgoingMessage, RawIncomingMessage};

    #[test]
    fn test_register_broadcaster() {
        let mut dispatcher = Dispatcher::new();
        let mut brd = TestBroadcaster::new();
        assert_eq!(dispatcher.num_broadcasters(), 0);
        dispatcher.register_broadcaster(&mut brd);
        assert_eq!(dispatcher.num_broadcasters(), 1);
    }

    #[test]
    fn test_register_subscriber() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 0);
        dispatcher.register_subscriber(&sub, OutgoingMessage);
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 1);
    }

    #[test]
    fn test_register_multiple_subscribers() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        let sub2 = TestSubscriber::new();

        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 0);
        dispatcher.register_subscriber(&sub, OutgoingMessage);
        dispatcher.register_subscriber(&sub2, OutgoingMessage);
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 2);
    }

    #[test]
    fn test_broadcast_simple_message() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        let mut brd = TestBroadcaster::new();
        dispatcher.register_broadcaster(&mut brd);
        dispatcher.register_subscriber(&sub, OutgoingMessage);

        dispatcher.start();

        brd.broadcast(OutgoingMessage, "Hello world!".to_string());
        let message = sub.receiver.recv().unwrap();
        assert_eq!(message.dispatch_type, OutgoingMessage);
        assert_eq!(message.payload, "Hello world!");
    }

    #[test]
    fn test_broadcast_multiple_to_one() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        let mut brd = TestBroadcaster::new();
        dispatcher.register_broadcaster(&mut brd);
        dispatcher.register_subscriber(&sub, OutgoingMessage);
        dispatcher.register_subscriber(&sub, RawIncomingMessage);

        dispatcher.start();

        brd.broadcast(OutgoingMessage, "Hello world!".to_string());
        let message = sub.receiver.recv().unwrap();
        assert_eq!(message.dispatch_type, OutgoingMessage);
        assert_eq!(message.payload, "Hello world!");
        brd.broadcast(RawIncomingMessage, "Hello world!".to_string());
        let message = sub.receiver.recv().unwrap();
        assert_eq!(message.dispatch_type, RawIncomingMessage);
        assert_eq!(message.payload, "Hello world!");
    }

    struct TestBroadcaster {
       sender: Option<SubscribeHandle>
    }

    impl TestBroadcaster {
       fn new() -> TestBroadcaster {
         TestBroadcaster { sender: None }
      }

      fn broadcast(&self, dispatch_type: DispatchType, payload: String) {
         let message = DispatchMessage { dispatch_type: dispatch_type, payload: payload };
         match self.sender {
            Some(ref s) => { s.send(message); },
            None => ()
         };
      }
    }

    impl Broadcast for TestBroadcaster {
      fn broadcast_handle(&mut self) -> BroadcastHandle {
         let (tx, rx) = mpsc::channel::<DispatchMessage>();
         self.sender = Some(tx);
         rx
      }

    }

    struct TestSubscriber {
      receiver: BroadcastHandle,
      sender: SubscribeHandle
    }

    impl TestSubscriber {
       fn new() -> TestSubscriber {
          let(tx, rx) = mpsc::channel::<DispatchMessage>();
          TestSubscriber { receiver: rx, sender: tx }
       }
    }

    impl Subscribe for TestSubscriber {
       fn subscribe_handle(&self) -> SubscribeHandle {
          self.sender.clone()
       }
    }
}
