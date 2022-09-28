fn main() {
    println!("Mintbase indexer integration tests.");
}

#[cfg(test)]
mod tests {
    use std::env;

    use diesel::{
        pg::PgConnection,
        Connection,
        ExpressionMethods,
        QueryDsl,
        RunQueryDsl,
    };
    use dotenv::dotenv;

    const PARAS_TOKEN_CONTRACT: &str = "paras-token-v2.testnet";
    const MB_STORE_CONTRACT: &str = "mb_store.mintspace2.testnet";

    pub fn establish_connection() -> PgConnection {
        dotenv().ok();
        let postgres_env_key = "POSTGRES";
        // let postgres_env_key = "POSTGRES_DEBUG"; // useful to debug the tests
        let database_url =
            env::var(postgres_env_key).expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }

    #[test]
    fn syncs_to_stop_height() {
        use minterop_common::schema::blocks::dsl::*;
        let conn = establish_connection();
        let expected_height = env::var("STOP_BLOCK_HEIGHT")
            .expect("Stop height needs to be specified")
            .parse::<i64>()
            .unwrap();
        let sync_height =
            blocks.select(synced_height).first::<i64>(&conn).unwrap();
        assert_eq!(sync_height, expected_height + 1);
    }

    #[test]
    fn indexes_contracts() {
        use minterop_common::schema::nft_contracts::dsl::*;
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
        use minterop_common::schema::nft_activities::dsl::*;
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
        use minterop_common::schema::nft_activities::dsl::*;
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
        use minterop_common::schema::nft_approvals::dsl::*;
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
        use minterop_common::schema::nft_earnings::dsl::*;
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
        use minterop_common::schema::nft_listings::dsl::*;
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
        use minterop_common::schema::nft_metadata::dsl::*;
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
        use minterop_common::schema::nft_offers::dsl::*;
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
        use minterop_common::schema::nft_offers::dsl::*;
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
        use minterop_common::schema::nft_tokens::dsl::*;
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
        use minterop_common::schema::nft_tokens::dsl::*;
        let conn = establish_connection();
        let num_tokens = nft_tokens
            .filter(nft_contract_id.eq(PARAS_TOKEN_CONTRACT.to_string()))
            .select(nft_contract_id)
            .count()
            .get_result::<i64>(&conn)
            .unwrap();
        assert!(num_tokens >= 1);
    }

    // TODO: Get nuts, but not brittle!
    // More activity, reference unwrapping etc.
}
