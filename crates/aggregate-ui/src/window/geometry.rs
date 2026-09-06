use bevy::{prelude::*, window::SystemCursorIcon};

#[derive(Clone, Copy, Debug)]
pub(super) struct WindowGeometry {
    pub position: Vec2,
    pub size: Vec2,
}

/// -1 moves the leading edge, 1 the trailing edge, 0 leaves that axis fixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResizeEdges(pub IVec2);

impl ResizeEdges {
    pub fn at(pointer: Vec2, geometry: WindowGeometry) -> Option<Self> {
        let local = pointer - geometry.position;
        let axis = |position: f32, extent: f32| {
            if position <= 6. {
                -1
            } else if position >= extent - 6. {
                1
            } else {
                0
            }
        };
        let edges = IVec2::new(
            axis(local.x, geometry.size.x),
            axis(local.y, geometry.size.y),
        );
        (edges != IVec2::ZERO).then_some(Self(edges))
    }

    pub fn cursor(self) -> SystemCursorIcon {
        match (self.0.x, self.0.y) {
            (0, _) => SystemCursorIcon::NsResize,
            (_, 0) => SystemCursorIcon::EwResize,
            (x, y) if x == y => SystemCursorIcon::NwseResize,
            _ => SystemCursorIcon::NeswResize,
        }
    }

    pub fn resize(
        self,
        mut geometry: WindowGeometry,
        delta: Vec2,
        minimum: Vec2,
        viewport: Vec2,
    ) -> WindowGeometry {
        for axis in 0..2 {
            let start = geometry.position[axis];
            let end = start + geometry.size[axis];
            let minimum = minimum[axis].min(viewport[axis]);
            match self.0[axis] {
                -1 => {
                    let leading = (start + delta[axis]).clamp(0., (end - minimum).max(0.));
                    geometry.position[axis] = leading;
                    geometry.size[axis] = end - leading;
                }
                1 => {
                    geometry.size[axis] = (geometry.size[axis] + delta[axis])
                        .clamp(minimum, (viewport[axis] - start).max(minimum))
                }
                _ => {}
            }
        }
        geometry
    }
}

pub(super) fn apply_geometry(node: &mut Node, geometry: WindowGeometry) {
    node.left = px(geometry.position.x);
    node.top = px(geometry.position.y);
    node.right = Val::Auto;
    node.bottom = Val::Auto;
    node.width = px(geometry.size.x);
    node.height = px(geometry.size.y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizing_leading_corner_keeps_opposite_corner_and_minimum_size() {
        let original = WindowGeometry {
            position: Vec2::new(100., 120.),
            size: Vec2::new(400., 300.),
        };
        let resized = ResizeEdges(IVec2::splat(-1)).resize(
            original,
            Vec2::splat(500.),
            Vec2::new(200., 150.),
            Vec2::new(1000., 800.),
        );
        assert_eq!(resized.size, Vec2::new(200., 150.));
        assert_eq!(
            resized.position + resized.size,
            original.position + original.size
        );
    }

    #[test]
    fn resizing_right_edge_keeps_height_and_stays_inside_viewport() {
        let original = WindowGeometry {
            position: Vec2::new(100., 120.),
            size: Vec2::new(400., 300.),
        };
        let resized = ResizeEdges(IVec2::X).resize(
            original,
            Vec2::splat(1000.),
            Vec2::new(200., 150.),
            Vec2::new(1000., 800.),
        );
        assert_eq!(resized.position, original.position);
        assert_eq!(resized.size, Vec2::new(900., 300.));
    }
}
