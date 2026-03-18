use crate::device::DeviceManager;
use crate::error::Result;
use crate::profile::{BehaviorPattern, PatternAction, UserProfileEngine};
use crate::types::{DeviceInfo, DeviceType};
use chrono::{Datelike, DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub actions: Vec<RecommendedAction>,
    pub confidence: f32,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    EnergySaving,
    Comfort,
    Security,
    Convenience,
    Routine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub device_id: String,
    pub device_name: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

pub struct RecommendationEngine {
    profile_engine: Arc<RwLock<UserProfileEngine>>,
    recommendation_history: Arc<RwLock<Vec<Recommendation>>>,
    rules: Vec<RecommendationRule>,
}

struct RecommendationRule {
    name: String,
    condition: Box<dyn Fn(&RecommendationContext) -> bool + Send + Sync>,
    generate: Box<dyn Fn(&RecommendationContext) -> Option<Recommendation> + Send + Sync>,
}

struct RecommendationContext {
    hour: u32,
    day_of_week: u32,
    devices: Vec<DeviceInfo>,
    user_preferences: Option<crate::types::UserPreferences>,
    recent_patterns: Vec<BehaviorPattern>,
}

impl RecommendationEngine {
    pub fn new(profile_engine: Arc<RwLock<UserProfileEngine>>) -> Self {
        Self {
            profile_engine,
            recommendation_history: Arc::new(RwLock::new(Vec::new())),
            rules: Vec::new(),
        }
    }

    pub fn with_default_profile() -> Self {
        Self::new(Arc::new(RwLock::new(UserProfileEngine::new())))
    }

    pub async fn generate_recommendations(
        &self,
        user_id: &str,
        devices: &[DeviceInfo],
    ) -> Result<Vec<Recommendation>> {
        let now = Utc::now();
        let hour = now.hour();
        let day_of_week = now.weekday().num_days_from_monday();

        let profile_engine = self.profile_engine.read().await;
        let user_preferences = profile_engine
            .get_profile(user_id)
            .await
            .map(|p| p.preferences);
        let recent_patterns = profile_engine.get_patterns(user_id).await;
        drop(profile_engine);

        let context = RecommendationContext {
            hour,
            day_of_week,
            devices: devices.to_vec(),
            user_preferences,
            recent_patterns,
        };

        let mut recommendations = Vec::new();

        recommendations.extend(self.check_time_based_rules(&context));
        recommendations.extend(self.check_energy_rules(&context));
        recommendations.extend(self.check_pattern_rules(&context));
        recommendations.extend(self.check_comfort_rules(&context));

        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut history = self.recommendation_history.write().await;
        for rec in &recommendations {
            history.push(rec.clone());
        }

        Ok(recommendations)
    }

    fn check_time_based_rules(&self, context: &RecommendationContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if context.hour >= 22 || context.hour < 6 {
            for device in &context.devices {
                if device.device_type == DeviceType::Light && device.state.online {
                    recommendations.push(Recommendation {
                        id: uuid::Uuid::new_v4().to_string(),
                        recommendation_type: RecommendationType::Routine,
                        title: "Bedtime Routine".to_string(),
                        description: "Turn off lights for bedtime".to_string(),
                        actions: vec![RecommendedAction {
                            device_id: device.id.clone(),
                            device_name: device.name.clone(),
                            action: "power_off".to_string(),
                            parameters: HashMap::new(),
                        }],
                        confidence: 0.9,
                        reason: "It's late at night and lights are still on".to_string(),
                        created_at: Utc::now(),
                    });
                }
            }
        }

        if context.hour >= 7 && context.hour < 9 {
            for device in &context.devices {
                if device.device_type == DeviceType::Curtain {
                    recommendations.push(Recommendation {
                        id: uuid::Uuid::new_v4().to_string(),
                        recommendation_type: RecommendationType::Routine,
                        title: "Morning Routine".to_string(),
                        description: "Open curtains for morning".to_string(),
                        actions: vec![RecommendedAction {
                            device_id: device.id.clone(),
                            device_name: device.name.clone(),
                            action: "open".to_string(),
                            parameters: HashMap::new(),
                        }],
                        confidence: 0.85,
                        reason: "Morning time - open curtains for natural light".to_string(),
                        created_at: Utc::now(),
                    });
                }
            }
        }

        recommendations
    }

    fn check_energy_rules(&self, context: &RecommendationContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        for device in &context.devices {
            if device.device_type == DeviceType::AirConditioner && device.state.online {
                if let Some(temp) = device.state.temperature {
                    if temp < 23.0 {
                        recommendations.push(Recommendation {
                            id: uuid::Uuid::new_v4().to_string(),
                            recommendation_type: RecommendationType::EnergySaving,
                            title: "Energy Saving Suggestion".to_string(),
                            description: format!(
                                "AC temperature is {}°C. Consider raising to 24°C to save energy",
                                temp
                            ),
                            actions: vec![RecommendedAction {
                                device_id: device.id.clone(),
                                device_name: device.name.clone(),
                                action: "set_temperature".to_string(),
                                parameters: vec![(
                                    "temperature".to_string(),
                                    serde_json::json!(24),
                                )]
                                .into_iter()
                                .collect(),
                            }],
                            confidence: 0.75,
                            reason: "Lower AC temperatures consume more energy".to_string(),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
        }

        recommendations
    }

    fn check_pattern_rules(&self, context: &RecommendationContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        for pattern in &context.recent_patterns {
            if pattern.confidence > 0.8 {
                let matches = pattern
                    .trigger
                    .conditions
                    .get("hour")
                    .and_then(|h: &serde_json::Value| h.as_u64())
                    .map(|h| h as u32 == context.hour)
                    .unwrap_or(false);

                if matches {
                    let actions: Vec<RecommendedAction> = pattern
                        .actions
                        .iter()
                        .map(|a| {
                            let device_name = context
                                .devices
                                .iter()
                                .find(|d| d.id == a.device_id)
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| a.device_id.clone());

                            RecommendedAction {
                                device_id: a.device_id.clone(),
                                device_name,
                                action: a.action.clone(),
                                parameters: a.parameters.clone(),
                            }
                        })
                        .collect();

                    if !actions.is_empty() {
                        recommendations.push(Recommendation {
                            id: uuid::Uuid::new_v4().to_string(),
                            recommendation_type: RecommendationType::Routine,
                            title: "Suggested Action".to_string(),
                            description: "Based on your usual routine".to_string(),
                            actions,
                            confidence: pattern.confidence,
                            reason: "This matches your learned behavior pattern".to_string(),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
        }

        recommendations
    }

    fn check_comfort_rules(&self, context: &RecommendationContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if let Some(ref prefs) = context.user_preferences {
            let season = Self::determine_season(context.day_of_week, context.hour);

            for device in &context.devices {
                if device.device_type == DeviceType::AirConditioner && device.state.online {
                    let preferred_temp = match season {
                        Season::Summer => prefs.temperature.preferred_summer,
                        Season::Winter => prefs.temperature.preferred_winter,
                        _ => 24.0,
                    };

                    if let Some(current_temp) = device.state.temperature {
                        if (current_temp - preferred_temp).abs() > 1.0 {
                            recommendations.push(Recommendation {
                                id: uuid::Uuid::new_v4().to_string(),
                                recommendation_type: RecommendationType::Comfort,
                                title: "Comfort Suggestion".to_string(),
                                description: format!(
                                    "Adjust AC to your preferred {}°C",
                                    preferred_temp
                                ),
                                actions: vec![RecommendedAction {
                                    device_id: device.id.clone(),
                                    device_name: device.name.clone(),
                                    action: "set_temperature".to_string(),
                                    parameters: vec![(
                                        "temperature".to_string(),
                                        serde_json::json!(preferred_temp),
                                    )]
                                    .into_iter()
                                    .collect(),
                                }],
                                confidence: 0.8,
                                reason: "Based on your temperature preferences".to_string(),
                                created_at: Utc::now(),
                            });
                        }
                    }
                }
            }
        }

        recommendations
    }

    fn determine_season(day: u32, hour: u32) -> Season {
        Season::Summer
    }

    pub async fn get_history(&self, limit: usize) -> Vec<Recommendation> {
        let history = self.recommendation_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    pub async fn clear_history(&self) {
        let mut history = self.recommendation_history.write().await;
        history.clear();
    }

    pub async fn dismiss_recommendation(&self, recommendation_id: &str) -> bool {
        let mut history = self.recommendation_history.write().await;
        let initial_len = history.len();
        history.retain(|r| r.id != recommendation_id);
        history.len() < initial_len
    }
}

enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new(Arc::new(RwLock::new(UserProfileEngine::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConnectionProtocol, DeviceState};

    fn create_test_device(id: &str, device_type: DeviceType, online: bool) -> DeviceInfo {
        DeviceInfo {
            id: id.to_string(),
            name: format!("Test {}", id),
            device_type,
            capabilities: vec!["power".to_string()],
            endpoint: "192.168.1.100:8080".to_string(),
            port: 8080,
            protocol: ConnectionProtocol::HTTP,
            last_seen: Utc::now(),
            state: DeviceState {
                online,
                temperature: Some(22.0),
                ..Default::default()
            },
        }
    }

    #[tokio::test]
    async fn engine_creation() {
        let engine = RecommendationEngine::default();
        let history = engine.get_history(10).await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn generate_recommendations() {
        let profile_engine = Arc::new(RwLock::new(UserProfileEngine::new()));
        profile_engine
            .write()
            .await
            .create_profile("user-1")
            .await
            .unwrap();

        let engine = RecommendationEngine::new(profile_engine);

        let devices = vec![
            create_test_device("light-1", DeviceType::Light, true),
            create_test_device("ac-1", DeviceType::AirConditioner, true),
        ];

        let recommendations = engine
            .generate_recommendations("user-1", &devices)
            .await
            .unwrap();

        assert!(!recommendations.is_empty());
    }

    #[tokio::test]
    async fn recommendation_history() {
        let profile_engine = Arc::new(RwLock::new(UserProfileEngine::new()));
        profile_engine
            .write()
            .await
            .create_profile("user-1")
            .await
            .unwrap();

        let engine = RecommendationEngine::new(profile_engine);

        let devices = vec![create_test_device("light-1", DeviceType::Light, true)];

        engine
            .generate_recommendations("user-1", &devices)
            .await
            .unwrap();

        let history = engine.get_history(10).await;
        assert!(!history.is_empty());

        engine.clear_history().await;
        let history = engine.get_history(10).await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn dismiss_recommendation() {
        let profile_engine = Arc::new(RwLock::new(UserProfileEngine::new()));
        profile_engine
            .write()
            .await
            .create_profile("user-1")
            .await
            .unwrap();

        let engine = RecommendationEngine::new(profile_engine);

        let devices = vec![create_test_device("light-1", DeviceType::Light, true)];

        let recs = engine
            .generate_recommendations("user-1", &devices)
            .await
            .unwrap();

        if !recs.is_empty() {
            let dismissed = engine.dismiss_recommendation(&recs[0].id).await;
            assert!(dismissed);
        }
    }
}
