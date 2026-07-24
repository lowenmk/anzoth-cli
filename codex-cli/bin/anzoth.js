#!/usr/bin/env node
// Unified entry point for the Anzoth CLI.

import { spawn } from "node:child_process";
import { existsSync, realpathSync } from "fs";
import { createRequire } from "node:module";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);
const anzothPackageRoot = realpathSync(path.join(__dirname, ".."));

const PLATFORM_PACKAGE_BY_TARGET = {
  "x86_64-unknown-linux-musl": "@anzoth/cli-linux-x64",
  "aarch64-unknown-linux-musl": "@anzoth/cli-linux-arm64",
  "x86_64-apple-darwin": "@anzoth/cli-darwin-x64",
  "aarch64-apple-darwin": "@anzoth/cli-darwin-arm64",
  "x86_64-pc-windows-msvc": "@anzoth/cli-win32-x64",
  "aarch64-pc-windows-msvc": "@anzoth/cli-win32-arm64",
};

const { platform, arch } = process;

let targetTriple = null;
switch (platform) {
  case "linux":
  case "android":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-unknown-linux-musl";
        break;
      case "arm64":
        targetTriple = "aarch64-unknown-linux-musl";
        break;
      default:
        break;
    }
    break;
  case "darwin":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-apple-darwin";
        break;
      case "arm64":
        targetTriple = "aarch64-apple-darwin";
        break;
      default:
        break;
    }
    break;
  case "win32":
    switch (arch) {
      case "x64":
        targetTriple = "x86_64-pc-windows-msvc";
        break;
      case "arm64":
        targetTriple = "aarch64-pc-windows-msvc";
        break;
      default:
        break;
    }
    break;
  default:
    break;
}

if (!targetTriple) {
  throw new Error(`Unsupported platform: ${platform} (${arch})`);
}

const platformPackage = PLATFORM_PACKAGE_BY_TARGET[targetTriple];
if (!platformPackage) {
  throw new Error(`Unsupported target triple: ${targetTriple}`);
}

function findAnzothExecutable() {
  let vendorRoot;
  try {
    const packageJsonPath = require.resolve(`${platformPackage}/package.json`);
    vendorRoot = path.join(path.dirname(packageJsonPath), "vendor");
  } catch {
    vendorRoot = path.join(__dirname, "..", "vendor");
  }

  const anzothExecutable = path.join(
    vendorRoot,
    targetTriple,
    "bin",
    process.platform === "win32" ? "anzoth.exe" : "anzoth",
  );
  if (existsSync(anzothExecutable)) {
    return anzothExecutable;
  }

  const packageManager = detectPackageManager();
  const updateCommand =
    packageManager === "bun"
      ? "bun install -g @anzoth/cli@latest"
      : packageManager === "pnpm"
        ? "pnpm add -g @anzoth/cli@latest"
        : "npm install -g @anzoth/cli@latest";
  throw new Error(
    `Missing optional dependency ${platformPackage}. Reinstall Anzoth: ${updateCommand}`,
  );
}

const binaryPath = findAnzothExecutable();

function isPnpmOwnedAnzothInstall(nodeModulesDir) {
  if (!existsSync(path.join(nodeModulesDir, ".modules.yaml"))) {
    return false;
  }

  try {
    return (
      realpathSync(path.join(nodeModulesDir, "@anzoth", "cli")) ===
      anzothPackageRoot
    );
  } catch {
    return false;
  }
}

function detectPackageManager() {
  const entrypointDir = path.dirname(path.resolve(process.argv[1]));
  for (const startDir of new Set([anzothPackageRoot, entrypointDir])) {
    const filesystemRoot = path.parse(startDir).root;
    for (
      let currentDir = startDir;
      currentDir !== filesystemRoot;
      currentDir = path.dirname(currentDir)
    ) {
      if (isPnpmOwnedAnzothInstall(path.join(currentDir, "node_modules"))) {
        return "pnpm";
      }
    }

    if (isPnpmOwnedAnzothInstall(path.join(filesystemRoot, "node_modules"))) {
      return "pnpm";
    }
  }

  const userAgent = process.env.npm_config_user_agent || "";
  if (/\bbun\//.test(userAgent)) {
    return "bun";
  }

  const execPath = process.env.npm_execpath || "";
  if (execPath.includes("bun")) {
    return "bun";
  }

  if (
    __dirname.includes(".bun/install/global") ||
    __dirname.includes(".bun\\install\\global")
  ) {
    return "bun";
  }

  return userAgent ? "npm" : null;
}

const packageManager = detectPackageManager();
const packageManagerEnvVar =
  packageManager === "bun"
    ? "ANZOTH_MANAGED_BY_BUN"
    : packageManager === "pnpm"
      ? "ANZOTH_MANAGED_BY_PNPM"
      : "ANZOTH_MANAGED_BY_NPM";
const env = {
  ...process.env,
  ANZOTH_MANAGED_PACKAGE_ROOT: anzothPackageRoot,
};
delete env.ANZOTH_MANAGED_BY_NPM;
delete env.ANZOTH_MANAGED_BY_BUN;
delete env.ANZOTH_MANAGED_BY_PNPM;
env[packageManagerEnvVar] = "1";

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env,
});

child.on("error", (err) => {
  console.error(err);
  process.exit(1);
});

const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }
  try {
    child.kill(signal);
  } catch {
    /* ignore */
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

const childResult = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    if (signal) {
      resolve({ type: "signal", signal });
    } else {
      resolve({ type: "code", exitCode: code ?? 1 });
    }
  });
});

if (childResult.type === "signal") {
  process.kill(process.pid, childResult.signal);
} else {
  process.exit(childResult.exitCode);
}
