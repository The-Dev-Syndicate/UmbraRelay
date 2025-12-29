use std::collections::HashMap;

// For now, we'll use a simple approach with Tauri state
// This will be replaced with proper secure storage
pub type TokenStore = HashMap<i64, String>;

