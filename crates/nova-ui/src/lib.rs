//! nova-ui: 9 primary navigation sections, UI component model, and mobile touch controls.

use nova_core::Id;
use nova_math::Vec2;
use serde::{Deserialize, Serialize};

/// The 9 primary application sections defined in the PRD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NavigationTab {
    Home,
    Discover,
    Avatar,
    Friends,
    Communities,
    Create,
    Profile,
    Notifications,
    Settings,
}

/// Mobile touch virtual joystick state.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct VirtualJoystick {
    pub center: Vec2,
    pub current: Vec2,
    pub radius: f32,
    pub is_active: bool,
}

impl VirtualJoystick {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            current: center,
            radius,
            is_active: false,
        }
    }

    pub fn on_touch_down(&mut self, touch_pos: Vec2) {
        self.center = touch_pos;
        self.current = touch_pos;
        self.is_active = true;
    }

    pub fn on_touch_move(&mut self, touch_pos: Vec2) {
        if self.is_active {
            let offset = Vec2::new(touch_pos.x - self.center.x, touch_pos.y - self.center.y);
            let len = offset.length();
            if len > self.radius {
                let norm = offset.normalize();
                self.current = Vec2::new(self.center.x + norm.x * self.radius, self.center.y + norm.y * self.radius);
            } else {
                self.current = touch_pos;
            }
        }
    }

    pub fn on_touch_up(&mut self) {
        self.is_active = false;
        self.current = self.center;
    }

    /// Normalized direction output [-1.0, 1.0].
    pub fn axis(&self) -> Vec2 {
        if !self.is_active || self.radius <= 0.0 {
            Vec2::ZERO
        } else {
            let dx = (self.current.x - self.center.x) / self.radius;
            let dy = (self.current.y - self.center.y) / self.radius;
            Vec2::new(dx, dy)
        }
    }
}

/// Mobile on-screen touch control layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileTouchControls {
    pub joystick: VirtualJoystick,
    pub jump_button_pressed: bool,
    pub action_button_pressed: bool,
    pub camera_drag_delta: Vec2,
}

impl Default for MobileTouchControls {
    fn default() -> Self {
        Self {
            joystick: VirtualJoystick::new(Vec2::new(120.0, 600.0), 60.0),
            jump_button_pressed: false,
            action_button_pressed: false,
            camera_drag_delta: Vec2::ZERO,
        }
    }
}

/// Platform Navigation State.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformNavigationState {
    pub current_tab: NavigationTab,
    pub active_experience_id: Option<Id>,
    pub in_creator_mode: bool,
}

impl Default for PlatformNavigationState {
    fn default() -> Self {
        Self {
            current_tab: NavigationTab::Home,
            active_experience_id: None,
            in_creator_mode: false,
        }
    }
}

impl PlatformNavigationState {
    pub fn switch_tab(&mut self, tab: NavigationTab) {
        self.current_tab = tab;
    }

    pub fn launch_experience(&mut self, id: Id) {
        self.active_experience_id = Some(id);
    }

    pub fn close_experience(&mut self) {
        self.active_experience_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_joystick() {
        let mut js = VirtualJoystick::new(Vec2::new(100.0, 100.0), 50.0);
        assert_eq!(js.axis(), Vec2::ZERO);

        js.on_touch_down(Vec2::new(100.0, 100.0));
        js.on_touch_move(Vec2::new(150.0, 100.0));
        let axis = js.axis();
        assert!((axis.x - 1.0).abs() < 1e-4);
        assert_eq!(axis.y, 0.0);

        js.on_touch_up();
        assert_eq!(js.axis(), Vec2::ZERO);
    }
}
