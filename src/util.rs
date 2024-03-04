use mb_sdk::near_sdk::AccountId;
use mb_sdk::types::nft_core::SafeFraction;
use std::collections::HashMap;

pub(crate) type SafeFractionMap = HashMap<AccountId, SafeFraction>;
pub(crate) type U16Map = HashMap<String, u16>;

pub(crate) fn map_fractions_to_u16(
    safe_fraction_map: &SafeFractionMap,
) -> U16Map {
    safe_fraction_map
        .iter()
        .map(|(account_id, safe_fraction)| {
            (account_id.to_string(), safe_fraction.numerator as u16)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn get_safe_fraction_map() -> SafeFractionMap {
        let mut m = HashMap::new();
        m.insert(
            AccountId::from_str("a.near").unwrap(),
            SafeFraction { numerator: 100 },
        );
        m.insert(
            AccountId::from_str("b.near").unwrap(),
            SafeFraction { numerator: 200 },
        );
        m
    }
    fn get_u16_map() -> U16Map {
        let mut m = HashMap::new();
        m.insert("a.near".to_string(), 100);
        m.insert("b.near".to_string(), 200);
        m
    }

    #[test]
    fn test_map_fractions_to_u16() {
        assert_eq!(
            map_fractions_to_u16(&get_safe_fraction_map()),
            get_u16_map()
        )
    }
}
