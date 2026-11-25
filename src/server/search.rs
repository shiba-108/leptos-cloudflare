use axum::http::Uri;

use crate::search::{parse_search_response, retry_after_seconds, SearchOutcome};

use super::{AppError, AppResult};

const MAX_SEARCH_RESPONSE_BYTES: usize = 262_144;

/// Fetches public search results during request-time SSR from the same origin
/// as the current page. The browser-facing API remains the contract's
/// `GET /api/search?q=...`; no token or Authorization header is used.
pub async fn search_public_content(query: String) -> AppResult<SearchOutcome> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(SearchOutcome::Loaded(Default::default()));
    }

    let uri = leptos_axum::extract::<Uri>()
        .await
        .map_err(|_| AppError::Internal)?;
    let Some(url) = search_api_url(&uri, query) else {
        return Ok(SearchOutcome::Unavailable);
    };

    let mut request_init = worker::RequestInit::new();
    request_init.with_cache(worker::CacheMode::NoStore);
    request_init
        .headers
        .set("Accept", "application/json")
        .map_err(|_| AppError::Internal)?;
    request_init
        .headers
        .set("Cache-Control", "no-store")
        .map_err(|_| AppError::Internal)?;

    let request = worker::Request::new_with_init(url.as_str(), &request_init)
        .map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;
    let status = response.status_code();
    let retry_after_header = response
        .headers()
        .get("retry-after")
        .map_err(|_| AppError::Internal)?;

    if !matches!(status, 200 | 429) {
        return Ok(SearchOutcome::Unavailable);
    }
    if response
        .headers()
        .get("content-length")
        .map_err(|_| AppError::Internal)?
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_SEARCH_RESPONSE_BYTES)
    {
        return Ok(SearchOutcome::Unavailable);
    }

    let body = response.text().await.map_err(|_| AppError::Internal)?;
    if body.len() > MAX_SEARCH_RESPONSE_BYTES {
        return Ok(SearchOutcome::Unavailable);
    }

    if status == 429 {
        return Ok(SearchOutcome::RateLimited {
            retry_after_seconds: retry_after_seconds(&body, retry_after_header.as_deref()),
        });
    }

    Ok(parse_search_response(&body)
        .map(SearchOutcome::Loaded)
        .unwrap_or(SearchOutcome::Unavailable))
}

fn search_api_url(uri: &Uri, query: &str) -> Option<worker::Url> {
    // The app and API share an origin. Restrict this SSR fetch to the VionSky
    // zone declared by the contract so an untrusted Host cannot turn it into a
    // generic server-side proxy.
    if uri.scheme_str() != Some("https") {
        return None;
    }
    let authority = uri.authority()?;
    let host = authority.host().trim_end_matches('.');
    let normalized_host = host.to_ascii_lowercase();
    if normalized_host != "vionsky.com" && !normalized_host.ends_with(".vionsky.com") {
        return None;
    }
    if authority.port_u16().is_some_and(|port| port != 443) {
        return None;
    }

    let mut url = worker::Url::parse(&format!("https://{host}/api/search")).ok()?;
    url.query_pairs_mut().append_pair("q", query);
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::search_api_url;
    use axum::http::Uri;

    #[test]
    fn builds_the_documented_same_origin_search_url_and_encodes_query() {
        let uri: Uri = "https://vionsky.com/vi/search?q=old"
            .parse()
            .expect("valid URI");
        let url = search_api_url(&uri, "áo khoác & gió").expect("VionSky origin");
        assert_eq!(url.path(), "/api/search");
        assert_eq!(
            url.query_pairs().find(|(key, _)| key == "q").unwrap().1,
            "áo khoác & gió"
        );
    }

    #[test]
    fn rejects_non_vionsky_origins_and_non_https_requests() {
        let external: Uri = "https://example.test/vi/search".parse().expect("valid URI");
        assert!(search_api_url(&external, "query").is_none());
        let insecure: Uri = "http://vionsky.com/vi/search".parse().expect("valid URI");
        assert!(search_api_url(&insecure, "query").is_none());
    }
}
