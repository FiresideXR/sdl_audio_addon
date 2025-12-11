use godot::prelude::*;
use crate::{VOIP_MIX_RATE, VOIP_FRAME_SIZE};



/// Encodes audio data from the default microphone
#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct VoipEncoder {
    pub encoder: opus::Encoder,

    pub stream: sdl3::audio::AudioStreamOwner,

    pub repacket: opus::Repacketizer,
    
    pub base: Base<RefCounted>,
}


#[godot_api]
impl VoipEncoder {

    /// Starts the microphone stream up and clears the buffer
    #[func]
    fn resume(&self) {
        self.stream.clear().expect("Could not clear stream");
        self.stream.resume().expect("Could not resume stream");
    }

    /// Stops the microphone stream
    #[func]
    fn pause(&self) {
        self.stream.pause().expect("Could not pause stream");
    }

    /// Reads off the data from the microphone as an Opus Audio packet
    #[func]
    fn get_packet(&mut self) -> PackedByteArray {

        let mut frames: Vec<Vec<u8>> = Vec::new();

        //let mut repacket = opus::Repacketizer::new().expect("Could not init Opus repacketizer");

        let mut buffer: [f32; VOIP_FRAME_SIZE as usize] = [0.0; VOIP_FRAME_SIZE as usize];

        // Collect frames from the microphone
        while (self.stream.available_bytes().expect("D: Couldn't get available bytes") / 4 > VOIP_FRAME_SIZE) && (frames.len() <= 10) { // 80 character rule dying a slow and painful death

            self.stream.read_f32_samples(&mut buffer).expect("Could not read samples into buffer");
            

            let new_frame = self.encoder.encode_vec_float(&buffer, 1024).expect("Something went terribly wrong");

            frames.push(new_frame);
        }

        if frames.len() == 0 {
            return PackedByteArray::new()
        }

        if frames.len() == 1 {
            return PackedArray::from(frames.pop().unwrap())
        }

        // Just realized I overcomplicated this :P
        // 
        // The repacketizer is designed to combine frames into larger packets
        // But I'm working with a 10-20ms frame size...
        //
        // ...and this code is gonna run 90 times a second (~11ms) so it's unlikely that this will be needed

        let mut state = self.repacket.begin();

        for frame in frames.iter() {
            // I'd prefer to have this in the while loop but the data doesn't live long enough :(
            state = state.cat_move(frame).expect("unable to cat frame");
        }

        // Today's magic number brought to you by https://www.opus-codec.org/docs/opus_api-1.5/group__opus__repacketizer.html
        let mut combined_frames: Vec<u8> = vec![0u8; 4_000];

        let size = state.out(&mut combined_frames).expect("Could not combine frames");

        //We intentionally allocate a decent chunk of memory for ✨reasons✨ so this trims off what we don't need
        let _ = combined_frames.split_off(size);

        PackedArray::from(combined_frames)

    }


}





