#!/usr/bin/env node
// bin/fluorite.js - Wrapper that invokes the Rust binary

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binDir = path.join(__dirname);
const ext = process.platform === 'win32' ? '.exe' : '';
const binaryPath = path.join(binDir, `fluorite${ext}`);

if (!fs.existsSync(binaryPath)) {
  console.error('Error: fluorite binary not found.');
  console.error('Please run: npm rebuild @zhxiaogg/fluorite-cli');
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
