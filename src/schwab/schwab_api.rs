use std::{fmt, sync::Arc};

use chrono::{DateTime, Utc};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;
use urlencoding::encode;

use crate::{schwab::schwab_auth::StoredTokenInfo, util::dedup_ordered};

/// Represents the type of contract for an options chain.
pub enum ContractType {
    /// Call options.
    Call,
    /// Put options.
    Put,
    /// All options (both call and put).
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

/// Represents the fields to be returned in a quote.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum QuoteFields {
    /// Quote data.
    Quote,
    /// Fundamental data.
    Fundamental,
    /// Extended data.
    Extended,
    /// Reference data.
    Reference,
    /// Regular data.
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

/// Represents the period type for price history.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum PeriodType {
    /// Day period type.
    Day,
    /// Month period type.
    Month,
    /// Year period type.
    Year,
    /// Year to date period type.
    Ytd,
}

impl fmt::Display for PeriodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodType::Day => write!(f, "day"),
            PeriodType::Month => write!(f, "month"),
            PeriodType::Year => write!(f, "year"),
            PeriodType::Ytd => write!(f, "ytd"),
        }
    }
}

/// Represents the frequency type for price history.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum FrequencyType {
    /// Minute frequency type.
    Minute,
    /// Daily frequency type.
    Daily,
    /// Weekly frequency type.
    Weekly,
    /// Monthly frequency type.
    Monthly,
}

impl fmt::Display for FrequencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrequencyType::Minute => write!(f, "minute"),
            FrequencyType::Daily => write!(f, "daily"),
            FrequencyType::Weekly => write!(f, "weekly"),
            FrequencyType::Monthly => write!(f, "monthly"),
        }
    }
}

/// Represents the sort order for movers.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Sort {
    /// Sort by volume.
    Volume,
    /// Sort by trades.
    Trades,
    /// Sort by percent change up.
    PercentChangeUp,
    /// Sort by percent change down.
    PercentChangeDown,
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sort::Volume => write!(f, "VOLUME"),
            Sort::Trades => write!(f, "TRADES"),
            Sort::PercentChangeUp => write!(f, "PERCENT_CHANGE_UP"),
            Sort::PercentChangeDown => write!(f, "PERCENT_CHANGE_DOWN"),
        }
    }
}

/// Represents the projection type for instruments.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Projection {
    /// Symbol search projection.
    SymbolSearch,
    /// Symbol regex projection.
    SymbolRegex,
    /// Description search projection.
    DescSearch,
    /// Description regex projection.
    DescRegex,
    /// Search projection.
    Search,
    /// Fundamental projection.
    Fundamental,
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Projection::SymbolSearch => write!(f, "symbol-search"),
            Projection::SymbolRegex => write!(f, "symbol-regex"),
            Projection::DescSearch => write!(f, "desc-search"),
            Projection::DescRegex => write!(f, "desc-regex"),
            Projection::Search => write!(f, "search"),
            Projection::Fundamental => write!(f, "fundamental"),
        }
    }
}

/// Represents the market symbols for market hours.
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum MarketSymbol {
    /// Equity market.
    Equity,
    /// Option market.
    Option,
    /// Bond market.
    Bond,
    /// Future market.
    Future,
    /// Forex market.
    Forex,
}

impl fmt::Display for MarketSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketSymbol::Equity => write!(f, "equity"),
            MarketSymbol::Option => write!(f, "option"),
            MarketSymbol::Bond => write!(f, "bond"),
            MarketSymbol::Future => write!(f, "future"),
            MarketSymbol::Forex => write!(f, "forex"),
        }
    }
}

/// A client for interacting with the Schwab API.
pub struct SchwabApi {
    reqwest_client: Arc<Client>,
    base_url: String,
    tokens_file_path: String,
}

impl SchwabApi {
    /// Creates a new `SchwabApi` instance.
    ///
    /// # Arguments
    ///
    /// * `reqwest_client` - An `Arc` wrapped `reqwest::Client` to be used for making HTTP requests.
    /// * `base_url` - The base URL for the Schwab API.
    /// * `tokens_file_path` - The path to the file where tokens are stored.
    ///
    /// # Returns
    ///
    /// A new `SchwabApi` instance.
    pub fn new(reqwest_client: Arc<Client>, base_url: String, tokens_file_path: String) -> Self {
        Self {
            reqwest_client,
            base_url,
            tokens_file_path,
        }
    }

    /// Parses optional parameters into a `Vec` of `(String, String)` tuples.
    /// Filters out `None` values and converts `Some` values to strings.
    fn parse_params<T: ToString>(params: Vec<(&str, Option<T>)>) -> Vec<(String, String)> {
        params
            .into_iter()
            .filter_map(|(key, value)| value.map(|v| (key.to_string(), v.to_string())))
            .collect()
    }

    /// Converts a `DateTime<Utc>` or `String` to an epoch timestamp in milliseconds.
    fn time_to_epoch_ms(date: Option<DateTime<Utc>>) -> Option<String> {
        date.map(|d| d.timestamp_millis().to_string())
    }

    /// Converts a `DateTime<Utc>` to a "YYYY-MM-DD" string.
    fn time_to_yyyymmdd(date: Option<DateTime<Utc>>) -> Option<String> {
        date.map(|d| d.format("%Y-%m-%d").to_string())
    }

    /// Gets quotes for a list of symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - A `Vec` of `String`s representing the symbols to get quotes for.
    /// * `fields` - An `Option`al `Vec` of `QuoteFields` to be returned in the quote.
    /// * `indicative` - An `Option`al `bool` indicating whether to return indicative quotes.
    ///
    /// # Returns
    ///
    /// An empty `Result` indicating success or failure.
    pub async fn get_quotes(
        &self,
        symbols: Vec<String>,
        fields: Option<Vec<QuoteFields>>,
        indicative: Option<bool>,
    ) -> anyhow::Result<Value, anyhow::Error> {
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

        let headers = self.construct_request_headers().await?;

        let request_url = format!(
            "{}/quotes?symbols={}&fields={}&indicative={}",
            self.base_url, symbols_string, fields_string, indicative_string
        );
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Gets an options chain for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The symbol to get the options chain for.
    /// * `contract_type` - The type of contract to get.
    /// * `strike_count` - The number of strikes to return.
    /// * `include_underlying_quote` - Whether to include the underlying quote in the response.
    ///
    /// # Returns
    ///
    /// An empty `Result` indicating success or failure.
    pub async fn get_chains(
        &self,
        symbol: String,
        contract_type: ContractType,
        strike_count: u64,
        include_underlying_quote: bool,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let request_url = format!(
            "{}/chains?symbol={}&contractType={}&strikeCount={}&includeUnderlyingQuote={}",
            self.base_url,
            symbol,
            contract_type.to_string(),
            strike_count.to_string(),
            include_underlying_quote.to_string()
        );
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get quote for a single symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol_id` - Ticker symbol.
    /// * `fields` - Fields to get ("all", "quote", "fundamental").
    ///
    /// # Returns
    ///
    /// `Result` containing quote for a single symbol or an error.
    pub async fn quote(
        &self,
        symbol_id: String,
        fields: Option<Vec<QuoteFields>>,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let fields_string = match fields {
            Some(v) => dedup_ordered(v)
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(","),
            None => "".to_owned(),
        };

        let params = SchwabApi::parse_params(vec![("fields", Some(fields_string))]);

        let request_url = format!("{}/{}/quotes", self.base_url, encode(&symbol_id));
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get an option expiration chain for a ticker.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Ticker symbol.
    ///
    /// # Returns
    ///
    /// `Result` containing option expiration chain or an error.
    pub async fn option_expiration_chain(
        &self,
        symbol: String,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let params = SchwabApi::parse_params(vec![("symbol", Some(symbol))]);

        let request_url = format!("{}/expirationchain", self.base_url);
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get price history for a ticker.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Ticker symbol.
    /// * `period_type` - Period type ("day"|"month"|"year"|"ytd").
    /// * `period` - Period.
    /// * `frequency_type` - Frequency type ("minute"|"daily"|"weekly"|"monthly").
    /// * `frequency` - Frequency (frequencyType: options), (minute: 1, 5, 10, 15, 30), (daily: 1), (weekly: 1), (monthly: 1).
    /// * `start_date` - Start date.
    /// * `end_date` - End date.
    /// * `need_extended_hours_data` - Need extended hours data (True|False).
    /// * `need_previous_close` - Need previous close (True|False).
    ///
    /// # Returns
    ///
    /// `Result` containing candle history or an error.
    pub async fn price_history(
        &self,
        symbol: String,
        period_type: Option<PeriodType>,
        period: Option<u64>,
        frequency_type: Option<FrequencyType>,
        frequency: Option<u64>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        need_extended_hours_data: Option<bool>,
        need_previous_close: Option<bool>,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let params = SchwabApi::parse_params(vec![
            ("symbol", Some(symbol)),
            ("periodType", period_type.map(|p| p.to_string())),
            ("period", period.map(|p| p.to_string())),
            ("frequencyType", frequency_type.map(|f| f.to_string())),
            ("frequency", frequency.map(|f| f.to_string())),
            ("startDate", SchwabApi::time_to_epoch_ms(start_date)),
            ("endDate", SchwabApi::time_to_epoch_ms(end_date)),
            (
                "needExtendedHoursData",
                need_extended_hours_data.map(|b| b.to_string()),
            ),
            (
                "needPreviousClose",
                need_previous_close.map(|b| b.to_string()),
            ),
        ]);

        let request_url = format!("{}/pricehistory", self.base_url);
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get movers in a specific index and direction.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Symbol ("$DJI"|"$COMPX"|"$SPX"|"NYSE"|"NASDAQ"|"OTCBB"|"INDEX_ALL"|"EQUITY_ALL"|"OPTION_ALL"|"OPTION_PUT"|"OPTION_CALL").
    /// * `sort` - Sort ("VOLUME"|"TRADES"|"PERCENT_CHANGE_UP"|"PERCENT_CHANGE_DOWN").
    /// * `frequency` - Frequency (0|1|5|10|30|60).
    ///
    /// # Notes
    ///
    /// Must be called within market hours (there aren't really movers outside of market hours).
    ///
    /// # Returns
    ///
    /// `Result` containing movers or an error.
    pub async fn movers(
        &self,
        symbol: String,
        sort: Option<Sort>,
        frequency: Option<u64>,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let params = SchwabApi::parse_params(vec![
            ("sort", sort.map(|s| s.to_string())),
            ("frequency", frequency.map(|f| f.to_string())),
        ]);

        let request_url = format!("{}/movers/{}", self.base_url, encode(&symbol));
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get Market Hours for dates in the future across different markets.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of market symbols ("equity", "option", "bond", "future", "forex").
    /// * `date` - Date.
    ///
    /// # Returns
    ///
    /// `Result` containing market hours or an error.
    pub async fn market_hours(
        &self,
        symbols: Vec<MarketSymbol>,
        date: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let symbols_string = symbols
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let params = SchwabApi::parse_params(vec![
            ("markets", Some(symbols_string)),
            ("date", SchwabApi::time_to_yyyymmdd(date)),
        ]);

        let request_url = format!("{}/markets", self.base_url);
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get Market Hours for dates in the future for a single market.
    ///
    /// # Arguments
    ///
    /// * `market_id` - Market id ("equity"|"option"|"bond"|"future"|"forex").
    /// * `date` - Date.
    ///
    /// # Returns
    ///
    /// `Result` containing market hours or an error.
    pub async fn market_hour(
        &self,
        market_id: MarketSymbol,
        date: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let params = SchwabApi::parse_params(vec![("date", SchwabApi::time_to_yyyymmdd(date))]);

        let request_url = format!("{}/markets/{}", self.base_url, market_id.to_string());
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get instruments for a list of symbols.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Symbol.
    /// * `projection` - Projection ("symbol-search"|"symbol-regex"|"desc-search"|"desc-regex"|"search"|"fundamental").
    ///
    /// # Returns
    ///
    /// `Result` containing instruments or an error.
    pub async fn instruments(
        &self,
        symbol: String,
        projection: Projection,
    ) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let params = SchwabApi::parse_params(vec![
            ("symbol", Some(symbol)),
            ("projection", Some(projection.to_string())),
        ]);

        let request_url = format!("{}/instruments", self.base_url);
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Get instrument for a single cusip.
    ///
    /// # Arguments
    ///
    /// * `cusip_id` - Cusip id.
    ///
    /// # Returns
    ///
    /// `Result` containing instrument or an error.
    pub async fn instrument_cusip(&self, cusip_id: String) -> anyhow::Result<Value, anyhow::Error> {
        let headers = self.construct_request_headers().await?;

        let request_url = format!("{}/instruments/{}", self.base_url, encode(&cusip_id));
        let response = self
            .reqwest_client
            .get(request_url)
            .headers(headers)
            .send()
            .await?;

        let response_json = serde_json::from_str(response.text().await?.as_str())?;
        Ok(response_json)
    }

    /// Constructs the request headers for a Schwab API request.
    ///
    /// # Returns
    ///
    /// A `HeaderMap` containing the required headers for a Schwab API request.
    async fn construct_request_headers(&self) -> anyhow::Result<HeaderMap, anyhow::Error> {
        let mut headers = HeaderMap::new();

        let json_string = tokio::fs::read_to_string(&self.tokens_file_path).await?;
        let data: StoredTokenInfo = serde_json::from_str(&json_string)?;
        let auth_header = format!("Bearer {}", data.access_token.as_str());

        headers.append("Accept", HeaderValue::from_str("application/json")?);
        headers.append(
            "Authorization",
            HeaderValue::from_str(auth_header.as_str())?,
        );

        Ok(headers)
    }
}
