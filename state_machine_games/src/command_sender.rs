use std::sync::mpsc;

use crate::prelude::*;

pub trait CommandSender<C: AnyGameCommand>: Clone + 'static + Send + Sync {
    fn send_command(&self, c: C);
}

impl CommandSender<()> for () {
    fn send_command(&self, _c: ()) {}
}

impl<C: AnyGameCommand> CommandSender<C> for mpsc::Sender<C> {
    fn send_command(&self, c: C) {
        self.send(c).expect("Could not send command")
    }
}
