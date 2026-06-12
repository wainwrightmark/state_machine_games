use leptos::prelude::Read;
use leptos::prelude::Memo;
use leptos::prelude::guards::Plain;
use leptos::prelude::guards::ReadGuard;

use crate::prelude::GetValueLens;

pub fn watch_value<L: GetValueLens>(
    signal: impl Read<Value = ReadGuard<L::Object, Plain<L::Object>>> + Send + Sync + 'static,
) -> Memo<L::Value>
where
    L::Object: Send + Sync + 'static,
    L::Value: PartialEq + Send + Sync + 'static,
{
    Memo::new(move |_| {
        let guard = signal.read();
        L::get_value(&guard)
    })
}
