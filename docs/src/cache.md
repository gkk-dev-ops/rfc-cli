# Caching and offline mode

`rfc-agent-cli` keeps fetched content in the platform's standard user cache directory. Override it when reproducible or sandboxed storage is useful:

```console
rfc --cache-dir ./rfc-cache show 9110
```

## Cache policy

- published RFC text is immutable and does not expire;
- RFC metadata is considered fresh for 24 hours;
- archived Internet-Drafts are cached;
- cache writes use an atomic replacement;
- stale cached data may be used when the network is unavailable and is identified in structured output.

## Strict offline operation

```console
rfc --offline show 9110
```

Offline mode never makes a network request. A missing cache entry produces exit code `6`.

## Refreshing and clearing

```console
rfc --refresh info 9110
rfc cache status
rfc cache clear
```

`--refresh` bypasses a fresh cached response and requests the authoritative source again.
