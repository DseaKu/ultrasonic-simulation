use bevy::prelude::*;

#[derive(Component)]
pub struct UiButton;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(10.0),
        ..default()
    }).with_children(|p| {
        p.spawn((
            Button,
            Node {
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
        )).with_child((
            Text::new("+"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::BLACK),
        ));
    });
}
