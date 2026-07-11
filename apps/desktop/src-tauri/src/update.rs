use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{command, State};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateMetadata {
    pub version: String,
    pub release_notes: String,
    pub download_url: String,
    pub signature: String,
    pub channel: String,
}

impl UpdateMetadata {
    pub fn canonical_string(&self) -> String {
        format!(
            "{}|{}|{}",
            self.version,
            self.download_url,
            self.channel
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub success: bool,
    pub update_available: bool,
    pub metadata: Option<UpdateMetadata>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SafeMigrationResult {
    pub success: bool,
    pub message: String,
}

// Separate Public Key for Update Signature validation (DEC-054 Compliance)
const UPDATE_PUBLIC_KEY: [u8; 32] = [
    96, 145, 14, 76, 146, 62, 190, 168, 8, 174, 137, 137, 122, 140, 102, 75, 244, 40, 125, 147, 91, 
    121, 202, 75, 37, 152, 224, 121, 175, 154, 106, 90
];

#[command]
pub async fn check_update(channel: String) -> Result<UpdateCheckResult, String> {
    // M10-001 Version check
    let current_version = env!("CARGO_PKG_VERSION");
    let mock_new_version = "1.1.0"; // Suppose we are at 1.0.0

    if current_version == mock_new_version {
        return Ok(UpdateCheckResult {
            success: true,
            update_available: false,
            metadata: None,
            error: None,
        });
    }

    // Sign using update private key (mocking server behavior)
    let mock_server_private_key: [u8; 32] = [
        164, 218, 5, 5, 153, 234, 82, 155, 205, 41, 152, 189, 213, 88, 227, 39, 70, 90, 222, 93, 157, 
        180, 50, 95, 218, 17, 222, 168, 171, 52, 182, 199
    ];
    let signing_key = SigningKey::from_bytes(&mock_server_private_key);
    assert_eq!(signing_key.verifying_key().to_bytes(), UPDATE_PUBLIC_KEY);

    let mut metadata = UpdateMetadata {
        version: mock_new_version.into(),
        release_notes: "Minor bug fixes and performance improvements.".into(),
        download_url: format!("https://updates.posq.example.com/posq-{}-setup.exe", mock_new_version),
        signature: "".into(),
        channel,
    };

    let message = metadata.canonical_string();
    let signature = signing_key.sign(message.as_bytes());
    metadata.signature = hex::encode(signature.to_bytes());

    Ok(UpdateCheckResult {
        success: true,
        update_available: true,
        metadata: Some(metadata),
        error: None,
    })
}

#[command]
pub async fn validate_update(metadata: UpdateMetadata, signature: String) -> Result<bool, String> {
    // M10-002 Signed update validation using Ed25519 (Blocker 3 Fixed)
    let verifying_key = VerifyingKey::from_bytes(&UPDATE_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key configuration: {}", e))?;
    
    let sig_bytes = hex::decode(&signature)
        .map_err(|_| "Signature is not valid hex".to_string())?;
    
    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|_| "Signature must be 64 bytes".to_string())?;
    
    let message = metadata.canonical_string();
    
    verifying_key.verify(message.as_bytes(), &signature)
        .map(|_| true)
        .map_err(|e| format!("Update signature validation failed: {}", e))
}

#[command]
pub async fn run_safe_migration(pool: State<'_, SqlitePool>) -> Result<SafeMigrationResult, String> {
    // M10-003 Migration backup gate
    println!("Starting safe migration. Taking pre-migration backup...");
    
    // We create an unencrypted backup for safety during migration
    let backup_res = crate::backup::create_local_backup(false, None).await;
    
    if let Err(e) = backup_res {
        return Err(format!("MIGRATION ABORTED: Pre-migration backup failed: {}", e));
    }
    
    let backup_data = backup_res.unwrap();
    if !backup_data.success {
        return Err(format!("MIGRATION ABORTED: Pre-migration backup returned failure: {}", backup_data.message));
    }
 
    println!("Backup successful at {:?}. Running SQL migrations...", backup_data.path);

    // Run migrations
    match crate::migration::run_migrations(pool.inner()).await {
        Ok(_) => Ok(SafeMigrationResult {
            success: true,
            message: "Migration completed successfully.".into(),
        }),
        Err(e) => {
            // M10-004 Failed migration recovery
            let err_msg = format!("MIGRATION FAILED! The system may be in an unstable state. Please restore the backup from: {:?} \nError details: {}", backup_data.path, e);
            Err(err_msg)
        }
    }
}
