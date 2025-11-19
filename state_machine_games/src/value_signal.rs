pub trait ValueSignal: Clone {
    type Value: Sized + 'static;

    fn new(value: Self::Value) -> Self;

    /// Updates the value of the signal, but only notifies subscribers if the function
    /// returns `true`.
    #[track_caller]
    fn maybe_update(&self, fun: impl FnOnce(&mut Self::Value) -> bool) {
        self.try_maybe_update(|val| {
            let did_update = fun(val);
            (did_update, ())
        });
    }

    /// Updates the value of the signal and notifies subscribers, returning the value that is
    /// returned by the update function, or `None` if the signal has already been disposed.
    #[track_caller]
    fn try_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> U) -> Option<U> {
        self.try_maybe_update(|val| (true, fun(val)))
    }

    /// Updates the value of the signal, notifying subscribers if the update function returns
    /// `(true, _)`, and returns the value returned by the update function,
    /// or `None` if the signal has already been disposed.
    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U>;
}