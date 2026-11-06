//! nova-app: Nova gaming platform and engine runner.

use chrono::Utc;
use nova_backend::{BackendService, CatalogExperience};
use nova_core::Id;
use nova_runtime::RuntimeSession;
use nova_ui::PlatformNavigationState;
use nova_world::Place;

pub struct NovaApp {
    pub backend: BackendService,
    pub nav_state: PlatformNavigationState,
    pub active_session: Option<RuntimeSession>,
    pub experience_places: std::collections::HashMap<Id, Place>,
}

impl NovaApp {
    pub fn new() -> Self {
        let mut app = Self {
            backend: BackendService::new(),
            nav_state: PlatformNavigationState::default(),
            active_session: None,
            experience_places: std::collections::HashMap::new(),
        };
        app.register_standard_catalog();
        app
    }

    /// Registers the 4 mandatory test experiences into the catalog and stores their places.
    pub fn register_standard_catalog(&mut self) {
        let system_creator_id = Id::new();

        // 1. NOVA Obby
        let obby_place = experience_nova_obby::build_obby_world(system_creator_id);
        let obby_exp = CatalogExperience {
            id: Id::new(),
            title: "NOVA Obby: Rainbow Tower".to_string(),
            description: "Colorful obstacle course testing jumping, checkpoints, and hazard triggers.".to_string(),
            creator_id: system_creator_id,
            creator_name: "NOVA Studio".to_string(),
            genre: "Obby".to_string(),
            thumbnail_url: "textures/thumbnails/obby.png".to_string(),
            place_id: obby_place.id,
            player_count: 0,
            total_visits: 1250,
            upvotes: 450,
            is_published: true,
            updated_at: Utc::now(),
        };
        self.experience_places.insert(obby_exp.id, obby_place);
        self.backend.publish_experience(obby_exp);

        // 2. NOVA Social World
        let social_place = experience_nova_social_world::build_social_world(system_creator_id);
        let social_exp = CatalogExperience {
            id: Id::new(),
            title: "NOVA Social World: Central Plaza".to_string(),
            description: "Multiplayer 3D plaza with fountain, gazebo, player list, and chat broadcast.".to_string(),
            creator_id: system_creator_id,
            creator_name: "NOVA Studio".to_string(),
            genre: "Social".to_string(),
            thumbnail_url: "textures/thumbnails/social.png".to_string(),
            place_id: social_place.id,
            player_count: 0,
            total_visits: 3420,
            upvotes: 1200,
            is_published: true,
            updated_at: Utc::now(),
        };
        self.experience_places.insert(social_exp.id, social_place);
        self.backend.publish_experience(social_exp);

        // 3. NOVA Simulator
        let sim_place = experience_nova_simulator::build_simulator_world(system_creator_id);
        let sim_exp = CatalogExperience {
            id: Id::new(),
            title: "NOVA Simulator: Crystal Miner".to_string(),
            description: "Resource harvesting simulator with backpack storage, upgrades, and persistence.".to_string(),
            creator_id: system_creator_id,
            creator_name: "NOVA Studio".to_string(),
            genre: "Simulator".to_string(),
            thumbnail_url: "textures/thumbnails/simulator.png".to_string(),
            place_id: sim_place.id,
            player_count: 0,
            total_visits: 890,
            upvotes: 310,
            is_published: true,
            updated_at: Utc::now(),
        };
        self.experience_places.insert(sim_exp.id, sim_place);
        self.backend.publish_experience(sim_exp);

        // 4. NOVA Racing
        let race_place = experience_nova_racing::build_racing_world(system_creator_id);
        let race_exp = CatalogExperience {
            id: Id::new(),
            title: "NOVA Racing: Grand Prix".to_string(),
            description: "High speed circuit racing with vehicle dynamics, checkpoints, and lap timing.".to_string(),
            creator_id: system_creator_id,
            creator_name: "NOVA Studio".to_string(),
            genre: "Racing".to_string(),
            thumbnail_url: "textures/thumbnails/racing.png".to_string(),
            place_id: race_place.id,
            player_count: 0,
            total_visits: 2100,
            upvotes: 820,
            is_published: true,
            updated_at: Utc::now(),
        };
        self.experience_places.insert(race_exp.id, race_place);
        self.backend.publish_experience(race_exp);
    }

    pub fn launch_by_id(&mut self, exp_id: Id) -> bool {
        if let Some(place) = self.experience_places.get(&exp_id).cloned() {
            self.launch_experience(exp_id, place);
            true
        } else {
            false
        }
    }

    pub fn launch_experience(&mut self, exp_id: Id, place: Place) {
        self.nav_state.launch_experience(exp_id);
        self.active_session = Some(RuntimeSession::new(place));
    }

    pub fn tick(&mut self, dt: f32) {
        if let Some(session) = &mut self.active_session {
            session.tick(dt);
        }
    }
}

impl Default for NovaApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_ui::NavigationTab;

    #[test]
    fn test_app_lifecycle() {
        let mut app = NovaApp::new();
        assert_eq!(app.nav_state.current_tab, NavigationTab::Home);

        // Verify that all 4 mandatory test experiences are published into the catalog
        assert_eq!(app.backend.catalog.len(), 4);
        assert_eq!(app.experience_places.len(), 4);

        // Search catalog for Obby, Social, Simulator, Racing
        assert_eq!(app.backend.search_catalog("Obby").len(), 1);
        assert_eq!(app.backend.search_catalog("Social").len(), 1);
        assert_eq!(app.backend.search_catalog("Simulator").len(), 1);
        assert_eq!(app.backend.search_catalog("Racing").len(), 1);

        // Verify launching each of the 4 test experiences through shared runtime
        let exp_ids: Vec<Id> = app.experience_places.keys().cloned().collect();
        for id in exp_ids {
            assert!(app.launch_by_id(id));
            assert!(app.active_session.is_some());
            app.tick(0.016);
            assert!(app.active_session.as_ref().unwrap().engine_time.frame_count > 0);
        }
    }
}
