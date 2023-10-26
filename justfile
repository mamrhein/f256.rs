#!/usr/bin/env just --justfile

release:
  cargo build --release

lint:
  cargo clippy

test-fast:
  cargo nextest run -r -E '!test(slowtest)'

test-slow:
  cargo nextest run -r -E 'test(slowtest)'

test-all:
  cargo hack test --release --feature-powerset --optional-deps num-traits
