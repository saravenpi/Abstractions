//! A plugin wrapper that keeps a Solari plugin inert on a device that cannot run it.

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::renderer::{RenderAdapter, RenderDevice};

use super::gpu::{Probe, Support, check};

/// Wraps a Solari plugin and runs its `finish` only when the render device passes [`check`].
///
/// Solari's own `finish` tests the wgpu features only, so a device with the features but too small binding
/// array limits would still register the render systems and fail validation at startup.
pub struct Guarded<P>(pub P);

/// Returns whether the render device of `app` can run Solari.
fn usable(app: &App) -> bool {
    let Some(render) = app.get_sub_app(RenderApp) else {
        return false;
    };
    let world = render.world();
    let (Some(device), Some(adapter)) = (
        world.get_resource::<RenderDevice>(),
        world.get_resource::<RenderAdapter>(),
    ) else {
        return false;
    };
    check(&Probe::of(device, adapter)) == Support::Ready
}

impl<P: Plugin> Plugin for Guarded<P> {
    /// Builds the wrapped plugin as usual: it only loads shaders and sets the render method.
    fn build(&self, app: &mut App) {
        self.0.build(app);
    }

    /// Waits for what the wrapped plugin waits for.
    fn ready(&self, app: &App) -> bool {
        self.0.ready(app)
    }

    /// Finishes the wrapped plugin only on a device that passes the check.
    fn finish(&self, app: &mut App) {
        if usable(app) {
            self.0.finish(app);
        } else {
            info!(
                "{} is not finished: this device cannot run Solari",
                self.0.name()
            );
        }
    }

    /// Cleans up the wrapped plugin.
    fn cleanup(&self, app: &mut App) {
        self.0.cleanup(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Marks that a plugin finished.
    #[derive(Resource)]
    struct Finished;

    /// A plugin that records its `finish`.
    struct Recorder;

    impl Plugin for Recorder {
        /// Does nothing.
        fn build(&self, _app: &mut App) {}

        /// Records that it finished.
        fn finish(&self, app: &mut App) {
            app.insert_resource(Finished);
        }
    }

    /// Without a render device the wrapped plugin never finishes.
    #[test]
    fn a_missing_device_keeps_the_plugin_inert() {
        let mut app = App::new();
        app.add_plugins(Guarded(Recorder));
        app.finish();
        assert!(app.world().get_resource::<Finished>().is_none());
    }
}
