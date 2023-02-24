use prelude::EpisodeAndAct;
//#![warn(missing_docs)]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AccountRegion {
    EU,
    NA,
    KR,
    AS,
}
impl AccountRegion {
    fn to_value(&self) -> String {
        match self {
            AccountRegion::EU => "eu",
            AccountRegion::NA => "na",
            AccountRegion::KR => "kr",
            AccountRegion::AS => "as",
        }
        .to_string()
    }
}

pub struct ValorantClient<'a> {
    api_end_point: &'a str,
}

impl<'a> ValorantClient<'a> {
    pub fn new() -> Self {
        ValorantClient::default()
    }

    pub fn change_api_endpoint(mut self, endpoint: &'a str) -> Self {
        self.api_end_point = endpoint;
        self
    }

    pub async fn request<T>(
        &self,
        api_type: ValorantApiType<'_>,
    ) -> Result<ApiResponse<T>, reqwest::Error>
    where
        T: DeserializeOwned + ValorantAPIData,
    {
        reqwest::get(format!("{}/{}", self.api_end_point, api_type.to_url()))
            .await?
            .json()
            .await
    }
}

impl Default for ValorantClient<'_> {
    fn default() -> Self {
        ValorantClient {
            api_end_point: "https://api.henrikdev.xyz/valorant",
        }
    }
}

pub enum ValorantApiType<'a> {
    MMRData {
        region: AccountRegion,
        name: &'a str,
        tag: &'a str,
        filter: Option<EpisodeAndAct>
    },
    AccountData {
        name: &'a str,
        tag: &'a str,
    },
}

impl<'a> ValorantApiType<'a> {
    pub fn to_url(&self) -> String {
        match self {
            Self::MMRData { region, name, tag, filter} => {
                format!("v2/mmr/{}/{}/{}", region.to_value(), name, tag)
            }
            Self::AccountData { name, tag } => {
                format!("v1/account/{}/{}", name, tag)
            }
        }
    }
}

pub mod prelude {
    pub use crate::account_data::AccountData;
    pub use crate::mmr_data::MMRData;
    pub use crate::AccountRegion;
    pub use crate::ApiResponse;
    pub use crate::ValorantApiType;
    pub use crate::ValorantClient;
    pub use crate::mmr_data::EpisodeAndAct;
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
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

        let result: ApiResponse<MMRData> = serde_json::from_str(response_404).unwrap();
        dbg!(result);
    }

    #[tokio::test]
    async fn making_a_call() {
        let api_user = ValorantClient::new();
        let result = api_user
            .request::<MMRData>(ValorantApiType::MMRData {
                region: AccountRegion::EU,
                name: "NitroSniper",
                tag: "NERD",
                filter: None
            })
            .await
            .unwrap();
        dbg!(result);
    }
}

pub mod mmr_data {
    use crate::ValorantAPIData;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MMRData {
        puuid: String,
        name: String,
        tag: String,
        current_data: CurrentActData,
        highest_rank: HighestRank,
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

    #[derive(Serialize, Deserialize, Debug)]
    struct HighestRank {
        old: bool,
        tier: u32,
        patched_tier: String,
        season: EpisodeAndAct
    }

    #[derive(Debug)]
    pub struct EpisodeAndAct {
        episode: u32,
        act: u32,
    }

    impl EpisodeAndAct {
        pub fn to_value(&self) -> String {
            format!("e{}a{}", self.episode, self.act)
        }
    }

    // Create a Serialize and Deserialize implementation for SeasonAndActData that turn season and
    // act into a string in the form of "s{season}a{act}"
    impl Serialize for EpisodeAndAct {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let s = format!("e{}a{}", self.episode, self.act);
            serializer.serialize_str(&s)
        }
    }

    impl<'de> Deserialize<'de> for EpisodeAndAct {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;

            let chars = string.chars().collect::<Vec<_>>();
            let (e, episode_number, a, act_number) = match chars.as_slice() {
                [e, episode_number, a, act_number] => (*e, *episode_number, *a, *act_number),
                _ => {
                    return Err(serde::de::Error::custom("Invalid length"));
                },
            };
            // write serde Errors where it checks them and return good error along with the values
            if e != 'e' || a != 'a' {
                return Err(serde::de::Error::custom(format!("Invalid format, format recieved: {string}")));
            }
            if !episode_number.is_numeric() || !act_number.is_numeric() {
                return Err(serde::de::Error::custom(format!("Invalid format, format recieved: {string}")));
            }

            // get the data
            Ok(Self { episode: episode_number.into(), act: act_number.into() })
        }
    }

    impl ValorantAPIData for MMRData {}

    struct ActRankStats {
        wins: u32,
        number_of_games: u32,
        final_rank: u32,

    }
    

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::ApiResponse;
        #[test]
        fn deserialize_response() {
            let response = r#"{
                "status": 200,
                "data": {
                    "name": "NitroSniper",
                    "tag": "NERD",
                    "puuid": "b44adaae-ab83-5001-a296-89ea0de0bce3",
                    "current_data": {
                        "currenttier": 16,
                        "currenttierpatched": "Platinum 2",
                        "images": {
                            "small": "https://media.valorant-api.com/competitivetiers/03621f52-342b-cf4e-4f86-9350a49c6d04/16/smallicon.png",
                            "large": "https://media.valorant-api.com/competitivetiers/03621f52-342b-cf4e-4f86-9350a49c6d04/16/largeicon.png",
                            "triangle_down": "https://media.valorant-api.com/competitivetiers/03621f52-342b-cf4e-4f86-9350a49c6d04/16/ranktriangledownicon.png",
                            "triangle_up": "https://media.valorant-api.com/competitivetiers/03621f52-342b-cf4e-4f86-9350a49c6d04/16/ranktriangleupicon.png"
                        },
                        "ranking_in_tier": 47,
                        "mmr_change_to_last_game": -11,
                        "elo": 1347,
                        "games_needed_for_rating": 0,
                        "old": false
                    },
                    "highest_rank": {
                        "old": false,
                        "tier": 18,
                        "patched_tier": "Diamond 1",
                        "season": "e5a3"
                    }
                }
            }"#;
            let result = serde_json::from_str::<ApiResponse<MMRData>>(response).unwrap();
            dbg!(result);
        }

        // write edge cases for season and act
        // 1. act can't be greater than 3, season must be length of 4 with value "s{season}a{act}"
        #[test]
        fn edge_cases() {
            let season_input = r#""e5a5""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_err());

            let season_input = r#""e5a-1""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_err());

            let season_input = r#""e5a1""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_ok());

            let season_input = r#""e5a3""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_ok());
            let season_input = r#""e5a2""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_ok());

            let season_input = r#""e5a0""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_err());

            let season_input = r#""sdfa""#;
            let result = serde_json::from_str::<EpisodeAndAct>(season_input);
            assert!(result.is_err());
        }
    }
}

mod account_data {
    use crate::{AccountRegion, ValorantAPIData};
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

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::{ApiResponse, ValorantClient};

        #[test]
        fn deserialize_response() {
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
        fn deserialize_response_na() {
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
        fn deserialize_response_br() {
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

    }
}
