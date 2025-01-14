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
pub struct ChatGetThreadHistoryResponse {
    /// Ordered old to new. If querying `rivet.api.chat.common#before_and_after`, this will be `count * 2` long.
    #[serde(rename = "chat_messages")]
    pub chat_messages: Vec<crate::models::ChatMessage>,
}

impl ChatGetThreadHistoryResponse {
    pub fn new(chat_messages: Vec<crate::models::ChatMessage>) -> ChatGetThreadHistoryResponse {
        ChatGetThreadHistoryResponse {
            chat_messages,
        }
    }
}


