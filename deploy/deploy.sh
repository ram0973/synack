#!/usr/bin/env bash
# Собирает фронтенд и бэкенд и выкладывает бинарник на сервер.
#   ./deploy/deploy.sh root@203.0.113.10
set -euo pipefail

HOST=${1:-${DEPLOY_HOST:-}}
if [ -z "$HOST" ]; then
	echo "usage: deploy/deploy.sh user@host"
	exit 1
fi

cd "$(dirname "$0")/.."

echo "==> сборка фронтенда"
(cd dashboard && bun run build)

echo "==> сборка бэкенда (SPA вшивается в бинарник)"
cargo build --release

echo "==> пользователь и каталоги на сервере"
ssh "$HOST" '
	set -e
	sudo mkdir -p /opt/synack /var/lib/synack /etc/synack
	if ! id synack >/dev/null 2>&1; then
		sudo useradd --system --home /var/lib/synack --shell /usr/sbin/nologin synack
	fi
	sudo chown -R synack:synack /var/lib/synack
'

echo "==> загрузка бинарника"
scp target/release/wiki "$HOST:/tmp/synack-wiki"
ssh "$HOST" 'sudo install -m 755 /tmp/synack-wiki /opt/synack/wiki && rm -f /tmp/synack-wiki'

echo "==> systemd"
scp deploy/synack.service "$HOST:/tmp/synack.service"
ssh "$HOST" '
	set -e
	sudo mv /tmp/synack.service /etc/systemd/system/synack.service
	sudo systemctl daemon-reload
	sudo systemctl enable synack
	sudo systemctl restart synack
	sleep 1
	sudo systemctl --no-pager --lines=5 status synack || true
'

echo "==> готово"
