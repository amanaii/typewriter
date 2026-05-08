use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventStream {
    rx: Receiver<AppEvent>,
}

impl EventStream {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        let input_tx = tx.clone();
        thread::spawn(move || loop {
            match event::read() {
                Ok(CrosstermEvent::Key(key)) if input_tx.send(AppEvent::Key(key)).is_err() => break,
                Ok(CrosstermEvent::Resize(width, height))
                    if input_tx.send(AppEvent::Resize(width, height)).is_err() =>
                {
                    break
                }
                Ok(_) => {}
                Err(_) => break,
            }
        });
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(16));
            if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        });
        Self { rx }
    }

    pub fn recv(&self) -> Result<AppEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
