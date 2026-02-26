# Deeria

Deeria is a small request dispatcher written in Rust.

It provides two execution paths:
- Remote fetch (GET → upstream → buffered response)
- Local file read (filesystem → buffered response)

Optional processing:
- Header injection / override
- Rewrite

# Configuration

Deeria is configured using a TOML file.

Example:

```toml
[server]
host = "127.0.0.1"
port = 4242

[proxies.stream]
type = "remote"
target = "https://example.com"

# Optional: rewrite (applied in order)
rewrite = { "http://old" = "http://new" }

# Optional: headers sent to upstream
upstream_headers = { Origin = "https://example.com" }

# Optional: headers returned to client
downstream_headers = { "Cache-Control" = "no-cache" }

[proxies.local_files]
type = "local"
target = "/home/user/assets/"

# Optional: rewrite (applied in order)
rewrite = { "http://old" = "http://new" }

# Optional: headers returned to client
downstream_headers = { "Cache-Control" = "no-cache" }
```
