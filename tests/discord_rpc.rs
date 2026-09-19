use sloth_tui::providers::models::{EpisodeRef, Media, MediaType};
use sloth_tui::tracking::DiscordRpc;
use sloth_tui::tui::screens::details;
use sloth_tui::tui::state::{AppState, SettingsCategory};
use sloth_tui::tui::theme::Theme;

#[test]
fn test_discord_rpc_disabled_lifecycle() {
    let mut rpc = DiscordRpc::new(false);
    assert!(!rpc.is_enabled());

    let media = Media {
        id: "tt1234567".to_string(),
        title: "Interstellar".to_string(),
        media_type: MediaType::Movie,
        year: Some(2014),
        overview: Some("A team of explorers travel through a wormhole in space.".to_string()),
        poster_url: None,
        backdrop_url: None,
        genres: vec!["Sci-Fi".to_string(), "Adventure".to_string()],
        rating: Some(8.7),
        duration_secs: Some(10140.0),
        seasons_count: None,
        episodes_count: None,
        provider_id: "moviebox",
        external_ids: Default::default(),
        cast: Vec::new(),
    };

    let episode = EpisodeRef {
        season: 1,
        episode: 1,
        title: Some("Pilot".to_string()),
        duration_secs: Some(3600.0),
    };

    // Disabled RPC should not panic or fail on any call
    rpc.set_watching(&media, Some(&episode));
    rpc.set_watching(&media, None);
    rpc.set_browsing();
    rpc.clear();
}

#[test]
fn test_discord_rpc_toggle() {
    let mut rpc = DiscordRpc::new(false);
    assert!(!rpc.is_enabled());

    rpc.set_enabled(true);
    #[cfg(feature = "discord")]
    assert!(rpc.is_enabled());
    #[cfg(not(feature = "discord"))]
    assert!(!rpc.is_enabled());

    rpc.set_enabled(false);
    assert!(!rpc.is_enabled());
}

#[test]
fn test_settings_general_has_four_rows_with_discord_toggle() {
    let mut state = AppState::default();
    state.settings_category = SettingsCategory::General;

    assert_eq!(SettingsCategory::General.row_count(), 4);
    assert!(state.discord_rpc_enabled);

    // Toggle Discord RPC
    state.discord_rpc_enabled = !state.discord_rpc_enabled;
    assert!(!state.discord_rpc_enabled);
}

#[test]
fn test_details_screen_renders_without_crash() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::mocha();
    let state = AppState::default();

    terminal
        .draw(|f| {
            let area = f.area();
            details::draw(f, area, &state, &theme);
        })
        .unwrap();
}
