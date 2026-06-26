#!/usr/bin/env bash
set -euo pipefail

version="${GITHUB_REF_NAME#v}"
if [[ -z "${version}" || "${version}" == "${GITHUB_REF_NAME}" ]]; then
  version="$(cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.name=="okftool-cli") | .version')"
fi

crate_published() {
  local crate="$1"
  curl --fail --silent --show-error \
    --retry 3 \
    --retry-delay 2 \
    --header "Accept: application/json" \
    --header "User-Agent: okftool-release-script" \
    "https://crates.io/api/v1/crates/${crate}/${version}" \
    >/dev/null 2>/dev/null
}

publish_or_skip() {
  local package="$1"
  local crate="$2"
  local log
  log="$(mktemp)"

  if crate_published "${crate}"; then
    echo "${crate}@${version} is already published; skipping"
    return 0
  fi

  echo "Publishing ${crate}@${version}"
  if cargo publish -p "${package}" 2>&1 | tee "${log}"; then
    return 0
  fi

  if grep -q "already exists on crates.io index" "${log}"; then
    echo "${crate}@${version} is already published; treating as success"
    return 0
  fi

  return 1
}

wait_for_crate() {
  local crate="$1"
  local attempts="${2:-80}"
  local sleep_seconds="${3:-15}"

  for ((i = 1; i <= attempts; i++)); do
    if crate_published "${crate}"; then
      echo "${crate}@${version} is visible in the crates.io index"
      return 0
    fi
    echo "Waiting for ${crate}@${version} in the crates.io index (${i}/${attempts})"
    sleep "${sleep_seconds}"
  done

  echo "::error::${crate} ${version} did not appear in the crates.io index" >&2
  return 1
}

publish_or_skip okftool-core okftool-core
wait_for_crate okftool-core
publish_or_skip okftool-cli okftool-cli
