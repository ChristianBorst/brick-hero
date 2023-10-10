use bevy::prelude::*;

#[derive(Resource)]
pub struct Scoreboard {
    pub score: usize,
}

// Marker for the ui text bundle
#[derive(Component)]
pub struct ScoreDisplay;

#[derive(Bundle)]
pub struct ScoreboardBundle {
    text: TextBundle,
    name: Name,
    marker: ScoreDisplay, // Used to uniquely identify the display bundle
}

impl ScoreboardBundle {
    pub fn new<L: Into<String>, V: Into<String>>(
        font_size: f32,
        label_color: Color,
        score_color: Color,
        label: L,
        top_left_placement: Vec2,
        initial_value: Option<V>,
    ) -> Self {
        ScoreboardBundle {
            text: TextBundle::from_sections([
                // Labels the score
                TextSection::new(
                    label,
                    TextStyle {
                        font_size: font_size,
                        color: label_color,
                        ..default()
                    },
                ),
                // The score value
                TextSection::new(
                    match initial_value {
                        Some(s) => s.into(),
                        None => "".to_string(),
                    },
                    TextStyle {
                        font_size: font_size,
                        color: score_color,
                        ..default()
                    },
                ),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(top_left_placement.y),
                left: Val::Px(top_left_placement.x),
                ..default()
            }),
            name: Name::new("Scoreboard"),
            marker: ScoreDisplay,
        }
    }
}

pub fn update_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut text_q: Query<&mut Text, With<ScoreDisplay>>,
) {
    // text_q holds the setup values put in the TextBundle
    let mut text = text_q.single_mut();
    // Update the empty section given the SCORE_COLOR
    text.sections[1].value = scoreboard.score.to_string();
}
