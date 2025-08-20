use std::{fmt, fs, sync::Arc};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use crate::{schwab::{common::TOKENS_FILE, schwab_auth::StoredTokenInfo}, util::dedup_ordered};

const SCHWAB_MARKET_DATA_API_URL: &str = "https://api.schwabapi.com/marketdata/v1";

pub enum ContractType {
    Call,
    Put,
    All,
}

impl fmt::Display for ContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractType::All => write!(f, "ALL"),
            ContractType::Call => write!(f, "CALL"),
            ContractType::Put => write!(f, "PUT"),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum QuoteFields {
    Quote,
    Fundamental,
    Extended,
    Reference,
    Regular,
}

impl fmt::Display for QuoteFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteFields::Quote => write!(f, "quote"),
            QuoteFields::Fundamental => write!(f, "fundamental"),
            QuoteFields::Extended => write!(f, "extended"),
            QuoteFields::Reference => write!(f, "reference"),
            QuoteFields::Regular => write!(f, "regular"),
        }
    }
}

pub struct SchwabApi {
    reqwest_client: Arc<Client>,
}

impl SchwabApi {
    pub fn new(reqwest_client: Arc<Client>) -> Self {
        Self { reqwest_client }
    }

    fn construct_request_headers() -> anyhow::Result<HeaderMap, anyhow::Error> {
        let mut headers = HeaderMap::new();

        let json_string = fs::read_to_string(TOKENS_FILE)?;
        let data: StoredTokenInfo = serde_json::from_str(&json_string)?;
        let auth_header = format!("Bearer {}", data.access_token.as_str());

        headers.append("Accept", HeaderValue::from_str("application/json")?);
        headers.append(
            "Authorization",
            HeaderValue::from_str(auth_header.as_str())?,
        );

        Ok(headers)
    }

    pub async fn get_quotes(
        &self,
        symbols: Vec<String>,
        fields: Option<Vec<QuoteFields>>,
        indicative: Option<bool>,
    ) -> anyhow::Result<(), anyhow::Error> {
        let symbols_string = symbols.join(",");
        let fields_string = match fields {
            Some(v) => dedup_ordered(v)
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(","),
            None => "".to_owned(),
        };
        let indicative_string = match indicative {
            Some(v) => v.to_string().to_lowercase(),
            None => "".to_owned(),
        };

        let headers = SchwabApi::construct_request_headers().unwrap();

        let response = self
            .reqwest_client
            .get(format!(
                "{SCHWAB_MARKET_DATA_API_URL}/quotes?symbols={}&fields={}&indicative={}",
                symbols_string, fields_string, indicative_string
            ))
            .headers(headers)
            .send()
            .await?;

        println!("Get quotes response: {:?}", response.status());

        let response_json = response.text().await?;

        println!("Quotes: {:?}", response_json);

        Ok(())
    }

    pub async fn get_chains(
        &self,
        symbol: String,
        contract_type: ContractType,
        strike_count: u64,
        include_underlying_quote: bool,
    ) -> anyhow::Result<(), anyhow::Error> {
        let headers = SchwabApi::construct_request_headers().unwrap();

        let response = self
            .reqwest_client
            .get(format!(
                "{SCHWAB_MARKET_DATA_API_URL}/chains?symbol={}&contractType={}&strikeCount={}&includeUnderlyingQuote={}",
                symbol, contract_type.to_string(), strike_count.to_string(), include_underlying_quote.to_string()
            ))
            .headers(headers)
            .send()
            .await?;

        println!("Get chains response: {:?}", response.status());

        let response_json = response.text().await?;

        println!("Chains: {:?}", response_json);

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokio;

    // NOTE: For these tests to pass, you must have a valid Schwab access token
    // in a file named "tokens.json" in your project's root directory.
    // The format should be:
    // {
    //   "access_token": "YOUR_ACCESS_TOKEN",
    //   "refresh_token": "YOUR_REFRESH_TOKEN",
    //   "expires_in": 1800,
    //   "refresh_token_expires_in": 7776000,
    //   "token_type": "Bearer"
    // }
    // These tests will hit the live Schwab API.

    #[tokio::test]
    async fn test_get_quotes_live() -> Result<()> {
        // Create a reqwest client for making actual HTTP requests.
        let client = Arc::new(reqwest::Client::new());
        let api = SchwabApi::new(client);

        // Call the get_quotes method with real symbols.
        // Make sure these symbols are valid on the Schwab API.
        let symbols = vec!["AAPL".to_string(), "GOOG".to_string()];
        let fields = Some(vec![QuoteFields::Quote, QuoteFields::Fundamental]);
        let indicative = Some(false);

        let result = api.get_quotes(symbols, fields, indicative).await;

        // Assert that the API call was successful.
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_chains_live() -> Result<()> {
        // Create a reqwest client for making actual HTTP requests.
        let client = Arc::new(reqwest::Client::new());
        let api = SchwabApi::new(client);

        // Call the get_chains method with a real symbol.
        let symbol = "SPY".to_string();
        let contract_type = ContractType::All;
        let strike_count = 5;
        let include_underlying_quote = true;

        let result = api.get_chains(symbol, contract_type, strike_count, include_underlying_quote).await;

        // Assert that the API call was successful.
        assert!(result.is_ok());
        
        Ok(())
    }
}