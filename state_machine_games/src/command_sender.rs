use std::{collections::VecDeque, sync::{Arc, Mutex}};

use crate::prelude::*;

pub trait CommandSender<C: GameCommand>: Clone + 'static + Send + Sync {
    fn send_command(&self, c: C);
}
#[derive(Debug, Clone)]
pub struct BasicCommandSender<C: GameCommand> {
    pub queue: Arc<Mutex<VecDeque<C>>>, //todo use a better queue
}

impl<C: GameCommand> CommandSender<C> for BasicCommandSender<C> {
    fn send_command(&self, c: C) {
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.push_back(c);
            }
            Err(err) => panic!("{err}"),
        }
    }
}

impl<C: GameCommand> BasicCommandSender<C> {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}
