#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;

#[cfg(not(target_arch = "wasm32"))]
const LEVEL_MUSIC: &[u8] = include_bytes!("../music/music.ogg");
#[cfg(not(target_arch = "wasm32"))]
const SFX_JUMP: &[u8] = include_bytes!("../sounds/jump.ogg");
#[cfg(not(target_arch = "wasm32"))]
const SFX_DEATH: &[u8] = include_bytes!("../sounds/death.ogg");
#[cfg(not(target_arch = "wasm32"))]
const SFX_SPRING: &[u8] = include_bytes!("../sounds/spring.ogg");
#[cfg(not(target_arch = "wasm32"))]
const SFX_STEPS: [&[u8]; 3] = [
    include_bytes!("../sounds/step1.ogg"),
    include_bytes!("../sounds/step2.ogg"),
    include_bytes!("../sounds/step3.ogg"),
];

pub struct Music {
    #[cfg(not(target_arch = "wasm32"))]
    _stream: OutputStream,
    #[cfg(not(target_arch = "wasm32"))]
    handle: OutputStreamHandle,
    #[cfg(not(target_arch = "wasm32"))]
    sink: Sink,
}

impl Music {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&handle).unwrap();
            Music { _stream, handle, sink }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Music {}
        }
    }

    pub fn play(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.sink.stop();
            let source = Decoder::new(Cursor::new(LEVEL_MUSIC))
                .unwrap()
                .skip_duration(std::time::Duration::from_secs(1));
            self.sink.append(source);
            self.sink.set_volume(0.5);
            self.sink.play();
        }
    }

    pub fn stop(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.sink.stop();
    }

    pub fn play_jump(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let source = Decoder::new(Cursor::new(SFX_JUMP)).unwrap();
            let _ = self.handle.play_raw(source.convert_samples());
        }
    }

    pub fn play_spring(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let source = Decoder::new(Cursor::new(SFX_SPRING)).unwrap();
            let _ = self.handle.play_raw(source.convert_samples());
        }
    }

    pub fn play_death(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let source = Decoder::new(Cursor::new(SFX_DEATH)).unwrap();
            let _ = self.handle.play_raw(source.convert_samples());
        }
    }

    pub fn play_step(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let idx = macroquad::rand::gen_range(0, 3);
            let source = Decoder::new(Cursor::new(SFX_STEPS[idx])).unwrap();
            let _ = self.handle.play_raw(source.convert_samples());
        }
    }
}
