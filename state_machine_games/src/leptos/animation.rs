use leptos::prelude::{Get, Memo, Signal};
use glam::{FloatExt, f32};


pub fn animate_value<V: Clone + Copy + PartialEq + Send + Sync + 'static>(
    start_time: f64,
    current_time: Signal<f64>,
    from: V,
    to: V,
    time_in_ms: f64,
    ease: impl EaseFunction<V>
)-> Memo<V>{
    if to == from{
        Memo::new(move |_|{to})
    }else{
        Memo::new(move|_|{
            let now = current_time.get();
            let ratio = ((now - start_time) / time_in_ms).clamp(0.0, 1.0);

            ease.ease(from, to, ratio)
        })
    }
}

pub trait EaseFunction<V>: Clone + Copy + Send + Sync + 'static{
        fn ease(self, from: V, to: V, ratio: f64)-> V;
}

#[derive(Debug, Clone, Copy)]
pub struct Lerp32;

impl EaseFunction<f32> for Lerp32{
    fn ease(self, from: f32, to: f32, ratio: f64)-> f32 {
        from.lerp(to, ratio as f32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Lerp64;

impl EaseFunction<f64> for Lerp64{
    fn ease(self, from: f64, to: f64, ratio: f64)-> f64 {
        from.lerp(to, ratio)
    }
}