#!/usr/bin/env node
"use strict";

const https = require("https");
const http = require("http");
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");
const os = require("os");

const pkg = require("./package.json");
const VERSION = `v${pkg.version}`;
const REPO = "Agora-Build/Hatch";
const BIN_DIR = path.join(__dirname, "bin");
const BIN_NAME = os.platform() === "win32" ? "hatch.exe" : "hatch";
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

function getPlatformKey() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    "linux-x64": "x86_64-unknown-linux-gnu",
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "darwin-x64": "x86_64-apple-darwin",
    "darwin-arm64": "aarch64-apple-darwin",
    "win32-x64": "x86_64-pc-windows-msvc",
  };

  const key = `${platform}-${arch}`;
  if (!map[key]) {
    console.error(`Unsupported platform: ${key}`);
    console.error("Supported: linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64");
    process.exit(1);
  }
  return map[key];
}

function getDownloadUrl() {
  const platformKey = getPlatformKey();
  const ext = os.platform() === "win32" ? ".exe" : "";
  return `https://github.com/${REPO}/releases/download/${VERSION}/hatch-${VERSION}-${platformKey}${ext}.tar.gz`;
}

function fetch(url, redirects = 0) {
  if (redirects > 5) {
    return Promise.reject(new Error("Too many redirects"));
  }
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https") ? https : http;
    client
      .get(url, { headers: { "User-Agent": "hatch-npm-installer" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return resolve(fetch(res.headers.location, redirects + 1));
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`Download failed: HTTP ${res.statusCode} for ${url}`));
        }
        const chunks = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

async function install() {
  const url = getDownloadUrl();
  console.log(`Downloading hatch ${VERSION} for ${getPlatformKey()}...`);
  console.log(`  ${url}`);

  const tarball = await fetch(url);

  const tmpFile = path.join(os.tmpdir(), `hatch-${Date.now()}.tar.gz`);
  fs.writeFileSync(tmpFile, tarball);

  fs.mkdirSync(BIN_DIR, { recursive: true });

  try {
    execSync(`tar -xzf "${tmpFile}" -C "${BIN_DIR}"`, { stdio: "pipe" });
  } finally {
    fs.unlinkSync(tmpFile);
  }

  fs.chmodSync(BIN_PATH, 0o755);
  console.log(`Installed hatch ${VERSION} to ${BIN_PATH}`);
}

install().catch((err) => {
  console.error(`Failed to install hatch: ${err.message}`);
  console.error("");
  console.error("You can manually download the binary from:");
  console.error(`  https://github.com/${REPO}/releases/tag/${VERSION}`);
  process.exit(1);
});
