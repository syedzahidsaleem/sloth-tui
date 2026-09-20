use std::path::Path;
use std::time::Duration;

use sloth_tui::config::{PlayerBackend, PlayerConfig};
use sloth_tui::player::vlc::{VlcStatus, build_vlc_args, poll_position};
use sloth_tui::providers::models::{Quality, StreamUrl};

static ENV_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

fn create_dummy_executable(dir: &Path, name: &str) {
    let exe_file = dir.join(format!("{name}.exe"));
    let bin_file = dir.join(name);

    let _ = std::fs::write(&exe_file, b"");
    let _ = std::fs::write(&bin_file, b"#!/bin/sh\nexit 0\n");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&bin_file, std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::set_permissions(&exe_file, std::fs::Permissions::from_mode(0o755));
    }
}

// 1. test_vlc_status_parse_playing
// Parse a mock status.json response when VLC is playing.
// Verify time = 3847.0, length = 7200.0, state = "playing"
#[test]
fn test_vlc_status_parse_playing() {
    let json = r#"{
        "time": 3847,
        "length": 7200,
        "state": "playing"
    }"#;
    let status = VlcStatus::parse(json).expect("should parse valid playing status");
    assert_eq!(status.time, 3847.0);
    assert_eq!(status.length, 7200.0);
    assert_eq!(status.state.as_deref(), Some("playing"));
    assert!(status.is_playing());
    assert!(!status.is_stopped());
    assert_eq!(status.position(), Some(3847.0));
    assert_eq!(status.duration(), Some(7200.0));
}

// 2. test_vlc_status_parse_stopped
// Parse a mock status.json response when VLC is stopped.
// Verify time = 0.0, state = "stopped"
#[test]
fn test_vlc_status_parse_stopped() {
    let json = r#"{
        "time": 0,
        "length": 0,
        "state": "stopped"
    }"#;
    let status = VlcStatus::parse(json).expect("should parse valid stopped status");
    assert_eq!(status.time, 0.0);
    assert_eq!(status.state.as_deref(), Some("stopped"));
    assert!(status.is_stopped());
    assert!(!status.is_playing());
    assert_eq!(status.position(), None);
    assert_eq!(status.duration(), None);
}

// 3. test_vlc_position_none_on_timeout
// Point poll_position at a port that doesn't respond (e.g. 19999).
// Verify it returns (None, None) within a reasonable timeout (< 5s), not hanging.
#[tokio::test]
async fn test_vlc_position_none_on_timeout() {
    let start = std::time::Instant::now();
    let (pos, dur) = poll_position(19999, "dummy_password").await;
    let elapsed = start.elapsed();

    assert_eq!(pos, None);
    assert_eq!(dur, None);
    assert!(
        elapsed < Duration::from_secs(5),
        "polling must not hang beyond 5s, took: {:?}",
        elapsed
    );
}

// 4. test_vlc_args_referer_header
// Build args for a stream with headers {"referer": "https://example.com"}.
// Verify args contain "--http-referrer=https://example.com"
#[test]
fn test_vlc_args_referer_header() {
    let stream = StreamUrl {
        url: "https://example.com/video.mp4".to_string(),
        headers: vec![("referer".to_string(), "https://example.com".to_string())],
        subtitle_url: None,
        is_hls: false,
        quality: Quality::Unknown,
        provider_id: "test",
    };
    let config = PlayerConfig::default();
    let args = build_vlc_args(&stream, None, 8080, "test_pw", &config);
    assert!(args.contains(&"--http-referrer=https://example.com".to_string()));
}

// 5. test_vlc_args_resume_position
// Build args with resume_pos = 120.0 (> 10s).
// Verify args contain "--start-time=120"
#[test]
fn test_vlc_args_resume_position() {
    let stream = StreamUrl {
        url: "https://example.com/video.mp4".to_string(),
        headers: vec![],
        subtitle_url: None,
        is_hls: false,
        quality: Quality::Unknown,
        provider_id: "test",
    };
    let config = PlayerConfig::default();
    let args = build_vlc_args(&stream, Some(120.0), 8080, "test_pw", &config);
    assert!(args.contains(&"--start-time=120".to_string()));
}

// 6. test_vlc_args_no_start_time_under_threshold
// Build args with resume_pos = 5.0 (<= 10s).
// Verify args do NOT contain "--start-time"
#[test]
fn test_vlc_args_no_start_time_under_threshold() {
    let stream = StreamUrl {
        url: "https://example.com/video.mp4".to_string(),
        headers: vec![],
        subtitle_url: None,
        is_hls: false,
        quality: Quality::Unknown,
        provider_id: "test",
    };
    let config = PlayerConfig::default();
    let args = build_vlc_args(&stream, Some(5.0), 8080, "test_pw", &config);
    assert!(!args.iter().any(|arg| arg.starts_with("--start-time")));
}

// 7. test_auto_detect_prefers_mpv
// Mock PATH or which to return both mpv and vlc.
// Verify detect_player() returns Some(PlayerBackend::Mpv)
#[test]
fn test_auto_detect_prefers_mpv() {
    let _lock = ENV_LOCK.lock();
    let orig_path = std::env::var_os("PATH");
    let temp_dir = tempfile::tempdir().expect("create temp dir");

    create_dummy_executable(temp_dir.path(), "mpv");
    create_dummy_executable(temp_dir.path(), "vlc");

    unsafe {
        std::env::set_var("PATH", temp_dir.path());
    }
    let detected = sloth_tui::config::detect_player();

    unsafe {
        if let Some(orig) = orig_path {
            std::env::set_var("PATH", orig);
        } else {
            std::env::remove_var("PATH");
        }
    }

    assert_eq!(detected, Some(PlayerBackend::Mpv));
}

// 8. test_auto_detect_falls_back_to_vlc
// Mock PATH to have only vlc.
// Verify detect_player() returns Some(PlayerBackend::Vlc)
#[test]
fn test_auto_detect_falls_back_to_vlc() {
    let _lock = ENV_LOCK.lock();
    let orig_path = std::env::var_os("PATH");
    let temp_dir = tempfile::tempdir().expect("create temp dir");

    create_dummy_executable(temp_dir.path(), "vlc");

    unsafe {
        std::env::set_var("PATH", temp_dir.path());
    }
    let detected = sloth_tui::config::detect_player();

    unsafe {
        if let Some(orig) = orig_path {
            std::env::set_var("PATH", orig);
        } else {
            std::env::remove_var("PATH");
        }
    }

    assert_eq!(detected, Some(PlayerBackend::Vlc));
}
