use godot::prelude::*;
use crate::VOIP_FRAME_SIZE;



/// Encodes audio data from the default microphone
#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct VoipEncoder {
    pub encoder: opus::Encoder,

    pub repacket: opus::Repacketizer,
    
    pub base: Base<RefCounted>,
}


#[godot_api]
impl VoipEncoder {

    /// Reads off the data from the microphone as an Opus Audio packet
    #[func]
    fn get_packet(&mut self) -> PackedByteArray {
        let mut frames: Vec<Vec<u8>> = Vec::new();

        //let mut buffer: [f32; VOIP_FRAME_SIZE as usize] = [0.0; VOIP_FRAME_SIZE as usize];

        //godot_print!("frames available: {}", godot::classes::AudioServer::singleton().get_input_frames_available());

        // Collect frames from the microphone
        while (godot::classes::AudioServer::singleton().get_input_frames_available() >= VOIP_FRAME_SIZE) && (frames.len() <= 10) { // 80 character rule dying a slow and painful death

            let frame_buffer = godot::classes::AudioServer::singleton().get_input_frames(VOIP_FRAME_SIZE);



            let f: Vec<f32> = frame_buffer.as_slice().iter().map(|e| (e.x + e.y) / 2.0).collect();

            //godot_print!("{:?}", f);


            //self.stream.read_f32_samples(&mut buffer).expect("Could not read samples into buffer");
            

            let new_frame = self.encoder.encode_vec_float(&f, 1024).expect("Something went terribly wrong");

            frames.push(new_frame);
        }

        if frames.len() == 0 {
            return PackedByteArray::new()
        }

        if frames.len() == 1 {
            return PackedArray::from(frames.pop().unwrap())
        }

        let mut state = self.repacket.begin();

        for frame in frames.iter() {
            // I'd prefer to have this in the while loop but the data doesn't live long enough :(
            // Maybe I just need to finagle the lifetimes though

            //Repacketizer can randomly fail. In this case we just drop the remaining packets.
            // TODO! This is probably a bad way to handle things. 
            match state.cat(frame) {
                Ok(()) => {},
                Err(_) => {
                    godot_print!("Repacketizer error: Dropping packets");
                    break
                },
            };
        }

        // Today's magic number brought to you by https://www.opus-codec.org/docs/opus_api-1.5/group__opus__repacketizer.html
        let mut combined_frames: Vec<u8> = vec![0u8; 4_000];

        let size = state.out(&mut combined_frames).expect("Could not combine frames");

        //We intentionally allocate a decent chunk of memory for ✨reasons✨ so this trims off what we don't need
        let _ = combined_frames.split_off(size);

        PackedArray::from(combined_frames)

    }


}





