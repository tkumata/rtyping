use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::TimerCommand;

pub fn spawn_timer_thread(
    timer: Arc<Mutex<i32>>,
    timeout: i32,
    timer_command_rx: mpsc::Receiver<TimerCommand>,
    timeout_tx: mpsc::Sender<()>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut running = false;

        loop {
            if running {
                match timer_command_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(TimerCommand::Start) => {
                        reset_timer(&timer);
                        running = true;
                    }
                    Ok(TimerCommand::Stop) => {
                        running = false;
                    }
                    Ok(TimerCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let mut elapsed = timer.lock().unwrap();
                        *elapsed += 1;
                        if *elapsed >= timeout {
                            running = false;
                            timeout_tx.send(()).ok();
                        }
                    }
                }
            } else {
                match timer_command_rx.recv() {
                    Ok(TimerCommand::Start) => {
                        reset_timer(&timer);
                        running = true;
                    }
                    Ok(TimerCommand::Stop) => {
                        running = false;
                    }
                    Ok(TimerCommand::Shutdown) | Err(_) => break,
                }
            }
        }
    })
}

pub(super) fn reset_timer(timer: &Arc<Mutex<i32>>) {
    *timer.lock().unwrap() = 0;
}

pub(super) fn current_timer(timer: &Arc<Mutex<i32>>) -> i32 {
    *timer.lock().unwrap()
}

pub(super) fn stop_timer(timer_command_tx: &mpsc::Sender<TimerCommand>) {
    timer_command_tx.send(TimerCommand::Stop).ok();
}

pub(super) fn finish_typing_session(
    app: &mut crate::presentation::ui::app::App,
    timer: &Arc<Mutex<i32>>,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
) {
    stop_timer(timer_command_tx);
    app.update_timer(current_timer(timer));
    app.finish_typing();
}

