/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// PartyInvite : A party invite.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartyInvite {
    #[serde(rename = "alias", skip_serializing_if = "Option::is_none")]
    pub alias: Option<Box<crate::models::PartyInviteAlias>>,
    /// RFC3339 timestamp.
    #[serde(rename = "create_ts")]
    pub create_ts: String,
    #[serde(rename = "external")]
    pub external: Box<crate::models::PartyInviteExternalLinks>,
    #[serde(rename = "invite_id")]
    pub invite_id: uuid::Uuid,
    /// A JSON Web Token. Slightly modified to include a description prefix and use Protobufs of JSON.
    #[serde(rename = "token")]
    pub token: String,
}

impl PartyInvite {
    /// A party invite.
    pub fn new(create_ts: String, external: crate::models::PartyInviteExternalLinks, invite_id: uuid::Uuid, token: String) -> PartyInvite {
        PartyInvite {
            alias: None,
            create_ts,
            external: Box::new(external),
            invite_id,
            token,
        }
    }
}


