#!/usr/bin/env bash
set -euo pipefail

echo "==> Building frontend..."
(cd frontend && npm ci && npm run build)

echo "==> Building backend (release)..."
(cd backend && cargo build --release)

echo ""
echo "==> Build complete!"
echo "    Binary: backend/target/release/friendship-service"
echo "    Static: dist/"
echo ""
echo "    Run with:"
echo "    FS_ENV=production ./backend/target/release/friendship-service"
