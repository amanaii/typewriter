use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TestTimer {
    started_at: Option<Instant>,
    limit: Option<Duration>,
}

impl TestTimer {
    pub fn new(limit: Option<Duration>) -> Self {
        Self {
            started_at: None,
            limit,
        }
    }

    pub fn start(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at
            .map(|instant| instant.elapsed())
            .unwrap_or_default()
    }

    pub fn remaining(&self) -> Option<Duration> {
        let limit = self.limit?;
        Some(limit.saturating_sub(self.elapsed()))
    }

    pub fn expired(&self) -> bool {
        self.limit
            .is_some_and(|limit| self.started_at.is_some() && self.elapsed() >= limit)
    }

    pub fn is_started(&self) -> bool {
        self.started_at.is_some()
    }
}
