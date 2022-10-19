use diesel::{
    ExpressionMethods,
    QueryDsl,
    RunQueryDsl,
};

// ------------------------------- constants -------------------------------- //

const PARAS_TOKEN_CONTRACT: &str = "paras-token-v2.testnet";
const MB_STORE_CONTRACT: &str = "mb_store.mintspace2.testnet";

// ---------------------------- helper functions ---------------------------- //

fn unwrap_env_var(varname: &str) -> String {
    std::env::var(varname).expect(&format!(
        "Environment variable {} needs to be defined!",
        varname
    ))
}

fn establish_connection() -> diesel::pg::PgConnection {
    use diesel::Connection;

    let database_url = unwrap_env_var("POSTGRES");
    diesel::pg::PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

//------------- TODO: REACTIVATE ALL ASSERTIONS WHEN RESOLVING WORKS WITH TESTING ---------------//
//
// #[test]
// fn syncs_to_stop_height() {
//     use minterop_data::schema::blocks::dsl::*;
//     let conn = establish_connection();
//     let expected_height = unwrap_env_var("STOP_BLOCK_HEIGHT")
//         .parse::<i64>()
//         .expect("STOP_BLOCK_HEIGHT needs to be an integer");
//     let sync_height = blocks.select(synced_height).first::<i64>(&conn).unwrap();
//     assert_eq!(sync_height, expected_height + 1);
// }

#[test]
fn indexes_contracts() {
    use minterop_data::schema::nft_contracts::dsl::*;
    let conn = establish_connection();
    let num_contracts = nft_contracts
        .select(id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_contracts >= 1);
}

#[test]
fn indexes_activities() {
    use minterop_data::schema::nft_activities::dsl::*;
    let conn = establish_connection();
    let activity_types = nft_activities
        .filter(nft_contract_id.eq(MB_STORE_CONTRACT.to_string()))
        .select(kind)
        .distinct()
        .get_results::<String>(&conn)
        .unwrap();
    assert!(activity_types.contains(&"mint".to_string()));
    assert!(activity_types.contains(&"transfer".to_string()));
    assert!(activity_types.contains(&"approve".to_string()));
    assert!(activity_types.contains(&"make_offer".to_string()));
    assert!(activity_types.contains(&"sale".to_string()));
    assert!(activity_types.contains(&"burn".to_string()));
}

#[test]
fn indexes_paras_activities() {
    use minterop_data::schema::nft_activities::dsl::*;
    let conn = establish_connection();
    let activity_types = nft_activities
        .filter(nft_contract_id.eq(PARAS_TOKEN_CONTRACT.to_string()))
        .select(kind)
        .distinct()
        .get_results::<String>(&conn)
        .unwrap();
    assert!(activity_types.contains(&"mint".to_string()));
    assert!(activity_types.contains(&"make_offer".to_string()));
    assert!(activity_types.contains(&"transfer".to_string()));
    assert!(activity_types.contains(&"list".to_string()));
    assert!(activity_types.contains(&"sale".to_string()));
}

#[test]
fn indexes_approvals() {
    use minterop_data::schema::nft_approvals::dsl::*;
    let conn = establish_connection();
    let num_approvals = nft_approvals
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_approvals >= 1);
}

#[test]
fn indexes_earnings() {
    use minterop_data::schema::nft_earnings::dsl::*;
    let conn = establish_connection();
    let num_earnings = nft_earnings
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_earnings >= 3);
}

#[test]
fn indexes_listings() {
    use minterop_data::schema::nft_listings::dsl::*;
    let conn = establish_connection();
    let listing_contracts = nft_listings
        .select(nft_contract_id)
        .distinct()
        .get_results::<String>(&conn)
        .unwrap();
    assert!(listing_contracts.contains(&PARAS_TOKEN_CONTRACT.to_string()));
    assert!(listing_contracts.contains(&MB_STORE_CONTRACT.to_string()));
}

#[test]
fn indexes_metadata() {
    use minterop_data::schema::nft_metadata::dsl::*;
    let conn = establish_connection();
    let metadata_contracts = nft_metadata
        .select(nft_contract_id)
        .distinct()
        .get_results::<String>(&conn)
        .unwrap();
    assert!(metadata_contracts.contains(&PARAS_TOKEN_CONTRACT.to_string()));
    assert!(metadata_contracts.contains(&MB_STORE_CONTRACT.to_string()));
}

#[test]
fn indexes_all_paras_offers() {
    use minterop_data::schema::nft_offers::dsl::*;
    let conn = establish_connection();
    let num_offers = nft_offers
        .filter(nft_contract_id.eq(PARAS_TOKEN_CONTRACT.to_string()))
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_offers >= 1);
}

#[test]
fn indexes_all_mb_offers() {
    use minterop_data::schema::nft_offers::dsl::*;
    let conn = establish_connection();
    let num_offers = nft_offers
        .filter(nft_contract_id.eq(MB_STORE_CONTRACT.to_string()))
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_offers >= 1);
}

#[test]
fn indexes_all_mb_tokens() {
    use minterop_data::schema::nft_tokens::dsl::*;
    let conn = establish_connection();
    let num_tokens = nft_tokens
        .filter(nft_contract_id.eq(MB_STORE_CONTRACT.to_string()))
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_tokens >= 3);
}

#[test]
fn indexes_all_paras_tokens() {
    use minterop_data::schema::nft_tokens::dsl::*;
    let conn = establish_connection();
    let num_tokens = nft_tokens
        .filter(nft_contract_id.eq(PARAS_TOKEN_CONTRACT.to_string()))
        .select(nft_contract_id)
        .count()
        .get_result::<i64>(&conn)
        .unwrap();
    assert!(num_tokens >= 1);
}
