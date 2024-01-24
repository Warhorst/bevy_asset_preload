use bevy_app::prelude::*;
use bevy_asset::{LoadedFolder, LoadState, RecursiveDependencyLoadState};
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;

/// Plugin that starts loading all assets in the asset folder for a given state and
/// automatically switches to another given state when everything is loaded.
pub struct AssetPreloadPlugin<LoadingState: States, NextState: States> {
    /// The state the plugin will start and keep loading all assets.
    loading_state: LoadingState,
    /// The state the plugin will switch to when all assets are loaded
    next_state: NextState
}

impl<LoadingState: States, NextState: States> AssetPreloadPlugin<LoadingState, NextState> {
    pub fn new(loading_state: LoadingState, next_state: NextState) -> Self {
        Self { loading_state, next_state }
    }
}

/// Resource that holds handles to all assets in the assets folder. This only exists to ensure
/// the assets don't get unloaded because nobody is using them.
#[derive(Resource)]
struct LoadedAssets(Vec<UntypedHandle>);

/// Resource that holds the handle to the currently loading asset folder.
#[derive(Resource)]
struct LoadingAssetFolder(Handle<LoadedFolder>);

impl LoadingAssetFolder {
    /// Tells if the asset folder is loaded. Might panic if the load failed.
    fn is_loaded(&self, asset_server: &AssetServer) -> bool {
        match asset_server.recursive_dependency_load_state(&self.0) {
            RecursiveDependencyLoadState::Failed => panic!("some assets failed loading, abort"),
            RecursiveDependencyLoadState::Loaded => true,
            _ => false
        }
    }
}

impl<LoadingState: States, NextState: States> Plugin for AssetPreloadPlugin<LoadingState, NextState> {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(self.loading_state.clone()),
                start_asset_loading
            )
            .add_systems(
                Update,
                switch_state_when_all_loaded(self.next_state.clone()).run_if(in_state(self.loading_state.clone()))
            )
        ;
    }
}

/// Start loading the assets folder, and store the folder handle in a resource.
#[cfg(not(target_arch = "wasm32"))]
fn start_asset_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let folder_handle = asset_server.load_folder("./");
    commands.insert_resource(LoadingAssetFolder(folder_handle));
}

/// Create a system that will check if the assets folder was loaded. If true, the system will switch to the
/// provided state and deletes the folder handle. Every handle from the folder will be preserved in a LoadedAssets resource.
#[cfg(not(target_arch = "wasm32"))]
fn switch_state_when_all_loaded<S: States>(followup_state: S) -> impl Fn(Commands, Res<AssetServer>, Res<LoadingAssetFolder>, Res<Assets<LoadedFolder>>, ResMut<NextState<S>>) {
    move |mut commands, asset_server, loading_asset_folder, loaded_folders, mut next_state| {
        if !loading_asset_folder.is_loaded(&asset_server) {
            return
        }

        let folder = loaded_folders.get(loading_asset_folder.0.id()).expect("the folder should be loaded");

        let loaded_assets = LoadedAssets(
            folder.handles
                .iter()
                .map(Clone::clone)
                .collect()
        );

        commands.insert_resource(loaded_assets);
        commands.remove_resource::<LoadingAssetFolder>();

        next_state.set(followup_state.clone())
    }
}

#[cfg(target_arch = "wasm32")]
fn start_asset_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let loaded_assets = LoadedAssets(wasm_load::wasm_load!());
    commands.insert_resource(loaded_assets);
}

#[cfg(target_arch = "wasm32")]
fn switch_state_when_all_loaded<S: States>(followup_state: S) -> impl Fn(Commands, Res<AssetServer>, Res<LoadedAssets>, ResMut<NextState<S>>) {
    move |mut commands, asset_server, loaded_assets, mut next_state| {
        let all_loaded = loaded_assets.0
            .iter()
            .all(|uh| match asset_server.load_state(uh.id()) {
                LoadState::Loaded => true,
                LoadState::Failed => panic!("load failed!"),
                _ => false
            });

        if all_loaded {
            next_state.set(followup_state.clone())
        }
    }
}