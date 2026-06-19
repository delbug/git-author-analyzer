const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');

// 1. 读取 tauri.conf.json
const confPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const conf = JSON.parse(fs.readFileSync(confPath, 'utf8'));

// 2. 读取 Cargo.toml
const cargoPath = path.join(root, 'src-tauri', 'Cargo.toml');
let cargoContent = fs.readFileSync(cargoPath, 'utf8');

// 3. 解析当前版本，递增 patch
const [major, minor, patch] = conf.version.split('.').map(Number);
const newVersion = `${major}.${minor}.${patch + 1}`;

console.log(`版本递增: ${conf.version} → ${newVersion}`);

// 4. 写回 tauri.conf.json
conf.version = newVersion;
fs.writeFileSync(confPath, JSON.stringify(conf, null, 2) + '\n');

// 5. 写回 Cargo.toml
const versionRegex = /^version\s*=\s*"[^"]*"/m;
cargoContent = cargoContent.replace(versionRegex, `version = "${newVersion}"`);
fs.writeFileSync(cargoPath, cargoContent);

console.log(`已更新: src-tauri/tauri.conf.json, src-tauri/Cargo.toml`);
