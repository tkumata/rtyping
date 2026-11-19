use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source, source::SineWave};
use std::io::{self, BufReader, Cursor};
use std::time::Duration;

#[derive(Default)]
pub struct Audio {
    bgm: Option<Player>,
    device: Option<MixerDeviceSink>,
    tried_opening_device: bool,
}

impl Audio {
    pub fn start_bgm(&mut self) -> io::Result<()> {
        let bytes = include_bytes!("../assets/audio/BGM.mp3");
        let decoder =
            Decoder::new_looped(BufReader::new(Cursor::new(bytes))).map_err(io::Error::other)?;
        if let Some(device) = self.device()? {
            let player = Player::connect_new(device.mixer());
            player.set_volume(0.4);
            player.append(decoder);
            self.bgm = Some(player);
        }
        Ok(())
    }

    pub fn start_round(&mut self) {
        self.tried_opening_device = self.device.is_some();
    }

    pub fn play_typing_sound(&mut self, frequency: f32) -> io::Result<()> {
        if let Some(device) = self.device()? {
            let source = SineWave::new(frequency).take_duration(Duration::from_millis(100));
            device.mixer().add(source);
        }
        Ok(())
    }

    fn device(&mut self) -> io::Result<Option<&MixerDeviceSink>> {
        if !self.tried_opening_device {
            self.tried_opening_device = true;
            let mut device = DeviceSinkBuilder::open_default_sink().map_err(io::Error::other)?;
            device.log_on_drop(false);
            self.device = Some(device);
        }
        Ok(self.device.as_ref())
    }
}
