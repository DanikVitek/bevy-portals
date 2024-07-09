use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub fn setup(
    main_camera: In<Entity>,
    mut commands: Commands,
    mut ui_materials: ResMut<Assets<CrosshairMaterial>>,
) {
    commands
        .spawn((
            Name::new("Crosshair"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            TargetCamera(main_camera.0),
        ))
        .with_children(|child| {
            child.spawn((MaterialNodeBundle {
                material: ui_materials.add(CrosshairMaterial {
                    color: Color::WHITE.rgba_to_vec4(),
                }),
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(18.),
                    height: Val::Px(18.),
                    ..Default::default()
                },
                ..Default::default()
            },));
        });
}

#[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
pub struct CrosshairMaterial {
    #[uniform(0)]
    color: Vec4,
}

impl UiMaterial for CrosshairMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/crosshair.wgsl".into()
    }
}
