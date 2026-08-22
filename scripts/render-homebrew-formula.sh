#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 4 ]]; then
  echo "usage: $0 TAG OWNER/REPO DIST_DIR OUTPUT" >&2
  exit 2
fi

tag="$1"
repository="$2"
dist_dir="$3"
output="$4"
version="${tag#v}"

checksum() {
  sha256sum "$dist_dir/run-code-$1.tar.gz" | awk '{print $1}'
}

macos_arm_sha="$(checksum aarch64-apple-darwin)"
macos_intel_sha="$(checksum x86_64-apple-darwin)"
linux_arm_sha="$(checksum aarch64-unknown-linux-gnu)"
linux_intel_sha="$(checksum x86_64-unknown-linux-gnu)"
base_url="https://github.com/$repository/releases/download/$tag"

mkdir -p "$(dirname "$output")"
cat >"$output" <<RUBY
class RunCode < Formula
  desc "Execute snippets with selected toolchains and temporary dependencies"
  homepage "https://github.com/$repository"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "$base_url/run-code-aarch64-apple-darwin.tar.gz"
      sha256 "$macos_arm_sha"
    else
      url "$base_url/run-code-x86_64-apple-darwin.tar.gz"
      sha256 "$macos_intel_sha"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "$base_url/run-code-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "$linux_arm_sha"
    else
      url "$base_url/run-code-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "$linux_intel_sha"
    end
  end

  def install
    bin.install "run-code"
  end

  test do
    assert_match "run-code #{version}", shell_output("#{bin}/run-code --version")
  end
end
RUBY
