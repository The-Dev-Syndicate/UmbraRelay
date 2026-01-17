.PHONY: start-dev release ts-check help

help:
	@echo "Available targets:"
	@echo "  make start-dev          - Start development server"
	@echo "  make ts-check           - Run TypeScript type checking"
	@echo "  make release VERSION=X - Create and push release tag (e.g., make release VERSION=0.0.1)"

start-dev:
	@echo "Starting development server..."
	npm run tauri dev || (echo ""; echo "❌ Development server failed! Make sure to run nvm use 20"; exit 1;)

ts-check:
	@echo "Running TypeScript type check..."
	@npm run ts-check || (echo ""; echo "❌ TypeScript check failed! Please fix errors before proceeding."; exit 1)
	@echo "✓ TypeScript check passed!"

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
	@echo "  1. Run TypeScript type checking"
	@echo "  2. Create git tag: v$(VERSION)"
	@echo "  3. Push tag to origin"
	@echo "  4. Trigger GitHub Actions workflow"
	@echo "  5. Create a draft release on GitHub for review"
	@echo ""
	@echo "Are you sure you want to proceed? (y/yes to continue, anything else to cancel)"
	@read -r confirm; \
	confirm_lower=$$(echo "$$confirm" | tr '[:upper:]' '[:lower:]'); \
	if [ "$$confirm_lower" = "y" ] || [ "$$confirm_lower" = "yes" ]; then \
		echo ""; \
		echo "Running pre-release checks..."; \
		if ! $(MAKE) ts-check; then \
			echo ""; \
			echo "❌ Pre-release checks failed! Aborting release."; \
			exit 1; \
		fi; \
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

