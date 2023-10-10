use bevy::prelude::*;

// Causes the associated entity to blink (if it also has a Visibility component)
#[derive(Component)]
pub struct Blinking(pub Timer);

pub fn blink(
    mut commands: Commands,
    mut blinker_q: Query<(Entity, &mut Blinking, &mut Visibility)>,
    time: Res<Time>,
) {
    for (blinker_ent, mut blink_time, mut visible) in blinker_q.iter_mut() {
        blink_time.0.tick(time.delta()); // Advance the blinker

        // Blink 5 times during the lifetime of the blinker

        // Mod by 1/5 to focus on fifths of a second, then rescale
        // so that 0 to 50 is half the time, 50 to 100 is the other half
        let r = ((blink_time.0.percent_left()) % (1. / 5.)) * 100. * 5.;
        if r <= 50.0 {
            *visible = Visibility::Inherited;
        } else {
            *visible = Visibility::Hidden;
        }

        // Delete the Blinking component if the contained timer finishes, reset visibility
        if blink_time.0.finished() {
            commands.entity(blinker_ent).remove::<Blinking>();
            *visible = Visibility::Inherited;
        }
    }
}
