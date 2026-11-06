//! nova-physics: 3D physics engine, colliders, character controller, moving platforms, and vehicle physics.

use nova_core::Id;
use nova_math::{AABB, Ray, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { radius: f32, half_height: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidBody {
    pub id: Id,
    pub body_type: BodyType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub use_gravity: bool,
    pub is_trigger: bool,
    pub shape: ColliderShape,
}

impl RigidBody {
    pub fn new_box(id: Id, body_type: BodyType, position: Vec3, half_extents: Vec3) -> Self {
        Self {
            id,
            body_type,
            position,
            velocity: Vec3::ZERO,
            mass: if body_type == BodyType::Dynamic { 1.0 } else { 0.0 },
            restitution: 0.0,
            friction: 0.5,
            use_gravity: body_type == BodyType::Dynamic,
            is_trigger: false,
            shape: ColliderShape::Box { half_extents },
        }
    }

    pub fn aabb(&self) -> AABB {
        match self.shape {
            ColliderShape::Box { half_extents } => {
                AABB::from_center_half_extents(self.position, half_extents)
            }
            ColliderShape::Sphere { radius } => {
                AABB::from_center_half_extents(self.position, Vec3::new(radius, radius, radius))
            }
            ColliderShape::Capsule { radius, half_height } => {
                AABB::from_center_half_extents(self.position, Vec3::new(radius, half_height + radius, radius))
            }
        }
    }
}

/// Character controller handling movement, gravity, jumping, and ground collision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterController {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub height: f32,
    pub is_grounded: bool,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_force: f32,
    pub gravity: f32,
}

impl CharacterController {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            radius: 0.4,
            height: 1.8,
            is_grounded: false,
            walk_speed: 6.0,
            run_speed: 10.0,
            jump_force: 8.5,
            gravity: 19.6, // realistic fast gravity for snappy platforming
        }
    }

    pub fn aabb(&self) -> AABB {
        AABB::from_center_half_extents(
            self.position + Vec3::new(0.0, self.height * 0.5, 0.0),
            Vec3::new(self.radius, self.height * 0.5, self.radius),
        )
    }

    pub fn step(&mut self, dt: f32, move_input: Vec3, jump: bool, world_bodies: &[RigidBody]) {
        // Horizontal movement
        let speed = self.walk_speed;
        let target_vel = move_input.normalize() * speed;
        self.velocity.x = target_vel.x;
        self.velocity.z = target_vel.z;

        // Apply gravity
        if !self.is_grounded {
            self.velocity.y -= self.gravity * dt;
        } else if jump {
            self.velocity.y = self.jump_force;
            self.is_grounded = false;
        } else if self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }

        // Integrate proposed position
        let delta = self.velocity * dt;
        let mut new_pos = self.position + delta;

        // Check ground and obstacles
        let mut on_ground = false;
        for body in world_bodies {
            if body.is_trigger {
                continue;
            }
            let b_aabb = body.aabb();
            let c_aabb = AABB::from_center_half_extents(
                new_pos + Vec3::new(0.0, self.height * 0.5, 0.0),
                Vec3::new(self.radius, self.height * 0.5, self.radius),
            );

            if c_aabb.intersects(&b_aabb) {
                // If character is falling and hits top of body
                if self.velocity.y <= 0.0 && self.position.y >= b_aabb.max.y - 0.2 {
                    new_pos.y = b_aabb.max.y;
                    self.velocity.y = 0.0;
                    on_ground = true;
                } else {
                    // Simple lateral obstacle resolution
                    if new_pos.x > b_aabb.min.x && new_pos.x < b_aabb.max.x {
                        new_pos.x = self.position.x;
                    }
                    if new_pos.z > b_aabb.min.z && new_pos.z < b_aabb.max.z {
                        new_pos.z = self.position.z;
                    }
                }
            }
        }

        self.position = new_pos;
        self.is_grounded = on_ground;
    }
}

/// Raycast hit result.
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub body_id: Id,
    pub distance: f32,
    pub point: Vec3,
}

/// 3D Vehicle Physics Simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Id,
    pub position: Vec3,
    pub velocity: Vec3,
    pub heading_angle: f32,
    pub engine_power: f32,
    pub max_speed: f32,
    pub steering_sensitivity: f32,
    pub drag: f32,
}

impl Vehicle {
    pub fn new(id: Id, position: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            heading_angle: 0.0,
            engine_power: 20.0,
            max_speed: 35.0,
            steering_sensitivity: 2.5,
            drag: 1.2,
        }
    }

    pub fn step(&mut self, dt: f32, throttle: f32, steering: f32) {
        // Update orientation
        let forward = Vec3::new(self.heading_angle.sin(), 0.0, -self.heading_angle.cos());
        let current_speed = self.velocity.dot(forward);

        if current_speed.abs() > 0.1 {
            self.heading_angle += steering * self.steering_sensitivity * dt * current_speed.signum();
        }

        let new_forward = Vec3::new(self.heading_angle.sin(), 0.0, -self.heading_angle.cos());

        // Apply engine acceleration
        let accel = new_forward * (throttle * self.engine_power);
        self.velocity += accel * dt;

        // Apply drag
        let speed = self.velocity.length();
        if speed > 0.0 {
            let drag_force = self.velocity.normalize() * (self.drag * speed * dt);
            if drag_force.length() > speed {
                self.velocity = Vec3::ZERO;
            } else {
                self.velocity -= drag_force;
            }
        }

        // Clamp to max speed
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        self.position += self.velocity * dt;
    }
}

/// Physics world manager.
#[derive(Debug, Default)]
pub struct PhysicsWorld {
    pub bodies: Vec<RigidBody>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self { bodies: Vec::new() }
    }

    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }

    pub fn raycast(&self, ray: &Ray, max_dist: f32) -> Option<RaycastHit> {
        let mut closest_hit: Option<RaycastHit> = None;
        for body in &self.bodies {
            if let Some(t) = ray.intersect_aabb(&body.aabb()) {
                if t >= 0.0 && t <= max_dist {
                    if closest_hit.is_none() || t < closest_hit.as_ref().unwrap().distance {
                        closest_hit = Some(RaycastHit {
                            body_id: body.id,
                            distance: t,
                            point: ray.origin + ray.direction * t,
                        });
                    }
                }
            }
        }
        closest_hit
    }

    pub fn step(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if body.body_type == BodyType::Dynamic && body.use_gravity {
                body.velocity.y -= 9.81 * dt;
                body.position += body.velocity * dt;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_gravity_and_ground() {
        let ground = RigidBody::new_box(
            Id::new(),
            BodyType::Static,
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(10.0, 1.0, 10.0),
        );
        let mut char_ctrl = CharacterController::new(Vec3::new(0.0, 2.0, 0.0));
        assert!(!char_ctrl.is_grounded);

        // Step character until landing on ground (surface is y=0.0)
        for _ in 0..50 {
            char_ctrl.step(0.02, Vec3::ZERO, false, &[ground.clone()]);
        }
        assert!(char_ctrl.is_grounded);
        assert!((char_ctrl.position.y - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_vehicle_acceleration() {
        let mut vehicle = Vehicle::new(Id::new(), Vec3::ZERO);
        assert_eq!(vehicle.velocity.length(), 0.0);
        vehicle.step(0.1, 1.0, 0.0);
        assert!(vehicle.velocity.length() > 0.0);
    }
}
