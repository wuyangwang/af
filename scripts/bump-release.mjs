#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');
process.chdir(repoRoot);

const cargoTomlPath = path.join(repoRoot, 'Cargo.toml');
const cargoLockPath = path.join(repoRoot, 'Cargo.lock');

function run(cmd, args, options = {}) {
  return execFileSync(cmd, args, {
    stdio: 'inherit',
    ...options,
  });
}

function runAllowFail(cmd, args) {
  try {
    execFileSync(cmd, args, { stdio: 'ignore' });
  } catch {
    // best effort
  }
}

function readText(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function writeUtf8NoBom(filePath, content) {
  fs.writeFileSync(filePath, content, 'utf8');
}

function readWorkspaceVersion(cargoText) {
  const match = cargoText.match(/^\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : null;
}

function autoBump(current) {
  const m = current.match(/^(\d+)\.(\d+)\.(\d+)(.*)$/);
  if (!m) {
    throw new Error(`Could not auto-bump version '${current}' (expected semver like 0.1.0)`);
  }
  const major = Number(m[1]);
  const minor = Number(m[2]);
  const patch = Number(m[3]) + 1;
  const suffix = m[4] || '';
  return `${major}.${minor}.${patch}${suffix}`;
}

function normalizeVersion(version) {
  if (!/^v?\d+\.\d+\.\d+.*$/.test(version)) {
    throw new Error('Version must look like 0.1.0 or v0.1.0');
  }
  const plain = version.replace(/^v/, '');
  return { plain, tag: `v${plain}` };
}

function updateCargoTomlVersion(cargoText, plain) {
  const re = /(^\[workspace\.package\][\s\S]*?^version\s*=\s*")[^"]+(")/m;
  if (!re.test(cargoText)) {
    throw new Error('Could not find [workspace.package] version in Cargo.toml');
  }
  return cargoText.replace(re, `$1${plain}$2`);
}

let versionArg = process.argv[2] || '';

if (!versionArg) {
  const cargoText = readText(cargoTomlPath);
  const current = readWorkspaceVersion(cargoText);
  if (!current) {
    throw new Error('Version not provided and could not be read from Cargo.toml');
  }
  versionArg = autoBump(current);
  console.log(`Auto-bumping version: ${current} -> ${versionArg}`);
}

const { plain, tag } = normalizeVersion(versionArg);

const cargoText = readText(cargoTomlPath);
const updatedCargo = updateCargoTomlVersion(cargoText, plain);
writeUtf8NoBom(cargoTomlPath, updatedCargo);

try {
  run('cargo', ['update', '-w'], { stdio: 'ignore' });
} catch {
  // best effort
}

const addArgs = ['add', 'Cargo.toml'];
if (fs.existsSync(cargoLockPath)) addArgs.push('Cargo.lock');
runAllowFail('git', addArgs);

run('git', ['commit', '-m', `chore(release): ${tag}`]);
run('git', ['tag', tag]);
run('git', ['push', 'origin', 'HEAD']);
run('git', ['push', 'origin', tag]);

console.log(`Released ${tag} (commit and tag pushed).`);
