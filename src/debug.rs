use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(WorldInspectorPlugin::new());
        }
        
        #[cfg(target_arch = "wasm32")]
        {
        app.add_plugin(bevy_web_resizer::Plugin);
        }
    }
}
