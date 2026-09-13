// use log::info;
// use voice_stream::VoiceStream;
// use voice_stream::cpal::traits::StreamTrait;
// // Disabled: voice-stream 0.4.0 has ort/voice_activity_detector conflicts with existing deps
// pub async fn voice_to_text() -> Result<(), Box<dyn std::error::Error>> {
//     info!("stream input voice...");
//
//     let (voice_stream, mut rx) = VoiceStream::default_device().unwrap();
//     voice_stream.play().unwrap();
//     loop {
//         match rx.recv().await {
//             Some(samples) => {
//                 println!("Received voice data chunk of size: {}", samples.len());
//             }
//             _ => {}
//         }
//     }
//     Ok(())
// }