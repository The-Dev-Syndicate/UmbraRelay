.PHONY: start-dev release help

help:
	@echo "Available targets:"
	@echo "  make start-dev          - Start development server"
	@echo "  make release VERSION=X  - Create and push release tag (e.g., make release VERSION=0.0.1)"

start-dev:
	@echo "Starting development server..."
	npm run tauri dev

release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make release VERSION=0.0.1"; \
		exit 1; \
	fi
	@echo ""
	@echo "=========================================="
	@echo "  RELEASE WARNING"
	@echo "=========================================="
	@echo "This will:"
	@echo "  1. Create git tag: v$(VERSION)"
	@echo "  2. Push tag to origin"
	@echo "  3. Trigger GitHub Actions workflow"
	@echo "  4. Create a draft release on GitHub for review"
	@echo ""
	@echo "Are you sure you want to proceed? (y/yes to continue, anything else to cancel)"
	@read -r confirm; \
	confirm_lower=$$(echo "$$confirm" | tr '[:upper:]' '[:lower:]'); \
	if [ "$$confirm_lower" = "y" ] || [ "$$confirm_lower" = "yes" ]; then \
		echo ""; \
		echo "Creating tag v$(VERSION)..."; \
		git tag v$(VERSION) || exit 1; \
		echo "Pushing tag to origin..."; \
		git push origin v$(VERSION) || exit 1; \
		echo ""; \
		echo "✓ Tag v$(VERSION) created and pushed successfully!"; \
		echo "✓ GitHub Actions workflow has been triggered."; \
		echo "✓ Check the Actions tab for build progress."; \
		echo "✓ Once builds complete, review the draft release in the Releases section."; \
	else \
		echo ""; \
		echo "Release cancelled."; \
		exit 0; \
	fi

