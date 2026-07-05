use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const WISHLIST_URL: &str =
    "https://api.steampowered.com/IWishlistService/GetWishlist/v1/";
const GET_ITEMS_URL: &str =
    "https://api.steampowered.com/IStoreBrowseService/GetItems/v1";
const TAG_LIST_URL: &str = "https://api.steampowered.com/IStoreService/GetTagList/v1";
const APP_DETAILS_URL: &str = "https://store.steampowered.com/api/appdetails";

#[derive(Clone)]
pub struct SteamClient {
    http: Client,
}

impl SteamClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    pub async fn fetch_wishlist(&self, steam_id: u64) -> AppResult<Vec<WishlistItem>> {
        let response = self
            .http
            .get(WISHLIST_URL)
            .query(&[("steamid", steam_id.to_string())])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Steam(format!("wishlist request failed: {e}")))?;

        let body: WishlistResponse = response.json().await?;

        Ok(body.response.items.unwrap_or_default())
    }

    pub async fn fetch_store_items(&self, appids: &[u32]) -> AppResult<Vec<StoreItem>> {
        if appids.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<serde_json::Value> = appids
            .iter()
            .map(|appid| serde_json::json!({ "appid": appid }))
            .collect();

        let input_json = serde_json::json!({
            "ids": ids,
            "context": {
                "language": "english",
                "country_code": "NL"
            },
            "data_request": {
                "include_basic_info": true,
                "include_assets": true,
                "include_tag_count": 20,
                "include_reviews": true,
                "include_release": true
            }
        });

        let response = self
            .http
            .get(GET_ITEMS_URL)
            .query(&[("input_json", input_json.to_string())])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Steam(format!("GetItems request failed: {e}")))?;

        let body: GetItemsResponse = response.json().await?;
        Ok(body.response.store_items.unwrap_or_default())
    }

    pub async fn fetch_tag_list(&self) -> AppResult<TagListResponse> {
        let response = self
            .http
            .get(TAG_LIST_URL)
            .query(&[("language", "english")])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Steam(format!("GetTagList request failed: {e}")))?;

        response.json().await.map_err(AppError::from)
    }

    pub async fn fetch_app_details(
        &self,
        appids: &[u32],
    ) -> AppResult<std::collections::HashMap<u32, AppDetailsData>> {
        use std::collections::HashMap;

        if appids.is_empty() {
            return Ok(HashMap::new());
        }

        let ids = appids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let response = self
            .http
            .get(APP_DETAILS_URL)
            .query(&[("appids", ids), ("filters", String::from("basic"))])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Steam(format!("appdetails request failed: {e}")))?;

        let body: HashMap<String, AppDetailsEntry> = response.json().await?;
        let mut results = HashMap::new();

        for (key, entry) in body {
            if !entry.success {
                continue;
            }
            let Some(data) = entry.data else {
                continue;
            };
            let appid = key.parse::<u32>().unwrap_or(data.steam_appid);
            results.insert(appid, data);
        }

        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
pub struct WishlistResponse {
    pub response: WishlistResponseInner,
}

#[derive(Debug, Deserialize)]
pub struct WishlistResponseInner {
    pub items: Option<Vec<WishlistItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WishlistItem {
    pub appid: u32,
    pub priority: i64,
    pub date_added: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetItemsResponse {
    pub response: GetItemsResponseInner,
}

#[derive(Debug, Deserialize)]
pub struct GetItemsResponseInner {
    pub store_items: Option<Vec<StoreItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoreItem {
    pub appid: u32,
    pub name: Option<String>,
    pub is_coming_soon: Option<bool>,
    pub basic_info: Option<BasicInfo>,
    pub assets: Option<Assets>,
    pub tags: Option<Vec<TagWeight>>,
    pub release: Option<ReleaseInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseInfo {
    pub steam_release_date: Option<i64>,
    pub is_coming_soon: Option<bool>,
    pub is_early_access: Option<bool>,
}

pub fn derive_is_released(item: &StoreItem) -> Option<bool> {
    if item.is_coming_soon == Some(true) {
        return Some(false);
    }
    if let Some(release) = &item.release {
        if release.is_coming_soon == Some(true) {
            return Some(false);
        }
        if release.steam_release_date.is_some() {
            return Some(true);
        }
    }
    None
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasicInfo {
    pub short_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Assets {
    pub asset_url_format: Option<String>,
    pub header: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TagWeight {
    pub tagid: u32,
    pub weight: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TagListResponse {
    pub response: TagListResponseInner,
}

#[derive(Debug, Deserialize)]
pub struct TagListResponseInner {
    pub version_hash: String,
    pub tags: Vec<TagEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagEntry {
    pub tagid: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AppDetailsEntry {
    pub success: bool,
    pub data: Option<AppDetailsData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppDetailsData {
    pub name: String,
    pub steam_appid: u32,
    pub short_description: Option<String>,
    pub header_image: Option<String>,
    pub coming_soon: Option<bool>,
}

pub fn derive_is_released_from_app_details(detail: &AppDetailsData) -> Option<bool> {
    if detail.coming_soon == Some(true) {
        Some(false)
    } else if detail.coming_soon == Some(false) {
        Some(true)
    } else {
        None
    }
}

const STORE_ASSETS_BASE: &str = "https://shared.fastly.steamstatic.com/store_item_assets/";

pub fn header_image_url(appid: u32, assets: &Option<Assets>) -> String {
    if let Some(assets) = assets {
        if let (Some(format), Some(header)) = (&assets.asset_url_format, &assets.header) {
            let path = format.replace("${FILENAME}", header);
            return format!("{STORE_ASSETS_BASE}{path}");
        }
    }
    format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg")
}

pub fn normalize_header_image_url(stored: &str, appid: u32) -> String {
    if stored.is_empty() {
        return format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg");
    }
    stored.replace(
        "/store_item_assets/steam/steam/",
        "/store_item_assets/steam/",
    )
}
