use crate::models::GamePlayModes;
use crate::steam::client::StoreItem;

pub const MULTIPLAYER: u32 = 1;
pub const COOP: u32 = 9;
pub const ONLINE_COOP: u32 = 38;
pub const REMOTE_PLAY_TOGETHER: u32 = 44;

pub fn play_modes_from_item(item: &StoreItem) -> Option<GamePlayModes> {
    let ids = item
        .categories
        .as_ref()?
        .supported_player_categoryids
        .as_ref()?;

    Some(play_modes_from_ids(ids))
}

pub fn play_modes_from_ids(ids: &[u32]) -> GamePlayModes {
    GamePlayModes {
        online_coop: ids.contains(&ONLINE_COOP),
        coop: ids.contains(&COOP),
        multiplayer: ids.contains(&MULTIPLAYER),
        remote_play_together: ids.contains(&REMOTE_PLAY_TOGETHER),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::client::{StoreCategories, StoreItem};

    fn item_with_players(ids: Vec<u32>) -> StoreItem {
        StoreItem {
            appid: 1,
            name: None,
            is_coming_soon: None,
            basic_info: None,
            assets: None,
            tags: None,
            release: None,
            screenshots: None,
            trailers: None,
            full_description_bbcode: None,
            best_purchase_option: None,
            categories: Some(StoreCategories {
                supported_player_categoryids: Some(ids),
                feature_categoryids: None,
            }),
        }
    }

    fn supports_multiplayer_together(modes: &GamePlayModes) -> bool {
        modes.online_coop || modes.coop || modes.multiplayer || modes.remote_play_together
    }

    #[test]
    fn elden_ring_style_categories() {
        let modes = play_modes_from_item(&item_with_players(vec![2, 1, 49, 36, 9, 38])).unwrap();
        assert!(modes.online_coop);
        assert!(modes.coop);
        assert!(modes.multiplayer);
        assert!(!modes.remote_play_together);
        assert!(supports_multiplayer_together(&modes));
    }

    #[test]
    fn single_player_only() {
        let modes = play_modes_from_item(&item_with_players(vec![2])).unwrap();
        assert!(!supports_multiplayer_together(&modes));
    }

    #[test]
    fn multiplayer_without_online_coop_tag() {
        let modes = play_modes_from_item(&item_with_players(vec![1, 9])).unwrap();
        assert!(!modes.online_coop);
        assert!(supports_multiplayer_together(&modes));
    }
}
