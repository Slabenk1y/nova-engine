//! NOVA Social World: Central Plaza environment and multiplayer synchronization logic.

use nova_core::Id;
use nova_math::Vec3;
use nova_network::{ClientPacket, ServerRoom};
#[cfg(test)]
use nova_network::ServerPacket;
use nova_physics::BodyType;
use nova_world::{Entity, Place, SpawnPoint};

pub fn build_social_world(creator_id: Id) -> Place {
    let mut place = Place::new("NOVA Social Plaza", creator_id);
    place.spawn_points = vec![
        SpawnPoint { position: Vec3::new(0.0, 1.0, 0.0), yaw_degrees: 0.0, team_id: None },
        SpawnPoint { position: Vec3::new(3.0, 1.0, 3.0), yaw_degrees: 45.0, team_id: None },
        SpawnPoint { position: Vec3::new(-3.0, 1.0, 3.0), yaw_degrees: -45.0, team_id: None },
    ];

    // Plaza Marble Floor
    let floor = Entity::new("Plaza_Floor", Vec3::new(0.0, -0.5, 0.0))
        .with_box_collider(Vec3::new(40.0, 0.5, 40.0), BodyType::Static);
    place.add_entity(floor);

    // Central Fountain
    let fountain = Entity::new("Central_Fountain", Vec3::new(0.0, 1.0, 0.0))
        .with_box_collider(Vec3::new(3.0, 1.0, 3.0), BodyType::Static);
    place.add_entity(fountain);

    // Social Gazebo / Pavilion
    let gazebo = Entity::new("Gazebo_Meeting_Spot", Vec3::new(15.0, 2.0, 15.0))
        .with_box_collider(Vec3::new(5.0, 2.0, 5.0), BodyType::Static);
    place.add_entity(gazebo);

    place
}

pub struct SocialWorldServerSession {
    pub room: ServerRoom,
}

impl SocialWorldServerSession {
    pub fn new(place_id: Id) -> Self {
        Self {
            room: ServerRoom::new(place_id, 32),
        }
    }

    pub fn player_join(&mut self, player_id: Id, username: &str) {
        self.room.handle_packet(
            player_id,
            ClientPacket::JoinPlace {
                player_id,
                username: username.to_string(),
            },
        );
    }

    pub fn player_chat(&mut self, player_id: Id, msg: &str) {
        self.room.handle_packet(
            player_id,
            ClientPacket::ChatMessage {
                message: msg.to_string(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplayer_social_interaction() {
        let exp_id = Id::new();
        let place = build_social_world(exp_id);
        let mut session = SocialWorldServerSession::new(place.id);

        let p1 = Id::new();
        let p2 = Id::new();

        session.player_join(p1, "Alice");
        session.player_join(p2, "Bob");

        assert_eq!(session.room.players.len(), 2);

        session.player_chat(p1, "Hello NOVA Social World!");
        let has_chat_broadcast = session.room.outgoing_messages.iter().any(|(_, packet)| {
            if let ServerPacket::ChatMessageBroadcast { message, sender_name, .. } = packet {
                message == "Hello NOVA Social World!" && sender_name == "Alice"
            } else {
                false
            }
        });
        assert!(has_chat_broadcast);
    }
}
