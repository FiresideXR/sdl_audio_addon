
use godot::{classes::{AudioStream, AudioStreamPlayback, Engine, IAudioStream, IAudioStreamPlayback, native::AudioFrame}, obj::Singleton, prelude::*};
use sdl3::{audio::{AudioFormat, AudioSpec}, sys::audio::SDL_GetAudioStreamData};

mod encoder;
mod decoder;

use encoder::VoipEncoder;
use decoder::VoipDecoder;

/// This bypasses Godot's audio system to get their microphone
/// 
/// This is a Singleton that handles SDL core and is used to create 
/// an audio stream that mirrors the default microphone
#[derive(GodotClass)]
#[class(base=Object)]
struct GdAudioBypass {
    sdl_audio: sdl3::AudioSubsystem,
    base: Base<Object>,
}



#[godot_api]
impl IObject for GdAudioBypass {
    fn init(base: Base<Object>) -> Self {
        Self{
            sdl_audio: sdl3::init().unwrap().audio().unwrap(),
            base,
        }
    }
}


pub const VOIP_MIX_RATE: i32 = 48000;
pub const VOIP_FRAME_SIZE: i32 = VOIP_MIX_RATE / 100; // 10ms frame size

const VOIP_SPEC: AudioSpec = AudioSpec{freq: Some(VOIP_MIX_RATE), channels: Some(1), format: Some(AudioFormat::F32LE)};


#[godot_api]
impl GdAudioBypass {


    #[func]
    fn create_default_mic_encoder(&self) -> Gd<VoipEncoder> {

        let device = self.sdl_audio.default_recording_device();

     
        // I fucking love letting other people write wrappers for C libraries

        let stream = device.open_device_stream(Some(&VOIP_SPEC)).expect("Failed to create microphone stream");

        // Me when I don't have to do as much work because beautiful people will lift us all up

        let encoder = opus::Encoder::new(VOIP_MIX_RATE as u32, opus::Channels::Mono, opus::Application::Voip).expect("Faild to create OPUS Encoder");

        let repacket = opus::Repacketizer::new().expect("Could not create repacketizer");

        Gd::from_init_fn(|base|{
            VoipEncoder{
                base,
                encoder,
                stream,
                repacket,
                paused: true,
            }
        })
    }

    #[func]
    fn create_voip_decoder_stream(&self) -> Gd<VoipDecoder> {
        
        let mix_rate = godot::classes::AudioServer::singleton().get_mix_rate();

        let godot_spec = AudioSpec::new(Some(mix_rate as i32), Some(1), Some(AudioFormat::F32LE));

        let stream = self.sdl_audio.new_stream(Some(&VOIP_SPEC), Some(&godot_spec)).expect("Failed to create voip to godot stream");


        let decoder = opus::Decoder::new(VOIP_MIX_RATE as u32, opus::Channels::Mono).expect("Failed to create Opus decoder");

        let repacket = opus::Repacketizer::new().expect("Failed to create Repacketizer for decoding");

        Gd::from_init_fn(|base| {
            VoipDecoder { 
                stream,
                base,
                repacket,
                decode_buffer: [0.0; VOIP_FRAME_SIZE as usize],
                repack_buffer: [0; 1276],
                decoder,
            }
        })
    }
}


struct SDLExtention;

// This code registeres the Singleton and frees the memory once we're done
#[gdextension]
unsafe impl ExtensionLibrary for SDLExtention {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton(
                &GdAudioBypass::class_id().to_string_name(),
                &GdAudioBypass::new_alloc(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            let singleton_name = &GdAudioBypass::class_id().to_string_name();

            if let Some(my_singleton) = engine.get_singleton(singleton_name){
                engine.unregister_singleton(singleton_name);
                my_singleton.free();
            } else {
                godot_error!("Failed to get audio bypass singleton");
            }
        }
    }
}