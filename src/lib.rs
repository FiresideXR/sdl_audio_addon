
use godot::{classes::{AudioStream, AudioStreamPlayback, Engine, IAudioStream, IAudioStreamPlayback, native::AudioFrame} , prelude::*};
use sdl3::{audio::{AudioFormat, AudioSpec}, sys::audio::SDL_GetAudioStreamData};




/// This bypasses Godot's audio system to get their microphone
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


#[godot_api]
impl GdAudioBypass {


    #[func]
    fn get_input_devices(&mut self) -> Dictionary {

        let mut out = Dictionary::new();

        for device_id in self.sdl_audio.audio_recording_device_ids().unwrap() {

            out.set(device_id.name().unwrap_or("NoNameDevice".into()), device_id.id());
        }

        return out
    }


    #[func]
    fn create_default_microphone_audio_stream(&mut self) -> Gd<VoipAudioStream> {



        let godot_spec = AudioSpec::new(Some(44100), Some(1), Some(AudioFormat::F32LE));

        let stream = self.sdl_audio.default_recording_device().open_device_stream(Some(&godot_spec)).expect("Failed to open default device stream");

        let godot_stream = Gd::from_init_fn(|base| {
            VoipAudioStream{
                base,
                stream,
            }
        });

        godot_stream
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
impl IAudioStream for VoipAudioStream {

    fn get_stream_name(&self) -> GString {
        "VOIP Audio".into()
    }

    fn instantiate_playback(&self) -> Option<Gd<AudioStreamPlayback>> {
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
        return 0.0;
    }
}

const BUFFER_SIZE: usize = 256;

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
    

    unsafe fn mix_rawptr(&mut self, buffer: *mut AudioFrame, _rate_scale: f32, frames: i32) -> i32 {

        for i in 0..frames {
            unsafe {
                    *buffer.add(i as usize) = AudioFrame { left: 1.0, right: 0.5 };
                }
        }

        return frames;

        let mut audio_frame_index: usize= 0;

        while frames > 0 {

            let frames_to_get = std::cmp::min(frames, BUFFER_SIZE as i32);

            frames -= frames_to_get;

            // Fill our internal buffer with audio frames
            unsafe {
                SDL_GetAudioStreamData(self.master_stream.bind_mut().stream.stream(),self.buffer.as_mut_ptr() as *mut std::ffi::c_void, frames_to_get);
            }

            for i in 0..frames_to_get as usize {
                let frame_data = self.buffer[i];

                let frame = AudioFrame{left: frame_data, right: frame_data};

                unsafe {
                    *buffer.add(audio_frame_index) = frame;
                }
            
                audio_frame_index += 1
            }
        }

        return 0;
    }


    fn is_playing(&self) -> bool {
        self.paused
    }
}



struct SDLExtention;


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