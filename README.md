# brokerage-api

A Rust library for integrating with various brokerage APIs, starting with a comprehensive implementation for Schwab. This library aims to provide a robust and easy-to-use interface for accessing market data, managing authentication, and eventually supporting trading operations.

## Current Status

This library is currently **under development**. The primary focus is on the Schwab Market Data Production API, which is partially implemented. Contributions are welcome!

## Features

*   **Schwab API Integration**:
    *   Authentication and token management (OAuth 2.0).
    *   Market data access:
        *   Real-time quotes for symbols.
        *   Options chain retrieval.
        *   Historical price data.
        *   Market movers.
        *   Market hours information.
        *   Instrument search by symbol, description, and CUSIP.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
brokerage-api = "0.1.5" # Or the latest version
tokio = { version = "1", features = ["full"] } # Required for async operations
reqwest = { version = "0.11", features = ["json"] } # For HTTP requests
anyhow = "1.0" # For error handling
```

## Usage

### Schwab API Authentication

To use the Schwab API, you must first authorize the application to obtain access and refresh tokens. These tokens are stored locally in a `tokens.json` file.

```rust
use std::sync::Arc;
use brokerage_api::SchwabAuth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_auth = SchwabAuth::new(client, "tokens.json".to_string());

    let app_key = std::env::var("SCHWAB_APP_KEY")
        .expect("SCHWAB_APP_KEY environment variable not set");
    let app_secret = std::env::var("SCHWAB_APP_SECRET")
        .expect("SCHWAB_APP_SECRET environment variable not set");

    // Authorize the application (first-time setup)
    schwab_auth.authorize(&app_key, &app_secret).await?;

    // To refresh tokens (after initial authorization)
    // schwab_auth.refresh_tokens(&app_key, &app_secret).await?;

    println!("Schwab API authorization successful!");

    Ok(())
}
```

**Authorization Steps:**

1.  Run the `authorize` function.
2.  The application will print an authorization URL to your console. Copy this URL and paste it into your web browser.
3.  Log in with your Schwab portfolio credentials and grant authorization to the application.
4.  You will be redirected to a blank page (e.g., `https://127.0.0.1`). Copy the **FULL URL** from your browser's address bar.
5.  Paste this URL back into your terminal when prompted and press Enter.
6.  The application will exchange the authorization code for access and refresh tokens, saving them to `tokens.json`.

### Schwab API Market Data Usage

Once authorized, you can use the `SchwabApi` client to access various market data endpoints.

#### Get Quotes for Multiple Symbols

```rust
use std::sync::Arc;
use brokerage_api::{SchwabApi, QuoteFields};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_api = SchwabApi::new(client, "tokens.json".to_string());

    let symbols = vec!["AAPL".to_string(), "GOOG".to_string()];
    let fields = Some(vec![QuoteFields::Quote, QuoteFields::Fundamental]);
    let indicative = Some(false);

    let quotes = schwab_api.get_quotes(symbols, fields, indicative).await?;
    println!("Quotes: {}", serde_json::to_string_pretty(&quotes)?);

    Ok(())
}
```

#### Get Options Chains

```rust
use std::sync::Arc;
use brokerage_api::{SchwabApi, ContractType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_api = SchwabApi::new(client, "tokens.json".to_string());

    let symbol = "SPY".to_string();
    let contract_type = ContractType::All;
    let strike_count = 5;
    let include_underlying_quote = true;

    let chains = schwab_api.get_chains(symbol, contract_type, strike_count, include_underlying_quote).await?;
    println!("Option Chains: {}", serde_json::to_string_pretty(&chains)?);

    Ok(())
}
```

#### Get Price History

```rust
use std::sync::Arc;
use chrono::{Utc, Duration};
use brokerage_api::{SchwabApi, PeriodType, FrequencyType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_api = SchwabApi::new(client, "tokens.json".to_string());

    let symbol = "MSFT".to_string();
    let period_type = Some(PeriodType::Month);
    let period = Some(1); // 1 month
    let frequency_type = Some(FrequencyType::Daily);
    let frequency = Some(1); // Daily frequency
    let end_date = Some(Utc::now());
    let start_date = Some(Utc::now() - Duration::days(30)); // Last 30 days

    let history = schwab_api.price_history(
        symbol,
        period_type,
        period,
        frequency_type,
        frequency,
        start_date,
        end_date,
        Some(false), // No extended hours data
        Some(true),  // Need previous close
    ).await?;
    println!("Price History: {}", serde_json::to_string_pretty(&history)?);

    Ok(())
}
```

## Contributing

We welcome contributions to this library! If you're interested in:

*   Implementing more Schwab API endpoints.
*   Adding support for other brokerage APIs.
*   Improving existing features or documentation.
*   Fixing bugs.

Please feel free to open an issue or submit a pull request.
