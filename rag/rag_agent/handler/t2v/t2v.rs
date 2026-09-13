use std::time::Instant;
use log::info;
use msedge_tts::tts::client::connect;
use msedge_tts::tts::SpeechConfig;
use msedge_tts::voice::get_voices_list;
use crate::shared::sharedUtils;

pub fn text2voice(textParam: String) {
    info!("get voices list...");
    let voices = get_voices_list().unwrap();
    for voice in &voices {
        info!("voice.name {:?}", voice.name);
        if voice.name.contains(sharedUtils::Parameter::T2V_INDONESIA_MAN.value) {
            info!("choose '{}' to synthesize...", voice.name);
            let config = SpeechConfig::from(voice);
            let mut tts = connect().unwrap();
            let start = Instant::now();
            let audio = tts.synthesize(&*textParam, &config).unwrap();
            info!("{:?}", audio.audio_metadata);
            info!("{:?}", Instant::now() - start);

            info!("play audio...");
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let sink = stream_handle
                .play_once(std::io::Cursor::new(audio.audio_bytes))
                .unwrap();
            sink.sleep_until_end();
            info!("play audio done.");

            break;
        }
    }
}
