//! Formula 1 streaming provider that fetches live broadcast channels from iptv-org.

use async_trait::async_trait;
use std::time::Duration;

use crate::SlothError;
use crate::providers::models::{EpisodeRef, Media, MediaType, ProviderError, Quality, StreamUrl};
use crate::providers::tv::parser::M3UParser;
use crate::providers::{Provider, ProviderCapabilities};

pub const SPORTS_M3U_URL: &str = "https://iptv-org.github.io/iptv/categories/sports.m3u";

/// Checks whether a channel name or group indicates a Formula 1 broadcast.
pub fn is_f1_channel(name: &str, group: &str) -> bool {
    let lower_name = name.to_lowercase();
    let lower_group = group.to_lowercase();

    if lower_name.contains("formula")
        || lower_name.contains("sky sports f1")
        || lower_group.contains("formula 1")
        || lower_group.contains("f1")
    {
        return true;
    }

    // Match "f1" with word delimiters to avoid false positives like "california"
    for part in lower_name.split(|c: char| !c.is_alphanumeric()) {
        if part == "f1" {
            return true;
        }
    }

    false
}

/// Parses raw M3U text and filters for F1 channels, returning them as [`StreamUrl`] items.
pub fn parse_f1_m3u(content: &str) -> Vec<StreamUrl> {
    let parser = M3UParser::new();
    let channels = parser.parse_m3u(content);

    channels
        .into_iter()
        .filter(|ch| is_f1_channel(&ch.name, &ch.group))
        .map(|ch| {
            let is_hls = ch.stream_url.contains(".m3u8") || !ch.stream_url.ends_with(".mp4");
            StreamUrl {
                url: ch.stream_url,
                quality: Quality::Auto,
                is_hls,
                headers: vec![("User-Agent".to_string(), "Sloth-TUI/0.1.0".to_string())],
                subtitle_url: None,
                provider_id: "f1",
            }
        })
        .collect()
}

/// Fetches sports channels from iptv-org and returns Formula 1 streams.
pub async fn fetch_f1_streams() -> Result<Vec<StreamUrl>, SlothError> {
    let parser = M3UParser::new();
    match parser
        .fetch_playlist(SPORTS_M3U_URL)
        .await
        .map_err(|e| e.to_string())
    {
        Ok(channels) => {
            let streams = channels
                .into_iter()
                .filter(|ch| is_f1_channel(&ch.name, &ch.group))
                .map(|ch| {
                    let is_hls =
                        ch.stream_url.contains(".m3u8") || !ch.stream_url.ends_with(".mp4");
                    StreamUrl {
                        url: ch.stream_url,
                        quality: Quality::Auto,
                        is_hls,
                        headers: vec![("User-Agent".to_string(), "Sloth-TUI/0.1.0".to_string())],
                        subtitle_url: None,
                        provider_id: "f1",
                    }
                })
                .collect();
            Ok(streams)
        }
        Err(err_msg) => {
            tracing::warn!(
                "Failed to fetch iptv-org sports playlist: {err_msg}, falling back to direct HTTP"
            );
            let client = crate::net::http_client_builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default();

            let resp = client.get(SPORTS_M3U_URL).send().await.map_err(|e| {
                SlothError::Provider(ProviderError::Network(format!(
                    "Failed to download F1 streams: {e}"
                )))
            })?;

            let text = resp.text().await.map_err(|e| {
                SlothError::Provider(ProviderError::Parsing(format!(
                    "Failed to read F1 streams playlist: {e}"
                )))
            })?;

            Ok(parse_f1_m3u(&text))
        }
    }
}

/// Provider implementation for Formula 1 streams.
#[derive(Debug, Default)]
pub struct F1StreamsProvider;

impl F1StreamsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for F1StreamsProvider {
    fn id(&self) -> &'static str {
        "f1"
    }

    fn name(&self) -> &'static str {
        "Formula 1 Live Streams"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: false,
            movies: false,
            tv_series: false,
            anime: false,
            live_sports: true,
            iptv: true,
            f1: true,
            subtitles: false,
            download: false,
            quality_selection: false,
            dub_sub_toggle: false,
            supports_search: false,
            supports_pagination: false,
            supports_series: false,
            supports_subtitles: false,
            supports_homepage: false,
        }
    }

    async fn search(&self, _query: &str, _kind: MediaType) -> Result<Vec<Media>, ProviderError> {
        Ok(Vec::new())
    }

    async fn resolve(
        &self,
        _media: &Media,
        _episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, ProviderError> {
        fetch_f1_streams()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))
    }

    async fn health(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_f1_channel() {
        assert!(is_f1_channel("Sky Sports F1 HD", "Sports"));
        assert!(is_f1_channel("Formula 1 TV", "Sports"));
        assert!(is_f1_channel("Canal+ F1", "Sports"));
        assert!(is_f1_channel("F1 Live", "Motorsport"));
        assert!(is_f1_channel("Live Racing", "Formula 1"));

        // Should not match random words containing f1 like california
        assert!(!is_f1_channel("California Music", "Entertainment"));
    }

    #[test]
    fn test_parse_f1_m3u() {
        let sample = r#"#EXTM3U
#EXTINF:-1 tvg-id="SkyF1" tvg-name="Sky Sports F1" group-title="Sports",Sky Sports F1 HD
https://stream.example.com/skysportsf1.m3u8
#EXTINF:-1 tvg-id="ESPN" group-title="Sports",ESPN HD
https://stream.example.com/espn.m3u8
#EXTINF:-1 tvg-name="Formula 1 Worldwide" group-title="Motorsport",Formula 1 TV
https://stream.example.com/f1tv.m3u8
"#;
        let streams = parse_f1_m3u(sample);
        assert_eq!(streams.len(), 2);
        assert_eq!(
            streams[0].url,
            "https://stream.example.com/skysportsf1.m3u8"
        );
        assert_eq!(streams[1].url, "https://stream.example.com/f1tv.m3u8");
    }
}
