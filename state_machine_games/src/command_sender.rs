use std::sync::mpsc;

pub trait CommandSender<C>: Clone + 'static + Send + Sync {
    fn send_command(&self, c: C);
}

impl CommandSender<()> for () {
    fn send_command(&self, _c: ()) {}
}

impl<C: Send + 'static> CommandSender<C> for mpsc::Sender<C> {
    fn send_command(&self, c: C) {
        self.send(c).expect("Could not send command")
    }
}
