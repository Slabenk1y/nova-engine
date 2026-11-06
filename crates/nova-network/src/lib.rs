//! nova-network: Authoritative client-server multiplayer protocol, state replication, and prediction.

use nova_core::Id;
use nova_math::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client-to-server action packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientPacket {
    JoinPlace { player_id: Id, username: String },
    InputTick { sequence: u64, move_vector: Vec3, jump: bool, yaw: f32 },
    ChatMessage { message: String },
    Interaction { target_entity_id: Id, action_name: String },
    Disconnect,
}

/// Character state replicated across network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPlayerState {
    pub player_id: Id,
    pub username: String,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub is_grounded: bool,
    pub score: i64,
}

/// Server-to-client replication packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerPacket {
    Welcome { assigned_player_id: Id, server_tick: u64, place_id: Id },
    WorldSnapshot { server_tick: u64, players: Vec<NetworkPlayerState> },
    ChatMessageBroadcast { sender_id: Id, sender_name: String, message: String },
    PlayerJoined { player_id: Id, username: String },
    PlayerLeft { player_id: Id },
    ServerNotification { text: String },
}

/// Authoritative Server Room for an active place instance.
#[derive(Debug)]
pub struct ServerRoom {
    pub place_id: Id,
    pub max_players: usize,
    pub current_tick: u64,
    pub players: HashMap<Id, NetworkPlayerState>,
    pub outgoing_messages: Vec<(Option<Id>, ServerPacket)>,
}

impl ServerRoom {
    pub fn new(place_id: Id, max_players: usize) -> Self {
        Self {
            place_id,
            max_players,
            current_tick: 0,
            players: HashMap::new(),
            outgoing_messages: Vec::new(),
        }
    }

    pub fn handle_packet(&mut self, sender_id: Id, packet: ClientPacket) {
        match packet {
            ClientPacket::JoinPlace { player_id, username } => {
                if self.players.len() < self.max_players {
                    let new_player = NetworkPlayerState {
                        player_id,
                        username: username.clone(),
                        position: Vec3::new(0.0, 1.0, 0.0),
                        velocity: Vec3::ZERO,
                        yaw: 0.0,
                        is_grounded: true,
                        score: 0,
                    };
                    self.players.insert(player_id, new_player);

                    // Send welcome
                    self.outgoing_messages.push((
                        Some(player_id),
                        ServerPacket::Welcome {
                            assigned_player_id: player_id,
                            server_tick: self.current_tick,
                            place_id: self.place_id,
                        },
                    ));

                    // Broadcast join
                    self.outgoing_messages.push((
                        None,
                        ServerPacket::PlayerJoined { player_id, username },
                    ));
                }
            }
            ClientPacket::InputTick { move_vector, jump, yaw, .. } => {
                if let Some(player) = self.players.get_mut(&sender_id) {
                    player.yaw = yaw;
                    let speed = 6.0;
                    player.velocity = move_vector.normalize() * speed;
                    player.position += player.velocity * 0.05;
                    if jump && player.is_grounded {
                        player.velocity.y = 8.0;
                        player.is_grounded = false;
                    }
                }
            }
            ClientPacket::ChatMessage { message } => {
                let sender_name = self.players.get(&sender_id)
                    .map(|p| p.username.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                self.outgoing_messages.push((
                    None,
                    ServerPacket::ChatMessageBroadcast {
                        sender_id,
                        sender_name,
                        message,
                    },
                ));
            }
            ClientPacket::Disconnect => {
                self.players.remove(&sender_id);
                self.outgoing_messages.push((None, ServerPacket::PlayerLeft { player_id: sender_id }));
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) -> ServerPacket {
        self.current_tick += 1;
        ServerPacket::WorldSnapshot {
            server_tick: self.current_tick,
            players: self.players.values().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_room_join_and_replication() {
        let place_id = Id::new();
        let mut server = ServerRoom::new(place_id, 16);
        let player_id = Id::new();

        server.handle_packet(
            player_id,
            ClientPacket::JoinPlace {
                player_id,
                username: "BuilderGuy".to_string(),
            },
        );

        assert_eq!(server.players.len(), 1);
        let snapshot = server.tick();
        if let ServerPacket::WorldSnapshot { players, .. } = snapshot {
            assert_eq!(players.len(), 1);
            assert_eq!(players[0].username, "BuilderGuy");
        } else {
            panic!("Expected WorldSnapshot packet");
        }
    }
}
