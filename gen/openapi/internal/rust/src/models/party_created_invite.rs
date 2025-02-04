/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// PartyCreatedInvite : Output from a created invite.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartyCreatedInvite {
    /// An alias used to join a given party. This alias must be unique for all invites for your game. Pass this alias to `rivet.api.party.common#CreatedInvite$alias` to consume the invite.
    #[serde(rename = "alias", skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// A JSON Web Token. Slightly modified to include a description prefix and use Protobufs of JSON.
    #[serde(rename = "token")]
    pub token: String,
}

impl PartyCreatedInvite {
    /// Output from a created invite.
    pub fn new(token: String) -> PartyCreatedInvite {
        PartyCreatedInvite {
            alias: None,
            token,
        }
    }
}


