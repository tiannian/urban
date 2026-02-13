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

### OrderResponse Structure

The `OrderResponse` structure represents the JSON returned from Binance's POST `/fapi/v1/order` (New Order) endpoint. All fields are deserialized from the API response using serde.

**Fields:**

- `client_order_id`: String - User-defined or system-assigned order id.
- `order_id`: i64 - System order id.
- `symbol`: String - Trading pair (e.g. `BTCUSDT`).
- `side`: String - Order side (`BUY`, `SELL`).
- `position_side`: String - Position side (`BOTH`, `LONG`, `SHORT`).
- `order_type`: String - Order type (e.g. `LIMIT`, `MARKET`, `TRAILING_STOP_MARKET`).
- `orig_type`: String - Original order type before trigger (for conditional orders).
- `status`: String - Order status (e.g. `NEW`, `FILLED`, `CANCELED`).
- `orig_qty`: String - Original order quantity.
- `executed_qty`: String - Executed quantity.
- `cum_qty`: String - Cumulative filled quantity.
- `cum_quote`: String - Cumulative filled quote amount.
- `price`: String - Limit price.
- `avg_price`: String - Average fill price.
- `stop_price`: String - Trigger price (not used for `TRAILING_STOP_MARKET`).
- `reduce_only`: bool - Whether the order is reduce-only.
- `close_position`: bool - Whether the order closes the position (conditional).
- `time_in_force`: String - Time in force (e.g. `GTC`, `IOC`, `GTD`).
- `update_time`: i64 - Last update time in milliseconds.
- `working_type`: String - Trigger price type (e.g. `CONTRACT_PRICE`).
- `price_protect`: bool - Whether trigger protection is enabled.
- `price_match`: String - Price match mode.
- `self_trade_prevention_mode`: String - Self-trade prevention mode.
- `good_till_date`: Option&lt;i64&gt; - Auto-cancel time when timeInForce is GTD.

**JSON Field Mapping:**

The structure uses `#[serde(rename = "...")]` to map camelCase JSON keys to snake_case Rust fields (e.g. `clientOrderId` → `client_order_id`, `orderId` → `order_id`, `origQty` → `orig_qty`, `executedQty` → `executed_qty`, `cumQuote` → `cum_quote`, `avgPrice` → `avg_price`, `stopPrice` → `stop_price`, `reduceOnly` → `reduce_only`, `closePosition` → `close_position`, `timeInForce` → `time_in_force`, `updateTime` → `update_time`, `workingType` → `working_type`, `priceProtect` → `price_protect`, `priceMatch` → `price_match`, `selfTradePreventionMode` → `self_trade_prevention_mode`, `goodTillDate` → `good_till_date`). The JSON field for order type is `type`; the Rust field may be named `order_type` with rename `"type"`, and `origType` → `orig_type`.

### Side, PositionSide, and OrderType Enums

**Side**

Order side. Serializes to the API string `BUY` or `SELL`.

- Variants: `Buy`, `Sell` (or equivalent; serialization to `"BUY"`, `"SELL"`).

**PositionSide**

Position side. One-way mode uses `Both`; hedge mode uses `Long` or `Short`. Serializes to the API string `BOTH`, `LONG`, or `SHORT`.

- Variants: `Both`, `Long`, `Short`.

**OrderType**

Order type. Serializes to the API string accepted by POST `/fapi/v1/order`.

- Variants (at least): `Limit`, `Market`. Other Binance types (e.g. `Stop`, `TakeProfit`, `StopMarket`, `TakeProfitMarket`, `TrailingStopMarket`) may be included as needed.
- API values: `LIMIT`, `MARKET`, `STOP`, `TAKE_PROFIT`, `STOP_MARKET`, `TAKE_PROFIT_MARKET`, `TRAILING_STOP_MARKET`.

These enums must derive `Serialize` and `Deserialize` (or equivalent) so that they serialize to the exact API strings above when building the request body.

### PlaceOrderRequest Structure

The `PlaceOrderRequest` structure holds exactly the following parameters for a single order. All fields are sent as form parameters to POST `/fapi/v1/order` (with `symbol` provided separately to `place_order`).

**Fields:**

- `side`: `Side` - Order side (Buy / Sell).
- `position_side`: `PositionSide` - Position side (Both / Long / Short).
- `order_type`: `OrderType` - Order type (e.g. Limit, Market).
- `quantity`: String - Order quantity (decimal string as required by the API for the given order type).
- `price`: Option&lt;String&gt; - Limit price (required for LIMIT orders; None for MARKET).
- `reduce_only`: bool - If true, order is reduce-only.

When building the request body, the implementation must convert each enum to its API string (e.g. `Side::Buy` → `"BUY"`, `PositionSide::Both` → `"BOTH"`, `OrderType::Limit` → `"LIMIT"`). For LIMIT orders, `timeInForce` is required by the API; the spec does not prescribe how it is chosen (e.g. default `GTC` in implementation).

### place_order Function

**Function Signature**

```rust
async fn place_order(
    &self,
    symbol: &str,
    req: &PlaceOrderRequest,
) -> Result<OrderResponse, Box<dyn std::error::Error>>
```

**Function Behavior**

The `place_order` function submits a single order to Binance's POST `/fapi/v1/order` endpoint. The function performs the following steps:

1. **Build Parameters**
   - Build a parameter vector from `symbol` and `req`: `symbol` (from the argument), `side` (from `req.side` serialized to API string), `positionSide` (from `req.position_side`), `type` (from `req.order_type`), `quantity` (from `req.quantity`), `reduceOnly` (from `req.reduce_only`, as `"true"` or `"false"`). For LIMIT orders include `price` from `req.price` (unwrap or use the value) and a `timeInForce` value (e.g. default `GTC`; implementation-defined). Parameter keys use Binance's camelCase.

2. **Send Signed Request**
   - Call `fapi_signed_request(self.client, self.base_url, "/fapi/v1/order", "POST", self.api_key, self.api_secret, params)`. The helper adds `timestamp` and `recvWindow` and signs the body as application/x-www-form-urlencoded.

3. **Parse Response**
   - Deserialize the JSON response body into an `OrderResponse` structure.
   - Return the result.

**Parameters:**
- `symbol`: The trading pair (e.g. `BTCUSDT`).
- `req`: A reference to a `PlaceOrderRequest` containing `side`, `position_side`, `order_type`, `quantity`, `price`, and `reduce_only`.

**Returns:** An `OrderResponse` instance with the order metadata and fill information returned by the API.

**Error Handling**

- Network errors, HTTP errors, or JSON deserialization errors are propagated as `Box<dyn std::error::Error>`.
- API-level errors (e.g. invalid symbol, rate limit, insufficient margin) are returned as errors from the HTTP client or JSON deserializer.

**Rate Limits**

- 10s order count (X-MBX-ORDER-COUNT-10S): 1; 1min order count (X-MBX-ORDER-COUNT-1M): 1. The implementation does not add rate limiting; callers must respect these limits.

### open_sell Function

**Function Signature**

```rust
async fn open_sell(
    &self,
    symbol: &str,
    amount: &str,
) -> Result<OrderResponse, Box<dyn std::error::Error>>
```

**Function Behavior**

The `open_sell` function places a limit sell order to open a short position at the current best ask (asks0). The function performs the following steps:

1. **Read Orderbook**
   - Call `get_orderbook(self, symbol, limit)` (e.g. `limit = Some(5)` or implementation-defined) to obtain the current orderbook.

2. **Obtain Price**
   - Take the first ask level: `asks[0]` from the orderbook. The price to use is `asks[0][0]` (the best ask price).

3. **Place Order**
   - Build a `PlaceOrderRequest` with: `side = Side::Sell`, `position_side` as per client/contract mode (e.g. `PositionSide::Both` or `Short`), `order_type = OrderType::Limit`, `quantity = amount`, `price = Some(asks[0][0].clone())`, `reduce_only = false`.
   - Call `place_order(self, symbol, &req)` and return its result.

**Parameters:**
- `symbol`: The trading pair (e.g. `BTCUSDT`).
- `amount`: Order quantity (decimal string).

**Returns:** The `OrderResponse` from the underlying `place_order` call.

**Error Handling**

- Propagates errors from `get_orderbook` (e.g. empty asks) and `place_order` as `Box<dyn std::error::Error>`.

### close_sell Function

**Function Signature**

```rust
async fn close_sell(
    &self,
    symbol: &str,
    amount: &str,
) -> Result<OrderResponse, Box<dyn std::error::Error>>
```

**Function Behavior**

The `close_sell` function places a limit sell order to close an existing short position at the current best bid (bids0). The function performs the following steps:

1. **Read Orderbook**
   - Call `get_orderbook(self, symbol, limit)` (e.g. `limit = Some(5)` or implementation-defined) to obtain the current orderbook.

2. **Obtain Price**
   - Take the first bid level: `bids[0]` from the orderbook. The price to use is `bids[0][0]` (the best bid price).

3. **Place Order**
   - Build a `PlaceOrderRequest` with: `side = Side::Sell`, `position_side` as per client/contract mode (e.g. `PositionSide::Both` or `Short`), `order_type = OrderType::Limit`, `quantity = amount`, `price = Some(bids[0][0].clone())`, `reduce_only = true`.
   - Call `place_order(self, symbol, &req)` and return its result.

**Parameters:**
- `symbol`: The trading pair (e.g. `BTCUSDT`).
- `amount`: Order quantity (decimal string) to close.

**Returns:** The `OrderResponse` from the underlying `place_order` call.

**Error Handling**

- Propagates errors from `get_orderbook` (e.g. empty bids) and `place_order` as `Box<dyn std::error::Error>`.

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

### Placing an Order

```rust
let req = PlaceOrderRequest {
    side: Side::Buy,
    position_side: PositionSide::Both,
    order_type: OrderType::Limit,
    quantity: "0.001".to_string(),
    price: Some("50000".to_string()),
    reduce_only: false,
};
let order_response = binance_client.place_order("BTCUSDT", &req).await?;
// order_response.order_id, order_response.status, order_response.executed_qty, etc.
```

### Open Sell and Close Sell

```rust
// Open a short: limit sell at best ask (asks0), quantity = amount
let open_resp = binance_client.open_sell("BTCUSDT", "0.001").await?;

// Close a short: limit sell at best bid (bids0), reduce_only, quantity = amount
let close_resp = binance_client.close_sell("BTCUSDT", "0.001").await?;
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

- [Binance USDT-Margined Futures – New Order (POST /fapi/v1/order)](https://developers.binance.com/docs/zh-CN/derivatives/usds-margined-futures/trade/rest-api) for request/response parameters, types, and rate limits.
- Binance Futures API documentation for other endpoint details and parameter requirements.
- Binance API authentication documentation for signature algorithm specifications.