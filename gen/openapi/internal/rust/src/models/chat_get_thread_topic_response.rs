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
pub struct ChatGetThreadTopicResponse {
    #[serde(rename = "topic")]
    pub topic: Box<crate::models::ChatSimpleTopic>,
}

impl ChatGetThreadTopicResponse {
    pub fn new(topic: crate::models::ChatSimpleTopic) -> ChatGetThreadTopicResponse {
        ChatGetThreadTopicResponse {
            topic: Box::new(topic),
        }
    }
}


