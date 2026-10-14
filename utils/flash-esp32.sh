#!/usr/bin/env bash
# Compatibility entry point; all configuration, compilation and upload logic is Node.
set -euo pipefail
exec node "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/flash-esp32" "$@"
