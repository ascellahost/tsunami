use crate::prelude::*;

/// EmbedData
///
/// Data for the embed
#[derive(Deserialize, Apiv2Schema, Clone, TS)]
#[ts(export)]
pub struct EmbedData {
    /// THe title of the embed this will be displayed
    #[openapi(example = "Hi")]
    pub title: Option<String>,
    /// The link that is used in your embed
    #[allow(dead_code)]
    pub link: Option<String>,
    pub url: Option<String>,
    #[openapi(example = "This is a image")]
    pub description: Option<String>,
    #[openapi(example = "#5f8f91")]
    pub color: Option<String>,
    #[openapi(example = "Some author")]
    pub author: Option<String>,
}
