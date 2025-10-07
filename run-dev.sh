#!/bin/bash

# First stop any existing processes
echo "Stopping any existing processes..."
./stop-dev.sh

# Wait a moment for ports to be released
sleep 2

# Setup Node.js environment
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
nvm use 20 >/dev/null 2>&1

# Get the current directory
ROOT_DIR=$(pwd)

# Start backend in background
echo "Starting backend on port 8080..."
cd "$ROOT_DIR/backend" && cargo run &
BACKEND_PID=$!

# Wait for backend to start
sleep 3

# Start React frontend
echo "Starting React frontend on port 5173..."
cd "$ROOT_DIR/frontend" && npm run dev &
FRONTEND_PID=$!

# Return to root directory
cd "$ROOT_DIR"

echo ""
echo "✅ Development servers started:"
echo "   Backend PID: $BACKEND_PID (http://localhost:8080)"
echo "   Frontend PID: $FRONTEND_PID (http://localhost:5173)"
echo "   GraphQL Playground: http://localhost:8080/graphiql"
echo ""
echo "Press Ctrl+C to stop all servers"

# Wait for user to press Ctrl+C
trap "echo 'Stopping servers...'; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit" INT
wait