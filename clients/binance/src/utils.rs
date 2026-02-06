use hmac::{Hmac, Mac};
use sha2::Sha256;
use url::form_urlencoded;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn binance_fapi_timestamp_ms() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

/// Encode params as query string (URL-encoded). The signature must be computed over this string.
fn build_query(params: &[(&str, String)]) -> String {
    let mut ser = form_urlencoded::Serializer::new(String::new());
    for (k, v) in params {
        ser.append_pair(k, v);
    }
    ser.finish()
}

/// HMAC-SHA256(secret, query) -> lowercase hex.
fn sign_query(api_secret: &str, query: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Append signature to params and return the full query string.
pub(crate) fn sign_params(api_secret: &str, params: &[(&str, String)]) -> String {
    let query = build_query(params);
    let sig = sign_query(api_secret, &query);
    format!("{}&signature={}", query, sig)
}

/// Signed request for USD-M futures (fapi). Supports GET, POST, DELETE.
///
/// - base_url: e.g. "https://fapi.binance.com"
/// - path: e.g. "/fapi/v2/account" or "/fapi/v1/order"
/// - method: "GET" | "POST" | "DELETE"
/// - params: without signature; timestamp/recvWindow are added if missing.
///
/// Many Binance POST/DELETE endpoints use application/x-www-form-urlencoded body.
pub async fn fapi_signed_request(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    method: &str,
    api_key: &str,
    api_secret: &str,
    mut params: Vec<(&str, String)>,
) -> Result<String, reqwest::Error> {
    if !params.iter().any(|(k, _)| *k == "timestamp") {
        params.push(("timestamp", binance_fapi_timestamp_ms()));
    }
    if !params.iter().any(|(k, _)| *k == "recvWindow") {
        params.push(("recvWindow", "5000".to_string()));
    }

    let signed_query = sign_params(api_secret, &params);

    let url = format!("{}{}", base_url, path);
    let req = match method {
        "GET" => client.get(format!("{}?{}", url, signed_query)),
        "POST" => client
            .post(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(signed_query),
        "DELETE" => client
            .delete(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(signed_query),
        _ => panic!("unsupported method: {}", method),
    };

    let resp = req
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}
