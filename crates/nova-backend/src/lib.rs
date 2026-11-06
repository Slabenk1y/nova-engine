//! nova-backend: Authentication, accounts, profiles, catalog, search, friends, communities, notifications, moderation.

use chrono::{DateTime, Utc};
use nova_core::Id;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// User Account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: Id,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub bio: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// Published Experience entry in catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogExperience {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub creator_id: Id,
    pub creator_name: String,
    pub genre: String,
    pub thumbnail_url: String,
    pub place_id: Id,
    pub player_count: u32,
    pub total_visits: u64,
    pub upvotes: u64,
    pub is_published: bool,
    pub updated_at: DateTime<Utc>,
}

/// Community Group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub owner_id: Id,
    pub members: Vec<Id>,
}

/// Moderation Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationReport {
    pub id: Id,
    pub reporter_id: Id,
    pub target_id: Id,
    pub target_type: String, // "Experience", "User", "Message"
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

/// In-memory platform backend services.
pub struct BackendService {
    pub users: HashMap<Id, UserAccount>,
    pub username_index: HashMap<String, Id>,
    pub catalog: HashMap<Id, CatalogExperience>,
    pub friends: HashMap<Id, Vec<Id>>,
    pub friend_requests: HashMap<Id, Vec<Id>>,
    pub communities: HashMap<Id, Community>,
    pub reports: Vec<ModerationReport>,
    pub player_storage: HashMap<(Id, Id), String>, // (PlayerId, ExperienceId) -> JSON data
}

impl Default for BackendService {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            username_index: HashMap::new(),
            catalog: HashMap::new(),
            friends: HashMap::new(),
            friend_requests: HashMap::new(),
            communities: HashMap::new(),
            reports: Vec::new(),
            player_storage: HashMap::new(),
        }
    }
}

impl BackendService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn register_user(&mut self, username: &str, display_name: &str, password: &str) -> Result<Id, String> {
        let normalized = username.to_lowercase();
        if self.username_index.contains_key(&normalized) {
            return Err("Username already taken".to_string());
        }

        let user_id = Id::new();
        let account = UserAccount {
            id: user_id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            password_hash: Self::hash_password(password),
            bio: String::new(),
            is_admin: false,
            created_at: Utc::now(),
        };

        self.users.insert(user_id, account);
        self.username_index.insert(normalized, user_id);
        Ok(user_id)
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Option<Id> {
        let user_id = self.username_index.get(&username.to_lowercase())?;
        let user = self.users.get(user_id)?;
        if user.password_hash == Self::hash_password(password) {
            Some(*user_id)
        } else {
            None
        }
    }

    pub fn publish_experience(&mut self, exp: CatalogExperience) -> Id {
        let id = exp.id;
        self.catalog.insert(id, exp);
        id
    }

    pub fn search_catalog(&self, query: &str) -> Vec<&CatalogExperience> {
        let q = query.to_lowercase();
        self.catalog
            .values()
            .filter(|e| e.is_published && (e.title.to_lowercase().contains(&q) || e.genre.to_lowercase().contains(&q)))
            .collect()
    }

    pub fn send_friend_request(&mut self, from: Id, to: Id) -> Result<(), String> {
        if from == to {
            return Err("Cannot friend self".to_string());
        }
        self.friend_requests.entry(to).or_default().push(from);
        Ok(())
    }

    pub fn accept_friend_request(&mut self, user_id: Id, friend_id: Id) -> Result<(), String> {
        if let Some(requests) = self.friend_requests.get_mut(&user_id) {
            if let Some(pos) = requests.iter().position(|id| *id == friend_id) {
                requests.remove(pos);
                self.friends.entry(user_id).or_default().push(friend_id);
                self.friends.entry(friend_id).or_default().push(user_id);
                return Ok(());
            }
        }
        Err("Friend request not found".to_string())
    }

    pub fn save_player_data(&mut self, player_id: Id, experience_id: Id, json_data: String) {
        self.player_storage.insert((player_id, experience_id), json_data);
    }

    pub fn load_player_data(&self, player_id: Id, experience_id: Id) -> Option<String> {
        self.player_storage.get(&(player_id, experience_id)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_and_persistence() {
        let mut backend = BackendService::new();
        let user_id = backend.register_user("player1", "CoolGamer", "secret123").unwrap();
        assert_eq!(backend.authenticate("player1", "secret123"), Some(user_id));
        assert_eq!(backend.authenticate("player1", "wrong_pass"), None);

        let exp_id = Id::new();
        backend.save_player_data(user_id, exp_id, "{\"level\": 10}".to_string());
        let loaded = backend.load_player_data(user_id, exp_id);
        assert_eq!(loaded, Some("{\"level\": 10}".to_string()));
    }
}
