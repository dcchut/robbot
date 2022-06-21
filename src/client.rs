pub fn make_client() -> reqwest::Client {
    static CLIENT: once_cell::sync::OnceCell<reqwest::Client> = once_cell::sync::OnceCell::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::ClientBuilder::new()
            .user_agent("robbot")
            .build()
            .expect("failed to build client")
    });

    // client is basically an Arc<_>
    client.clone()
}
