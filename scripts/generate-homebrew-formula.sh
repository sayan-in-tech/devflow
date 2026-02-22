#!/usr/bin/env bash
set -euo pipefail

mkdir -p packaging/homebrew
awk 'BEGIN { print "class Devflow < Formula"; print "  desc \"Developer workflow automation\""; print "  homepage \"https://github.com/example/devflow\""; print "  url \"https://github.com/example/devflow/releases/download/v0.1.0/devflow-macos-x86_64.tar.gz\""; print "  sha256 \"REPLACE_ME\""; print "  version \"0.1.0\""; print "  def install"; print "    bin.install \"devflow\""; print "  end"; print "end" }' > packaging/homebrew/devflow.rb

echo "Generated packaging/homebrew/devflow.rb"
