#!/usr/bin/env node
// install.js - Post-install script for @blossomstack/fluorite-cli
// Binary is provided via optionalDependencies (platform-specific packages)

const fs = require('fs');
const path = require('path');

const PLATFORM_PACKAGES = {
  'darwin-x64': '@blossomstack/fluorite-darwin-x64',
  'darwin-arm64': '@blossomstack/fluorite-darwin-arm64',
  'linux-x64': '@blossomstack/fluorite-linux-x64',
  'linux-arm64': '@blossomstack/fluorite-linux-arm64',
  'win32-x64': '@blossomstack/fluorite-win32-x64',
};

function checkBinaryInstalled() {
  const platformKey = `${process.platform}-${process.arch}`;
  const packageName = PLATFORM_PACKAGES[platformKey];

  if (!packageName) {
    console.warn(`Warning: No pre-built binary available for ${platformKey}`);
    console.warn('You can build from source: cargo install fluorite_codegen');
    return;
  }

  try {
    const ext = process.platform === 'win32' ? '.exe' : '';
    const packagePath = require.resolve(`${packageName}/package.json`);
    const packageDir = path.dirname(packagePath);
    const binaryPath = path.join(packageDir, 'bin', `fluorite${ext}`);

    if (fs.existsSync(binaryPath)) {
      // Ensure binary has execute permission (npm may strip it during publish/install)
      if (process.platform !== 'win32') {
        try {
          fs.chmodSync(binaryPath, 0o755);
        } catch (e) {
          console.warn(`Warning: Could not set execute permission on ${binaryPath}: ${e.message}`);
        }
      }
      console.log(`fluorite binary installed successfully for ${platformKey}`);
    } else {
      console.warn(`Warning: Binary not found in ${packageName}`);
      console.warn('Try reinstalling: npm uninstall @blossomstack/fluorite-cli && npm install @blossomstack/fluorite-cli');
    }
  } catch (e) {
    // Platform package not installed - this can happen if npm skipped the optional dependency
    console.warn(`Warning: Platform package ${packageName} not installed`);
    console.warn('This may happen if npm skipped the optional dependency.');
    console.warn('');
    console.warn('Alternative installation methods:');
    console.warn('  1. cargo install fluorite_codegen');
    console.warn('  2. Download binary from https://github.com/blossomstack/fluorite/releases');
  }
}

checkBinaryInstalled();
