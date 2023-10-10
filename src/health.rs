use bevy::prelude::*;

// The player's health, dead if equal to 0
#[derive(Resource, Deref, DerefMut)]
pub struct Health(pub usize);

// Marker for the ui text bundle
#[derive(Component)]
pub struct HealthDisplay;

#[derive(Bundle)]
pub struct HealthDisplayBundle {
    text: TextBundle,
    name: Name,
    marker: HealthDisplay, // Used to uniquely identify the display bundle
}

impl HealthDisplayBundle {
    pub fn new<L: Into<String>, V: Into<String>>(
        font_size: f32,
        label_color: Color,
        score_color: Color,
        label: L,
        top_left_placement: Vec2,
        initial_value: Option<V>,
    ) -> Self {
        HealthDisplayBundle {
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
            name: Name::new("Health"),
            marker: HealthDisplay,
        }
    }
}
