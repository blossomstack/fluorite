#!/usr/bin/env node
// install.js - Downloads the appropriate binary for the current platform

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = '0.1.0';
const REPO = 'zhxiaogg/fluorite';

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;

  const platformMap = {
    'darwin': 'apple-darwin',
    'linux': 'unknown-linux-gnu',
    'win32': 'pc-windows-msvc'
  };

  const archMap = {
    'x64': 'x86_64',
    'arm64': 'aarch64'
  };

  const targetPlatform = platformMap[platform];
  const targetArch = archMap[arch];

  if (!targetPlatform || !targetArch) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  return {
    target: `${targetArch}-${targetPlatform}`,
    extension: platform === 'win32' ? '.exe' : ''
  };
}

async function downloadBinary() {
  const { target, extension } = getPlatformInfo();
  const binaryName = `fluorite${extension}`;
  const assetName = `fluorite-${target}${extension}`;
  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;

  const binDir = path.join(__dirname, 'bin');
  const binaryPath = path.join(binDir, binaryName);

  // Skip if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('Binary already installed.');
    return;
  }

  console.log(`Downloading fluorite binary for ${target}...`);
  console.log(`URL: ${downloadUrl}`);

  // Create bin directory if it doesn't exist
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // For now, just create a placeholder that tells users to build from source
  // TODO: Implement actual binary download when releases are available
  const placeholderScript = `#!/bin/sh
echo "Error: Pre-built binaries not yet available."
echo "Please build from source: cargo build --release --package fluorite_codegen"
exit 1
`;

  fs.writeFileSync(binaryPath, placeholderScript);
  fs.chmodSync(binaryPath, '755');

  console.log('Note: Pre-built binaries not yet available. See README for build instructions.');
}

downloadBinary().catch(err => {
  console.error('Failed to install:', err.message);
  process.exit(1);
});
