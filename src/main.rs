mod schwab;
mod util;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tracing::{info, Level};

use crate::schwab::{schwab_api::SchwabApi, schwab_auth::SchwabAuth};

const TOKEN_REFRESH_INTERVAL: u64 = 1800;
const SCHWAB_APP_KEY_ENV_VAR: &str = "SCHWAB_APP_KEY";
const SCHWAB_APP_SECRET_ENV_VAR: &str = "SCHWAB_APP_SECRET";

// Example library usage
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging information.
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Get the application key from environment variables.
    let client_app_key = Arc::new(
        std::env::var(SCHWAB_APP_KEY_ENV_VAR).expect("SCHWAB_APP_KEY environment variable not set"),
    );

    // Get the application secret from environment variables.
    let client_secret = Arc::new(
        std::env::var(SCHWAB_APP_SECRET_ENV_VAR)
            .expect("SCHWAB_APP_SECRET environment variable not set"),
    );

    // Create a shared Reqwest client for making HTTP requests.
    let reqwest_client = Arc::new(reqwest::Client::new());
    let schwab_auth = Arc::new(SchwabAuth::new(Arc::clone(&reqwest_client)));
    let schwab_api = SchwabApi::new(Arc::clone(&reqwest_client));

    // Begin the authorization flow.
    schwab_auth.authorize(&client_app_key, &client_secret).await?;

    loop {
        // Sleep for the specified duration.
        thread::sleep(Duration::from_secs(TOKEN_REFRESH_INTERVAL));

        let schwab_auth_clone = Arc::clone(&schwab_auth);
        let client_app_key_clone = Arc::clone(&client_app_key);
        let client_secret_clone = Arc::clone(&client_secret);

        // Spawn a new async task. The 'move' keyword moves the cloned variables into the task.
        let _handle = tokio::spawn(async move {
            match schwab_auth_clone.refresh_tokens(
                &client_app_key_clone,
                &client_secret_clone,
            )
            .await
            {
                Ok(_) => {
                    info!("Token refresh succeeded!");
                }
                Err(e) => {
                    info!("Token refresh failed: {:?}", e);
                }
            }
        });
    }
}
