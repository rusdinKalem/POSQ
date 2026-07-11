use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;
use chrono::{DateTime, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LicenseToken {
    pub merchant_id: String,
    pub device_id: String,
    pub plan: String,
    pub entitlements: Vec<String>,
    pub issued_at: String,
    pub valid_until: String,
    pub grace_until: String,
    pub status: String,
    pub signature: String, // Hex encoded signature
}

impl LicenseToken {
    pub fn canonical_string(&self) -> String {
        format!(
            "{}|{}|{}|{:?}|{}|{}|{}|{}",
            self.merchant_id,
            self.device_id,
            self.plan,
            self.entitlements,
            self.issued_at,
            self.valid_until,
            self.grace_until,
            self.status
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct LicenseStateResult {
    pub mode: String, // "Active", "Grace", "RestrictedExpired", "Unlicensed", "SuspiciousTime"
    pub token: Option<LicenseToken>,
    pub error: Option<String>,
}

fn get_license_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("POSQ");
    fs::create_dir_all(&path).unwrap_or(());
    path.push("license.json");
    path
}

fn get_last_seen_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("POSQ");
    fs::create_dir_all(&path).unwrap_or(());
    path.push("last_seen_time.txt");
    path
}

// Fixed public key of the Control Plane for verification
const SERVER_PUBLIC_KEY: [u8; 32] = [
    138, 165, 194, 189, 11, 14, 75, 103, 100, 102, 177, 229, 105, 66, 70, 127, 43, 47, 54, 4, 36, 
    253, 91, 154, 207, 20, 179, 236, 159, 190, 151, 128
];

#[command]
pub async fn activate_device(device_name: String) -> Result<LicenseStateResult, String> {
    // Generate private key for signing (mocking server behavior)
    let mock_server_private_key: [u8; 32] = [
        19, 138, 78, 187, 221, 203, 100, 22, 8, 28, 92, 23, 14, 152, 14, 165, 69, 252, 79, 205, 190, 
        240, 67, 15, 255, 93, 111, 165, 49, 3, 169, 214
    ];
    let signing_key = SigningKey::from_bytes(&mock_server_private_key);
    
    // Ensure it matches the compiled public key
    assert_eq!(signing_key.verifying_key().to_bytes(), SERVER_PUBLIC_KEY);

    let mut mock_token = LicenseToken {
        merchant_id: "merchant-123".into(),
        device_id: format!("device-{}-456", device_name),
        plan: "pro".into(),
        entitlements: vec!["offline_checkout".into(), "inventory".into()],
        issued_at: Utc::now().to_rfc3339(),
        valid_until: (Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
        grace_until: (Utc::now() + chrono::Duration::days(33)).to_rfc3339(),
        status: "active".into(),
        signature: "".into(),
    };

    // Sign the canonical representation of the token
    let message = mock_token.canonical_string();
    let signature = signing_key.sign(message.as_bytes());
    mock_token.signature = hex::encode(signature.to_bytes());

    let path = get_license_path();
    let json_str = serde_json::to_string_pretty(&mock_token).map_err(|e| e.to_string())?;
    fs::write(&path, json_str).map_err(|e| e.to_string())?;

    // Reset last seen time on new activation
    let last_seen_path = get_last_seen_path();
    let _ = fs::write(&last_seen_path, Utc::now().to_rfc3339());

    Ok(LicenseStateResult {
        mode: "Active".into(),
        token: Some(mock_token),
        error: None,
    })
}

#[command]
pub async fn verify_license() -> Result<LicenseStateResult, String> {
    let path = get_license_path();
    
    if !path.exists() {
        return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("No license found. Please activate device.".into()),
        });
    }

    let json_str = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("Failed to read license file.".into()),
        }),
    };

    let token: LicenseToken = match serde_json::from_str(&json_str) {
        Ok(t) => t,
        Err(_) => return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: None,
            error: Some("Invalid license format.".into()),
        }),
    };
    
    // M13 QA Hardening / Blocker 3: Verify signature using Ed25519
    let verifying_key = VerifyingKey::from_bytes(&SERVER_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key configuration: {}", e))?;
    
    let sig_bytes = hex::decode(&token.signature)
        .map_err(|_| "Signature field is not valid hex".to_string())?;
    
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|_| "Signature must be 64 bytes".to_string())?;
    
    let message = token.canonical_string();
    if verifying_key.verify(message.as_bytes(), &signature).is_err() {
        return Ok(LicenseStateResult {
            mode: "Unlicensed".into(),
            token: Some(token),
            error: Some("License token signature is invalid. TAMPERED!".into()),
        });
    }

    let now = Utc::now();

    // SEC-002: Clock Rollback Bypass Vulnerability Fix
    let last_seen_path = get_last_seen_path();
    let last_seen = if last_seen_path.exists() {
        let content = fs::read_to_string(&last_seen_path).unwrap_or_default();
        DateTime::parse_from_rfc3339(content.trim())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc))
    } else {
        DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc)
    };

    if now < last_seen {
        return Ok(LicenseStateResult {
            mode: "SuspiciousTime".into(),
            token: Some(token),
            error: Some("Clock rollback detected. Please correct your system time.".into()),
        });
    }

    // Save current time as last seen
    let _ = fs::write(&last_seen_path, now.to_rfc3339());

    let valid_until = DateTime::parse_from_rfc3339(&token.valid_until)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or(now);
    let grace_until = DateTime::parse_from_rfc3339(&token.grace_until)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or(now);

    let mut mode = "Active".to_string();
    if now > grace_until {
        mode = "RestrictedExpired".to_string();
    } else if now > valid_until {
        mode = "Grace".to_string();
    }

    Ok(LicenseStateResult {
        mode,
        token: Some(token),
        error: None,
    })
}

// SEC-001: Centralized Active License Enforcer
pub async fn enforce_active_license() -> Result<(), String> {
    let state = verify_license().await?;
    if state.mode == "RestrictedExpired" || state.mode == "Unlicensed" || state.mode == "SuspiciousTime" {
        return Err(format!(
            "AKSES DITOLAK: Mode lisensi saat ini ({}) memblokir operasi ini. Silakan periksa koneksi internet atau perbarui langganan Anda.",
            state.mode
        ));
    }
    Ok(())
}

#[command]
pub async fn refresh_license() -> Result<LicenseStateResult, String> {
    // Stub implementation: Simulate contacting CP server to renew token
    let mut res = verify_license().await?;
    
    if let Some(mut token) = res.token {
        // Sign renewal using mock server private key
        let mock_server_private_key: [u8; 32] = [
            19, 138, 78, 187, 221, 203, 100, 22, 8, 28, 92, 23, 14, 152, 14, 165, 69, 252, 79, 205, 190, 
            240, 67, 15, 255, 93, 111, 165, 49, 3, 169, 214
        ];
        let signing_key = SigningKey::from_bytes(&mock_server_private_key);

        token.valid_until = (Utc::now() + chrono::Duration::days(30)).to_rfc3339();
        token.grace_until = (Utc::now() + chrono::Duration::days(33)).to_rfc3339();
        token.issued_at = Utc::now().to_rfc3339();

        // Sign new token
        token.signature = "".into();
        let message = token.canonical_string();
        let signature = signing_key.sign(message.as_bytes());
        token.signature = hex::encode(signature.to_bytes());

        let path = get_license_path();
        let json_str = serde_json::to_string_pretty(&token).map_err(|e| e.to_string())?;
        fs::write(&path, json_str).map_err(|e| e.to_string())?;

        // Reset last seen
        let last_seen_path = get_last_seen_path();
        let _ = fs::write(&last_seen_path, Utc::now().to_rfc3339());

        res.token = Some(token);
        res.mode = "Active".into();
    } else {
        return Err("Cannot refresh: no active license found to renew.".into());
    }

    Ok(res)
}
