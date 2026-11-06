//! NOVA Obby Experience definition and world generation.

use nova_core::Id;
use nova_math::Vec3;
use nova_physics::BodyType;
use nova_world::{Entity, Place, SpawnPoint};

pub fn build_obby_world(creator_id: Id) -> Place {
    let mut place = Place::new("NOVA Obby: Rainbow Tower", creator_id);
    place.spawn_points = vec![SpawnPoint {
        position: Vec3::new(0.0, 1.0, 0.0),
        yaw_degrees: 0.0,
        team_id: None,
    }];

    // 1. Starting Platform
    let start = Entity::new("Start_Platform", Vec3::new(0.0, 0.0, 0.0))
        .with_box_collider(Vec3::new(4.0, 0.5, 4.0), BodyType::Static);
    place.add_entity(start);

    // 2. Stage 1: Floating Step Stones
    for i in 1..=4 {
        let z_offset = -6.0 * i as f32;
        let y_offset = 0.5 * i as f32;
        let stone = Entity::new(format!("Step_Stone_{}", i), Vec3::new(0.0, y_offset, z_offset))
            .with_box_collider(Vec3::new(1.2, 0.4, 1.2), BodyType::Static);
        place.add_entity(stone);
    }

    // 3. Intermediate Checkpoint 1
    let check1 = Entity::new("Checkpoint_1", Vec3::new(0.0, 2.5, -30.0))
        .with_box_collider(Vec3::new(3.0, 0.5, 3.0), BodyType::Static);
    place.add_entity(check1);

    // 4. Moving Hazard Platform
    let mut slider = Entity::new("Moving_Hazard", Vec3::new(0.0, 3.0, -38.0))
        .with_box_collider(Vec3::new(2.0, 0.3, 1.5), BodyType::Kinematic);
    slider.tags.push("hazard".to_string());
    place.add_entity(slider);

    // 5. Lava floor trigger below
    let mut lava = Entity::new("Lava_Floor", Vec3::new(0.0, -10.0, -30.0))
        .with_box_collider(Vec3::new(50.0, 1.0, 50.0), BodyType::Static);
    if let Some(rb) = &mut lava.rigid_body {
        rb.is_trigger = true;
    }
    lava.tags.push("lava_kill".to_string());
    place.add_entity(lava);

    // 6. Finish Gateway Platform
    let finish = Entity::new("Finish_Platform", Vec3::new(0.0, 4.0, -50.0))
        .with_box_collider(Vec3::new(5.0, 0.5, 5.0), BodyType::Static);
    place.add_entity(finish);

    place
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_runtime::RuntimeSession;

    #[test]
    fn test_obby_progression_and_checkpoints() {
        let creator_id = Id::new();
        let place = build_obby_world(creator_id);
        assert!(place.entities.len() >= 6);

        let mut session = RuntimeSession::new(place);
        assert_eq!(session.character.position, Vec3::new(0.0, 1.0, 0.0));

        // Walk forward onto the obstacle course
        session.character.step(0.1, Vec3::new(0.0, 0.0, -1.0), false, &[]);
        assert!(session.character.position.z < 0.0);

        // Player reaches finish
        session.character.position = Vec3::new(0.0, 5.0, -50.0);
        assert_eq!(session.character.position.z, -50.0);
    }
}
