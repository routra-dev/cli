# Routra CLI

Command-line interface for managing Routra API keys, routing policies, usage, batch jobs, and cost reporting.

## Installation

```bash
cargo install routra
```

## Authentication

```bash
# Login - securely stores your API key in ~/.routra/config.json
routra login

# Logout - removes stored credentials
routra logout
```

Get your API key from [routra.dev/dashboard/keys](https://www.routra.dev/dashboard/keys).

You can also pass your API key directly:

```bash
# Via environment variable
export ROUTRA_API_KEY="rtr-..."

# Via flag (overrides env and config)
routra --api-key "rtr-..." keys list
```

## Commands

### API Keys

```bash
# List all keys
routra keys list

# Create a new key (optionally attach a routing policy)
routra keys create --name production
routra keys create --name staging --policy cheapest

# Rotate a key (old key stays active 24h)
routra keys rotate <key-id>

# Revoke a key (with confirmation prompt)
routra keys revoke <key-id>
```

### Routing Policies

```bash
# Push policies from a YAML file
routra policy push routra.yaml

# List all policies in your workspace
routra policy list

# Show full details of a policy
routra policy get <policy-name>

# Delete a policy (with confirmation prompt)
routra policy delete <policy-name>
```

Example `routra.yaml`:

```yaml
policies:
  cheapest:
    strategy: cheapest
  production:
    strategy: balanced
    constraints:
      allowed_providers: [coreweave, lambda]
  gdpr-eu:
    strategy: cheapest
    constraints:
      allowed_regions: [eu-west, eu-central]
```

### Usage

```bash
# View usage summary for the current billing period
routra usage
```

### Cost Breakdown

```bash
# View cost breakdown by model and provider
routra cost
```

### Batch Jobs

```bash
# Submit a JSONL file as a batch job
routra batch create requests.jsonl
routra batch create requests.jsonl --policy cheapest --window 1h

# Check job status
routra batch status <job-id>

# Get results URL for a completed batch
routra batch results <job-id>

# Cancel a queued or processing job
routra batch cancel <job-id>

# List all batch jobs
routra batch list
```

## Global Options

| Flag | Env Variable | Description |
|------|-------------|-------------|
| `--api-key` | `ROUTRA_API_KEY` | API key (overrides config file) |
| `--base-url` | `ROUTRA_BASE_URL` | API base URL (default: `https://api.routra.dev/v1`) |

## Configuration

Credentials are stored in `~/.routra/config.json` with owner-only file permissions (0600 on Unix).

```json
{
  "api_key": "rtr-...",
  "base_url": "https://api.routra.dev/v1"
}
```

## Security

- API key input is masked during `routra login` (not visible on screen)
- Config file permissions are restricted to owner-only on Unix
- Destructive operations (`keys revoke`, `policy delete`) require confirmation
- All communication uses HTTPS with rustls (pure Rust TLS)

## License

MIT
