#!/usr/bin/env bash
set -euo pipefail

# Start both frontend and backend servers for development.
# Ctrl-C kills both processes.

trap 'kill 0' EXIT

echo "Starting Friendship&Service development servers..."

(cd backend && cargo run) &
BACKEND_PID=$!

(cd frontend && npm run dev) &
FRONTEND_PID=$!

echo "Backend (Actix Web): http://127.0.0.1:8080"
echo "Frontend (Vite):     http://localhost:5173"
echo ""
echo "Press Ctrl-C to stop both servers."

wait
