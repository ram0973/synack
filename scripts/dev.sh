#!/usr/bin/env bash
# Оба процесса: Rust с автоперезапуском и Vite с HMR.
# Останавливает только своих детей, чужие процессы не трогает.
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

pids=()

cleanup() {
	trap - EXIT INT TERM
	for pid in "${pids[@]:-}"; do
		[ -n "$pid" ] && kill "$pid" 2>/dev/null || true
	done
	wait 2>/dev/null || true
}
trap cleanup EXIT INT TERM

cargo watch -w src -w migrations -x run &
pids+=("$!")

cd dashboard
bun run dev &
pids+=("$!")

echo
echo "  API:  http://127.0.0.1:3000"
echo "  UI:   http://127.0.0.1:5173  (HMR)"
echo

wait
