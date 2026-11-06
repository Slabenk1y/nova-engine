//! nova-world: Scene graph, entity components, spawn points, places, and versioned serialization.

use chrono::{DateTime, Utc};
use nova_core::Id;
use nova_math::{Transform, Vec3};
use nova_physics::{BodyType, RigidBody};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Material definition for rendering and visual appearance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Material {
    pub name: String,
    pub base_color_rgba: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission_rgba: [f32; 4],
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            base_color_rgba: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission_rgba: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Mesh definition (procedural or asset reference).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeshComponent {
    Cube { half_extents: Vec3 },
    Sphere { radius: f32 },
    Cylinder { radius: f32, height: f32 },
    AssetRef { asset_id: Id, path: String },
}

/// Light source component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LightComponent {
    Directional { color_rgb: [f32; 3], intensity: f32 },
    Point { color_rgb: [f32; 3], intensity: f32, radius: f32 },
}

/// Spawn point definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnPoint {
    pub position: Vec3,
    pub yaw_degrees: f32,
    pub team_id: Option<u32>,
}

/// An entity in a 3D Place.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: Id,
    pub name: String,
    pub parent_id: Option<Id>,
    pub transform: Transform,
    pub mesh: Option<MeshComponent>,
    pub material: Option<Material>,
    pub light: Option<LightComponent>,
    pub rigid_body: Option<RigidBody>,
    pub script_ids: Vec<Id>,
    pub tags: Vec<String>,
}

impl Entity {
    pub fn new(name: impl Into<String>, position: Vec3) -> Self {
        Self {
            id: Id::new(),
            name: name.into(),
            parent_id: None,
            transform: Transform::from_translation(position),
            mesh: None,
            material: None,
            light: None,
            rigid_body: None,
            script_ids: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_box_collider(mut self, half_extents: Vec3, body_type: BodyType) -> Self {
        self.rigid_body = Some(RigidBody::new_box(self.id, body_type, self.transform.position, half_extents));
        self.mesh = Some(MeshComponent::Cube { half_extents });
        self
    }
}

/// Environment settings for a place.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentSettings {
    pub ambient_light: [f32; 3],
    pub fog_color: [f32; 3],
    pub fog_density: f32,
    pub skybox_preset: String,
    pub gravity: Vec3,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            ambient_light: [0.2, 0.2, 0.25],
            fog_color: [0.7, 0.8, 0.9],
            fog_density: 0.001,
            skybox_preset: "clear_day".to_string(),
            gravity: Vec3::new(0.0, -19.6, 0.0),
        }
    }
}

/// Place: A playable 3D world belonging to an Experience.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Place {
    pub id: Id,
    pub experience_id: Id,
    pub name: String,
    pub version: u32,
    pub entities: HashMap<Id, Entity>,
    pub spawn_points: Vec<SpawnPoint>,
    pub environment: EnvironmentSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Place {
    pub fn new(name: impl Into<String>, experience_id: Id) -> Self {
        Self {
            id: Id::new(),
            experience_id,
            name: name.into(),
            version: 1,
            entities: HashMap::new(),
            spawn_points: vec![SpawnPoint {
                position: Vec3::new(0.0, 1.0, 0.0),
                yaw_degrees: 0.0,
                team_id: None,
            }],
            environment: EnvironmentSettings::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> Id {
        let id = entity.id;
        self.entities.insert(id, entity);
        self.updated_at = Utc::now();
        id
    }

    pub fn remove_entity(&mut self, id: &Id) -> Option<Entity> {
        let removed = self.entities.remove(id);
        if removed.is_some() {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Serialize place to versioned JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize place from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Multi-place Experience configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperienceManifest {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub creator_user_id: Id,
    pub starting_place_id: Id,
    pub place_ids: Vec<Id>,
    pub max_players_per_server: u32,
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_creation_and_serialization() {
        let exp_id = Id::new();
        let mut place = Place::new("Obby World", exp_id);

        let entity = Entity::new("Platform_1", Vec3::new(0.0, 0.0, 0.0))
            .with_box_collider(Vec3::new(5.0, 0.5, 5.0), BodyType::Static);
        let entity_id = place.add_entity(entity);

        assert_eq!(place.entities.len(), 1);
        assert!(place.entities.contains_key(&entity_id));

        let json = place.to_json().expect("Serialization failed");
        let decoded = Place::from_json(&json).expect("Deserialization failed");
        assert_eq!(decoded.name, "Obby World");
        assert_eq!(decoded.entities.len(), 1);
        assert_eq!(decoded.entities.get(&entity_id).unwrap().name, "Platform_1");
    }
}
