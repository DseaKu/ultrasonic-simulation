use bevy::prelude::*;
use bevy::camera::Viewport;

#[derive(Component)]
pub struct SimulationCamera;

#[derive(Component)]
pub struct PlotCamera;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, update_camera_viewports);
    }
}

fn setup_camera(mut commands: Commands) {
    // Upper pane camera (Simulation)
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0), // Look at simulation
        SimulationCamera,
    ));

    // Lower pane camera (Plot)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        Transform::from_xyz(0.0, -5000.0, 0.0), // Look at plot
        PlotCamera,
    ));
}

fn update_camera_viewports(
    windows: Query<&Window>,
    mut sim_cam: Query<&mut Camera, (With<SimulationCamera>, Without<PlotCamera>)>,
    mut plot_cam: Query<&mut Camera, With<PlotCamera>>,
) {
    let Ok(window) = windows.single() else { return };
    
    let physical_width = window.physical_width();
    let physical_height = window.physical_height();
    
    if physical_width == 0 || physical_height == 0 {
        return;
    }
    
    let half_height = physical_height / 2;
    
    let sim_pos = UVec2::new(0, 0);
    let sim_size = UVec2::new(physical_width, half_height);

    if let Ok(mut cam) = sim_cam.single_mut() {
        let needs_update = match &cam.viewport {
            Some(v) => v.physical_position != sim_pos || v.physical_size != sim_size,
            None => true,
        };
        if needs_update {
            cam.viewport = Some(Viewport {
                physical_position: sim_pos,
                physical_size: sim_size,
                ..default()
            });
        }
    }
    
    // Lower Pane Viewport
    let plot_pos = UVec2::new(0, half_height);
    let plot_size = UVec2::new(physical_width, physical_height - half_height);

    if let Ok(mut cam) = plot_cam.single_mut() {
        let needs_update = match &cam.viewport {
            Some(v) => v.physical_position != plot_pos || v.physical_size != plot_size,
            None => true,
        };
        if needs_update {
            cam.viewport = Some(Viewport {
                physical_position: plot_pos,
                physical_size: plot_size,
                ..default()
            });
        }
    }
}
