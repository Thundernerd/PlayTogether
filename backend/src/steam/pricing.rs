use crate::models::GamePrice;
use crate::steam::client::{PurchaseOption, StoreItem};

pub fn price_from_store_item(item: &StoreItem) -> Option<GamePrice> {
    let option = item.best_purchase_option.as_ref()?;
    price_from_purchase_option(option)
}

pub fn price_from_purchase_option(option: &PurchaseOption) -> Option<GamePrice> {
    let final_cents = parse_cents(option.final_price_in_cents.as_deref()?)?;
    let discount_percent = option.discount_pct.filter(|pct| *pct > 0);
    let initial_cents = option
        .original_price_in_cents
        .as_ref()
        .and_then(|value| parse_cents(value))
        .or_else(|| {
            if discount_percent.is_some() {
                None
            } else {
                Some(final_cents)
            }
        });

    let discount_end_date = option
        .active_discounts
        .as_ref()
        .and_then(|discounts| discounts.first())
        .and_then(|discount| discount.discount_end_date);

    Some(GamePrice {
        initial_cents,
        final_cents: Some(final_cents),
        discount_percent,
        formatted_initial: option.formatted_original_price.clone(),
        formatted_final: option.formatted_final_price.clone(),
        discount_end_date,
    })
}

fn parse_cents(value: &str) -> Option<i32> {
    value.parse().ok()
}

pub fn store_item_has_pricing(item: &StoreItem) -> bool {
    item.best_purchase_option.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::client::{ActiveDiscount, PurchaseOption};

    #[test]
    fn discounted_game() {
        let option = PurchaseOption {
            final_price_in_cents: Some("1799".to_string()),
            original_price_in_cents: Some("5999".to_string()),
            formatted_final_price: Some("17,99€".to_string()),
            formatted_original_price: Some("59,99€".to_string()),
            discount_pct: Some(70),
            active_discounts: Some(vec![ActiveDiscount {
                discount_end_date: Some(1_783_616_400),
                discount_amount: None,
                discount_description: None,
            }]),
            ..default_purchase_option()
        };

        let price = price_from_purchase_option(&option).unwrap();
        assert_eq!(price.discount_percent, Some(70));
        assert_eq!(price.final_cents, Some(1799));
        assert_eq!(price.initial_cents, Some(5999));
        assert_eq!(price.discount_end_date, Some(1_783_616_400));
    }

    #[test]
    fn full_price_game() {
        let option = PurchaseOption {
            final_price_in_cents: Some("5999".to_string()),
            formatted_final_price: Some("59,99€".to_string()),
            ..default_purchase_option()
        };

        let price = price_from_purchase_option(&option).unwrap();
        assert_eq!(price.discount_percent, None);
        assert_eq!(price.final_cents, Some(5999));
        assert_eq!(price.initial_cents, Some(5999));
    }

    fn default_purchase_option() -> PurchaseOption {
        PurchaseOption {
            packageid: None,
            purchase_option_name: None,
            final_price_in_cents: None,
            original_price_in_cents: None,
            formatted_final_price: None,
            formatted_original_price: None,
            discount_pct: None,
            active_discounts: None,
        }
    }
}
