mod check;
mod update;

use chrono::{Duration, Timelike, Utc};
use crate::util::context::Context;
use std::sync::Arc;
use tokio::time::{Instant, self};

fn next_threshold(ms: i64) -> Instant {
    let instant = Instant::now();
    let now = Utc::now();
    let next_threshold_from_now = (Utc::now() + Duration::milliseconds(ms))
        .with_second(0)
        .unwrap();
    let difference = next_threshold_from_now - now;

    instant + difference.to_std().unwrap()
}


pub async fn start(context: Arc<Context>) {
    loop {
        time::sleep_until(next_threshold(600_000)).await;
        check::unchecked_codes(context.clone(), 4).await;
        time::sleep_until(next_threshold(600_000)).await;
        update::checked_codes(context.clone(), 4).await;
    }
}