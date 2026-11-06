//! nova-runtime: Unified execution runtime for loading places, updating physics, characters, scripts, and inputs.

use nova_character::{AnimationStateMachine, AvatarConfig};
use nova_core::{EngineTime, Id};
use nova_math::Vec3;
use nova_physics::{CharacterController, RigidBody};
use nova_scripting::{ScriptExecutionContext, ScriptRuntime, VisualScriptGraph};
use nova_ui::MobileTouchControls;
use nova_world::Place;
use std::collections::HashMap;

/// An active runtime session running a specific place.
pub struct RuntimeSession {
    pub place: Place,
    pub character: CharacterController,
    pub avatar_config: AvatarConfig,
    pub anim_state: AnimationStateMachine,
    pub engine_time: EngineTime,
    pub script_runtime: ScriptRuntime,
    pub script_graphs: HashMap<Id, VisualScriptGraph>,
    pub script_context: ScriptExecutionContext,
    pub touch_controls: MobileTouchControls,
    pub active_spawn_point: Vec3,
    pub session_score: i64,
    pub session_timer: f32,
    pub is_completed: bool,
}

impl RuntimeSession {
    pub fn new(place: Place) -> Self {
        let spawn_pos = place
            .spawn_points
            .first()
            .map(|sp| sp.position)
            .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

        Self {
            place,
            character: CharacterController::new(spawn_pos),
            avatar_config: AvatarConfig::default(),
            anim_state: AnimationStateMachine::new(),
            engine_time: EngineTime::new(),
            script_runtime: ScriptRuntime::new(),
            script_graphs: HashMap::new(),
            script_context: ScriptExecutionContext::default(),
            touch_controls: MobileTouchControls::default(),
            active_spawn_point: spawn_pos,
            session_score: 0,
            session_timer: 0.0,
            is_completed: false,
        }
    }

    pub fn respawn(&mut self) {
        self.character.position = self.active_spawn_point;
        self.character.velocity = Vec3::ZERO;
        self.script_context.player_respawn_requested = false;
    }

    pub fn tick(&mut self, dt: f32) {
        self.engine_time.tick(dt);
        self.session_timer += dt;

        // Process touch controls to movement input vector
        let joy_axis = self.touch_controls.joystick.axis();
        let move_input = Vec3::new(joy_axis.x, 0.0, -joy_axis.y);
        let jump = self.touch_controls.jump_button_pressed;

        // Collect rigid bodies in the world
        let bodies: Vec<RigidBody> = self
            .place
            .entities
            .values()
            .filter_map(|e| e.rigid_body.clone())
            .collect();

        // Step physics character controller
        self.character.step(dt, move_input, jump, &bodies);

        // Update animation state machine
        self.anim_state.update(dt, &self.character);

        // Check if player fell into the abyss
        if self.character.position.y < -30.0 || self.script_context.player_respawn_requested {
            self.respawn();
        }

        // Handle teleport triggers if any
        if let Some(target) = self.script_context.teleport_target.take() {
            self.character.position = Vec3::new(target[0], target[1], target[2]);
            self.character.velocity = Vec3::ZERO;
        }

        self.session_score = self.script_context.player_score;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_physics::BodyType;
    use nova_world::Entity;

    #[test]
    fn test_runtime_session_tick_and_fall_respawn() {
        let exp_id = Id::new();
        let mut place = Place::new("Test Realm", exp_id);
        let ground = Entity::new("Ground", Vec3::new(0.0, -1.0, 0.0))
            .with_box_collider(Vec3::new(10.0, 1.0, 10.0), BodyType::Static);
        place.add_entity(ground);

        let mut session = RuntimeSession::new(place);
        assert_eq!(session.character.position, Vec3::new(0.0, 1.0, 0.0));

        // Step session for a frame
        session.tick(0.016);
        assert!(session.engine_time.frame_count > 0);

        // Simulate falling off platform
        session.character.position = Vec3::new(0.0, -35.0, 0.0);
        session.tick(0.016);
        // Character should respawn back at spawn point
        assert_eq!(session.character.position, Vec3::new(0.0, 1.0, 0.0));
    }
}
