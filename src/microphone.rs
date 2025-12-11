use godot::prelude::*;


#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct VoipEncoder {
    pub encoder: opus::Encoder,

    pub stream: sdl3::audio::AudioStreamOwner,
    
    pub base: Base<RefCounted>,
}


#[godot_api]
impl VoipEncoder {




}





