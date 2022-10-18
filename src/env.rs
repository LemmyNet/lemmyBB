/// Address where lemmyBB should listen for incoming requests.
pub fn listen_address() -> String {
    std::env::var("LEMMY_BB_LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:1244".to_string())
}

/// Address where Lemmy API is available.
pub fn lemmy_backend() -> String {
    std::env::var("LEMMY_BB_BACKEND").unwrap_or_else(|_| "http://localhost:8536".to_string())
}

/// Set true if Lemmy backend runs with increased message rate limit. This is necessary to show
/// last replies for threads and forums.
pub fn increased_rate_limit() -> bool {
    !std::env::var("LEMMY_BB_INCREASED_RATE_LIMIT")
        .unwrap_or_default()
        .is_empty()
}
