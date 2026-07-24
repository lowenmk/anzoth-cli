"""Shared release/package naming for the public Anzoth CLI distribution."""

from __future__ import annotations

BRAND_NAME = "Anzoth"
GITHUB_REPOSITORY = "lowenmk/anzoth-cli"
NPM_SCOPE = "@anzoth"
MAIN_NPM_PACKAGE = f"{NPM_SCOPE}/cli"
BIN_NAME = "anzoth"
INSTALLER_NAME = "Anzoth"

PLATFORM_PACKAGES: dict[str, dict[str, str]] = {
    "anzoth-linux-x64": {
        "npm_name": f"{NPM_SCOPE}/cli-linux-x64",
        "npm_tag": "linux-x64",
        "target_triple": "x86_64-unknown-linux-musl",
        "os": "linux",
        "cpu": "x64",
    },
    "anzoth-linux-arm64": {
        "npm_name": f"{NPM_SCOPE}/cli-linux-arm64",
        "npm_tag": "linux-arm64",
        "target_triple": "aarch64-unknown-linux-musl",
        "os": "linux",
        "cpu": "arm64",
    },
    "anzoth-darwin-x64": {
        "npm_name": f"{NPM_SCOPE}/cli-darwin-x64",
        "npm_tag": "darwin-x64",
        "target_triple": "x86_64-apple-darwin",
        "os": "darwin",
        "cpu": "x64",
    },
    "anzoth-darwin-arm64": {
        "npm_name": f"{NPM_SCOPE}/cli-darwin-arm64",
        "npm_tag": "darwin-arm64",
        "target_triple": "aarch64-apple-darwin",
        "os": "darwin",
        "cpu": "arm64",
    },
    "anzoth-win32-x64": {
        "npm_name": f"{NPM_SCOPE}/cli-win32-x64",
        "npm_tag": "win32-x64",
        "target_triple": "x86_64-pc-windows-msvc",
        "os": "win32",
        "cpu": "x64",
    },
    "anzoth-win32-arm64": {
        "npm_name": f"{NPM_SCOPE}/cli-win32-arm64",
        "npm_tag": "win32-arm64",
        "target_triple": "aarch64-pc-windows-msvc",
        "os": "win32",
        "cpu": "arm64",
    },
}

PACKAGE_TARBALL_PREFIX = "anzoth-npm"
PACKAGE_ARTIFACT_PREFIX = "anzoth-package"
PACKAGE_CHECKSUM_NAME = f"{PACKAGE_ARTIFACT_PREFIX}_SHA256SUMS"
PACKAGE_ARCHIVE_TEMPLATE = f"{PACKAGE_ARTIFACT_PREFIX}" + "-{target}.tar.gz"
PLATFORM_TARBALL_TEMPLATE = f"{PACKAGE_TARBALL_PREFIX}" + "-{platform}-{version}.tgz"
MAIN_TARBALL_TEMPLATE = f"{PACKAGE_TARBALL_PREFIX}" + "-{version}.tgz"
SDK_TARBALL_TEMPLATE = "anzoth-sdk-npm-{version}.tgz"
RESPONSES_PROXY_TARBALL_TEMPLATE = "anzoth-responses-api-proxy-npm-{version}.tgz"
PACKAGE_CHOICES = (
    "anzoth",
    *PLATFORM_PACKAGES,
    "anzoth-responses-api-proxy",
    "anzoth-sdk",
)
