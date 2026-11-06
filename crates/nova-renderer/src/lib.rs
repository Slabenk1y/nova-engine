//! Renderer-facing scene extraction for NOVA.
//!
//! This crate deliberately separates engine-world data from a graphics API.
//! The current milestone produces a deterministic render scene; a GPU backend
//! and Android Surface integration are follow-up milestones.

pub mod gles;

use nova_core::Id;
use nova_math::{Transform, Vec3};
use nova_world::{EnvironmentSettings, LightComponent, MeshComponent, Place};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3d {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub vertical_fov_radians: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::UP,
            vertical_fov_radians: 60.0_f32.to_radians(),
            near_plane: 0.1,
            far_plane: 1_000.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderEntity {
    pub id: Id,
    pub name: String,
    pub transform: Transform,
    pub mesh: MeshComponent,
    pub base_color_rgba: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderLight {
    Directional { color_rgb: [f32; 3], intensity: f32 },
    Point { position: Vec3, color_rgb: [f32; 3], intensity: f32, radius: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderScene {
    pub place_id: Id,
    pub place_name: String,
    pub environment: EnvironmentSettings,
    pub entities: Vec<RenderEntity>,
    pub lights: Vec<RenderLight>,
}

impl RenderScene {
    /// Build a stable renderer-facing snapshot from a Place.
    /// Entities are sorted by ID so snapshots are deterministic across runs.
    pub fn from_place(place: &Place) -> Self {
        let mut source_entities: Vec<_> = place.entities.values().collect();
        source_entities.sort_by_key(|entity| entity.id);

        let mut entities = Vec::new();
        let mut lights = Vec::new();

        for entity in source_entities {
            if let Some(mesh) = &entity.mesh {
                let material = entity.material.as_ref();
                let defaults = nova_world::Material::default();
                let material = material.unwrap_or(&defaults);

                entities.push(RenderEntity {
                    id: entity.id,
                    name: entity.name.clone(),
                    transform: entity.transform,
                    mesh: mesh.clone(),
                    base_color_rgba: material.base_color_rgba,
                    metallic: material.metallic,
                    roughness: material.roughness,
                });
            }

            match &entity.light {
                Some(LightComponent::Directional { color_rgb, intensity }) => {
                    lights.push(RenderLight::Directional {
                        color_rgb: *color_rgb,
                        intensity: *intensity,
                    });
                }
                Some(LightComponent::Point { color_rgb, intensity, radius }) => {
                    lights.push(RenderLight::Point {
                        position: entity.transform.position,
                        color_rgb: *color_rgb,
                        intensity: *intensity,
                        radius: *radius,
                    });
                }
                None => {}
            }
        }

        Self {
            place_id: place.id,
            place_name: place.name.clone(),
            environment: place.environment.clone(),
            entities,
            lights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_world::Entity;

    #[test]
    fn extracts_meshes_materials_and_environment() {
        let mut place = Place::new("Render Test", Id::new());
        let mut platform = Entity::new("Platform", Vec3::new(1.0, 0.0, 2.0));
        platform.mesh = Some(MeshComponent::Cube { half_extents: Vec3::new(2.0, 0.5, 2.0) });
        place.add_entity(platform);

        let scene = RenderScene::from_place(&place);
        assert_eq!(scene.place_id, place.id);
        assert_eq!(scene.place_name, "Render Test");
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, "Platform");
        assert_eq!(scene.entities[0].base_color_rgba, [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(scene.environment, place.environment);
    }

    #[test]
    fn default_camera_has_valid_clip_range() {
        let camera = Camera3d::default();
        assert!(camera.near_plane > 0.0);
        assert!(camera.far_plane > camera.near_plane);
        assert!(camera.vertical_fov_radians > 0.0);
    }
}
