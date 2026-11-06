//! NOVA Racing: Track layout, vehicle physics, lap tracking, and checkpoint detection.

use nova_core::Id;
use nova_math::Vec3;
use nova_physics::{BodyType, Vehicle};
use nova_world::{Entity, Place, SpawnPoint};

pub fn build_racing_world(creator_id: Id) -> Place {
    let mut place = Place::new("NOVA Racing: Grand Prix", creator_id);
    place.spawn_points = vec![SpawnPoint {
        position: Vec3::new(0.0, 0.5, 0.0),
        yaw_degrees: 0.0,
        team_id: None,
    }];

    // Main Track Base
    let track = Entity::new("Track_Floor", Vec3::new(0.0, -0.5, 0.0))
        .with_box_collider(Vec3::new(100.0, 0.5, 100.0), BodyType::Static);
    place.add_entity(track);

    // Left Barrier
    let left_wall = Entity::new("Barrier_Left", Vec3::new(-15.0, 1.0, 0.0))
        .with_box_collider(Vec3::new(0.5, 1.0, 50.0), BodyType::Static);
    place.add_entity(left_wall);

    // Right Barrier
    let right_wall = Entity::new("Barrier_Right", Vec3::new(15.0, 1.0, 0.0))
        .with_box_collider(Vec3::new(0.5, 1.0, 50.0), BodyType::Static);
    place.add_entity(right_wall);

    place
}

pub struct RaceSession {
    pub vehicle: Vehicle,
    pub current_lap: u32,
    pub total_laps: u32,
    pub lap_time: f32,
    pub best_lap_time: Option<f32>,
    pub next_checkpoint: u32,
    pub is_race_finished: bool,
}

impl RaceSession {
    pub fn new() -> Self {
        Self {
            vehicle: Vehicle::new(Id::new(), Vec3::ZERO),
            current_lap: 1,
            total_laps: 3,
            lap_time: 0.0,
            best_lap_time: None,
            next_checkpoint: 1,
            is_race_finished: false,
        }
    }

    pub fn tick(&mut self, dt: f32, throttle: f32, steering: f32) {
        if self.is_race_finished {
            return;
        }

        self.lap_time += dt;
        self.vehicle.step(dt, throttle, steering);

        // Checkpoint 1 at z = -25.0
        if self.next_checkpoint == 1 && self.vehicle.position.z <= -25.0 {
            self.next_checkpoint = 2;
        }

        // Finish line at z >= 0.0 after passing checkpoint 2
        if self.next_checkpoint == 2 && self.vehicle.position.z >= 0.0 {
            if self.best_lap_time.is_none() || self.lap_time < self.best_lap_time.unwrap() {
                self.best_lap_time = Some(self.lap_time);
            }

            if self.current_lap >= self.total_laps {
                self.is_race_finished = true;
            } else {
                self.current_lap += 1;
                self.lap_time = 0.0;
                self.next_checkpoint = 1;
            }
        }
    }
}

impl Default for RaceSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_race_progression_and_lap_completion() {
        let mut race = RaceSession::new();
        assert_eq!(race.current_lap, 1);
        assert!(!race.is_race_finished);

        // Simulate driving forward towards checkpoint 1
        for _ in 0..30 {
            race.tick(0.1, 1.0, 0.0);
        }
        assert_eq!(race.next_checkpoint, 2);

        // Simulate turnaround and driving across finish line
        race.vehicle.velocity = Vec3::ZERO;
        race.vehicle.position.z = 5.0;
        race.tick(0.1, 0.0, 0.0);
        assert_eq!(race.current_lap, 2);
        assert!(race.best_lap_time.is_some());
    }
}
