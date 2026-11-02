#!/usr/bin/env bash
# Только бэкенд: пересборка и перезапуск на изменения src/ и migrations/.
# Фронтенд запускается отдельно: cd dashboard && bun run dev
set -uo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-watch >/dev/null 2>&1; then
	echo "cargo-watch не установлен: cargo install cargo-watch --locked"
	exit 1
fi

addr="${WIKI_ADDR:-127.0.0.1:3000}"
port="${addr##*:}"

if ss -ltn 2>/dev/null | grep -q ":${port} "; then
	echo "порт ${port} уже занят — останови старый процесс (ss -ltnp | grep :${port})"
	exit 1
fi

export RUST_LOG="${RUST_LOG:-wiki=info}"

exec cargo watch -w src -w migrations -x run
