use std::io::{BufReader, Cursor};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;

use rodio::{DeviceSinkBuilder, Player};

pub struct BgmHandler {
    receiver: Receiver<()>,
}

impl BgmHandler {
    pub fn new(receiver: Receiver<()>) -> Self {
        Self { receiver }
    }

    pub fn start(self) {
        thread::spawn(move || {
            let Ok(handle) = DeviceSinkBuilder::open_default_sink() else {
                return;
            };
            let sink = Player::connect_new(handle.mixer());

            loop {
                match self.receiver.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {
                        if append_audio(&sink).is_err() {
                            break;
                        }
                        sink.sleep_until_end();
                    }
                }
            }
        });
    }
}

fn append_audio(sink: &Player) -> Result<(), rodio::decoder::DecoderError> {
    let bytes = include_bytes!("../../src/assets/audio/BGM.mp3");
    let cursor = Cursor::new(bytes);
    let decoder = rodio::Decoder::try_from(BufReader::new(cursor))?;
    sink.append(decoder);
    sink.set_volume(0.4);
    Ok(())
}
