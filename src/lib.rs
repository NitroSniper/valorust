//#![warn(missing_docs)]
use serde::{Deserialize, Serialize};

const API_END_POINT: &str = "https://api.henrikdev.xyz/valorant";

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse<T: ValorantAPIData> {
    Success { status: u32, data: T },
    Failure { status: u32, errors: Vec<ApiError> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    message: String,
    code: u32,
    details: String,
}

pub trait ValorantAPIData {}

mod mmr_data {
    use super::{AccountRegion, ValorantAPIData};
    use crate::{ApiResponse, API_END_POINT};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MMRData {
        puuid: String,
        name: String,
        tag: String,
        current_data: CurrentActData,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CurrentActData {
        #[serde(rename = "currenttier")]
        current_tier: u32,
        #[serde(rename = "currenttierpatched")]
        current_tier_patched: String,
        images: RankImages,
        ranking_in_tier: u32,
        mmr_change_to_last_game: i32,
        elo: u32,
        games_needed_for_rating: u32,
        old: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct RankImages {
        small: String,
        large: String,
        triangle_down: String,
        triangle_up: String,
    }

    impl ValorantAPIData for MMRData {}

    impl MMRData {
        async fn new(
            region: AccountRegion,
            name: &str,
            tag: &str,
        ) -> Result<ApiResponse<Self>, reqwest::Error> {
            Ok(reqwest::get(format!(
                "{API_END_POINT}/v2/mmr/{}/{name}/{tag}",
                region.to_string()
            ))
            .await?
            .json()
            .await?)
        }
    }
    #[cfg(test)]
    mod test {
        use super::*;
        use crate::ApiResponse;

        #[tokio::test]
        async fn make_request() {
            let result = MMRData::new(AccountRegion::EU, "NitroSniper", "NERD")
                .await
                .unwrap();
            dbg!(result);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AccountRegion {
    EU,
    NA,
    KR,
    AS,
}
impl AccountRegion {
    fn to_string(&self) -> String {
        match self {
            AccountRegion::EU => "eu",
            AccountRegion::NA => "na",
            AccountRegion::KR => "kr",
            AccountRegion::AS => "as",
        }
        .to_string()
    }
}

pub mod account_data {
    use super::{AccountRegion, ValorantAPIData};
    use crate::{ApiResponse, API_END_POINT};
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    pub struct AccountData {
        puuid: String,
        region: AccountRegion,
        account_level: u32,
        name: String,
        tag: String,
        card: ProfileBanner,
        last_update: String,
        last_update_raw: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProfileBanner {
        small: String,
        large: String,
        wide: String,
        id: String,
    }

    impl ValorantAPIData for AccountData {}

    impl AccountData {
        async fn new(name: &str, tag: &str) -> Result<ApiResponse<Self>, reqwest::Error> {
            Ok(
                reqwest::get(format!("{API_END_POINT}/v1/account/{name}/{tag}"))
                    .await?
                    .json()
                    .await?,
            )
        }
    }
    #[cfg(test)]
    mod test {
        use super::*;
        use crate::ApiResponse;

        #[test]
        fn get_account_data_404() {
            let response_404 = r#"{
            "status": 404,
            "errors": [
            {
                "message": "Not found",
                "code": 0,
                "details": "null"
            }
            ]
        }"#;

            let result: ApiResponse<AccountData> = serde_json::from_str(response_404).unwrap();
            dbg!(result);
        }

        #[test]
        fn get_account_data_eu_200() {
            let response_200 = r#"{
            "status": 200,
            "data": {
                "puuid": "b44adaae-ab83-5001-a296-89ea0de0bce3",
                "region": "eu",
                "account_level": 125,
                "name": "NitroSniper",
                "tag": "NERD",
                "card": {
                    "small": "https://media.valorant-api.com/playercards/bb6ae873-43ec-efb4-3ea6-93ac00a82d4e/smallart.png",
                    "large": "https://media.valorant-api.com/playercards/bb6ae873-43ec-efb4-3ea6-93ac00a82d4e/largeart.png",
                    "wide": "https://media.valorant-api.com/playercards/bb6ae873-43ec-efb4-3ea6-93ac00a82d4e/wideart.png",
                    "id": "bb6ae873-43ec-efb4-3ea6-93ac00a82d4e"
                },
                "last_update": "12 minutes ago",
                "last_update_raw": 1676749780
            }
        }"#;

            let result: ApiResponse<AccountData> = serde_json::from_str(response_200).unwrap();
            dbg!(result);
        }

        #[test]
        fn get_account_data_na_200() {
            let response_200 = r#"{
            "status": 200,
            "data": {
                "puuid": "f14bab04-d739-564b-9704-0c0add689aa5",
                "region": "na",
                "account_level": 76,
                "name": "mads",
                "tag": "ana",
                "card": {
                    "small": "https://media.valorant-api.com/playercards/eba5be7e-4ec7-753b-8678-fa88da1e46ab/smallart.png",
                    "large": "https://media.valorant-api.com/playercards/eba5be7e-4ec7-753b-8678-fa88da1e46ab/largeart.png",
                    "wide": "https://media.valorant-api.com/playercards/eba5be7e-4ec7-753b-8678-fa88da1e46ab/wideart.png",
                    "id": "eba5be7e-4ec7-753b-8678-fa88da1e46ab"
                },
                "last_update": "Now",
                "last_update_raw": 1676761988
            }

        }"#;

            let result: ApiResponse<AccountData> = serde_json::from_str(response_200).unwrap();
            dbg!(result);
        }

        #[test]
        fn get_account_data_br_200() {
            let response_200 = r#"{
            "status": 200,
            "data": {
                "puuid": "8c5b5846-87e1-54ce-8bc9-38ceb3c5629b",
                "region": "na",
                "account_level": 23,
                "name": "anoca",
                "tag": "3945",
                "card": {
                    "small": "https://media.valorant-api.com/playercards/bdc0c02c-441c-8ebc-ec5e-27bff0888ae0/smallart.png",
                    "large": "https://media.valorant-api.com/playercards/bdc0c02c-441c-8ebc-ec5e-27bff0888ae0/largeart.png",
                    "wide": "https://media.valorant-api.com/playercards/bdc0c02c-441c-8ebc-ec5e-27bff0888ae0/wideart.png",
                    "id": "bdc0c02c-441c-8ebc-ec5e-27bff0888ae0"
                },
                "last_update": "Now",
                "last_update_raw": 1676762616
            }
        }"#;
            let result: ApiResponse<AccountData> = serde_json::from_str(response_200).unwrap();
            dbg!(result);
        }

        // TODO - write test cases for korea and asia

        #[tokio::test]
        async fn make_request() {
            let result = AccountData::new("NitroSniper", "NERD").await.unwrap();
            dbg!(result);
        }
    }
}
