# Command-line usage

## Read a document

RFC identifiers may be passed as a number or with an `RFC` prefix:

```console
rfc show 9110
rfc show RFC9110
```

Internet-Drafts use their complete archive identifier:

```console
rfc show draft-ietf-httpbis-semantics-19
```

## Read RFC metadata

```console
rfc info 9110
```

Metadata includes the title, authors, publication status, dates, DOI, errata URL, and RFC relationships when supplied by the RFC Editor.

## Structured output

Use `--json` as a shortcut for a single JSON response:

```console
rfc --json info 9110
rfc --output json show 9110
```

Use JSON Lines when a line-oriented response is easier to consume:

```console
rfc --output jsonl info 9110
```

Structured data is written to stdout. Diagnostics and structured error envelopes are written to stderr.

## Cache commands

```console
rfc cache status
rfc cache clear
rfc --offline show 9110
rfc --refresh info 9110
```

## Global options

Run `rfc --help` or `rfc <command> --help` for the complete option reference. Important global controls include:

- `--json`
- `--output human|json|jsonl`
- `--offline`
- `--refresh`
- `--cache-dir <path>`
- `--timeout-seconds <seconds>`

`rfc show <identifier>` remains compatible with the original Python CLI. The legacy `--pager` flag is accepted but does not open an implicit pager, keeping stdout under the caller's control.

## Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | Success |
| `1` | Internal or MCP error |
| `2` | Invalid input |
| `4` | RFC or draft not found |
| `5` | Network error |
| `6` | Offline cache miss |
| `7` | Cache error |
