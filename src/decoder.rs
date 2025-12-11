
use godot::{classes::AudioStream, prelude::*};
use crate::{VOIP_MIX_RATE, VOIP_FRAME_SIZE};


#[derive(GodotClass)]
#[class(base=AudioStream, no_init)]
pub struct VoipDecoder {
    pub decoder: opus::Decoder,

    pub base: Gd<AudioStream>,
}


#[godot_api]
impl VoipDecoder {


    #[func]
    fn decode_frames(&mut self, frames: PackedByteArray) {


    }


}


