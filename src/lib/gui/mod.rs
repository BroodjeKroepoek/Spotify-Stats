use belly::prelude::*;
use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(eml! {
        <body s:padding="500px">
            "Hello, "<strong>"world"</strong>"!"
        </body>
    });
}
