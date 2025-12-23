// src/bin_utils.rs
//! Utility functions for finding binaries

use std::path::Path;
use crate::platform;

/// Find binary ONLY in bin folder - no PATH fallback
pub fn find_binary(name: &str) -> Result<String, String> {
    // Check in bin folder first
    let bin_path = Path::new("bin").join(name);
    if bin_path.exists() {
        return Ok(bin_path.to_string_lossy().into_owned());
    }
    
    // Check in bin folder with .exe extension on Windows
    if platform::IS_WINDOWS {
        let bin_exe_path = Path::new("bin").join(format!("{}.exe", name));
        if bin_exe_path.exists() {
            return Ok(bin_exe_path.to_string_lossy().into_owned());
        }
    }
    
    // No fallback to system PATH - strict requirement
    Err(format!("Binary '{}' not found in bin/ folder", name))
}

/// Check if a binary exists in bin folder
pub fn binary_exists_in_bin(name: &str) -> bool {
    let bin_path = Path::new("bin").join(name);
    if bin_path.exists() {
        return true;
    }
    
    if platform::IS_WINDOWS {
        let bin_exe_path = Path::new("bin").join(format!("{}.exe", name));
        return bin_exe_path.exists();
    }
    
    false
}

/// Get list of binaries available in bin folder
pub fn list_binaries_in_bin() -> Vec<String> {
    let mut binaries = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir("bin") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        binaries.push(file_name.to_string());
                    }
                }
            }
        }
    }
    
    binaries
}