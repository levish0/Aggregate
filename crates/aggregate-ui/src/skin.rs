//! Texture surfaces and proportional borders, separate from layout and interaction.
use crate::button::{ButtonTone, UiButton};
use bevy::{
    prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, ui::widget::NodeImageMode,
};

#[derive(Component, Clone, Copy)]
pub enum PanelSkin {
    Panel,
    Tooltip,
}

#[derive(Component)]
pub struct TextureFrame {
    pub path: &'static str,
    pub border: Vec2,
    pub corner_scale: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SurfaceMaterial {
    /// Kind, selected/disabled atlas frame, hover and press.
    #[uniform(0)]
    pub parameters: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub background: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub detail: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub shading: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    pub pattern: Handle<Image>,
}
impl UiMaterial for SurfaceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ui_surface.wgsl".into()
    }
}

#[derive(Component)]
struct ButtonBevel;

fn sliced(image: Handle<Image>, border: Vec2, scale: f32) -> ImageNode {
    ImageNode {
        image,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: border,
                max_inset: border,
            },
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: scale,
        }),
        ..default()
    }
}

fn layer(commands: &mut Commands, parent: Entity, image: ImageNode) -> Entity {
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            image,
            bevy::ui::FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

pub fn apply_surfaces(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
    panels: Query<(Entity, &PanelSkin), Added<PanelSkin>>,
    buttons: Query<(Entity, &UiButton), Added<UiButton>>,
    frames: Query<(Entity, &TextureFrame), Added<TextureFrame>>,
) {
    for (entity, skin) in &panels {
        let tooltip = matches!(skin, PanelSkin::Tooltip);
        commands
            .entity(entity)
            .remove::<BackgroundGradient>()
            .insert((
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                MaterialNode(materials.add(SurfaceMaterial {
                    parameters: Vec4::ZERO,
                    background: server.load(if tooltip {
                        "gfx/interface/tooltip/tooltip_bg.dds"
                    } else {
                        "gfx/interface/backgrounds/default_bg.dds"
                    }),
                    detail: server.load("gfx/interface/textures/velvet_texture.dds"),
                    shading: server.load("gfx/interface/backgrounds/default_bg_shading.dds"),
                    pattern: server.load("gfx/interface/backgrounds/bg_tiling_pattern.dds"),
                })),
            ));
        layer(
            &mut commands,
            entity,
            sliced(
                server.load(if tooltip {
                    "gfx/interface/tooltip/tooltip_frame.dds"
                } else {
                    "gfx/interface/backgrounds/simple_frame.dds"
                }),
                Vec2::splat(12.),
                0.5,
            ),
        );
    }
    for (entity, button) in &buttons {
        commands
            .entity(entity)
            .insert(MaterialNode(materials.add(SurfaceMaterial {
                parameters: Vec4::new(1., 0., 0., 0.),
                background: server.load("gfx/interface/buttons/default_button_bg.dds"),
                detail: server.load("gfx/interface/buttons/default_button_texture.dds"),
                shading: server.load("gfx/interface/buttons/default_button_bg_gradient.dds"),
                pattern: server.load("gfx/interface/buttons/default_button_wood_border.dds"),
            })));
        let bevel = layer(
            &mut commands,
            entity,
            sliced(
                server.load("gfx/interface/buttons/default_button_bevel.dds"),
                Vec2::new(150., 70.),
                0.5,
            ),
        );
        commands.entity(bevel).insert(ButtonBevel);
        if button.tone == ButtonTone::Primary {
            layer(
                &mut commands,
                entity,
                sliced(
                    server.load("gfx/interface/buttons/default_button_frame_fancy_small.dds"),
                    Vec2::new(100., 40.),
                    0.5,
                ),
            );
        }
    }
    for (entity, frame) in &frames {
        commands.entity(entity).insert(sliced(
            server.load(frame.path),
            frame.border,
            frame.corner_scale,
        ));
    }
}

pub fn animate_surfaces(
    buttons: Query<(
        &UiButton,
        &crate::button::ButtonMotion,
        &MaterialNode<SurfaceMaterial>,
    )>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    for (button, motion, handle) in &buttons {
        if let Some(mut material) = materials.get_mut(handle) {
            material.parameters = Vec4::new(
                1.,
                if !button.enabled {
                    2.
                } else if button.selected {
                    1.
                } else {
                    0.
                },
                motion.hover_amount(),
                motion.press_amount(),
            );
        }
    }
}
