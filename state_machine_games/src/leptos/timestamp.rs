use std::time::Duration;

use web_sys::js_sys;

pub struct Timestamp {
    pub now: f64,
}

impl Timestamp {
    pub fn now() -> Self {
        
        // let Some(window) = web_sys::window() else {
        //     return Timestamp {
        //         now: 0.0,
        //     };
        // };

        // let Some(performance) = window.performance() else {
        //     return Timestamp {
        //         now: 0.0,
        //     };
        // };
        //todo switch to performance now
        let now = js_sys::Date::now();// performance.now();
        Timestamp {
            now,
        }
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let ms_since_origin = self.now + rhs.as_millis() as f64;

        Self { now: ms_since_origin }
    }
}
