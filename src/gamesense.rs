// GameSense Server Module
// HTTP API compatible with SteelSeries GameSense protocol

use axum::{
    extract::State,
    routing::{get},
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// GameSense state structure
#[derive(Debug, Clone, Default)]
pub struct GameSenseState {
    /// Battery levels for connected devices
    pub battery_levels: BATTERY_MAP<String, i32>,
    
    /// Device temperatures
    pub temperatures: HashMap<String, i32>, // in Celsius
    
    /// Volume levels
    pub volume_levels: VolumeLevels,
    
    /// Device activity status
    pub active_devices: Vec<String>,
}

pub type BATTERY_MAP<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug, Clone, Default)]
pub struct VolumeLevels {
    pub master: i32, // percentage 0-100
    pub game: i32,   // percentage 0-100
    pub chat: i32,   // percentage 0-100
}

/// GameSense endpoint data
#[derive(Debug, Clone, Serialize)]
pub struct EndpointData {
    #[serde(rename = "type")]
    pub endpoint_type: String,
    
    pub key: String,
    
    pub value: serde_json::Value,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Create new GameSense state
pub fn create_gamesense_state(active_devices: Vec<String>) -> Arc<RwLock<GameSenseState>> {
    Arc::new(RwLock::new(GameSenseState {
        active_devices,
        ..Default::default()
    }))
}

/// Custom error types
#[derive(Error, Debug)]
pub enum GameSenseError {
    #[error("Server start failed: {0}")]
    ServerStart(String),
    
    #[error("Invalid request format")]
    InvalidRequest,
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

pub type GameSenseResult<T> = Result<T, GameSenseError>;

/// GameSenseServer implementation
pub struct GameSenseServer {
    bind_address: String,
    port: u16,
}

impl GameSenseServer {
    pub fn new() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 27301,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_bind(mut self, address: String) -> Self {
        self.bind_address = address;
        self
    }

    /// Create Axum router
    pub fn create_router(&self, state: Arc<RwLock<GameSenseState>>) -> Router {
        Router::new()
            .route("/state", get(handle_state))
            .route("/battery", get(handle_battery))
            .route("/volume", get(handle_volume).post(update_volume))
            .route("/temperature", get(handle_temperature))
            .route("/devices", get(handle_devices))
            .with_state(state)
    }

    /// Start the server
    pub async fn serve(self, state: Arc<RwLock<GameSenseState>>) -> GameSenseResult<()> {
        let addr = format!("{}:{}", self.bind_address, self.port);
        let app = self.create_router(state);
        
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| GameSenseError::ServerStart(e.to_string()))?;
        
        tracing::info!("GameSense server starting on {}", addr);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| GameSenseError::ServerStart(e.to_string()))?;
        
        Ok(())
    }
}

// Request handlers
async fn handle_state(State(state): State<Arc<RwLock<GameSenseState>>>) 
    -> axum::response::Json<serde_json::Value>
{
    let state_read = state.read().unwrap();
    
    let endpoints: Vec<EndpointData> = vec![
        EndpointData {
            endpoint_type: "device".to_string(),
            key: "status".to_string(),
            value: serde_json::json!(true),
            label: Some("Active".to_string()),
        },
    ];
    
    axum::response::Json(serde_json::json!({ "endpoints": endpoints }))
}

async fn handle_battery(State(state): State<Arc<RwLock<GameSenseState>>>) 
    -> axum::response::Json<serde_json::Value>
{
    let state_read = state.read().unwrap();
    
    let endpoints: Vec<EndpointData> = state_read.battery_levels
        .iter()
        .map(|(dev_id, level)| EndpointData {
            endpoint_type: "battery".to_string(),
            key: dev_id.clone(),
            value: serde_json::json!(*level),
            label: Some(format!("Battery: {}%", level)),
        })
        .collect();
    
    axum::response::Json(serde_json::json!({ "endpoints": endpoints }))
}

async fn handle_volume(State(state): State<Arc<RwLock<GameSenseState>>>) 
    -> axum::response::Json<serde_json::Value> 
{
    let state_read = state.read().unwrap();
    
    let endpoints: Vec<EndpointData> = vec![
        EndpointData {
            endpoint_type: "volume".to_string(),
            key: "master".to_string(),
            value: serde_json::json!(state_read.volume_levels.master),
            label: Some(format!("Master: {}%", state_read.volume_levels.master)),
        },
        EndpointData {
            endpoint_type: "volume".to_string(),
            key: "game".to_string(),
            value: serde_json::json!(state_read.volume_levels.game),
            label: Some(format!("Game: {}%", state_read.volume_levels.game)),
        },
        EndpointData {
            endpoint_type: "volume".to_string(),
            key: "chat".to_string(),
            value: serde_json::json!(state_read.volume_levels.chat),
            label: Some(format!("Chat: {}%", state_read.volume_levels.chat)),
        },
    ];
    
    axum::response::Json(serde_json::json!({ "endpoints": endpoints }))
}

async fn update_volume(
    State(state): State<Arc<RwLock<GameSenseState>>>,
    body: axum::Form<serde_json::Value>,
) -> axum::response::Json<serde_json::Value> {
    // Parse request body
    // Example: {"channel": "game", "value": 75}
    
    let channel = body.get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("master");
    
    let value = body.get("value")
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
        .unwrap_or(50);
    
    if value < 0 || value > 100 {
        return axum::response::Json(serde_json::json!({"error": "Value must be 0-100"}));
    }
    
    let mut state_write = state.write().unwrap();
    
    {
        match channel {
            "master" => state_write.volume_levels.master = value,
            "game" => state_write.volume_levels.game = value,
            "chat" => state_write.volume_levels.chat = value,
            _ => return axum::response::Json(serde_json::json!({"error": "Invalid channel"})),
        }
    }
    
    axum::response::Json(serde_json::json!({"updated": true}))
}

async fn handle_temperature(State(state): State<Arc<RwLock<GameSenseState>>>) 
    -> axum::response::Json<serde_json::Value>
{
    let state_read = state.read().unwrap();
    
    let endpoints: Vec<EndpointData> = state_read.temperatures
        .iter()
        .map(|(dev_id, temp)| EndpointData {
            endpoint_type: "temp".to_string(),
            key: dev_id.clone(),
            value: serde_json::json!(*temp),
            label: Some(format!("Temp: {}°C", temp)),
        })
        .collect();
    
    axum::response::Json(serde_json::json!({ "endpoints": endpoints }))
}

async fn handle_devices(State(state): State<Arc<RwLock<GameSenseState>>>) 
    -> axum::response::Json<serde_json::Value>
{
    let state_read = state.read().unwrap();
    
    axum::response::Json(serde_json::json!({
        "devices": state_read.active_devices,
        "count": state_read.active_devices.len()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_state() {
        let state = create_gamesense_state(vec!["keyboard_1".to_string()]);
        let read = state.read().unwrap();
        assert!(read.active_devices.contains(&"keyboard_1".to_string()));
    }
    
    #[tokio::test]
    async fn test_server_creation() {
        let state = create_gamesense_state(vec![]);
        let _state_read = state.read().unwrap();
        let server = GameSenseServer::new();
        let router = server.create_router(state);
        // Test passes if we can create the router without panicking
        println!("Router created successfully: {:?}", std::any::type_name_of_val(&router));
    }
}
