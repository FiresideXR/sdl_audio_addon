use godot::{classes::{Engine, }, prelude::*};
use sdl3::audio::{AudioDevice, AudioStream};



fn main() {
    println!("Hello, world!");
}


#[derive(GodotClass)]
#[class(init, base=Object)]
struct GdAudioBypass {
    base: Base<Object>,
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
                godot_error!("Failed to get singleton");
            }
        }
    }
}