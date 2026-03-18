use crate::error::Result;
use crate::types::{TemperaturePreference, UserPreferences};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub preferences: UserPreferences,
    pub behavior_patterns: Vec<BehaviorPattern>,
    pub device_usage: HashMap<String, DeviceUsageStats>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            preferences: UserPreferences::default(),
            behavior_patterns: Vec::new(),
            device_usage: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub trigger: PatternTrigger,
    pub actions: Vec<PatternAction>,
    pub frequency: u32,
    pub last_triggered: Option<DateTime<Utc>>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    TimeBased,
    LocationBased,
    EventBased,
    Sequence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTrigger {
    pub trigger_type: String,
    pub conditions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAction {
    pub device_id: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUsageStats {
    pub device_id: String,
    pub total_uses: u32,
    pub last_used: Option<DateTime<Utc>>,
    pub average_duration_secs: f64,
    pub preferred_settings: HashMap<String, serde_json::Value>,
    pub usage_by_hour: [u32; 24],
    pub usage_by_day: [u32; 7],
}

impl DeviceUsageStats {
    pub fn new(device_id: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            total_uses: 0,
            last_used: None,
            average_duration_secs: 0.0,
            preferred_settings: HashMap::new(),
            usage_by_hour: [0; 24],
            usage_by_day: [0; 7],
        }
    }
}

pub struct UserProfileEngine {
    profiles: Arc<RwLock<HashMap<String, UserProfile>>>,
    current_user: Arc<RwLock<Option<String>>>,
    learning_enabled: Arc<RwLock<bool>>,
}

impl UserProfileEngine {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            current_user: Arc::new(RwLock::new(None)),
            learning_enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn create_profile(&self, user_id: &str) -> Result<UserProfile> {
        let profile = UserProfile {
            user_id: user_id.to_string(),
            preferences: UserPreferences::default(),
            behavior_patterns: Vec::new(),
            device_usage: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut profiles = self.profiles.write().await;
        profiles.insert(user_id.to_string(), profile.clone());

        tracing::info!("Created profile for user: {}", user_id);
        Ok(profile)
    }

    pub async fn get_profile(&self, user_id: &str) -> Option<UserProfile> {
        let profiles = self.profiles.read().await;
        profiles.get(user_id).cloned()
    }

    pub async fn get_current_profile(&self) -> Option<UserProfile> {
        let current = self.current_user.read().await;
        if let Some(user_id) = current.as_ref() {
            self.get_profile(user_id).await
        } else {
            None
        }
    }

    pub async fn set_current_user(&self, user_id: &str) -> Result<()> {
        let mut current = self.current_user.write().await;
        *current = Some(user_id.to_string());
        tracing::info!("Set current user: {}", user_id);
        Ok(())
    }

    pub async fn update_preferences(
        &self,
        user_id: &str,
        preferences: UserPreferences,
    ) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            profile.preferences = preferences;
            profile.updated_at = Utc::now();
            tracing::info!("Updated preferences for user: {}", user_id);
        }
        Ok(())
    }

    pub async fn record_device_usage(
        &self,
        user_id: &str,
        device_id: &str,
        action: &str,
        parameters: &HashMap<String, serde_json::Value>,
        duration_secs: Option<f64>,
    ) -> Result<()> {
        let mut profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            let stats = profile
                .device_usage
                .entry(device_id.to_string())
                .or_insert_with(|| DeviceUsageStats::new(device_id));

            stats.total_uses += 1;
            stats.last_used = Some(Utc::now());

            if let Some(dur) = duration_secs {
                let total = stats.average_duration_secs * (stats.total_uses - 1) as f64;
                stats.average_duration_secs = (total + dur) / stats.total_uses as f64;
            }

            let hour = Utc::now().hour() as usize;
            stats.usage_by_hour[hour] += 1;

            let day = Utc::now().weekday().num_days_from_monday() as usize;
            stats.usage_by_day[day] += 1;

            for (key, value) in parameters {
                stats.preferred_settings.insert(key.clone(), value.clone());
            }

            profile.updated_at = Utc::now();
        }
        Ok(())
    }

    pub async fn learn_pattern(
        &self,
        user_id: &str,
        trigger: PatternTrigger,
        actions: Vec<PatternAction>,
    ) -> Result<String> {
        let mut profiles = self.profiles.write().await;
        if let Some(profile) = profiles.get_mut(user_id) {
            let pattern_id = uuid::Uuid::new_v4().to_string();

            let pattern = BehaviorPattern {
                pattern_id: pattern_id.clone(),
                pattern_type: PatternType::EventBased,
                trigger,
                actions,
                frequency: 1,
                last_triggered: Some(Utc::now()),
                confidence: 0.5,
            };

            profile.behavior_patterns.push(pattern);
            profile.updated_at = Utc::now();

            tracing::info!("Learned new pattern for user {}: {}", user_id, pattern_id);
            return Ok(pattern_id);
        }
        Ok(String::new())
    }

    pub async fn get_patterns(&self, user_id: &str) -> Vec<BehaviorPattern> {
        let profiles = self.profiles.read().await;
        profiles
            .get(user_id)
            .map(|p| p.behavior_patterns.clone())
            .unwrap_or_default()
    }

    pub async fn find_matching_patterns(
        &self,
        user_id: &str,
        trigger_type: &str,
        conditions: &HashMap<String, serde_json::Value>,
    ) -> Vec<BehaviorPattern> {
        let profiles = self.profiles.read().await;
        if let Some(profile) = profiles.get(user_id) {
            profile
                .behavior_patterns
                .iter()
                .filter(|p| {
                    p.trigger.trigger_type == trigger_type
                        && p.confidence > 0.7
                        && self.conditions_match(&p.trigger.conditions, conditions)
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    fn conditions_match(
        &self,
        pattern_conditions: &HashMap<String, serde_json::Value>,
        current_conditions: &HashMap<String, serde_json::Value>,
    ) -> bool {
        for (key, value) in pattern_conditions {
            if let Some(current) = current_conditions.get(key) {
                if current != value {
                    return false;
                }
            }
        }
        true
    }

    pub async fn get_device_preferences(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> Option<HashMap<String, serde_json::Value>> {
        let profiles = self.profiles.read().await;
        profiles
            .get(user_id)
            .and_then(|p| p.device_usage.get(device_id))
            .map(|s| s.preferred_settings.clone())
    }

    pub async fn get_peak_usage_hours(&self, user_id: &str, device_id: &str) -> Vec<u8> {
        let profiles = self.profiles.read().await;
        if let Some(profile) = profiles.get(user_id) {
            if let Some(stats) = profile.device_usage.get(device_id) {
                let mut hours: Vec<(u8, u32)> = stats
                    .usage_by_hour
                    .iter()
                    .enumerate()
                    .map(|(i, &count)| (i as u8, count))
                    .collect();
                hours.sort_by(|a, b| b.1.cmp(&a.1));
                return hours.iter().take(3).map(|(h, _)| *h).collect();
            }
        }
        Vec::new()
    }

    pub async fn set_learning_enabled(&self, enabled: bool) {
        let mut learning = self.learning_enabled.write().await;
        *learning = enabled;
    }

    pub async fn is_learning_enabled(&self) -> bool {
        *self.learning_enabled.read().await
    }

    pub async fn export_profile(&self, user_id: &str) -> Option<String> {
        let profiles = self.profiles.read().await;
        profiles
            .get(user_id)
            .and_then(|p| serde_json::to_string(p).ok())
    }

    pub async fn import_profile(&self, json: &str) -> Result<UserProfile> {
        let profile: UserProfile = serde_json::from_str(json)?;
        let user_id = profile.user_id.clone();

        let mut profiles = self.profiles.write().await;
        profiles.insert(user_id.clone(), profile.clone());

        tracing::info!("Imported profile for user: {}", user_id);
        Ok(profile)
    }
}

impl Default for UserProfileEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get_profile() {
        let engine = UserProfileEngine::new();
        let profile = engine.create_profile("user-1").await.unwrap();

        assert_eq!(profile.user_id, "user-1");

        let retrieved = engine.get_profile("user-1").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn set_current_user() {
        let engine = UserProfileEngine::new();
        engine.create_profile("user-2").await.unwrap();

        engine.set_current_user("user-2").await.unwrap();

        let current = engine.get_current_profile().await;
        assert!(current.is_some());
        assert_eq!(current.unwrap().user_id, "user-2");
    }

    #[tokio::test]
    async fn record_device_usage() {
        let engine = UserProfileEngine::new();
        engine.create_profile("user-3").await.unwrap();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(24));

        engine
            .record_device_usage("user-3", "ac-1", "set_temperature", &params, Some(3600.0))
            .await
            .unwrap();

        let profile = engine.get_profile("user-3").await.unwrap();
        let stats = profile.device_usage.get("ac-1").unwrap();

        assert_eq!(stats.total_uses, 1);
        assert_eq!(stats.average_duration_secs, 3600.0);
    }

    #[tokio::test]
    async fn learn_pattern() {
        let engine = UserProfileEngine::new();
        engine.create_profile("user-4").await.unwrap();

        let trigger = PatternTrigger {
            trigger_type: "time".to_string(),
            conditions: vec![("hour".to_string(), serde_json::json!(22))]
                .into_iter()
                .collect(),
        };

        let actions = vec![PatternAction {
            device_id: "light-1".to_string(),
            action: "power_off".to_string(),
            parameters: HashMap::new(),
        }];

        let pattern_id = engine
            .learn_pattern("user-4", trigger, actions)
            .await
            .unwrap();
        assert!(!pattern_id.is_empty());

        let patterns = engine.get_patterns("user-4").await;
        assert_eq!(patterns.len(), 1);
    }

    #[tokio::test]
    async fn export_import_profile() {
        let engine = UserProfileEngine::new();
        engine.create_profile("user-5").await.unwrap();

        let exported = engine.export_profile("user-5").await.unwrap();
        assert!(!exported.is_empty());

        engine.profiles.write().await.remove("user-5");

        let imported = engine.import_profile(&exported).await.unwrap();
        assert_eq!(imported.user_id, "user-5");
    }

    #[tokio::test]
    async fn learning_toggle() {
        let engine = UserProfileEngine::new();
        assert!(engine.is_learning_enabled().await);

        engine.set_learning_enabled(false).await;
        assert!(!engine.is_learning_enabled().await);
    }
}
