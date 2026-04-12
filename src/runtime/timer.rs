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
        let mut current_timeout = timeout;

        loop {
            if running {
                match timer_command_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(TimerCommand::Start(next_timeout)) => {
                        reset_timer(&timer);
                        current_timeout = next_timeout;
                        running = true;
                    }
                    Ok(TimerCommand::Stop) => {
                        running = false;
                    }
                    Ok(TimerCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let mut elapsed = timer.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        *elapsed += 1;
                        if current_timeout > 0 && *elapsed >= current_timeout {
                            running = false;
                            timeout_tx.send(()).ok();
                        }
                    }
                }
            } else {
                match timer_command_rx.recv() {
                    Ok(TimerCommand::Start(next_timeout)) => {
                        reset_timer(&timer);
                        current_timeout = next_timeout;
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
    *timer.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
}

pub(super) fn current_timer(timer: &Arc<Mutex<i32>>) -> i32 {
    *timer.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
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

pub(super) fn cancel_typing_session(
    app: &mut crate::presentation::ui::app::App,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
) {
    stop_timer(timer_command_tx);
    app.return_to_menu();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TimerCommand;
    use std::sync::mpsc;

    #[test]
    fn timeout_zero_does_not_emit_timeout_signal() {
        let timer = Arc::new(Mutex::new(0));
        let (timer_command_tx, timer_command_rx) = mpsc::channel();
        let (timeout_tx, timeout_rx) = mpsc::channel();
        let handle = spawn_timer_thread(Arc::clone(&timer), 0, timer_command_rx, timeout_tx);

        timer_command_tx.send(TimerCommand::Start(0)).ok();
        std::thread::sleep(Duration::from_millis(1100));

        assert!(timeout_rx.try_recv().is_err());

        timer_command_tx.send(TimerCommand::Shutdown).ok();
        handle.join().ok();
    }
}
