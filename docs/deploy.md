# Деплой на VPS

Ниже — Ubuntu 24.04, домен и Caddy с автоматическим HTTPS. Бэкенд слушает
`127.0.0.1:3000`, наружу смотрит только Caddy.

## 1. Подготовка сервера

```bash
# локально
ssh-copy-id root@203.0.113.10          # свой IP
ssh root@203.0.113.10

# на сервере
apt update && apt upgrade -y
apt install -y caddy ufw

ufw allow OpenSSH
ufw allow 80,443/tcp
ufw --force enable
```

DNS: A-запись домена (например `synack.example.com`) → IP сервера.
Подожди, пока запись разойдётся, иначе Caddy не получит сертификат.

## 2. Каталоги и пользователь

```bash
mkdir -p /opt/synack /var/lib/synack /etc/synack
useradd --system --home /var/lib/synack --shell /usr/sbin/nologin synack
chown -R synack:synack /var/lib/synack
```

## 3. Переменные окружения

```bash
cp /path/to/repo/deploy/synack.env.example /etc/synack/synack.env
chmod 600 /etc/synack/synack.env

# по желанию: задать пароль сразу, без admin/admin
printf 'мой-пароль' | ./wiki --hash-password
# вставь строку в /etc/synack/synack.env:
# WIKI_PASSWORD_HASH=$argon2id$v=19$...
```

Без `WIKI_PASSWORD_HASH` при первом запуске создаётся `admin` / `admin`,
а в админке висит баннер, пока пароль не сменён.

## 4. Caddy

```bash
cp deploy/Caddyfile /etc/caddy/Caddyfile   # замени домен на свой
systemctl reload caddy
```

## 5. Первый деплой

С машины, где стоит Rust и Bun:

```bash
./deploy/deploy.sh root@203.0.113.10
```

Скрипт соберёт SPA, соберёт release-бинарник (SPA вшита внутрь), зальёт его
в `/opt/synack/wiki`, поставит systemd-юнит и запустит сервис.

Проверка:

```bash
curl -s https://synack.example.com/health     # {"ok":true,...}
systemctl status synack
journalctl -u synack -n 50 --no-pager
```

## 6. Обновление

```bash
./deploy/deploy.sh root@203.0.113.10
```

Бинарник заменяется, сервис перезапускается; база и медиа лежат отдельно
и не затрагиваются.

## 7. Бэкап

```bash
# на сервере
sqlite3 /var/lib/synack/wiki.db ".backup '/var/backups/wiki-$(date +%F).db'"
tar czf /var/backups/media-$(date +%F).tar.gz -C /var/lib/synack media
```

Достаточно двух файлов: база и каталог `media`.

## Если что-то не так

- `502` от Caddy — сервис не запустился: `journalctl -u synack -n 50`.
- `permission denied` на базу — `chown -R synack:synack /var/lib/synack`.
- Нет сертификата — проверь A-запись и что порт 80 открыт.
- Порт занят — `ss -ltnp | grep :3000`.

## CI/CD (GitHub Actions)

- .github/workflows/ci.yml — на каждый push и PR: cargo test --all-targets, release-сборка,
  un run check и un run build.
- .github/workflows/deploy.yml — на push в main: собирает SPA и бинарник, прогоняет тесты,
  заливает бинарник на сервер и перезапускает systemd-сервис.

Включается переменной репозитория: Settings → Secrets and variables → Actions → **Variables** →
DEPLOY_ENABLED = true.

Секреты (Settings → Secrets and variables → Actions → **Secrets**):

| Имя | Значение |
|---|---|
| DEPLOY_HOST | IP сервера |
| DEPLOY_USER | 
oot |
| DEPLOY_KEY | приватный ed25519-ключ деплоя (публичный — в ~/.ssh/authorized_keys на сервере) |

Пока DEPLOY_ENABLED не задана, workflow деплоя просто не запускается — CI продолжает работать.

## Если ТСПУ режет QUIC

Симптом: браузер пишет `QUIC protocol error`, сайт «не дышит», хотя TCP-порты отвечают.
Caddy по умолчанию отдаёт HTTP/3 по UDP 443, а DPI в РФ часто его дропает — браузер
не всегда успевает тихо откатиться на HTTP/2.

В `/etc/caddy/Caddyfile` добавь глобальный блок (он уже есть в `deploy/Caddyfile`):

```
{
	servers {
		protocols h1 h2
	}
}
```

Применить:

```bash
caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
systemctl reload caddy
```

Проверка: в ответе не должно быть заголовка `Alt-Svc`:

```bash
curl -sI https://домен | grep -i alt-svc
```

Готовый скрипт: `./scripts/tests/disable-quic.sh` (заходит по SSH и правит Caddyfile).

Пока сервер недоступен, в браузере можно выключить QUIC вручную:
Chrome — `chrome://flags/#enable-quic` → Disabled, Firefox — `network.http.http3.enable = false`.
