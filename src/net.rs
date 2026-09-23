//! Network client factory, DNS utilities, and HTTP helpers.

pub const DEFAULT_BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
pub const APP_HTTP_USER_AGENT: &str =
    "Sloth-TUI/0.1.0 (https://github.com/syedzahidsaleem/sloth-tui)";

/// Constructs a preconfigured `reqwest::ClientBuilder` with standard connection pooling and user agent.
pub fn http_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .tcp_nodelay(true)
        .tcp_keepalive(Some(std::time::Duration::from_secs(45)))
        .pool_idle_timeout(Some(std::time::Duration::from_secs(90)))
        .pool_max_idle_per_host(8)
        .user_agent(APP_HTTP_USER_AGENT)
}

/// Probes a given URL with HEAD and GET requests to check reachability.
pub async fn probe_url(url: &str, timeout: std::time::Duration) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(timeout)
        .build()
    else {
        return false;
    };
    if let Ok(resp) = client.head(url).send().await {
        if resp.status().is_success() || resp.status().is_redirection() {
            return true;
        }
    }
    if let Ok(resp) = client.get(url).send().await {
        return resp.status().is_success() || resp.status().is_redirection();
    }
    false
}

/// Validates whether a source string begins with http:// or https://.
pub fn is_http_url(source: &str) -> bool {
    let trimmed = source.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

/// Validates whether a source string is a playable media stream URL
/// (HTTP, HTTPS, RTMP, RTMPS, RTSP, RTSPS, MMS, MMSH, UDP, SRT).
pub fn is_playable_stream_url(source: &str) -> bool {
    let trimmed = source.trim();
    trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("rtmp://")
        || trimmed.starts_with("rtmps://")
        || trimmed.starts_with("rtsp://")
        || trimmed.starts_with("rtsps://")
        || trimmed.starts_with("mms://")
        || trimmed.starts_with("mmsh://")
        || trimmed.starts_with("udp://")
        || trimmed.starts_with("srt://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_http_url() {
        assert!(is_http_url("http://example.com"));
        assert!(is_http_url("https://example.com/playlist.m3u8"));
        assert!(is_http_url("   https://example.com   "));
        assert!(!is_http_url("/local/path/file.m3u"));
        assert!(!is_http_url("stremio://addon.example.com"));
        assert!(!is_http_url(""));
    }

    #[test]
    fn test_is_playable_stream_url() {
        assert!(is_playable_stream_url("http://example.com/live.m3u8"));
        assert!(is_playable_stream_url("https://example.com/video.mp4"));
        assert!(is_playable_stream_url("rtmp://example.com/live/stream"));
        assert!(is_playable_stream_url("rtsp://example.com:554/live"));
        assert!(is_playable_stream_url("mms://example.com/tv"));
        assert!(is_playable_stream_url("udp://@239.1.1.1:1234"));
        assert!(is_playable_stream_url("srt://example.com:9000"));
        assert!(!is_playable_stream_url("/local/path/file.m3u"));
        assert!(!is_playable_stream_url("invalid-url"));
        assert!(!is_playable_stream_url(""));
    }
}
