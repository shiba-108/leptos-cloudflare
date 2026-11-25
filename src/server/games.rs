use serde_json::Value;

use crate::components::home_page::ProductSummary;

use super::{AppError, AppResult};

const PUBLIC_GAMES_API_URL: &str = "https://vionsky.com/api/games/";
const HOME_GAME_LIMIT: usize = 6;

fn parse_public_games(payload: &Value) -> AppResult<Vec<ProductSummary>> {
    let rows = payload
        .get("games")
        .and_then(Value::as_array)
        .ok_or(AppError::Internal)?;

    Ok(rows
        .iter()
        .filter_map(ProductSummary::from_public_row)
        .take(HOME_GAME_LIMIT)
        .collect())
}

/// Fetches the platform's public approved-games API during request-time SSR.
/// The API returns raw rows, so each row is parsed against the confirmed public
/// contract before it can reach the page or the browser.
pub async fn get_public_games() -> AppResult<Vec<ProductSummary>> {
    let url = PUBLIC_GAMES_API_URL
        .parse()
        .map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Url(url)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    if !(200..300).contains(&response.status_code()) {
        return Err(AppError::Internal);
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| AppError::Internal)?;

    parse_public_games(&payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_an_empty_public_list() {
        assert!(parse_public_games(&json!({ "games": [] }))
            .expect("empty games is valid")
            .is_empty());
    }

    #[test]
    fn rejects_a_payload_without_the_confirmed_games_array() {
        assert!(parse_public_games(&json!({ "items": [] })).is_err());
    }

    #[test]
    fn filters_unapproved_rows_and_limits_output_to_six_cards() {
        let mut rows = vec![json!({
            "status": "pending",
            "title": "Not public",
            "slug": "not-public"
        })];
        rows.extend((0..7).map(|index| {
            json!({
                "status": "approved",
                "title": format!("Public game {index}"),
                "slug": format!("public-game-{index}"),
                "category": "ignored",
                "is_learning": true
            })
        }));

        let games =
            parse_public_games(&json!({ "games": rows })).expect("confirmed public list contract");
        assert_eq!(games.len(), HOME_GAME_LIMIT);
    }
}
