/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// GroupExternalLinks : External links for this group.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GroupExternalLinks {
    /// A link to this group's chat page.
    #[serde(rename = "chat")]
    pub chat: String,
    /// A link to this group's profile page.
    #[serde(rename = "profile")]
    pub profile: String,
}

impl GroupExternalLinks {
    /// External links for this group.
    pub fn new(chat: String, profile: String) -> GroupExternalLinks {
        GroupExternalLinks {
            chat,
            profile,
        }
    }
}

