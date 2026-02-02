#!/usr/bin/env node
// bin/fluorite.js - Wrapper that invokes the Rust binary

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Platform-specific package mapping
const PLATFORM_PACKAGES = {
  'darwin-x64': '@zhxiaogg/fluorite-darwin-x64',
  'darwin-arm64': '@zhxiaogg/fluorite-darwin-arm64',
  'linux-x64': '@zhxiaogg/fluorite-linux-x64',
  'linux-arm64': '@zhxiaogg/fluorite-linux-arm64',
  'win32-x64': '@zhxiaogg/fluorite-win32-x64',
};

function getBinaryPath() {
  const ext = process.platform === 'win32' ? '.exe' : '';
  const platformKey = `${process.platform}-${process.arch}`;

  // Try platform-specific package first
  const packageName = PLATFORM_PACKAGES[platformKey];
  if (packageName) {
    try {
      const packagePath = require.resolve(`${packageName}/package.json`);
      const packageDir = path.dirname(packagePath);
      const binaryPath = path.join(packageDir, 'bin', `fluorite${ext}`);
      if (fs.existsSync(binaryPath)) {
        return binaryPath;
      }
    } catch (e) {
      // Package not installed, fall through to local binary
    }
  }

  // Fallback to local binary (for development or manual installation)
  const localBinaryPath = path.join(__dirname, `fluorite${ext}`);
  if (fs.existsSync(localBinaryPath)) {
    return localBinaryPath;
  }

  return null;
}

const binaryPath = getBinaryPath();

if (!binaryPath) {
  const platformKey = `${process.platform}-${process.arch}`;
  console.error('Error: fluorite binary not found.');
  console.error('');
  console.error(`Platform: ${platformKey}`);
  console.error('');
  if (PLATFORM_PACKAGES[platformKey]) {
    console.error('Try reinstalling the package:');
    console.error('  npm uninstall @zhxiaogg/fluorite-cli');
    console.error('  npm install @zhxiaogg/fluorite-cli');
  } else {
    console.error(`Your platform (${platformKey}) is not supported.`);
    console.error('');
    console.error('You can build from source instead:');
    console.error('  cargo install fluorite_codegen');
  }
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('error', (err) => {
  console.error('Failed to execute fluorite:', err.message);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
