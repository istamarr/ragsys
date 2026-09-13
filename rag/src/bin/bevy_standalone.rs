// bevy_standalone.rs - Standalone Bevy animation without Burn dependencies
use bevy::prelude::*;
use bevy::window::WindowResolution;
use std::time::Duration;

#[derive(Component)]
struct Human {
    is_talking: bool,
    talk_timer: Timer,
    mouth_state: MouthState,
    blink_timer: Timer,
    eye_state: EyeState,
}

#[derive(Component)] struct MouthSprite;
#[derive(Component)] struct EyeSprite;
#[derive(Component)] struct HeadSprite;
#[derive(Component)] struct BodySprite;

#[derive(Clone, Copy, PartialEq)]
enum MouthState { Closed, Open, Smile, Speaking }

#[derive(Clone, Copy, PartialEq)]
enum EyeState { Open, Closed, Blinking }

#[derive(Resource)]
struct TalkingState {
    current_text: String,
    is_speaking: bool,
    speech_duration: Timer,
}

impl Default for TalkingState {
    fn default() -> Self {
        Self {
            current_text: "Welcome to AIMS Automotive AI System".to_string(),
            is_speaking: false,
            speech_duration: Timer::new(Duration::from_secs(3), TimerMode::Once),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AIMS Automotive AI - Human Avatar".to_string(),
                resolution: WindowResolution::new(800.0, 600.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<TalkingState>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            handle_input,
            update_talking_animation,
            update_blinking_animation,
            animate_head_movement,
            display_ui,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut talking_state: ResMut<TalkingState>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.96, 0.96, 0.86),
                custom_size: Some(Vec2::new(100.0, 150.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Human {
            is_talking: false,
            talk_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
            mouth_state: MouthState::Closed,
            blink_timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
            eye_state: EyeState::Open,
        },
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(120.0, 200.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -100.0, 1.0)),
            ..default()
        },
        BodySprite,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.75, 0.80),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 2.0)),
            ..default()
        },
        HeadSprite,
    ));

    for x in [-15.0f32, 15.0] {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, 60.0, 3.0)),
                ..default()
            },
            EyeSprite,
        ));
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(20.0, 8.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 35.0, 3.0)),
            ..default()
        },
        MouthSprite,
    ));

    commands.spawn(
        TextBundle::from_section(
            "AIMS Automotive AI Assistant\nPress SPACE to talk, T to change text, R to reset",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );

    talking_state.current_text = "Selamat datang di AIMS Automotive AI System".to_string();
    info!("Avatar ready. SPACE=talk  T=text  R=reset");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut talking_state: ResMut<TalkingState>,
    mut human_query: Query<&mut Human>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        talking_state.is_speaking = !talking_state.is_speaking;
        talking_state.speech_duration.reset();
        for mut human in human_query.iter_mut() {
            human.is_talking = talking_state.is_speaking;
            if human.is_talking {
                human.talk_timer.reset();
                human.mouth_state = MouthState::Speaking;
            } else {
                human.mouth_state = MouthState::Closed;
            }
        }
        info!("{}", if talking_state.is_speaking { "Started talking" } else { "Stopped talking" });
    }

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        let texts = [
            "Apa itu mobil hybrid?",
            "Bagaimana cara kerja GADAI kendaraan Pegadaian?",
            "Jelaskan teknologi Toyota Hybrid Synergy Drive",
            "Buatkan flowchart proses kredit mobil",
            "What are the benefits of electric vehicles?",
            "How does automotive financing work?",
        ];
        let idx = texts.iter().position(|&x| x == talking_state.current_text).unwrap_or(0);
        talking_state.current_text = texts[(idx + 1) % texts.len()].to_string();
        info!("Text: {}", talking_state.current_text);
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        talking_state.is_speaking = false;
        for mut human in human_query.iter_mut() {
            human.is_talking = false;
            human.mouth_state = MouthState::Closed;
            human.eye_state = EyeState::Open;
            human.talk_timer.reset();
            human.blink_timer.reset();
        }
        info!("Animation reset");
    }
}

fn update_talking_animation(
    time: Res<Time>,
    mut talking_state: ResMut<TalkingState>,
    mut human_query: Query<&mut Human>,
    mut mouth_query: Query<&mut Sprite, With<MouthSprite>>,
) {
    talking_state.speech_duration.tick(time.delta());
    for mut human in human_query.iter_mut() {
        if human.is_talking {
            human.talk_timer.tick(time.delta());
            if human.talk_timer.just_finished() {
                human.mouth_state = match human.mouth_state {
                    MouthState::Closed   => MouthState::Open,
                    MouthState::Open     => MouthState::Speaking,
                    MouthState::Speaking => MouthState::Closed,
                    MouthState::Smile    => MouthState::Open,
                };
            }
            for mut sprite in mouth_query.iter_mut() {
                match human.mouth_state {
                    MouthState::Closed => {
                        sprite.custom_size = Some(Vec2::new(20.0, 4.0));
                        sprite.color = Color::rgb(1.0, 0.0, 0.0);
                    }
                    MouthState::Open => {
                        sprite.custom_size = Some(Vec2::new(25.0, 15.0));
                        sprite.color = Color::rgb(0.5, 0.0, 0.0);
                    }
                    MouthState::Speaking => {
                        sprite.custom_size = Some(Vec2::new(22.0, 12.0));
                        sprite.color = Color::rgb(1.0, 0.27, 0.0);
                    }
                    MouthState::Smile => {
                        sprite.custom_size = Some(Vec2::new(30.0, 8.0));
                        sprite.color = Color::rgb(1.0, 0.75, 0.80);
                    }
                }
            }
            if talking_state.speech_duration.just_finished() {
                human.is_talking = false;
                human.mouth_state = MouthState::Smile;
                talking_state.is_speaking = false;
                info!("Finished talking");
            }
        }
    }
}

fn update_blinking_animation(
    time: Res<Time>,
    mut human_query: Query<&mut Human>,
    mut eye_query: Query<&mut Sprite, With<EyeSprite>>,
) {
    for mut human in human_query.iter_mut() {
        human.blink_timer.tick(time.delta());
        if human.blink_timer.just_finished() {
            human.eye_state = EyeState::Blinking;
            human.blink_timer.set_duration(Duration::from_millis(150));
            human.blink_timer.reset();
        } else if human.eye_state == EyeState::Blinking
            && human.blink_timer.elapsed() > Duration::from_millis(100)
        {
            human.eye_state = EyeState::Open;
            human.blink_timer.set_duration(Duration::from_secs(3));
            human.blink_timer.reset();
        }
        for mut sprite in eye_query.iter_mut() {
            match human.eye_state {
                EyeState::Open => {
                    sprite.custom_size = Some(Vec2::new(15.0, 15.0));
                    sprite.color = Color::BLACK;
                }
                EyeState::Closed | EyeState::Blinking => {
                    sprite.custom_size = Some(Vec2::new(15.0, 3.0));
                    sprite.color = Color::rgb(0.5, 0.5, 0.5);
                }
            }
        }
    }
}

fn animate_head_movement(
    time: Res<Time>,
    mut head_query: Query<&mut Transform, With<HeadSprite>>,
    human_query: Query<&Human>,
) {
    for human in human_query.iter() {
        for mut transform in head_query.iter_mut() {
            if human.is_talking {
                // let t = time.elapsed_secs() * 2.0;
                let t = time.elapsed_seconds() * 2.0;
                transform.translation.x = (t.sin() * 2.0).clamp(-1.0, 1.0);
                transform.translation.y = 50.0 + (t.cos() * 1.0).clamp(-0.5, 0.5);
            } else {
                transform.translation.x = 0.0;
                transform.translation.y = 50.0;
            }
        }
    }
}

fn display_ui(
    talking_state: Res<TalkingState>,
    mut text_query: Query<&mut Text>,
) {
    for mut text in text_query.iter_mut() {
        let status = if talking_state.is_speaking { "SPEAKING" } else { "READY" };
        text.sections[0].value = format!(
            "AIMS Automotive AI Assistant [{}]\nCurrent: {}\nSPACE=talk  T=text  R=reset",
            status, talking_state.current_text
        );
    }
}
