//! NOVA Simulator: Crystal Miner environment, resource nodes, upgrades, and persistent state.

use nova_backend::BackendService;
use nova_core::Id;
use nova_math::Vec3;
use nova_physics::BodyType;
use nova_world::{Entity, Place, SpawnPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulatorPlayerData {
    pub coins: u64,
    pub ore_in_bag: u32,
    pub backpack_capacity: u32,
    pub pickaxe_power: u32,
}

impl Default for SimulatorPlayerData {
    fn default() -> Self {
        Self {
            coins: 0,
            ore_in_bag: 0,
            backpack_capacity: 25,
            pickaxe_power: 1,
        }
    }
}

pub fn build_simulator_world(creator_id: Id) -> Place {
    let mut place = Place::new("NOVA Simulator: Crystal Miner", creator_id);
    place.spawn_points = vec![SpawnPoint {
        position: Vec3::new(0.0, 1.0, 0.0),
        yaw_degrees: 0.0,
        team_id: None,
    }];

    // Cave Floor
    let floor = Entity::new("Cave_Floor", Vec3::new(0.0, -0.5, 0.0))
        .with_box_collider(Vec3::new(30.0, 0.5, 30.0), BodyType::Static);
    place.add_entity(floor);

    // Crystal Node 1
    let mut crystal1 = Entity::new("Crystal_Node_1", Vec3::new(5.0, 1.0, 5.0))
        .with_box_collider(Vec3::new(1.0, 1.5, 1.0), BodyType::Static);
    crystal1.tags.push("crystal_ore".to_string());
    place.add_entity(crystal1);

    // Crystal Node 2
    let mut crystal2 = Entity::new("Crystal_Node_2", Vec3::new(-6.0, 1.0, 7.0))
        .with_box_collider(Vec3::new(1.0, 1.5, 1.0), BodyType::Static);
    crystal2.tags.push("crystal_ore".to_string());
    place.add_entity(crystal2);

    // Sell Shop Area
    let mut sell_area = Entity::new("Sell_Ring", Vec3::new(0.0, 0.1, -10.0))
        .with_box_collider(Vec3::new(3.0, 0.2, 3.0), BodyType::Static);
    sell_area.tags.push("sell_zone".to_string());
    place.add_entity(sell_area);

    place
}

pub struct SimulatorSession {
    pub player_data: SimulatorPlayerData,
    pub experience_id: Id,
    pub player_id: Id,
}

impl SimulatorSession {
    pub fn new(player_id: Id, experience_id: Id, backend: &BackendService) -> Self {
        let player_data = if let Some(json) = backend.load_player_data(player_id, experience_id) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            SimulatorPlayerData::default()
        };

        Self {
            player_data,
            experience_id,
            player_id,
        }
    }

    pub fn harvest_crystal(&mut self) -> bool {
        if self.player_data.ore_in_bag < self.player_data.backpack_capacity {
            let harvest_amount = self.player_data.pickaxe_power;
            self.player_data.ore_in_bag = (self.player_data.ore_in_bag + harvest_amount)
                .min(self.player_data.backpack_capacity);
            true
        } else {
            false
        }
    }

    pub fn sell_ore(&mut self) -> u64 {
        let earned = (self.player_data.ore_in_bag as u64) * 10;
        self.player_data.coins += earned;
        self.player_data.ore_in_bag = 0;
        earned
    }

    pub fn upgrade_pickaxe(&mut self) -> bool {
        let cost = (self.player_data.pickaxe_power as u64) * 50;
        if self.player_data.coins >= cost {
            self.player_data.coins -= cost;
            self.player_data.pickaxe_power += 1;
            true
        } else {
            false
        }
    }

    pub fn save_to_backend(&self, backend: &mut BackendService) {
        if let Ok(json) = serde_json::to_string(&self.player_data) {
            backend.save_player_data(self.player_id, self.experience_id, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_harvest_sell_upgrade_and_persistence() {
        let mut backend = BackendService::new();
        let player_id = Id::new();
        let exp_id = Id::new();

        {
            let mut session = SimulatorSession::new(player_id, exp_id, &backend);
            assert_eq!(session.player_data.ore_in_bag, 0);

            // Mine crystal
            assert!(session.harvest_crystal());
            assert_eq!(session.player_data.ore_in_bag, 1);

            // Sell ore
            let earned = session.sell_ore();
            assert_eq!(earned, 10);
            assert_eq!(session.player_data.coins, 10);
            assert_eq!(session.player_data.ore_in_bag, 0);

            // Give extra coins and test pickaxe upgrade
            session.player_data.coins = 100;
            assert!(session.upgrade_pickaxe());
            assert_eq!(session.player_data.pickaxe_power, 2);

            // Save session to backend
            session.save_to_backend(&mut backend);
        }

        // New session for same player restores state from backend
        let restored_session = SimulatorSession::new(player_id, exp_id, &backend);
        assert_eq!(restored_session.player_data.pickaxe_power, 2);
        assert_eq!(restored_session.player_data.coins, 50);
    }
}
