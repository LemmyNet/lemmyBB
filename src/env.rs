/// Address where lemmyBB should listen for incoming requests.
pub fn listen_address() -> String {
    std::env::var("LEMMY_BB_LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:1244".to_string())
}

/// Address where Lemmy API is available.
pub fn lemmy_backend() -> String {
    std::env::var("LEMMY_BB_BACKEND").unwrap_or_else(|_| "http://localhost:8536".to_string())
}

/// The domain under which lemmyBB is running.
pub fn external_domain() -> String {
    std::env::var("LEMMY_BB_DOMAIN").unwrap_or_else(|_| "http://127.0.0.1:1244/".to_string())
}
