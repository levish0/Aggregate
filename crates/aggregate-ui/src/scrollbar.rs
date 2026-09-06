//! Visible position and direct manipulation for every ScrollRegion.
use crate::{motion::MotionValue, scroll::ScrollRegion, theme};
use bevy::{prelude::*, ui::ComputedStackIndex};

#[derive(Component)]
pub struct ScrollbarTrack { region: Entity }
#[derive(Component)]
pub struct ScrollbarThumb { region: Entity }
#[derive(Resource, Default)]
pub struct ScrollbarInteraction { active: Option<(Entity, f32)> }

pub fn attach(mut commands: Commands, regions: Query<Entity, Added<ScrollRegion>>) {
    for region in &regions {
        let track = commands.spawn((Node { position_type: PositionType::Absolute, right: px(2), top: px(0), width: px(9), ..default() }, BackgroundColor(Color::WHITE.with_alpha(0.06)), ScrollbarTrack { region }, ZIndex(5))).id();
        let thumb = commands.spawn((Node { position_type: PositionType::Absolute, width: percent(100), min_height: px(22), border_radius: BorderRadius::all(px(4)), ..default() }, BackgroundColor(theme::MUTED.with_alpha(0.7)), ScrollbarThumb { region })).id();
        commands.entity(region).add_child(track);
        commands.entity(track).add_child(thumb);
    }
}

pub fn update(
    regions: Query<(&ComputedNode, &ScrollPosition), With<ScrollRegion>>,
    mut tracks: Query<(&ScrollbarTrack, &mut Node), Without<ScrollbarThumb>>,
    mut thumbs: Query<(&ScrollbarThumb, &mut Node), Without<ScrollbarTrack>>,
) {
    for (track, mut node) in &mut tracks {
        let Ok((computed, position)) = regions.get(track.region) else { continue; };
        let height = computed.size().y * computed.inverse_scale_factor();
        let content = computed.content_size().y * computed.inverse_scale_factor();
        node.display = if content > height + 1. { Display::Flex } else { Display::None };
        node.top = px(position.y);
        node.height = px(height);
    }
    for (thumb, mut node) in &mut thumbs {
        let Ok((computed, position)) = regions.get(thumb.region) else { continue; };
        let height = computed.size().y * computed.inverse_scale_factor();
        let content = computed.content_size().y * computed.inverse_scale_factor();
        let thumb_height = (height * height / content.max(1.)).clamp(22_f32.min(height), height.max(0.));
        node.height = px(thumb_height);
        node.top = px(position.y / (content - height).max(1.) * (height - thumb_height));
    }
}

pub fn interact(
    windows: Query<&Window>, mouse: Res<ButtonInput<MouseButton>>, mut interaction: ResMut<ScrollbarInteraction>,
    tracks: Query<(&ScrollbarTrack, &ComputedNode, &UiGlobalTransform, &ComputedStackIndex)>,
    thumbs: Query<(&ScrollbarThumb, &ComputedNode, &UiGlobalTransform)>,
    mut regions: Query<(&ComputedNode, &mut ScrollRegion, &mut ScrollPosition)>,
) {
    if !mouse.pressed(MouseButton::Left) { interaction.active = None; return; }
    let Some(cursor) = windows.single().ok().and_then(Window::physical_cursor_position) else { return; };
    if mouse.just_pressed(MouseButton::Left) {
        if let Some((track, _, _, _)) = tracks.iter().filter(|(_, node, transform, _)| node.size().min_element() > 0. && Rect::from_center_size(transform.translation, node.size()).contains(cursor)).max_by_key(|(_, _, _, stack)| stack.0) {
            let offset = thumbs.iter().find(|(thumb, _, _)| thumb.region == track.region).map(|(_, node, transform)| {
                let rect = Rect::from_center_size(transform.translation, node.size());
                if rect.contains(cursor) { cursor.y - rect.min.y } else { node.size().y / 2. }
            }).unwrap_or(0.);
            interaction.active = Some((track.region, offset));
        }
    }
    if let Some((entity, offset)) = interaction.active {
        let Some((_, track, transform, _)) = tracks.iter().find(|(track, _, _, _)| track.region == entity) else { interaction.active = None; return; };
        let Some((_, thumb, _)) = thumbs.iter().find(|(thumb, _, _)| thumb.region == entity) else { return; };
        let Ok((computed, mut region, mut position)) = regions.get_mut(entity) else { return; };
        let top = transform.translation.y - track.size().y / 2.;
        let fraction = ((cursor.y - top - offset) / (track.size().y - thumb.size().y).max(1.)).clamp(0.,1.);
        let maximum = ((computed.content_size().y - computed.size().y)*computed.inverse_scale_factor()).max(0.);
        region.target = fraction * maximum;
        region.motion = MotionValue::new(region.target);
        position.y = region.target;
    }
}
