/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartyGetInviteResponse {
    #[serde(rename = "party")]
    pub party: Box<crate::models::PartySummary>,
}

impl PartyGetInviteResponse {
    pub fn new(party: crate::models::PartySummary) -> PartyGetInviteResponse {
        PartyGetInviteResponse {
            party: Box::new(party),
        }
    }
}


