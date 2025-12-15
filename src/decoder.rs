
use godot::{classes::{AudioStream, AudioStreamPlayback, IAudioStream, IAudioStreamPlayback, native::AudioFrame}, prelude::*};
use sdl3::sys::audio::SDL_GetAudioStreamData;
use crate::{VOIP_FRAME_SIZE};


#[derive(GodotClass)]
#[class(base=AudioStream, no_init)]
pub struct VoipDecoder {
    pub decoder: opus::Decoder,

    pub stream: sdl3::audio::AudioStreamOwner,

    pub repacket: opus::Repacketizer,

    //Magic number: https://opus-codec.org/docs/opus_api-1.3.1/group__opus__repacketizer.html#gac591b550d92125b4abfa11a4b609f51f
    pub repack_buffer: [u8; 1276],

    pub decode_buffer: [f32; VOIP_FRAME_SIZE as usize],

    pub base: Base<AudioStream>,
}


#[godot_api]
impl VoipDecoder {

    #[func]
    fn stream_size(&self) -> i32 {
        return self.stream.available_bytes().unwrap() / 4
    }


    /// Call this to clear the stream data (such as when there is a large latency)
    #[func]
    fn clear_stream(&self) {
        self.stream.clear().expect("Failed to clear stream")
    }

    #[func]
    fn decode_packet(&mut self, packet: PackedByteArray) {

        if packet.len() == 0 {
            return;
        }


        let mut rp = self.repacket.begin();

        rp.cat(packet.as_slice()).expect("Failed to cat packed while decoding");


        for i in 0..rp.get_nb_frames() {

            let rp_size = rp.out_range(i, i + 1, &mut self.repack_buffer).expect("Failed to split packet");

            let size = self.decoder.decode_float(&self.repack_buffer[0..rp_size], &mut self.decode_buffer, false).expect("Failed to decode packet");

            self.stream.put_data_f32(&self.decode_buffer[0..size]).expect("Failed to put decoded packet into stream");
        }
    }


}

#[godot_api]
impl IAudioStream for VoipDecoder {

    fn get_stream_name(&self) -> GString {
        "VOIP Audio Decoder".into()
    }

    fn instantiate_playback(&self) -> Option<Gd<AudioStreamPlayback>> {

        self.stream.clear().expect("could not clear stream");

        let playback = Gd::from_init_fn(|base| {
            VoipPlackback {
                base,
                buffer: [1.23; BUFFER_SIZE],
                master_stream: self.to_gd(),
                paused: false,
            }
        });

        godot_print!("Created playback");

        Some(playback.upcast())
    }

    fn get_length(&self) -> f64 {
        0.0
    }

}

const BUFFER_SIZE: usize = 1024;

#[derive(GodotClass)]
#[class(base=AudioStreamPlayback, no_init)]
struct VoipPlackback {
    base: Base<AudioStreamPlayback>,
    buffer: [f32; BUFFER_SIZE],
    master_stream: Gd<VoipDecoder>,
    paused: bool,
}

#[godot_api]
impl IAudioStreamPlayback for VoipPlackback {

    fn start (&mut self, _from: f64) {
        //self.master_stream.bind().stream.resume().expect("I dunno what happened");
        self.paused = false;
    }

    fn stop (&mut self) {
        //self.master_stream.bind().stream.pause().expect("Wuh");
        self.paused = true;
    }

    fn is_playing(&self) -> bool {
        !self.paused
    }

    fn get_playback_position(&self) -> f64 {
        0.0
    }

    fn get_loop_count(&self) -> i32 {
        0
    }

    /// Wuh
    unsafe fn mix_rawptr(&mut self, buffer: *mut AudioFrame, _rate_scale: f32, frames: i32) -> i32 {

        let mut audio_frame_index: usize = 0;

        let mut remaining_frames = frames;

        while remaining_frames > 0 {

            let frames_to_get = std::cmp::min(remaining_frames, BUFFER_SIZE as i32);

            let filled_bytes;

            unsafe {
                filled_bytes = SDL_GetAudioStreamData(self.master_stream.bind_mut().stream.stream(),self.buffer.as_mut_ptr() as *mut std::ffi::c_void, frames_to_get * 4);
            }


            for i in 0..(filled_bytes / 4) as usize {
                let frame_data = self.buffer[i];

                let frame = AudioFrame{left: frame_data, right: frame_data};

                unsafe {
                    *buffer.add(audio_frame_index) = frame;
                }
            
                audio_frame_index += 1;
                remaining_frames -= 1;
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
}