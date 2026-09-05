use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct UiFonts {
    pub regular: Handle<Font>,
    pub semibold: Handle<Font>,
}

/// Compile-time bundled fonts use Bevy's Font assets directly, with no cwd dependency.
pub fn load_fonts(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    commands.insert_resource(UiFonts {
        regular: fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/Pretendard-Regular.ttf").to_vec(),
        )),
        semibold: fonts.add(Font::from_bytes(
            include_bytes!("../assets/fonts/Pretendard-SemiBold.ttf").to_vec(),
        )),
    });
}
