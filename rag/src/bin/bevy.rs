// // bevy.rs - Human talking animation using Bevy engine
// use bevy::prelude::*;
// use bevy::window::WindowResolution;
// use std::collections::HashMap;
// use std::time::Duration;
// use nom::Input;
//
// #[derive(Component)]
// struct Human {
//     is_talking: bool,
//     talk_timer: Timer,
//     mouth_state: MouthState,
//     blink_timer: Timer,
//     eye_state: EyeState,
// }
//
// #[derive(Component)]
// struct MouthSprite;
//
// #[derive(Component)]
// struct EyeSprite;
//
// #[derive(Component)]
// struct HeadSprite;
//
// #[derive(Component)]
// struct BodySprite;
//
// #[derive(Clone, Copy, PartialEq)]
// enum MouthState {
//     Closed,
//     Open,
//     Smile,
//     Speaking,
// }
//
// #[derive(Clone, Copy, PartialEq)]
// enum EyeState {
//     Open,
//     Closed,
//     Blinking,
// }
//
// #[derive(Resource)]
// struct AnimationAssets {
//     mouth_textures: HashMap<MouthState, Handle<Image>>,
//     eye_textures: HashMap<EyeState, Handle<Image>>,
//     head_texture: Handle<Image>,
//     body_texture: Handle<Image>,
// }
//
// #[derive(Resource)]
// struct TalkingState {
//     current_text: String,
//     is_speaking: bool,
//     speech_duration: Timer,
// }
//
// impl Default for TalkingState {
//     fn default() -> Self {
//         Self {
//             current_text: String::new(),
//             is_speaking: false,
//             speech_duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
//         }
//     }
// }
//
// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins.set(WindowPlugin {
//             primary_window: Some(Window {
//                 title: "AIMS Automotive AI - Human Avatar".to_string(),
//                 resolution: WindowResolution::new(800.0, 600.0),
//                 resizable: false,
//                 ..default()
//             }),
//             ..default()
//         }))
//         .init_resource::<TalkingState>()
//         .add_systems(Startup, setup_scene)
//         .add_systems(Update, (
//             handle_input,
//             update_talking_animation,
//             update_blinking_animation,
//             update_mouth_sprites,
//             update_eye_sprites,
//         ))
//         .run();
// }
//
// fn setup_scene(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut talking_state: ResMut<TalkingState>,
// ) {
//     // Setup camera
//     commands.spawn(Camera2dBundle::default());
//
//     // Create animation assets
//     let mut mouth_textures = HashMap::new();
//     mouth_textures.insert(MouthState::Closed, asset_server.load("mouth_closed.png"));
//     mouth_textures.insert(MouthState::Open, asset_server.load("mouth_open.png"));
//     mouth_textures.insert(MouthState::Smile, asset_server.load("mouth_smile.png"));
//     mouth_textures.insert(MouthState::Speaking, asset_server.load("mouth_speaking.png"));
//
//     let mut eye_textures = HashMap::new();
//     eye_textures.insert(EyeState::Open, asset_server.load("eyes_open.png"));
//     eye_textures.insert(EyeState::Closed, asset_server.load("eyes_closed.png"));
//     eye_textures.insert(EyeState::Blinking, asset_server.load("eyes_blinking.png"));
//
//     let head_texture = asset_server.load("head.png");
//     let body_texture = asset_server.load("body.png");
//
//     commands.insert_resource(AnimationAssets {
//         mouth_textures,
//         eye_textures,
//         head_texture: head_texture.clone(),
//         body_texture: body_texture.clone(),
//     });
//
//     // Create human avatar
//     let human_entity = commands.spawn((
//         SpriteBundle {
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//             ..default()
//         },
//         Human {
//             is_talking: false,
//             talk_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
//             mouth_state: MouthState::Closed,
//             blink_timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
//             eye_state: EyeState::Open,
//         },
//     )).id();
//
//     // Add body sprite
//     commands.spawn((
//         SpriteBundle {
//             texture: body_texture,
//             transform: Transform::from_translation(Vec3::new(0.0, -100.0, 1.0))
//                 .with_scale(Vec3::splat(2.0)),
//             ..default()
//         },
//         BodySprite,
//     ));
//
//     // Add head sprite
//     commands.spawn((
//         SpriteBundle {
//             texture: head_texture,
//             transform: Transform::from_translation(Vec3::new(0.0, 50.0, 2.0))
//                 .with_scale(Vec3::splat(1.5)),
//             ..default()
//         },
//         HeadSprite,
//     ));
//
//     // Add eye sprites
//     commands.spawn((
//         SpriteBundle {
//             texture: asset_server.load("eyes_open.png"),
//             transform: Transform::from_translation(Vec3::new(0.0, 70.0, 3.0))
//                 .with_scale(Vec3::splat(1.0)),
//             ..default()
//         },
//         EyeSprite,
//     ));
//
//     // Add mouth sprite
//     commands.spawn((
//         SpriteBundle {
//             texture: asset_server.load("mouth_closed.png"),
//             transform: Transform::from_translation(Vec3::new(0.0, 30.0, 3.0))
//                 .with_scale(Vec3::splat(1.0)),
//             ..default()
//         },
//         MouthSprite,
//     ));
//
//     // Initialize talking state
//     talking_state.current_text = "Welcome to AIMS Automotive AI System".to_string();
//
//     info!("Human avatar initialized");
//     info!("Press SPACE to start talking animation");
//     info!("Press T to toggle talking state");
//     info!("Press R to reset animation");
// }
//
// fn handle_input(
//     keyboard_input: Res<Input<KeyCode>>,
//     mut talking_state: ResMut<TalkingState>,
//     mut human_query: Query<&mut Human>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::Space) {
//         talking_state.is_speaking = !talking_state.is_speaking;
//         talking_state.speech_duration.reset();
//
//         for mut human in human_query.iter_mut() {
//             human.is_talking = talking_state.is_speaking;
//             if human.is_talking {
//                 human.talk_timer.reset();
//                 human.mouth_state = MouthState::Speaking;
//             } else {
//                 human.mouth_state = MouthState::Closed;
//             }
//         }
//
//         if talking_state.is_speaking {
//             info!("Started talking: {}", talking_state.current_text);
//         } else {
//             info!("Stopped talking");
//         }
//     }
//
//     if keyboard_input.just_pressed(KeyCode::T) {
//         // Toggle between different talking texts
//         let texts = vec![
//             "Apa itu mobil hybrid?",
//             "Bagaimana cara kerja GADAI kendaraan?",
//             "Jelaskan teknologi Toyota Hybrid Synergy Drive",
//             "Buatkan flowchart proses kredit mobil",
//             "What are the benefits of electric vehicles?",
//             "How does automotive financing work?",
//         ];
//
//         let current_index = texts.iter().position(|&x| x == talking_state.current_text).unwrap_or(0);
//         let next_index = (current_index + 1) % texts.len();
//         talking_state.current_text = texts[next_index].to_string();
//
//         info!("Changed text to: {}", talking_state.current_text);
//     }
//
//     if keyboard_input.just_pressed(KeyCode::R) {
//         // Reset animation
//         talking_state.is_speaking = false;
//         for mut human in human_query.iter_mut() {
//             human.is_talking = false;
//             human.mouth_state = MouthState::Closed;
//             human.eye_state = EyeState::Open;
//             human.talk_timer.reset();
//             human.blink_timer.reset();
//         }
//         info!("Animation reset");
//     }
// }
//
// fn update_talking_animation(
//     time: Res<Time>,
//     mut talking_state: ResMut<TalkingState>,
//     mut human_query: Query<&mut Human>,
// ) {
//     talking_state.speech_duration.tick(time.delta());
//
//     for mut human in human_query.iter_mut() {
//         if human.is_talking {
//             human.talk_timer.tick(time.delta());
//
//             if human.talk_timer.just_finished() {
//                 // Cycle through mouth states for talking animation
//                 human.mouth_state = match human.mouth_state {
//                     MouthState::Closed => MouthState::Open,
//                     MouthState::Open => MouthState::Speaking,
//                     MouthState::Speaking => MouthState::Closed,
//                     MouthState::Smile => MouthState::Open,
//                 };
//             }
//
//             // Auto-stop talking after duration
//             if talking_state.speech_duration.just_finished() {
//                 human.is_talking = false;
//                 human.mouth_state = MouthState::Smile;
//                 talking_state.is_speaking = false;
//                 info!("Finished talking");
//             }
//         }
//     }
// }
//
// fn update_blinking_animation(
//     time: Res<Time>,
//     mut human_query: Query<&mut Human>,
// ) {
//     for mut human in human_query.iter_mut() {
//         human.blink_timer.tick(time.delta());
//
//         if human.blink_timer.just_finished() {
//             // Start blinking
//             human.eye_state = EyeState::Blinking;
//             human.blink_timer.set_duration(Duration::from_millis(150));
//             human.blink_timer.reset();
//         } else if human.eye_state == EyeState::Blinking &&
//             human.blink_timer.elapsed() > Duration::from_millis(100) {
//             // End blinking
//             human.eye_state = EyeState::Open;
//             human.blink_timer.set_duration(Duration::from_secs(3));
//             human.blink_timer.reset();
//         }
//     }
// }
//
// fn update_mouth_sprites(
//     animation_assets: Res<AnimationAssets>,
//     human_query: Query<&Human>,
//     mut mouth_query: Query<&mut Handle<Image>, With<MouthSprite>>,
// ) {
//     for human in human_query.iter() {
//         for mut mouth_texture in mouth_query.iter_mut() {
//             if let Some(texture) = animation_assets.mouth_textures.get(&human.mouth_state) {
//                 *mouth_texture = texture.clone();
//             }
//         }
//     }
// }
//
// fn update_eye_sprites(
//     animation_assets: Res<AnimationAssets>,
//     human_query: Query<&Human>,
//     mut eye_query: Query<&mut Handle<Image>, With<EyeSprite>>,
// ) {
//     for human in human_query.iter() {
//         for mut eye_texture in eye_query.iter_mut() {
//             if let Some(texture) = animation_assets.eye_textures.get(&human.eye_state) {
//                 *eye_texture = texture.clone();
//             }
//         }
//     }
// }
//
// // Additional systems for advanced animations
// fn create_facial_expressions(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     // Create different facial expressions for different contexts
//     let expressions = vec![
//         ("happy", "mouth_smile.png"),
//         ("neutral", "mouth_closed.png"),
//         ("speaking", "mouth_open.png"),
//         ("thinking", "mouth_small.png"),
//     ];
//
//     for (name, texture_path) in expressions {
//         info!("Loaded expression: {} from {}", name, texture_path);
//     }
// }
//
// fn animate_head_movement(
//     time: Res<Time>,
//     mut head_query: Query<&mut Transform, With<HeadSprite>>,
//     human_query: Query<&Human>,
// ) {
//     for human in human_query.iter() {
//         for mut transform in head_query.iter_mut() {
//             if human.is_talking {
//                 // Subtle head movement while talking
//                 let time_factor = time.elapsed_seconds() * 2.0;
//                 let offset_x = (time_factor.sin() * 2.0).clamp(-1.0, 1.0);
//                 let offset_y = (time_factor.cos() * 1.0).clamp(-0.5, 0.5);
//
//                 transform.translation.x = offset_x;
//                 transform.translation.y = 50.0 + offset_y;
//             } else {
//                 // Return to neutral position
//                 transform.translation.x = 0.0;
//                 transform.translation.y = 50.0;
//             }
//         }
//     }
// }
//
// // Integration with automotive AI system
// fn integrate_with_ai_system(
//     mut talking_state: ResMut<TalkingState>,
//     mut human_query: Query<&mut Human>,
// ) {
//     // This function would integrate with the automotive AI responses
//     // When the AI generates a response, trigger talking animation
//
//     let ai_responses = vec![
//         "Mobil hybrid menggunakan kombinasi mesin bensin dan motor listrik",
//         "GADAI kendaraan adalah layanan Pegadaian untuk pinjaman dengan jaminan kendaraan",
//         "Toyota Hybrid Synergy Drive menggabungkan efisiensi dan performa",
//         "Flowchart proses kredit meliputi aplikasi, verifikasi, dan persetujuan",
//     ];
//
//     // Simulate AI response trigger
//     if !talking_state.is_speaking {
//         // Could be triggered by external AI system
//         for response in ai_responses.iter().take(1) {
//             talking_state.current_text = response.to_string();
//             info!("AI Response ready: {}", response);
//         }
//     }
// }
//
// // Audio integration for lip sync
// fn setup_audio_system(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     // Load audio files for different phonemes
//     let audio_files = vec![
//         "phoneme_a.ogg",
//         "phoneme_e.ogg",
//         "phoneme_i.ogg",
//         "phoneme_o.ogg",
//         "phoneme_u.ogg",
//     ];
//
//     for audio_file in audio_files {
//         let audio_handle = asset_server.load(audio_file);
//         info!("Loaded audio: {}", audio_file);
//     }
// }
//
// // Text-to-speech integration
// fn text_to_speech_system(
//     talking_state: Res<TalkingState>,
//     mut human_query: Query<&mut Human>,
// ) {
//     // This would integrate with TTS system to generate audio
//     // and synchronize mouth movements with speech
//
//     if talking_state.is_speaking {
//         for mut human in human_query.iter_mut() {
//             // Analyze text for phonemes and adjust mouth shapes accordingly
//             let text = &talking_state.current_text;
//
//             // Simple phoneme detection
//             if text.contains("a") || text.contains("A") {
//                 human.mouth_state = MouthState::Open;
//             } else if text.contains("i") || text.contains("I") {
//                 human.mouth_state = MouthState::Speaking;
//             } else {
//                 human.mouth_state = MouthState::Closed;
//             }
//         }
//     }
// }
//
// // Export animation data for other systems
// #[derive(Debug)]
// pub struct AnimationFrame {
//     pub mouth_state: MouthState,
//     pub eye_state: EyeState,
//     pub head_position: Vec3,
//     pub timestamp: f32,
// }
//
// impl AnimationFrame {
//     pub fn new(mouth: MouthState, eyes: EyeState, head_pos: Vec3, time: f32) -> Self {
//         Self {
//             mouth_state: mouth,
//             eye_state: eyes,
//             head_position: head_pos,
//             timestamp: time,
//         }
//     }
// }
//
// // Animation recording system
// fn record_animation_frames(
//     time: Res<Time>,
//     human_query: Query<&Human>,
//     head_query: Query<&Transform, With<HeadSprite>>,
// ) {
//     for human in human_query.iter() {
//         for transform in head_query.iter() {
//             let frame = AnimationFrame::new(
//                 human.mouth_state,
//                 human.eye_state,
//                 transform.translation,
//                 time.elapsed_seconds(),
//             );
//
//             // Store frame data for analysis or export
//             debug!("Animation frame: {:?}", frame);
//         }
//     }
// }

//https://github.com/bevyengine/bevy-assets/tree/main
fn main() {}