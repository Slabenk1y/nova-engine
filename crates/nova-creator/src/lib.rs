//! nova-creator: NOVA Creator Studio model, scene hierarchy, inspector, undo/redo stack, templates, and publishing.

use nova_core::Id;
use nova_math::Vec3;
use nova_physics::BodyType;
use nova_world::{Entity, Place};
use serde::{Deserialize, Serialize};

/// An undoable/redoable editor action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorAction {
    AddEntity(Entity),
    RemoveEntity(Entity),
    TranslateEntity { id: Id, old_pos: Vec3, new_pos: Vec3 },
    RenameEntity { id: Id, old_name: String, new_name: String },
}

/// Project starter templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarterTemplate {
    EmptyWorld,
    Obby,
    Simulator,
    SocialPlaza,
    RacingTrack,
}

/// Creator Studio session state.
#[derive(Debug)]
pub struct CreatorSession {
    pub place: Place,
    pub selected_entity_id: Option<Id>,
    pub undo_stack: Vec<EditorAction>,
    pub redo_stack: Vec<EditorAction>,
    pub is_playtesting: bool,
    pub saved_state_json: Option<String>,
}

impl CreatorSession {
    pub fn new(place: Place) -> Self {
        Self {
            place,
            selected_entity_id: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_playtesting: false,
            saved_state_json: None,
        }
    }

    pub fn from_template(template: StarterTemplate, creator_id: Id) -> Self {
        let place = match template {
            StarterTemplate::EmptyWorld => Place::new("Empty World", creator_id),
            StarterTemplate::Obby => {
                let mut p = Place::new("Obby Template", creator_id);
                let start_plat = Entity::new("Start_Platform", Vec3::new(0.0, 0.0, 0.0))
                    .with_box_collider(Vec3::new(4.0, 0.5, 4.0), BodyType::Static);
                p.add_entity(start_plat);
                p
            }
            StarterTemplate::Simulator => {
                let mut p = Place::new("Simulator Template", creator_id);
                let ground = Entity::new("Ground", Vec3::new(0.0, -0.5, 0.0))
                    .with_box_collider(Vec3::new(20.0, 0.5, 20.0), BodyType::Static);
                p.add_entity(ground);
                p
            }
            StarterTemplate::SocialPlaza => {
                let mut p = Place::new("Social Plaza Template", creator_id);
                let ground = Entity::new("Plaza_Ground", Vec3::new(0.0, -0.5, 0.0))
                    .with_box_collider(Vec3::new(30.0, 0.5, 30.0), BodyType::Static);
                p.add_entity(ground);
                p
            }
            StarterTemplate::RacingTrack => {
                let mut p = Place::new("Racing Track Template", creator_id);
                let track = Entity::new("Track_Floor", Vec3::new(0.0, -0.5, 0.0))
                    .with_box_collider(Vec3::new(50.0, 0.5, 50.0), BodyType::Static);
                p.add_entity(track);
                p
            }
        };

        Self::new(place)
    }

    pub fn execute_action(&mut self, action: EditorAction) {
        match &action {
            EditorAction::AddEntity(entity) => {
                self.place.add_entity(entity.clone());
            }
            EditorAction::RemoveEntity(entity) => {
                self.place.remove_entity(&entity.id);
                if self.selected_entity_id == Some(entity.id) {
                    self.selected_entity_id = None;
                }
            }
            EditorAction::TranslateEntity { id, new_pos, .. } => {
                if let Some(entity) = self.place.entities.get_mut(id) {
                    entity.transform.position = *new_pos;
                }
            }
            EditorAction::RenameEntity { id, new_name, .. } => {
                if let Some(entity) = self.place.entities.get_mut(id) {
                    entity.name = new_name.clone();
                }
            }
        }
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match &action {
                EditorAction::AddEntity(entity) => {
                    self.place.remove_entity(&entity.id);
                }
                EditorAction::RemoveEntity(entity) => {
                    self.place.add_entity(entity.clone());
                }
                EditorAction::TranslateEntity { id, old_pos, .. } => {
                    if let Some(entity) = self.place.entities.get_mut(id) {
                        entity.transform.position = *old_pos;
                    }
                }
                EditorAction::RenameEntity { id, old_name, .. } => {
                    if let Some(entity) = self.place.entities.get_mut(id) {
                        entity.name = old_name.clone();
                    }
                }
            }
            self.redo_stack.push(action);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match &action {
                EditorAction::AddEntity(entity) => {
                    self.place.add_entity(entity.clone());
                }
                EditorAction::RemoveEntity(entity) => {
                    self.place.remove_entity(&entity.id);
                }
                EditorAction::TranslateEntity { id, new_pos, .. } => {
                    if let Some(entity) = self.place.entities.get_mut(id) {
                        entity.transform.position = *new_pos;
                    }
                }
                EditorAction::RenameEntity { id, new_name, .. } => {
                    if let Some(entity) = self.place.entities.get_mut(id) {
                        entity.name = new_name.clone();
                    }
                }
            }
            self.undo_stack.push(action);
            true
        } else {
            false
        }
    }

    pub fn start_playtest(&mut self) {
        if !self.is_playtesting {
            self.saved_state_json = self.place.to_json().ok();
            self.is_playtesting = true;
        }
    }

    pub fn stop_playtest(&mut self) {
        if self.is_playtesting {
            if let Some(saved) = &self.saved_state_json {
                if let Ok(restored) = Place::from_json(saved) {
                    self.place = restored;
                }
            }
            self.is_playtesting = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creator_undo_redo() {
        let mut session = CreatorSession::from_template(StarterTemplate::EmptyWorld, Id::new());
        let entity = Entity::new("Rock", Vec3::new(1.0, 0.0, 1.0));
        let ent_id = entity.id;

        session.execute_action(EditorAction::AddEntity(entity));
        assert!(session.place.entities.contains_key(&ent_id));

        session.undo();
        assert!(!session.place.entities.contains_key(&ent_id));

        session.redo();
        assert!(session.place.entities.contains_key(&ent_id));
    }

    #[test]
    fn test_playtest_state_preservation() {
        let mut session = CreatorSession::from_template(StarterTemplate::EmptyWorld, Id::new());
        let entity = Entity::new("Coin", Vec3::new(0.0, 1.0, 0.0));
        session.place.add_entity(entity);

        session.start_playtest();
        assert!(session.is_playtesting);

        // Modify world during playtest
        session.place.entities.clear();
        assert_eq!(session.place.entities.len(), 0);

        session.stop_playtest();
        assert!(!session.is_playtesting);
        assert_eq!(session.place.entities.len(), 1);
    }
}
