use bevy::prelude::*;

#[derive(serde::Deserialize, Debug)]
pub struct GltfExtrasValue {
    #[serde(rename = "TLM_Lightmap")]
    pub tlm_lightmap: Option<String>,
}

#[derive(Component)]
pub struct LightmapInfo {
    pub lightmap_name: String,
    pub exposure: f32,
}

#[derive(Resource, Debug, Default)]
pub struct LightmapRegistry {
    pub map: bevy::utils::HashMap<String, String>,
}

pub fn setup(
    query: Query<(Entity, &Name, Option<&GltfExtras>)>,
    mut commands: Commands,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mut single_run: Local<bool>
){

    if *single_run {
        return;
    }

    //Setup lightmap registry
    for (entity, name, extras) in query.iter() {

        //println!("{:?}", name);

        let mut lightmap_found = false;
        let mut lightmap_name = String::new();

        // If there's any extras at all
        if let Some(extras) = extras {
            match serde_json::from_str::<GltfExtrasValue>(&extras.value) {
                Ok(parsed) => {
                    if let Some(lightmap) = parsed.tlm_lightmap {
                        lightmap_name = lightmap.to_string();
                        lightmap_found = true;
                        println!("Found LM...");
                    }
                }
                Err(err) => {
                    println!("Failed to parse extras for {}: {}", name.as_str(), err);
                }
            }
        }

        if lightmap_found {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok(_child_name) = name_query.get(*child) {
                        commands.entity(*child).insert(LightmapInfo {

                            lightmap_name: lightmap_name.clone(),
                            exposure: 1000.0, // Default exposure value
                            
                        });
                        println!("Insert LM...");
                    }
                }
            }
        }

    }

    println!("Applying single run");

    *single_run = true;

}



pub fn apply_lightmaps(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &LightmapInfo)>,
    mut lightmap_handles: Local<Vec<Handle<bevy::prelude::Image>>>,
    mut single_run: Local<bool>
) {

    if *single_run {
        return;
    }

    if lightmap_handles.is_empty() {

        println!("Applying lightmaps");

        // Start loading lightmaps
        for (entity, material, lightmap_info) in query.iter() {
            let lightmap_path = format!("lightmaps/{}.ktx2", lightmap_info.lightmap_name);

            if let Some(mat) = materials.get_mut(&material.0) {
                mat.lightmap_exposure = lightmap_info.exposure;
                mat.reflectance = 0.0;
            }

            let lightmap_handle = asset_server.load(lightmap_path);
            lightmap_handles.push(lightmap_handle.clone());

            commands.entity(entity).insert(bevy::pbr::Lightmap {
                image: lightmap_handle,
                ..default()
            });
        }

    } else {
        // Check if all handles are loaded
        let all_loaded = lightmap_handles.iter().all(|handle| {
            match asset_server.get_load_state(handle) {
                Some(bevy::asset::LoadState::Loaded) => true,
                Some(_) => false, // Handle is in a state other than Loaded
                None => false,    // Handle's load state is unknown
            }
        });

        if all_loaded {
            println!("Clearing lightmaps");
            lightmap_handles.clear(); // Clear the handles after loading is complete
            *single_run = true;
        }
    }

}
