pub(crate) type SafeFractionMap = std::collections::HashMap<
    mb_sdk::near_sdk::AccountId,
    mb_sdk::types::nft_core::SafeFraction,
>;
pub(crate) type U16Map = std::collections::HashMap<String, u16>;

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
