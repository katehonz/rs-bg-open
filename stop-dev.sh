#!/bin/bash

echo "Stopping all development servers..."

# Kill backend processes (cargo run on port 8080)
echo "Stopping backend..."
pkill -f "target/debug/backend" 2>/dev/null
lsof -ti:8080 | xargs kill -9 2>/dev/null

# Kill React frontend processes (npm run dev on port 5173)
echo "Stopping React frontend..."
pkill -f "npm run dev" 2>/dev/null
pkill -f "vite" 2>/dev/null
lsof -ti:5173 | xargs kill -9 2>/dev/null

# Kill old Leptos frontend processes if any
pkill -f "trunk serve" 2>/dev/null
lsof -ti:3000 | xargs kill -9 2>/dev/null

# Kill any cargo watch processes
pkill -f "cargo watch" 2>/dev/null

# Kill any remaining cargo run processes
pkill -f "cargo run" 2>/dev/null

# Kill any Node.js processes that might be hanging
pkill -f "node.*vite" 2>/dev/null

echo "All servers stopped."