#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

use basic_bevy_pixel_camera::prelude::*;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam 05".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3),
                    },
                    ..default()
                }),
        );

        // Add other plugins.
        app.add_plugins((
            game::plugin,
            screen::plugin,
            ui::plugin,
            BasicPixelCameraPlugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

pub const PIXELS_PER_UNIT: f32 = 32.0;
pub const HIGH_RES_LAYER: RenderLayers = RenderLayers::layer(0);
pub const PIXEL_PERFECT_LAYER: RenderLayers = RenderLayers::layer(1);

pub const UNIT_HEIGHT: f32 = 16.0;
pub const UNIT_WIDTH: f32 = 8.0;

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands, images: ResMut<Assets<Image>>) {
    // Normal camera setup
    let mut cam_2d = Camera2dBundle::default();
    cam_2d.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: UNIT_HEIGHT,
        min_height: UNIT_WIDTH,
    };
    let main_cam = commands
        .spawn((
            Name::new("Camera"),
            cam_2d,
            HIGH_RES_LAYER,
            MainCamera,
            IsDefaultUiCamera,
        ))
        .id();

    // Pixel camera setup
    let image = create_pixel_image(images);

    let pixel_camera = create_pixel_camera(commands.reborrow(), image.clone(), PIXEL_PERFECT_LAYER);

    let pixel_canvas = create_pixel_canvas(
        &PixelCanvasConfig::new(PIXELS_PER_UNIT, UNIT_HEIGHT, UNIT_WIDTH),
        commands.reborrow(),
        image,
        pixel_camera,
        HIGH_RES_LAYER,
    );

    commands.entity(pixel_camera).insert(PixelCameraSnapping);
    commands.entity(pixel_canvas).insert(PixelCanvasSmoothing);
    commands
        .entity(main_cam)
        .push_children(&[pixel_camera, pixel_canvas]);
}
