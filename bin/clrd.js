#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Try to load native binding
function loadBinding() {
  const bindingPath = path.join(__dirname, '..', 'index.js');

  if (fs.existsSync(bindingPath)) {
    return require(bindingPath);
  }

  // Fallback: try platform-specific binding
  const platform = process.platform;
  const arch = process.arch;

  const triples = {
    'darwin-x64': 'darwin-x64',
    'darwin-arm64': 'darwin-arm64',
    'linux-x64': 'linux-x64-gnu',
    'linux-arm64': 'linux-arm64-gnu',
    'win32-x64': 'win32-x64-msvc',
    'win32-arm64': 'win32-arm64-msvc',
  };

  const triple = triples[`${platform}-${arch}`];
  if (!triple) {
    console.error(`Unsupported platform: ${platform}-${arch}`);
    process.exit(1);
  }

  try {
    return require(`clrd-${triple}`);
  } catch (e) {
    console.error('Failed to load native binding:', e.message);
    process.exit(1);
  }
}

// Main entry point
async function main() {
  const args = process.argv.slice(2);

  try {
    const binding = loadBinding();

    // Pass CLI args to Rust and execute
    const exitCode = await binding.run(args);
    process.exit(exitCode);
  } catch (error) {
    console.error('clrd error:', error.message);
    process.exit(1);
  }
}

main();
