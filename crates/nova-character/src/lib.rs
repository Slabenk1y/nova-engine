//! nova-character: Block-style avatar rig, customization, animation state machine, and inventory.

use nova_core::Id;
use nova_math::Vec3;
use nova_physics::CharacterController;
use serde::{Deserialize, Serialize};

/// Cosmetic item slot types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemSlot {
    Head,
    Face,
    Hair,
    TorsoClothing,
    LegClothing,
    Shoes,
    BackAccessory,
    WaistAccessory,
    HandHeld,
}

/// Catalog cosmetic item definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmeticItem {
    pub id: Id,
    pub name: String,
    pub slot: ItemSlot,
    pub mesh_asset: String,
    pub texture_asset: String,
    pub color_tint: [f32; 4],
}

/// Block avatar body parts with customized colors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyColors {
    pub head_color: [f32; 4],
    pub torso_color: [f32; 4],
    pub left_arm_color: [f32; 4],
    pub right_arm_color: [f32; 4],
    pub left_leg_color: [f32; 4],
    pub right_leg_color: [f32; 4],
}

impl Default for BodyColors {
    fn default() -> Self {
        Self {
            head_color: [0.98, 0.82, 0.65, 1.0],      // skin tone
            torso_color: [0.15, 0.45, 0.85, 1.0],     // blue shirt
            left_arm_color: [0.98, 0.82, 0.65, 1.0],
            right_arm_color: [0.98, 0.82, 0.65, 1.0],
            left_leg_color: [0.2, 0.6, 0.2, 1.0],     // green pants
            right_leg_color: [0.2, 0.6, 0.2, 1.0],
        }
    }
}

/// Full avatar configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvatarConfig {
    pub scale: Vec3,
    pub colors: BodyColors,
    pub equipped_items: Vec<CosmeticItem>,
}

impl Default for AvatarConfig {
    fn default() -> Self {
        Self {
            scale: Vec3::ONE,
            colors: BodyColors::default(),
            equipped_items: Vec::new(),
        }
    }
}

/// Animation state for the block character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocomotionState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
    Swimming,
    Sitting,
}

/// Character animation state machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationStateMachine {
    pub state: LocomotionState,
    pub state_time: f32,
    pub limb_swing_phase: f32,
}

impl AnimationStateMachine {
    pub fn new() -> Self {
        Self {
            state: LocomotionState::Idle,
            state_time: 0.0,
            limb_swing_phase: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, controller: &CharacterController) {
        self.state_time += dt;

        let horizontal_speed = Vec3::new(controller.velocity.x, 0.0, controller.velocity.z).length();

        if !controller.is_grounded {
            if controller.velocity.y > 0.0 {
                self.state = LocomotionState::Jumping;
            } else {
                self.state = LocomotionState::Falling;
            }
        } else if horizontal_speed > 6.5 {
            self.state = LocomotionState::Running;
            self.limb_swing_phase += dt * 14.0;
        } else if horizontal_speed > 0.1 {
            self.state = LocomotionState::Walking;
            self.limb_swing_phase += dt * 8.0;
        } else {
            self.state = LocomotionState::Idle;
            self.limb_swing_phase = 0.0;
        }
    }
}

impl Default for AnimationStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Player Inventory with server ownership validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInventory {
    pub owned_items: Vec<CosmeticItem>,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self { owned_items: Vec::new() }
    }

    pub fn owns_item(&self, item_id: &Id) -> bool {
        self.owned_items.iter().any(|item| &item.id == item_id)
    }

    pub fn grant_item(&mut self, item: CosmeticItem) {
        if !self.owns_item(&item.id) {
            self.owned_items.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state_transitions() {
        let mut sm = AnimationStateMachine::new();
        let mut controller = CharacterController::new(Vec3::ZERO);

        controller.is_grounded = true;
        controller.velocity = Vec3::ZERO;
        sm.update(0.1, &controller);
        assert_eq!(sm.state, LocomotionState::Idle);

        controller.velocity = Vec3::new(4.0, 0.0, 0.0);
        sm.update(0.1, &controller);
        assert_eq!(sm.state, LocomotionState::Walking);

        controller.is_grounded = false;
        controller.velocity = Vec3::new(0.0, 5.0, 0.0);
        sm.update(0.1, &controller);
        assert_eq!(sm.state, LocomotionState::Jumping);
    }

    #[test]
    fn test_inventory_ownership() {
        let mut inv = PlayerInventory::new();
        let item = CosmeticItem {
            id: Id::new(),
            name: "Gold Crown".to_string(),
            slot: ItemSlot::Head,
            mesh_asset: "crown.obj".to_string(),
            texture_asset: "crown.png".to_string(),
            color_tint: [1.0, 0.84, 0.0, 1.0],
        };
        let item_id = item.id;
        assert!(!inv.owns_item(&item_id));
        inv.grant_item(item);
        assert!(inv.owns_item(&item_id));
    }
}
