use serde::{Deserialize, Serialize};

use crate::{
    file::EncryptedContent,
    room::{OwnedMxcUri, UInt},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
/// Encrypted Image Preview info
pub struct EncryptedPreviewImage {
    /// The target URI
    pub url: OwnedMxcUri,
    /// Encrypted PreviewImage data information
    #[serde(flatten)]
    pub encrypted_image: Box<EncryptedContent>,
}

/// UrlPreview Metadata about an image.
/// modelled after [OpenGraph Image Properties](https://ogp.me/#structured)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PreviewImage {
    /// MXC url to the image
    #[serde(rename = "og:image", alias = "og:image:url", skip_serializing_if = "Option::is_none")]
    pub url: Option<OwnedMxcUri>,

    /// Encrypted PreviewImage data
    #[serde(rename = "matrix:image:encryption", skip_serializing_if = "Option::is_none")]
    pub encrypted_image: Option<EncryptedPreviewImage>,

    /// The size of the image in bytes
    #[serde(
        rename = "matrix:image:size",
        alias = "og:image:size",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<UInt>,

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
impl PreviewImage {
    /// Whether this is effectively empty
    pub fn is_none(&self) -> bool {
        self.url.is_none() && self.encrypted_image.is_none()
    }
    /// Whether this has any preview image information
    pub fn is_some(&self) -> bool {
        self.url.is_some() || self.encrypted_image.is_some()
    }
    pub fn is_encrypted_image(&self) -> bool {
        self.encrypted_image.is_some()
    }
}

/// Preview Information for a URL matched in the message text, according to
/// [MSC 4095](https://github.com/matrix-org/matrix-spec-proposals/pull/4095)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UrlPreview {
    /// The url this was matching on
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

    /// Metadata of a preview image if given
    #[serde(flatten, skip_serializing_if = "PreviewImage::is_none")]
    pub image: PreviewImage,
}

impl UrlPreview {
    /// Whether this preview is empty and thus the users homeserver should be
    /// asked for preview data instead.
    // According to the [MSC](https://github.com/beeper/matrix-spec-proposals/blob/bundled-url-previews/proposals/4095-bundled-url-previews.md)
    // if there is a match but no fields (other than the matched_url), the client
    // should fallback to asking the homeserver
    pub fn should_ask_homeserver(&self) -> bool {
        self.url.is_none()
            && self.title.is_none()
            && self.description.is_none()
            && self.image.is_none()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::{super::text::TextMessageEventContent, *};

    #[test]
    fn parsing_regular_example() {
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
        assert!(image.is_some());
        assert!(!image.is_encrypted_image());
        let PreviewImage { size, url, height, width, mimetype, encrypted_image } = image.clone();
        assert!(encrypted_image.is_none());
        assert_eq!(u64::from(size.unwrap()), 16588);
        assert_eq!(
            url.clone().unwrap().to_string(),
            "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO".to_owned()
        );
        assert_eq!(u64::from(height.unwrap()), 400);
        assert_eq!(u64::from(width.unwrap()), 800);
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }

    #[test]
    fn parsing_example_no_previews() {
        let normal_preview = json!({
                      "msgtype": "m.text",
                      "body": "https://matrix.org",
                      "m.url_previews": [],
                      "m.mentions": {}
        });
        let message_with_preview: TextMessageEventContent = from_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        assert!(url_previews.clone().unwrap().is_empty(), "Unexpectedly found url previews");
    }

    #[test]
    fn parsing_example_empty_previews() {
        let normal_preview = json!({
                "msgtype": "m.text",
                "body": "https://matrix.org",
                "m.url_previews": [
                  {
                    "matrix:matched_url": "https://matrix.org"
                  }
                ],
                "m.mentions": {}
        });

        let message_with_preview: TextMessageEventContent = from_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let preview = previews.first().unwrap();
        assert_eq!(preview.matched_url, "https://matrix.org");
        assert!(preview.should_ask_homeserver());
    }

    #[test]
    fn parsing_encrypted_image_example() {
        let normal_preview = json!({
              "msgtype": "m.text",
              "body": "https://matrix.org",
              "m.url_previews": [
                {
                  "matrix:matched_url": "https://matrix.org",
                  "og:title": "Matrix.org",
                  "og:url": "https://matrix.org/",
                  "og:description": "Matrix, the open protocol for secure decentralised communications",
                  "matrix:image:size": 16588,
                  "og:image:height": 400,
                  "og:image:type": "image/jpeg",
                  "og:image:width": 800,

        "matrix:image:encryption": {
            "key": {
              "k": "GRAgOUnbbkcd-UWoX5kTiIXJII81qwpSCnxLd5X6pxU",
              "alg": "A256CTR",
              "ext": true,
              "kty": "oct",
              "key_ops": [
                "encrypt",
                "decrypt"
              ]
            },
            "iv": "kZeoJfx4ehoAAAAAAAAAAA",
            "hashes": {
              "sha256": "WDOJYFegjAHNlaJmOhEPpE/3reYeD1pRvPVcta4Tgbg"
            },
            "v": "v2",
            "url": "mxc://beeper.com/53207ac52ce3e2c722bb638987064bfdc0cc257b"
          }
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
        assert!(image.is_some());
        assert!(image.is_encrypted_image());
        let PreviewImage { size, url, height, width, mimetype, encrypted_image } = image.clone();
        assert!(encrypted_image.is_some());
        assert!(url.is_none());
        assert_eq!(u64::from(size.unwrap()), 16588);
        assert_eq!(
            encrypted_image.clone().unwrap().url.to_string(),
            "mxc://beeper.com/53207ac52ce3e2c722bb638987064bfdc0cc257b".to_owned()
        );
        assert_eq!(u64::from(height.unwrap()), 400);
        assert_eq!(u64::from(width.unwrap()), 800);
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }

    #[test]
    #[cfg(feature = "unstable-msc1767")]
    fn parsing_extensible_example() {
        use crate::message::MessageEventContent;
        let normal_preview = json!({
              "m.text": [
                {"body": "matrix.org/support"}
              ],
              "m.url_previews": [
                {
                  "matrix:matched_url": "matrix.org/support",
                  "matrix:image:size": 16588,
                  "og:description": "Matrix, the open protocol for secure decentralised communications",
                  "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                  "og:image:height": 400,
                  "og:image:type": "image/jpeg",
                  "og:image:width": 800,
                  "og:title": "Support Matrix",
                  "og:url": "https://matrix.org/support/"
                }
              ],
              "m.mentions": {}
            }
        );

        let message_with_preview: MessageEventContent = from_value(normal_preview).unwrap();
        let MessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let preview = previews.first().unwrap();
        assert!(!preview.should_ask_homeserver());
        let UrlPreview { image, matched_url, title, url, description } = preview;
        assert_eq!(matched_url, "matrix.org/support");
        assert_eq!(title.as_ref().unwrap(), "Support Matrix");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/support/");

        // Check the preview image parsed:
        assert!(image.is_some());
        assert!(!image.is_encrypted_image());
        let PreviewImage { size, url, height, width, mimetype, encrypted_image } = image.clone();
        assert!(encrypted_image.is_none());
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
