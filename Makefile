.PHONY: dev backend frontend build clean

dev:
	@echo "Starting development servers..."
	@make -j2 backend frontend

backend:
	@echo "Starting backend server..."
	cd backend && cargo watch -x run

frontend:
	@echo "Starting frontend server..."
	cd frontend && trunk serve

build:
	@echo "Building release versions..."
	cd backend && cargo build --release
	cd frontend && trunk build --release

clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf frontend/dist

db-create:
	createdb rs-ac-bg

db-drop:
	dropdb rs-ac-bg

db-reset: db-drop db-create
	@echo "Database reset complete"

test:
	cargo test --workspace