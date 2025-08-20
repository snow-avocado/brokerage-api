# NOTE
This library is currently **under development**, and the only supported brokerage API is Schwab (Market Data Production), which is partially implemented.
Contributions are welcome!

# brokerage-api
A Rust library for integrating with various brokerage APIs.

## Schwab API

### Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
brokerage-api = "0.1.3"
```

### Authentication

To use the Schwab API, you must first authorize the application. This is done by running the `authorize` function, which will prompt you to log in to your Schwab account and authorize the application.

```rust
use std::sync::Arc;
use brokerage_api::SchwabAuth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_auth = SchwabAuth::new(client);

    let app_key = std::env::var("SCHWAB_APP_KEY")?;
    let app_secret = std::env::var("SCHWAB_APP_SECRET")?;

    schwab_auth.authorize(&app_key, &app_secret).await?;

    Ok(())
}
```

After running this code, you will be prompted to open a URL in your browser. After logging in and authorizing the application, you will be redirected to a blank page. Copy the URL of this page and paste it into the terminal. The application will then retrieve and store your access and refresh tokens in a `tokens.json` file.

### Usage

Once you have authorized the application, you can use the `SchwabApi` to make requests to the Schwab API.

#### Get Quotes

```rust
use std::sync::Arc;
use brokerage_api::{SchwabApi, QuoteFields};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_api = SchwabApi::new(client);

    let symbols = vec!["AAPL".to_string(), "GOOG".to_string()];
    let fields = Some(vec![QuoteFields::Quote, QuoteFields::Fundamental]);
    let indicative = Some(false);

    schwab_api.get_quotes(symbols, fields, indicative).await?;

    Ok(())
}
```

#### Get Option Chains

```rust
use std::sync::Arc;
use brokerage_api::{SchwabApi, ContractType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(reqwest::Client::new());
    let schwab_api = SchwabApi::new(client);

    let symbol = "SPY".to_string();
    let contract_type = ContractType::All;
    let strike_count = 5;
    let include_underlying_quote = true;

    schwab_api.get_chains(symbol, contract_type, strike_count, include_underlying_quote).await?;

    Ok(())
}
