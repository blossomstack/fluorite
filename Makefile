.PHONY: build test clean fmt lint check all release-build interop-test \
        publish-crates publish-npm sync-versions bump-version release

all: fmt-check lint test

build:
	cargo build

release-build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check

interop-test:
	./tests/interop/run-interop-test.sh

# Release targets

# Sync npm package versions with Cargo workspace version
sync-versions:
	@VERSION=$$(grep -m1 'version = ' Cargo.toml | sed 's/.*"\(.*\)"/\1/'); \
	echo "Syncing npm packages to version $$VERSION"; \
	for pkg in fluorite-cli fluorite-darwin-x64 fluorite-darwin-arm64 fluorite-linux-x64 fluorite-linux-arm64 fluorite-win32-x64; do \
		cd npm/$$pkg && npm version $$VERSION --no-git-tag-version --allow-same-version && cd ../..; \
	done; \
	cd npm/fluorite-cli && node -e " \
		const fs = require('fs'); \
		const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8')); \
		for (const dep of Object.keys(pkg.optionalDependencies || {})) { \
			pkg.optionalDependencies[dep] = '$$VERSION'; \
		} \
		fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n'); \
	"

# Publish Rust crates to crates.io (run in order: runtime first, then codegen)
publish-crates:
	@echo "Publishing fluorite (runtime) to crates.io..."
	cargo publish --package fluorite
	@echo "Waiting for crates.io to index..."
	sleep 30
	@echo "Publishing fluorite_codegen to crates.io..."
	cargo publish --package fluorite_codegen

# Publish npm package (requires binaries to be available in GitHub releases)
publish-npm:
	@echo "Publishing @zhxiaogg/fluorite-cli to npm..."
	cd npm/fluorite-cli && npm publish --access public

# Bump minor version in Cargo.toml and sync to npm packages
bump-version:
	@CURRENT=$$(grep -m1 'version = ' Cargo.toml | sed 's/.*"\(.*\)"/\1/'); \
	MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	PATCH=$$(echo $$CURRENT | cut -d. -f3); \
	NEW_MINOR=$$((MINOR + 1)); \
	NEW_VERSION="$$MAJOR.$$NEW_MINOR.0"; \
	echo "Bumping version from $$CURRENT to $$NEW_VERSION"; \
	sed -i.bak "s/^version = \"$$CURRENT\"/version = \"$$NEW_VERSION\"/" Cargo.toml && rm Cargo.toml.bak; \
	echo "Updating codegen dependency version to $$NEW_VERSION"; \
	sed -i.bak "/^fluorite = /s/version = \"[^\"]*\"/version = \"$$NEW_VERSION\"/" codegen/Cargo.toml && rm codegen/Cargo.toml.bak; \
	$(MAKE) sync-versions

# Release: bump version, commit, tag, and push
release:
	@echo "Running pre-release checks..."
	$(MAKE) all
	@echo ""
	@echo "Bumping version..."
	$(MAKE) bump-version
	@VERSION=$$(grep -m1 'version = ' Cargo.toml | sed 's/.*"\(.*\)"/\1/'); \
	echo ""; \
	echo "Creating release commit and tag for v$$VERSION..."; \
	git add -A; \
	git commit -m "chore: release v$$VERSION"; \
	git tag "v$$VERSION"; \
	echo ""; \
	echo "Pushing to remote..."; \
	git push && git push --tags; \
	echo ""; \
	echo "Release v$$VERSION pushed! GitHub Actions will handle publishing."
