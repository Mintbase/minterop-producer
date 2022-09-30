crate::forward_mod!(nft_list);
crate::forward_mod!(nft_update_list);
crate::forward_mod!(nft_unlist);
crate::forward_mod!(nft_sold);
crate::forward_mod!(nft_make_offer);
crate::forward_mod!(nft_withdraw_offer);

// TODO: make this a macro with implicit error logging and return
fn parse_list_id(list_id: &str) -> Option<(&str, &str, u64)> {
    list_id
        .split_once(':')
        .and_then(|(token_id, rem)| {
            rem.split_once(':').map(|(approval_id, nft_contract)| {
                (token_id, approval_id, nft_contract)
            })
        })
        .and_then(|(token_id, approval_id, nft_contract)| {
            approval_id
                .parse::<u64>()
                .ok()
                .map(|approval_id| (nft_contract, token_id, approval_id))
        })
}
