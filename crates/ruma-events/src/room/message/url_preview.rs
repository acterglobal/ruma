use serde::{Deserialize, Serialize};

use crate::room::{OwnedMxcUri, UInt};

/// UrlPreview Metadata about an image.
/// modelled after [OpenGraph Image Properties](https://ogp.me/#structured)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PreviewImage {
    /// The size of the image in bytes
    #[serde(
        rename = "matrix:image:size",
        alias = "og:image:size",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<UInt>,

    /// MXC url to the image
    #[serde(rename = "og:image", alias = "og:image:url", skip_serializing_if = "Option::is_none")]
    pub url: Option<OwnedMxcUri>,

    /// The width of the image in pixels.
    #[serde(rename = "og:image:width", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The height of the image in pixels.
    #[serde(rename = "og:image:height", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The mime_type of the image
    #[serde(rename = "og:image:type", skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UrlPreview {
    #[serde(alias = "matrix:matched_url")]
    pub matched_url: String,

    /// Canonical URL
    #[serde(rename = "og:url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Title to use for the preview
    #[serde(rename = "og:title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description to use for the preview
    #[serde(rename = "og:description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // extra fields
    #[serde(flatten)]
    pub image: Option<PreviewImage>,
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::{super::text::TextMessageEventContent, *};

    #[test]
    fn parsing_normal_example() {
        let normal_preview = json!({
              "msgtype": "m.text",
              "body": "https://matrix.org",
              "m.url_previews": [
                {
                  "matrix:matched_url": "https://matrix.org",
                  "matrix:image:size": 16588,
                  "og:description": "Matrix, the open protocol for secure decentralised communications",
                  "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                  "og:image:height": 400,
                  "og:image:type": "image/jpeg",
                  "og:image:width": 800,
                  "og:title": "Matrix.org",
                  "og:url": "https://matrix.org/"
                }
              ],
              "m.mentions": {}
            }
        );

        let message_with_preview: TextMessageEventContent = from_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let UrlPreview { image, matched_url, title, url, description } = previews.first().unwrap();
        assert_eq!(matched_url, "https://matrix.org");
        assert_eq!(title.as_ref().unwrap(), "Matrix.org");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/");

        // Check the preview image parsed:

        let PreviewImage { size, url, height, width, mimetype } = image.clone().unwrap();
        assert_eq!(u64::from(size.unwrap()), 16588);
        assert_eq!(
            url.clone().unwrap().to_string(),
            "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO".to_owned()
        );
        assert_eq!(u64::from(height.unwrap()), 400);
        assert_eq!(u64::from(width.unwrap()), 800);
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }
}
