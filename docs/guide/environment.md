# Environment variables

Every setting has a configuration-file form; environment variables override
files, and CLI flags override environment variables. Secrets are only ever
read from the environment — never stored in configuration files.

## Agent settings

| Variable | Overrides | Default | Notes |
| --- | --- | --- | --- |
| `VOX_PROVIDER` | `general.provider` | `openai` | Provider registry id. |
| `VOX_MODEL` | `general.model` | `gpt-4o-mini` | Model identifier. |
| `VOX_BASE_URL` | `general.base_url` | provider default | HTTPS, or HTTP on loopback only. |
| `VOX_API_KEY_ENV` | `general.api_key_env` | `OPENAI_API_KEY` | Name of the variable holding the key. |
| `VOX_LOG` | `general.log` | `info` | `error`, `warn`, `info`, `debug`, `trace`, or directives like `vox=debug`. |

## Paths

| Variable | Overrides | Default |
| --- | --- | --- |
| `VOX_DATA_DIR` | session and log storage | platform data directory |
| `VOX_CONFIG_DIR` | user configuration lookup | platform config directory |

## Credentials

| Variable | Purpose |
| --- | --- |
| `OPENAI_API_KEY` | Default API key for OpenAI-compatible providers. |

The variable named by `general.api_key_env` is read at run time; a missing
value stops the command with a generic error and a diagnostic in the log.
Change the variable name with `--api-key-env` or `VOX_API_KEY_ENV` when using
several providers.

## Example session

```sh
export OPENAI_API_KEY="..."        # from your secret manager, not this file
export VOX_LOG="vox=debug"
vox -p "summarise the failing test"
```
