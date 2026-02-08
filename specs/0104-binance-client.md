# Binance Client Specification

## Overview

This specification describes a client for interacting with Binance perpetual futures (USDT-M) API. The client provides functionality to query position information and execute signed API requests. The client handles authentication, request signing, and API communication using HMAC-SHA256 signatures as required by Binance's API security model.

## Scope and Assumptions

- **API Endpoint**: The client interacts with Binance Futures API (fapi) endpoints, specifically the USDT-M perpetual futures API.
- **Authentication**: All authenticated requests require API key and secret. The client uses HMAC-SHA256 signatures for request authentication.
- **HTTP Client**: The client uses a shared `Arc<reqwest::Client>` instance for making HTTP requests.
- **Base URL**: The base URL for API endpoints is configurable (e.g., `https://fapi.binance.com`).
- **Request Signing**: All signed requests include a timestamp and optional `recvWindow` parameter, with the signature computed over the URL-encoded query string.

## Terminology and Variables

- `api_key`: The Binance API key used for authentication.
- `api_secret`: The Binance API secret used for HMAC-SHA256 signature generation.
- `base_url`: The base URL for Binance Futures API endpoints (e.g., `https://fapi.binance.com`).
- `symbol`: Trading pair symbol (e.g., `BTCUSDT`).
- `pair`: Alternative term for trading symbol.
- `timestamp`: Unix timestamp in milliseconds for request signing.
- `recvWindow`: Request receive window in milliseconds (default: 5000ms).
- `signature`: HMAC-SHA256 signature computed over the URL-encoded query string.

## Detailed Specifications

### BinancePerpsClient Structure

The `BinancePerpsClient` type contains:

- `client`: An `Arc<reqwest::Client>` instance for making HTTP requests.
- `api_key`: String containing the Binance API key.
- `api_secret`: String containing the Binance API secret.
- `base_url`: String containing the base URL for API endpoints.

**BinancePerpsClientConfig Structure**

The `BinancePerpsClientConfig` structure contains:

- `client`: An `Arc<reqwest::Client>` instance for making HTTP requests.
- `api_key`: String containing the Binance API key.
- `api_secret`: String containing the Binance API secret.
- `base_url`: String containing the base URL for API endpoints.

The `BinancePerpsClientConfig` structure must derive `serde::Serialize` and `serde::Deserialize` for serialization support.

**Constructor**

```rust
fn new(config: BinancePerpsClientConfig) -> Self
```

- Creates a new `BinancePerpsClient` instance.
- **Parameters:**
  - `config`: A `BinancePerpsClientConfig` instance containing all configuration parameters and the HTTP client instance.
- **Returns:** A new `BinancePerpsClient` instance with the provided configuration.

### Position Structure

The `Position` structure represents position information returned from Binance's `/fapi/v3/positionRisk` endpoint. All fields are deserialized from JSON responses using serde.

**Fields:**

- `symbol`: String - Trading pair symbol (e.g., `BTCUSDT`).
- `position_side`: String - Position side (`LONG`, `SHORT`, or `BOTH`).
- `position_amt`: String - Position amount (positive for long, negative for short).
- `entry_price`: String - Average entry price.
- `break_even_price`: String - Break-even price.
- `mark_price`: String - Current mark price.
- `unrealized_pnl`: String - Unrealized profit/loss.
- `liquidation_price`: String - Liquidation price.
- `isolated_margin`: String - Isolated margin amount.
- `notional`: String - Position notional value.
- `margin_asset`: String - Margin asset (e.g., `USDT`).
- `isolated_wallet`: String - Isolated wallet balance.
- `initial_margin`: String - Initial margin requirement.
- `maint_margin`: String - Maintenance margin requirement.
- `position_initial_margin`: String - Position initial margin.
- `open_order_initial_margin`: String - Open order initial margin.
- `adl`: i32 - Auto-deleveraging indicator.
- `bid_notional`: String - Bid notional value.
- `ask_notional`: String - Ask notional value.
- `update_time`: i64 - Last update timestamp in milliseconds.

**JSON Field Mapping:**

The structure uses `#[serde(rename = "...")]` attributes to map camelCase JSON field names to snake_case Rust field names:
- `positionSide` → `position_side`
- `positionAmt` → `position_amt`
- `entryPrice` → `entry_price`
- `breakEvenPrice` → `break_even_price`
- `markPrice` → `mark_price`
- `unRealizedProfit` → `unrealized_pnl`
- `liquidationPrice` → `liquidation_price`
- `isolatedMargin` → `isolated_margin`
- `marginAsset` → `margin_asset`
- `isolatedWallet` → `isolated_wallet`
- `initialMargin` → `initial_margin`
- `maintMargin` → `maint_margin`
- `positionInitialMargin` → `position_initial_margin`
- `openOrderInitialMargin` → `open_order_initial_margin`
- `bidNotional` → `bid_notional`
- `askNotional` → `ask_notional`
- `updateTime` → `update_time`

### Orderbook Structure

The `Orderbook` structure represents the order book (market depth) returned from Binance's `/fapi/v1/depth` endpoint. All fields are deserialized from JSON responses using serde.

**Fields:**

- `last_update_id`: i64 - Last update ID for the order book.
- `e`: i64 - Event time in milliseconds.
- `t`: i64 - Transaction time in milliseconds.
- `bids`: Vec<[String; 2]> - Array of bid levels; each element is `[price, quantity]` as strings.
- `asks`: Vec<[String; 2]> - Array of ask levels; each element is `[price, quantity]` as strings.

**JSON Field Mapping:**

The structure uses `#[serde(rename = "...")]` attributes to map JSON field names to Rust field names:
- `lastUpdateId` → `last_update_id`
- `E` → `e`
- `T` → `t`

**Example Response (JSON):**

```json
{
  "lastUpdateId": 9861306016934,
  "E": 1770548779357,
  "T": 1770548779348,
  "bids": [["641.600","0.01"],["641.590","0.16"],["641.580","0.85"]],
  "asks": [["641.620","0.40"],["641.640","0.15"],["641.660","2.75"]]
}
```

### get_orderbook Function

**Function Signature**

```rust
async fn get_orderbook(
    &self,
    symbol: &str,
    limit: Option<u16>,
) -> Result<Orderbook, Box<dyn std::error::Error>>
```

**Function Behavior**

The `get_orderbook` function queries Binance's `/fapi/v1/depth` endpoint to retrieve the order book (market depth) for a specific symbol. The endpoint is public and does not require request signing. The function performs the following steps:

1. **Build Query Parameters**
   - Add required parameter: `symbol` — the trading pair symbol (e.g., `BTCUSDT`).
   - If `limit` is `Some(n)`, add query parameter `limit` with value `n`. Valid limits are 5, 10, 20, 50, 100, or 500; if not provided, the API uses its default.

2. **Build Request URL**
   - Construct the full URL: `{base_url}/fapi/v1/depth?symbol={symbol}[&limit={limit}]`.

3. **Send HTTP Request**
   - Make a GET request to the constructed URL.
   - No API key or signature is required for this public endpoint.
   - Await the response.

4. **Parse Response**
   - Deserialize the JSON response body into an `Orderbook` structure.
   - Return the orderbook.

**Parameters:**
- `symbol`: The trading pair symbol (e.g., `BTCUSDT`).
- `limit`: Optional. Number of depth levels to return (5, 10, 20, 50, 100, or 500). If `None`, the API default is used.

**Returns:** An `Orderbook` instance containing `last_update_id`, `e`, `t`, `bids`, and `asks`.

**Error Handling**

- Network errors, HTTP errors, or JSON deserialization errors are propagated as `Box<dyn std::error::Error>`.

### get_position Function

**Function Signature**

```rust
async fn get_position(
    &self,
    pair: &str,
) -> Result<Vec<Position>, Box<dyn std::error::Error>>
```

**Function Behavior**

The `get_position` function queries Binance's `/fapi/v3/positionRisk` endpoint to retrieve position information for a specific trading pair. The function performs the following steps:

1. **Prepare Parameters**
   - Create a parameter vector containing:
     - `symbol`: The trading pair symbol (e.g., `BTCUSDT`).
     - `timestamp`: Current Unix timestamp in milliseconds (generated via `binance_fapi_timestamp_ms()`).

2. **Sign Request**
   - Call `sign_params(&self.api_secret, &params)` to generate a signed query string.
   - The signature is computed using HMAC-SHA256 over the URL-encoded query string.

3. **Build Request URL**
   - Construct the full URL: `{base_url}/fapi/v3/positionRisk?{signed_query}`.

4. **Send HTTP Request**
   - Make a GET request to the constructed URL.
   - Include the `X-MBX-APIKEY` header with the API key value.
   - Await the response.

5. **Parse Response**
   - Deserialize the JSON response body into a `Vec<Position>`.
   - Return the vector of positions.

**Error Handling**

- Network errors, HTTP errors, or JSON deserialization errors are propagated as `Box<dyn std::error::Error>`.
- The function does not handle API-level errors (e.g., invalid API key, rate limiting) explicitly; these are returned as errors from the HTTP client or JSON deserializer.

### Utility Functions

#### binance_fapi_timestamp_ms

**Function Signature**

```rust
fn binance_fapi_timestamp_ms() -> String
```

- Generates the current Unix timestamp in milliseconds as a string.
- **Returns:** String representation of milliseconds since Unix epoch.

#### sign_params

**Function Signature**

```rust
fn sign_params(api_secret: &str, params: &[(&str, String)]) -> String
```

**Function Behavior**

The `sign_params` function creates a signed query string for Binance API requests:

1. **Build Query String**
   - URL-encode all parameters using `form_urlencoded::Serializer`.
   - Parameters are encoded as `key=value` pairs, separated by `&`.

2. **Compute Signature**
   - Compute HMAC-SHA256 signature over the query string using the API secret.
   - Convert the signature bytes to lowercase hexadecimal string.

3. **Append Signature**
   - Append `&signature={hex_signature}` to the query string.
   - Return the complete signed query string.

**Parameters:**
- `api_secret`: The Binance API secret for signature generation.
- `params`: Slice of key-value parameter pairs.

**Returns:** Complete signed query string including the signature parameter.

#### fapi_signed_request

**Function Signature**

```rust
async fn fapi_signed_request(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    method: &str,
    api_key: &str,
    api_secret: &str,
    mut params: Vec<(&str, String)>,
) -> Result<String, reqwest::Error>
```

**Function Behavior**

The `fapi_signed_request` function is a generic helper for making signed requests to Binance Futures API endpoints. It supports GET, POST, and DELETE methods. The function performs the following steps:

1. **Add Default Parameters**
   - If `timestamp` is not present in params, add it with the current timestamp (via `binance_fapi_timestamp_ms()`).
   - If `recvWindow` is not present in params, add it with value `"5000"`.

2. **Sign Parameters**
   - Call `sign_params(api_secret, &params)` to generate the signed query string.

3. **Build Request**
   - Construct the full URL: `{base_url}{path}`.
   - Based on `method`:
     - **GET**: Append the signed query string as URL query parameters.
     - **POST**: Set the request body to the signed query string and set `Content-Type: application/x-www-form-urlencoded`.
     - **DELETE**: Set the request body to the signed query string and set `Content-Type: application/x-www-form-urlencoded`.
   - Add the `X-MBX-APIKEY` header with the API key value.

4. **Send Request**
   - Send the HTTP request and await the response.
   - Read the response body as text.

5. **Return Response**
   - Return the response body as a string.

**Parameters:**
- `client`: The `reqwest::Client` instance for making HTTP requests.
- `base_url`: Base URL for the API (e.g., `https://fapi.binance.com`).
- `path`: API endpoint path (e.g., `/fapi/v2/account` or `/fapi/v1/order`).
- `method`: HTTP method (`"GET"`, `"POST"`, or `"DELETE"`).
- `api_key`: Binance API key.
- `api_secret`: Binance API secret.
- `params`: Vector of key-value parameter pairs (without signature).

**Returns:** Response body as a string, or a `reqwest::Error` if the request fails.

**Error Handling**

- Network errors and HTTP errors are returned as `reqwest::Error`.
- Unsupported HTTP methods cause a panic.

## Usage Patterns

### Initializing the Client

```rust
let client = Arc::new(reqwest::Client::new());
let api_key = "your_api_key".to_string();
let api_secret = "your_api_secret".to_string();
let base_url = "https://fapi.binance.com".to_string();

let config = BinancePerpsClientConfig {
    client,
    api_key,
    api_secret,
    base_url,
};
let binance_client = BinancePerpsClient::new(config);
```

### Querying Position Information

```rust
let positions = binance_client.get_position("BTCUSDT").await?;
for position in positions {
    // Access position fields: symbol, position_amt, entry_price, etc.
}
```

### Querying Order Book

```rust
let orderbook = binance_client.get_orderbook("BTCUSDT", Some(10)).await?;
// orderbook.bids and orderbook.asks are Vec<[String; 2]> (price, quantity)
// orderbook.last_update_id, orderbook.e, orderbook.t for metadata
```

### Using the Generic Signed Request Function

```rust
let params = vec![
    ("symbol", "BTCUSDT".to_string()),
    ("side", "BUY".to_string()),
    ("type", "LIMIT".to_string()),
    ("quantity", "0.001".to_string()),
    ("price", "50000".to_string()),
];

let response = fapi_signed_request(
    &client,
    "https://fapi.binance.com",
    "/fapi/v1/order",
    "POST",
    &api_key,
    &api_secret,
    params,
).await?;
```

## Configuration Parameters

- **API Credentials**
  - `api_key`: Binance API key (obtained from Binance account settings).
  - `api_secret`: Binance API secret (obtained from Binance account settings).
- **Base URL**
  - Production: `https://fapi.binance.com`
  - Testnet: `https://testnet.binancefuture.com`
- **Request Parameters**
  - `recvWindow`: Default receive window is 5000ms. Can be customized in `fapi_signed_request` params.
  - `timestamp`: Automatically generated for each request.

## Security Considerations

- **API Secret Protection**: The API secret should never be logged or exposed in error messages.
- **Request Signing**: All authenticated requests must include a valid signature computed over the exact query string.
- **Timestamp Validation**: Binance validates request timestamps to prevent replay attacks. The `recvWindow` parameter defines the acceptable time window.
- **HTTPS Only**: All API requests must be made over HTTPS to protect credentials and data in transit.

## References

- Binance Futures API documentation for endpoint details and parameter requirements.
- Binance API authentication documentation for signature algorithm specifications.