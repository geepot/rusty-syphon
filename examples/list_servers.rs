//! Lists currently available Syphon servers from the shared directory.
//! Run on macOS: cargo run --example list_servers

fn main() {
    #[cfg(target_os = "macos")]
    {
        let dir = match rusty_syphon::ServerDirectory::shared() {
            Some(d) => d,
            None => {
                eprintln!("Failed to get Syphon server directory");
                return;
            }
        };
        let count = dir.servers_count();
        println!("Syphon servers available: {}", count);
        for i in 0..count {
            if let Some(desc) = dir.server_at_index(i) {
                let name = desc.name().unwrap_or_else(|| "(no name)".into());
                let app = desc.app_name().unwrap_or_else(|| "(unknown app)".into());
                let uuid = desc.uuid().unwrap_or_else(|| "(no uuid)".into());
                println!("  [{}] {} (app: {}) uuid={}", i, name, app, uuid);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("Syphon is macOS-only. This example does nothing on other platforms.");
    }
}
