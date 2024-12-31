use std::io::{BufReader, Cursor};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;

pub struct BgmHandler {
    receiver: Receiver<()>,
}

impl BgmHandler {
    pub fn new(receiver: Receiver<()>) -> Self {
        Self { receiver }
    }

    pub fn start(self) {
        thread::spawn(move || loop {
            match self.receiver.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    play_audio();
                }
            }
        });
    }
}

fn play_audio() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    let bytes = include_bytes!("../../src/assets/audio/BGM.mp3");
    let cursor = Cursor::new(bytes);

    sink.append(rodio::Decoder::new(BufReader::new(cursor)).unwrap());
    sink.set_volume(0.4);
    sink.sleep_until_end();
}
