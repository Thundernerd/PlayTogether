use crate::steam::client::{
    AppDetailsData, Assets, StoreItem, StoreScreenshots, StoreTrailers,
    StoreTrailer,
};

pub const STORE_ASSETS_BASE: &str = "https://shared.fastly.steamstatic.com/store_item_assets/";
pub const STEAM_MOVIE_CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps/";
pub const STEAM_TRAILER_VIDEO_CDN: &str = "https://video.akamai.steamstatic.com/store_trailers/";

pub fn normalize_store_asset_url(url: &str) -> String {
    url.replace("shared.akamai.steamstatic.com", "shared.fastly.steamstatic.com")
        .replace(
            "/store_item_assets/steam/steam/",
            "/store_item_assets/steam/",
        )
}

pub fn resolve_asset_url(appid: u32, assets: &Option<Assets>, filename: &str) -> String {
    if filename.starts_with("http://") || filename.starts_with("https://") {
        return normalize_store_asset_url(filename);
    }

    if filename.starts_with("steam/") {
        return normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}{filename}"));
    }

    if let Some(assets) = assets {
        if let Some(format) = &assets.asset_url_format {
            let path = format.replace("${FILENAME}", filename);
            if path.starts_with("http://") || path.starts_with("https://") {
                return normalize_store_asset_url(&path);
            }
            return normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}{path}"));
        }
    }

    if filename.contains('/') {
        return normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}steam/apps/{appid}/{filename}"));
    }

    normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}steam/apps/{appid}/{filename}"))
}

pub fn header_image_url(appid: u32, assets: &Option<Assets>) -> String {
    if let Some(assets) = assets {
        if let Some(header) = &assets.header {
            return resolve_asset_url(appid, &Some(assets.clone()), header);
        }
    }
    normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg"))
}

pub fn header_image_url_from_item(item: &StoreItem) -> String {
    header_image_url(item.appid, &item.assets)
}

pub fn header_image_url_from_app_details(detail: &AppDetailsData) -> Option<String> {
    detail
        .header_image
        .as_ref()
        .map(|url| normalize_store_asset_url(url))
}

pub fn normalize_header_image_url(stored: &str, appid: u32) -> String {
    if stored.is_empty() {
        return normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}steam/apps/{appid}/header.jpg"));
    }
    normalize_store_asset_url(stored)
}

pub fn screenshot_urls(appid: u32, assets: &Option<Assets>, screenshots: &StoreScreenshots) -> Vec<String> {
    let mut urls = Vec::new();
    for list in [
        screenshots.all_ages_screenshots.as_deref(),
        screenshots.mature_content_screenshots.as_deref(),
    ] {
        if let Some(items) = list {
            for shot in items {
                if let Some(filename) = &shot.filename {
                    urls.push(resolve_asset_url(appid, assets, filename));
                }
            }
        }
    }
    urls
}

pub fn screenshot_urls_from_item(item: &StoreItem) -> Vec<String> {
    item.screenshots
        .as_ref()
        .map(|shots| screenshot_urls(item.appid, &item.assets, shots))
        .unwrap_or_default()
}

pub fn trailer_movie_id(trailer: &StoreTrailer) -> Option<String> {
    for path in [&trailer.screenshot_medium, &trailer.screenshot_full] {
        if let Some(path) = path {
            if let Some(id) = path.split('/').next() {
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

pub fn trailer_thumbnail_url(trailer: &StoreTrailer) -> Option<String> {
    let medium = trailer.screenshot_medium.as_ref()?;

    if let Some(format) = &trailer.trailer_url_format {
        let path = format.replace("${FILENAME}", medium);
        if path.starts_with("http://") || path.starts_with("https://") {
            return Some(normalize_store_asset_url(&path));
        }
        return Some(normalize_store_asset_url(&format!("{STORE_ASSETS_BASE}{path}")));
    }

    if medium.contains('/') {
        return Some(normalize_store_asset_url(&format!(
            "{STORE_ASSETS_BASE}steam/apps/{medium}"
        )));
    }

    Some(normalize_store_asset_url(&format!(
        "{STORE_ASSETS_BASE}steam/apps/{medium}/movie.293x165.jpg"
    )))
}

fn microtrailer_video_url(trailer: &StoreTrailer, mime: &str) -> Option<String> {
    trailer.microtrailer.as_ref()?.iter().find_map(|source| {
        if source.source_type.as_deref() == Some(mime) {
            source
                .filename
                .as_ref()
                .map(|filename| format!("{STEAM_TRAILER_VIDEO_CDN}{filename}"))
        } else {
            None
        }
    })
}

pub fn trailer_video_mp4_url(trailer: &StoreTrailer) -> Option<String> {
    microtrailer_video_url(trailer, "video/mp4").or_else(|| {
        let movie_id = trailer_movie_id(trailer)?;
        Some(format!("{STEAM_MOVIE_CDN}{movie_id}/movie480.mp4"))
    })
}

pub fn trailer_video_webm_url(trailer: &StoreTrailer) -> Option<String> {
    microtrailer_video_url(trailer, "video/webm").or_else(|| {
        let movie_id = trailer_movie_id(trailer)?;
        Some(format!("{STEAM_MOVIE_CDN}{movie_id}/movie480.webm"))
    })
}

pub fn is_allowed_video_cdn_url(url: &str) -> bool {
    url.starts_with("https://cdn.cloudflare.steamstatic.com/steam/apps/")
        || url.starts_with("https://cdn.akamai.steamstatic.com/steam/apps/")
        || url.starts_with(STEAM_TRAILER_VIDEO_CDN)
        || url.starts_with("https://video.fastly.steamstatic.com/store_trailers/")
}

#[derive(Debug, Clone)]
pub struct ResolvedTrailer {
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub video_webm_url: Option<String>,
    pub category: Option<i32>,
}

pub fn collect_trailers(trailers: &StoreTrailers) -> Vec<&StoreTrailer> {
    let mut items = Vec::new();
    if let Some(highlights) = &trailers.highlights {
        items.extend(highlights.iter());
    }
    if let Some(other) = &trailers.other_trailers {
        items.extend(other.iter());
    }
    items
}

pub fn trailer_entries(trailers: &StoreTrailers) -> Vec<ResolvedTrailer> {
    collect_trailers(trailers)
        .into_iter()
        .filter_map(resolve_trailer)
        .collect()
}

fn resolve_trailer(trailer: &StoreTrailer) -> Option<ResolvedTrailer> {
    if trailer_video_mp4_url(trailer).is_none() {
        return None;
    }
    Some(ResolvedTrailer {
        name: trailer
            .trailer_name
            .clone()
            .unwrap_or_else(|| "Trailer".to_string()),
        thumbnail_url: trailer_thumbnail_url(trailer),
        video_webm_url: trailer_video_webm_url(trailer),
        category: trailer.trailer_category,
    })
}

pub fn trailer_proxy_video_url(appid: u32, index: usize, webm: bool) -> String {
    if webm {
        format!("/api/games/{appid}/trailers/{index}/video?format=webm")
    } else {
        format!("/api/games/{appid}/trailers/{index}/video")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::client::{Assets, StoreScreenshot, StoreScreenshots, StoreTrailer, TrailerVideoSource};

    fn assets_standard(appid: u32) -> Assets {
        Assets {
            asset_url_format: Some(format!("steam/apps/{appid}/${{FILENAME}}?t=123")),
            header: Some("header.jpg".to_string()),
        }
    }

    fn assets_hash_header(appid: u32) -> Assets {
        Assets {
            asset_url_format: Some(format!("steam/apps/{appid}/${{FILENAME}}?t=456")),
            header: Some("a2a7b6a6846c5a206874a93f1fe403872666e889/header.jpg".to_string()),
        }
    }

    #[test]
    fn header_standard_app() {
        let assets = assets_standard(570);
        let url = header_image_url(570, &Some(assets));
        assert_eq!(
            url,
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570/header.jpg?t=123"
        );
    }

    #[test]
    fn header_hash_filename_app() {
        let assets = assets_hash_header(583270);
        let url = header_image_url(583270, &Some(assets));
        assert_eq!(
            url,
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/583270/a2a7b6a6846c5a206874a93f1fe403872666e889/header.jpg?t=456"
        );
    }

    #[test]
    fn screenshot_full_path_filename() {
        let assets = assets_standard(570);
        let screenshots = StoreScreenshots {
            all_ages_screenshots: Some(vec![StoreScreenshot {
                filename: Some(
                    "steam/apps/570/ss_ad8eee787704745ccdecdfde3a5cd2733704898d.jpg?t=1769535998"
                        .to_string(),
                ),
            }]),
            mature_content_screenshots: None,
        };
        let urls = screenshot_urls(570, &Some(assets), &screenshots);
        assert_eq!(
            urls[0],
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570/ss_ad8eee787704745ccdecdfde3a5cd2733704898d.jpg?t=1769535998"
        );
    }

    #[test]
    fn trailer_urls_from_movie_id() {
        let trailer = StoreTrailer {
            trailer_name: Some("Test Trailer".to_string()),
            trailer_url_format: None,
            trailer_category: Some(0),
            microtrailer: None,
            adaptive_trailers: None,
            screenshot_medium: Some("256692021/movie.293x165.jpg".to_string()),
            screenshot_full: Some("256692021/movie_full.jpg".to_string()),
        };
        assert_eq!(
            trailer_thumbnail_url(&trailer).unwrap(),
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/256692021/movie.293x165.jpg"
        );
        assert_eq!(
            trailer_video_mp4_url(&trailer).unwrap(),
            "https://cdn.cloudflare.steamstatic.com/steam/apps/256692021/movie480.mp4"
        );
        assert_eq!(
            trailer_video_webm_url(&trailer).unwrap(),
            "https://cdn.cloudflare.steamstatic.com/steam/apps/256692021/movie480.webm"
        );
    }

    #[test]
    fn normalize_fixes_double_steam_and_akamai() {
        let url = normalize_store_asset_url(
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/steam/apps/570/header.jpg",
        );
        assert_eq!(
            url,
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570/header.jpg"
        );
    }

    fn trailer_4680930() -> StoreTrailer {
        StoreTrailer {
            trailer_name: Some("Gameplay Trailer".to_string()),
            trailer_url_format: Some("steam/apps/${FILENAME}?t=1781477184".to_string()),
            trailer_category: Some(0),
            microtrailer: Some(vec![
                TrailerVideoSource {
                    filename: Some("4680930/2027708481/b7da0efc55abd03e22cb42dee71bd45d841cc8b4/1781476898/microtrailer.webm".to_string()),
                    source_type: Some("video/webm".to_string()),
                },
                TrailerVideoSource {
                    filename: Some("4680930/2027708481/b7da0efc55abd03e22cb42dee71bd45d841cc8b4/1781476898/microtrailer.mp4".to_string()),
                    source_type: Some("video/mp4".to_string()),
                },
            ]),
            adaptive_trailers: None,
            screenshot_medium: Some(
                "257330559/b277a21c3fa79f7c9f898ba28e3dd121c6f03831/movie_600x337.jpg".to_string(),
            ),
            screenshot_full: Some(
                "257330559/b277a21c3fa79f7c9f898ba28e3dd121c6f03831/movie_full.jpg".to_string(),
            ),
        }
    }

    fn trailer_2478130_cinematic() -> StoreTrailer {
        StoreTrailer {
            trailer_name: Some("Cinematic Trailer".to_string()),
            trailer_url_format: Some("steam/apps/${FILENAME}?t=1782839518".to_string()),
            trailer_category: Some(0),
            microtrailer: Some(vec![
                TrailerVideoSource {
                    filename: Some("2478130/974384988/73d35ea544793b39f27d2e76cfcf4038c76fadaa/1782815061/microtrailer.webm".to_string()),
                    source_type: Some("video/webm".to_string()),
                },
                TrailerVideoSource {
                    filename: Some("2478130/974384988/73d35ea544793b39f27d2e76cfcf4038c76fadaa/1782815061/microtrailer.mp4".to_string()),
                    source_type: Some("video/mp4".to_string()),
                },
            ]),
            adaptive_trailers: None,
            screenshot_medium: Some(
                "257368258/0a5b04666a1c3f9fe338b22e8cd5d0ab6fa5f7e4/movie_600x337.jpg".to_string(),
            ),
            screenshot_full: Some(
                "257368258/0a5b04666a1c3f9fe338b22e8cd5d0ab6fa5f7e4/movie_full.jpg".to_string(),
            ),
        }
    }

    fn trailer_2478130_gameplay() -> StoreTrailer {
        StoreTrailer {
            trailer_name: Some("Gameplay Trailer".to_string()),
            trailer_url_format: Some("steam/apps/${FILENAME}?t=1782839521".to_string()),
            trailer_category: Some(0),
            microtrailer: Some(vec![
                TrailerVideoSource {
                    filename: Some("2478130/40678319/70dadcd605c85cd015ef9dc9f2f78cb1f100920f/1782810621/microtrailer.webm".to_string()),
                    source_type: Some("video/webm".to_string()),
                },
                TrailerVideoSource {
                    filename: Some("2478130/40678319/70dadcd605c85cd015ef9dc9f2f78cb1f100920f/1782810621/microtrailer.mp4".to_string()),
                    source_type: Some("video/mp4".to_string()),
                },
            ]),
            adaptive_trailers: None,
            screenshot_medium: Some(
                "257368262/c8410529edf5f759650485d274b2074acdb85c45/movie_600x337.jpg".to_string(),
            ),
            screenshot_full: Some(
                "257368262/c8410529edf5f759650485d274b2074acdb85c45/movie_full.jpg".to_string(),
            ),
        }
    }

    #[test]
    fn header_app_4680930() {
        let assets = Assets {
            asset_url_format: Some("steam/apps/4680930/${FILENAME}?t=1782722252".to_string()),
            header: Some("09bac78bf3307c5f49025e2ae60132f6ad5f53af/header.jpg".to_string()),
        };
        assert_eq!(
            header_image_url(4680930, &Some(assets)),
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/4680930/09bac78bf3307c5f49025e2ae60132f6ad5f53af/header.jpg?t=1782722252"
        );
    }

    #[test]
    fn screenshot_app_4680930() {
        let assets = Assets {
            asset_url_format: Some("steam/apps/4680930/${FILENAME}?t=1782722252".to_string()),
            header: None,
        };
        let screenshots = StoreScreenshots {
            all_ages_screenshots: Some(vec![StoreScreenshot {
                filename: Some(
                    "steam/apps/4680930/0316acc98bd72fcbc1299d09b7eae6d2954d9a07/ss_0316acc98bd72fcbc1299d09b7eae6d2954d9a07.jpg?t=1782722252"
                        .to_string(),
                ),
            }]),
            mature_content_screenshots: None,
        };
        let urls = screenshot_urls(4680930, &Some(assets), &screenshots);
        assert_eq!(
            urls[0],
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/4680930/0316acc98bd72fcbc1299d09b7eae6d2954d9a07/ss_0316acc98bd72fcbc1299d09b7eae6d2954d9a07.jpg?t=1782722252"
        );
    }

    #[test]
    fn trailer_urls_app_4680930() {
        let trailer = trailer_4680930();
        assert_eq!(
            trailer_thumbnail_url(&trailer).unwrap(),
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/257330559/b277a21c3fa79f7c9f898ba28e3dd121c6f03831/movie_600x337.jpg?t=1781477184"
        );
        assert_eq!(
            trailer_video_mp4_url(&trailer).unwrap(),
            "https://video.akamai.steamstatic.com/store_trailers/4680930/2027708481/b7da0efc55abd03e22cb42dee71bd45d841cc8b4/1781476898/microtrailer.mp4"
        );
        assert_eq!(
            trailer_video_webm_url(&trailer).unwrap(),
            "https://video.akamai.steamstatic.com/store_trailers/4680930/2027708481/b7da0efc55abd03e22cb42dee71bd45d841cc8b4/1781476898/microtrailer.webm"
        );
        assert!(is_allowed_video_cdn_url(&trailer_video_mp4_url(&trailer).unwrap()));
    }

    #[test]
    fn trailer_urls_app_2478130() {
        let cinematic = trailer_2478130_cinematic();
        let gameplay = trailer_2478130_gameplay();

        assert_eq!(
            trailer_thumbnail_url(&cinematic).unwrap(),
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/257368258/0a5b04666a1c3f9fe338b22e8cd5d0ab6fa5f7e4/movie_600x337.jpg?t=1782839518"
        );
        assert_eq!(
            trailer_video_mp4_url(&cinematic).unwrap(),
            "https://video.akamai.steamstatic.com/store_trailers/2478130/974384988/73d35ea544793b39f27d2e76cfcf4038c76fadaa/1782815061/microtrailer.mp4"
        );

        assert_eq!(
            trailer_thumbnail_url(&gameplay).unwrap(),
            "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/257368262/c8410529edf5f759650485d274b2074acdb85c45/movie_600x337.jpg?t=1782839521"
        );
        assert_eq!(
            trailer_video_mp4_url(&gameplay).unwrap(),
            "https://video.akamai.steamstatic.com/store_trailers/2478130/40678319/70dadcd605c85cd015ef9dc9f2f78cb1f100920f/1782810621/microtrailer.mp4"
        );
        assert_ne!(
            trailer_video_mp4_url(&cinematic).unwrap(),
            trailer_video_mp4_url(&gameplay).unwrap()
        );
    }
}
