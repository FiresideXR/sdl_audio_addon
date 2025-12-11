
use godot::{classes::{AudioStream, AudioStreamPlayback, Engine, IAudioStream, IAudioStreamPlayback, native::AudioFrame}, obj::Singleton, prelude::*};
use sdl3::{audio::{AudioFormat, AudioSpec}, sys::audio::SDL_GetAudioStreamData};

mod microphone;
mod stream;


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


const VOIP_MIX_RATE: i32 = 44100;


#[godot_api]
impl GdAudioBypass {


    #[func]
    fn create_default_mic_encoder(&self) -> Gd<microphone::VoipEncoder> {

        let voip_spec = AudioSpec::new(Some(VOIP_MIX_RATE), Some(1), Some(AudioFormat::F32LE));

        let device = self.sdl_audio.default_recording_device();

        // I fucking love letting other people write wrappers for C libraries

        let stream = device.open_device_stream(Some(&voip_spec)).expect("Failed to create microphone stream");

        // Me when I don't have to do as much work because beautiful people will lift us all up

        let encoder = opus::Encoder::new(VOIP_MIX_RATE as u32, opus::Channels::Mono, opus::Application::Voip).expect("Faild to create OPUS Encoder");

        Gd::from_init_fn(|base|{
            microphone::VoipEncoder{
                base,
                encoder,
                stream,
            }
        })
    }

    #[func]
    fn create_voip_decover_stream(&self) {
        
    }


    #[func]
    fn create_default_microphone_audio_stream(&mut self) -> Gd<VoipAudioStream> {

        let mix_rate = godot::classes::AudioServer::singleton().get_mix_rate();

        godot_print!("Mix rate: {mix_rate}");

        let godot_spec = AudioSpec::new(Some(mix_rate as i32), Some(1), Some(AudioFormat::F32LE));

        
        let device = self.sdl_audio.default_recording_device();

        godot_print!("{:?}", device.id());
        

        let stream = device.open_device_stream(Some(&godot_spec)).expect("Failed to open default device stream");
        

        //let _ = stream.resume();

        //let spec = stream.get_format().expect("bazinga");

        //godot_print!("{spec:?}");

        

        Gd::from_init_fn(|base| {
            VoipAudioStream{
                base,
                stream,
            }
        })
    }



    /// This is a doc comment
    #[func]
    fn test(&mut self) {
        for id in self.sdl_audio.audio_recording_device_ids().expect("Found no recording devices") {
            godot_print!("Recording Device: {}", id.name().unwrap())
        }

    }
}



#[derive(GodotClass)]
#[class(base=AudioStream, no_init)]
struct VoipAudioStream {
    stream: sdl3::audio::AudioStreamOwner,
    base: Base<AudioStream>,
}

#[godot_api]
impl VoipAudioStream {
    #[func]
    pub fn get_stream_size(&mut self) -> i32 {
        self.stream.available_bytes().expect("Something went wrong getting avilable bytes") / 4
    }
}

#[godot_api]
impl IAudioStream for VoipAudioStream {

    fn get_stream_name(&self) -> GString {
        "VOIP Audio".into()
    }

    fn instantiate_playback(&self) -> Option<Gd<AudioStreamPlayback>> {

        // Create the audio playback object that is used to mix the audio with the Godot audio system
        let playback = Gd::from_init_fn(|base| {
            VoipAudioPlayback{
                base,
                buffer: [0f32; BUFFER_SIZE],
                master_stream: self.to_gd(),
                paused: true,
            }
        });

        godot_print!("Playback created");

    
        Some(playback.upcast())
    }

    fn get_length(&self) -> f64 {
        0.0
    }
}

const BUFFER_SIZE: usize = 1024;

#[derive(GodotClass)]
#[class(base=AudioStreamPlayback, no_init)]
struct VoipAudioPlayback {
    base: Base<AudioStreamPlayback>,
    master_stream: Gd<VoipAudioStream>,
    buffer: [f32; BUFFER_SIZE],
    paused: bool,
}


#[godot_api]
impl IAudioStreamPlayback for VoipAudioPlayback {


    fn start (&mut self, _from: f64) {
        self.master_stream.bind().stream.resume().expect("I dunno what happened");
        self.paused = false;
    }

    fn stop (&mut self) {
        self.master_stream.bind().stream.pause().expect("Wuh");
        self.paused = true;
    }

    fn get_playback_position(&self) -> f64 {
        0.0
    }

    fn get_loop_count(&self) -> i32 {
        0
    }
    
    // This function mixes the data from the microphone stream into the Godot audio system
    unsafe fn mix_rawptr(&mut self, buffer: *mut AudioFrame, _rate_scale: f32, frames: i32) -> i32 {

        let mut audio_frame_index: usize = 0;

        let mut remaining_frames = frames;

        while remaining_frames > 0 {

            let frames_to_get = std::cmp::min(remaining_frames, BUFFER_SIZE as i32);

            remaining_frames -= frames_to_get;

            let filled_bytes;

            // Fill our internal buffer with audio frames
            unsafe {
                filled_bytes = SDL_GetAudioStreamData(self.master_stream.bind_mut().stream.stream(),self.buffer.as_mut_ptr() as *mut std::ffi::c_void, frames_to_get * 4);
            }

            for i in 0..(filled_bytes / 4) as usize {
                let frame_data = self.buffer[i];

                let frame = AudioFrame{left: frame_data, right: frame_data};

                unsafe {
                    *buffer.add(audio_frame_index) = frame;
                }
            
                audio_frame_index += 1
            }
        }

        for i in audio_frame_index..(frames as usize) {
            let frame = AudioFrame{left: 0.0, right: 0.0};

            unsafe {
                *buffer.add(i) = frame;
            }
        }

        frames
    }


    fn is_playing(&self) -> bool {
        self.paused
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