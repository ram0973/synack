# Стек: wiki

Статус: черновик. Зафиксированы решения для MVP, остальное — в «Открытых вопросах».

## Что строим

Личная вики: дерево категорий, страницы в Markdown, один автор.
Один сервер, один бинарь, одна база-файл.

## Решения коротко

| Область | Выбор | Почему |
|---|---|---|
| Бэкенд | Rust + axum 0.8 + tokio | выбранный стек, один бинарь |
| БД | SQLite (sqlx 0.8, WAL) | один сервер, ноль ops, FTS5 на будущее |
| Контент | Markdown в колонке `body_md` | канон — текст, легко выгрузить, одна правда |
| Фронтенд | SvelteKit 2 + Svelte 5, `adapter-static`, `ssr = false` | SPA без Node в проде |
| Сборка SPA | Vite, TS | стандарт |
| Статика | вшита в бинарь через `rust-embed` | деплой = один файл |
| Рендер Markdown | `pulldown-cmark` + `ammonia` на сервере | один рендерер, один санитайзер |
| Авторизация | один аккаунт, argon2id, HttpOnly-cookie | MVP |
| Деплой | systemd + Caddy | привычно, HTTPS из коробки |

## Архитектура

```
браузер
  │  SvelteKit SPA (статика из бинаря)
  │  fetch /api/*  (JSON)
  ▼
axum (Rust)
  ├── sqlx ──► SQLite: wiki.db
  └── rust-embed: dashboard/build/
```

Один процесс, один порт. Caddy спереди терминирует TLS.

## Бэкенд

- Rust (stable), axum 0.8, tokio.
- sqlx 0.8 + SQLite, режим WAL, миграции в `migrations/*.sql`.
- `pulldown-cmark` — Markdown → HTML; `ammonia` — санитайз (только разрешённые теги).
- Пароль: `argon2id`. Сессия: случайный токен в HttpOnly + SameSite=Lax cookie, серверная таблица `sessions`.
- Ошибки: единый JSON `{"error":{"code":"...","message":"..."}}` + корректный HTTP-статус.
- Логи: `tracing` + `tracing-subscriber`.

## Фронтенд

- SvelteKit 2 / Svelte 5, TypeScript, `adapter-static` с `fallback: 'index.html'` и `ssr = false`.
- Страницы: список/дерево, просмотр статьи, редактор, логин.
- Редактор MVP: `<textarea>` + живое превью Markdown. Канон — Markdown, конвертаций нет.
- TipTap — опция на потом (см. открытые вопросы).

## Схема данных (MVP)

```sql
CREATE TABLE categories (
  id          INTEGER PRIMARY KEY,
  parent_id   INTEGER REFERENCES categories(id) ON DELETE RESTRICT,
  position    INTEGER NOT NULL DEFAULT 0,
  name        TEXT    NOT NULL,
  slug        TEXT    NOT NULL UNIQUE,
  created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pages (
  id          INTEGER PRIMARY KEY,
  category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
  slug        TEXT    NOT NULL,
  title       TEXT    NOT NULL,
  body_md     TEXT    NOT NULL DEFAULT '',
  status      TEXT    NOT NULL DEFAULT 'draft'
              CHECK (status IN ('draft','published')),
  created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT    NOT NULL DEFAULT (datetime('now')),
  UNIQUE (category_id, slug)
);

CREATE TABLE sessions (
  token       TEXT PRIMARY KEY,
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  expires_at  TEXT NOT NULL
);
```

Дерево — adjacency list (`parent_id`). Выборка — рекурсивный CTE.
Перенос категории — `UPDATE parent_id, position` в транзакции с проверкой
«нельзя вложить в собственного потомка» (тот же CTE по `descendants`).

Статья живёт ровно в одной категории. Переезд = смена `category_id`.
Уникальность slug — в пределах категории.

## API (v1)

| Метод | Путь | Что делает |
|---|---|---|
| GET | `/api/wiki/tree` | дерево категорий (id, name, slug, parent_id, position) |
| POST | `/api/wiki/categories` | создать категорию |
| PATCH | `/api/wiki/categories/:id` | переименовать / переместить / отсортировать |
| DELETE | `/api/wiki/categories/:id` | удалить (только пустую) |
| GET | `/api/wiki/pages/:id` | страница с отрендеренным HTML |
| POST | `/api/wiki/pages` | создать |
| PATCH | `/api/wiki/pages/:id` | сохранить (title, body_md, category_id, status) |
| DELETE | `/api/wiki/pages/:id` | удалить |
| GET | `/api/wiki/pages?category=:id` | список в категории |
| POST | `/api/auth/login` | логин |
| POST | `/api/auth/logout` | выход |
| GET | `/api/auth/me` | кто я |


## Разделы сайта

Сайт — набор разделов, вики сейчас единственный готовый.

| Раздел | Публичные URL | API | Статус |
|---|---|---|---|
| Вики | `/wiki`, `/wiki/<путь>`, `/wiki/search` | `/api/wiki/*` | готов |
| Блог | `/blog/...` | `/api/blog/*` | позже |
| Страницы | `/pages/...` | `/api/pages/*` | позже |

- Админка: `/admin/wiki` (категории и статьи), `/admin/password`; `/admin` редиректит на `/admin/wiki`.
  Вкладки разделов — `routes/admin/+layout.svelte`, будущие («Блог», «Страницы») пока неактивны.
- Общее между разделами: `auth`, `media`, `error`; всё вики — в `src/wiki/`.

## Сборка и деплой

Node (или Bun) нужен **только для сборки** SPA:

```
dashboard/  →  bun run build  →  dashboard/build/  →  rust-embed  →  cargo build --release  →  wiki
```

В проде Node нет. Артефакт: `wiki` + `wiki.db` + `media/`.
Бэкап: `sqlite3 wiki.db "VACUUM INTO 'backup.db'"` + копия `media/`.

## Что НЕ входит в MVP

- drag&drop дерева категорий (модель данных готова, UI — следующий шаг);
- поиск (FTS5);
- история версий и откат;
- загрузка картинок, диаграммы Mermaid, подсветка кода;
- несколько пользователей и роли;
- i18n и темы.

## Решено по MVP

- Редактор — `textarea` + превью, канон Markdown.
- URL — пути категорий: `/rust/async/vvedenie` (slug'и латиницей, транслит на бэке).
- Деплой в MVP не делаем: запуск локально в WSL.
- Проект — `wiki/` внутри `C:\src\sh0-lite`.

## Открытые вопросы

1. TipTap — когда и нужен ли, если канон Markdown (понадобится конвертация HTML↔Markdown).
2. Как показывать корень `/` и пустые категории.
3. Позже: деплой (systemd+Caddy или через sh0-lite).

## Окружение разработки

- WSL: Ubuntu-26.04, ядро 6.18.
- Rust: `cargo`/`rustc` в `/root/.cargo/bin`.
- Bun: `/usr/local/bin/bun` — сборка SPA, Node не нужен.
- Код лежит на Windows-диске (`/mnt/c/src/sh0-lite/wiki`), сборка и запуск — внутри WSL.
- Docker Desktop есть, но текущему пользователю недоступен — не используем.

## Статика и вшивание

`rust-embed` держит `dashboard/build/` внутри бинаря.

- **debug**: файлы читаются с диска (`/home/ra/wiki/dashboard/build`) — SPA можно пересобрать и перезапустить Rust, вшивать ничего не нужно.
- **release**: файлы вшиты, бинарь самодостаточен.

Отдача: `fallback(static_handler)` — не-API путь отдаёт файл, если он есть, иначе `index.html` (SPA-роутинг).
`/api/*` без совпадения — 404 JSON. Ассеты `_app/immutable/*` кешируются на год.

## Фронтенд (актуально)

- SvelteKit 3 + Svelte 5, `adapter-static`, `ssr = false`.
- Tailwind v4 + shadcn-svelte 1.7: button, card, input, badge, dialog, dropdown-menu,
  tabs, sidebar, sonner, skeleton, separator.
- Шрифт Inter Variable (`@fontsource-variable/inter`), тёмная тема через `mode-watcher`.
- Сайдбар сворачивается (`collapsible="offcanvas"`): кнопка-гамбургер в верхней панели,
  горячая клавиша Ctrl/Cmd+B, состояние запоминается в cookie `sidebar_state`.
  На мобильном открывается как Sheet и закрывается по Escape/клику по оверлею.
- В редакторе поля с подписями (Заголовок, Категория, Ссылка), кнопка «Сохранить» и переключатель
  публикации: у черновика — «Опубликовать», у опубликованной — «Отменить публикацию».
  После сохранения редактор остаётся на месте, статус обновляется бейджем.
- Текст редактируется в CodeMirror 6 (`codemirror`, `@codemirror/lang-markdown`,
  `@codemirror/language-data`, тема `one-dark` в тёмном режиме): подсветка Markdown и кода
  внутри ```-блоков, языки (TS, Rust, Java, …) подгружаются лениво через `language-data`.
  Превью остаётся серверным (`POST /api/wiki/preview`).
- Видео: кнопка «Видео» → загрузка файла (mp4/webm, до 200 МБ) или ссылка YouTube.
  В текст пишутся директивы `::video{src="/media/…"}` и `::youtube{id="…"}`;
  сервер заменяет их на `<video>`/`<iframe youtube-nocookie>` уже после санитайза,
  `src` проверяется (только `/media/` или http(s)), id — по маске.
- Переключатель «Текст / Превью» — сегментированная группа кнопок, а не вкладки shadcn.
- Кнопка «Таблица» — диалог с числом столбцов и строк, вставляет Markdown-таблицу с заголовками
  «Столбец N» и пустыми строками.
- Проверка вёрстки: Playwright + headless Chromium, `wiki/shots3.sh`.

Грабли: shadcn-svelte ожидает кастомные варианты Tailwind — `data-open`, `data-closed`,
`data-checked`, `data-unchecked`, `data-active (=true)`, `data-horizontal`, `data-vertical`.
Без них табы рендерятся в строку, а сайдбар подсвечивает все пункты.

## Авторизация

Один пользователь. Пароль берётся из `WIKI_PASSWORD` (если не задан — `admin` и предупреждение в логе),
хеш argon2id считается при старте.

- `POST /api/auth/login {password}` → 200 + `Set-Cookie: session=…; HttpOnly; SameSite=Lax; Max-Age=2592000`
- `POST /api/auth/logout` → удаляет сессию и чистит cookie
- `GET /api/auth/me` → `{ "authenticated": bool, "default_password": bool }`

Сессии живут в таблице `sessions` 30 дней. Middleware `auth_guard` пропускает GET/HEAD/OPTIONS,
а любые изменения требует с валидной сессией (исключение — `/api/auth/login`). UI: `/admin` и редактор
редиректят на `/login`, в футере сайдбара — вход/выход.

## Картинки и видео

- `POST /api/media` (multipart, только с сессией): картинки `png/jpeg/gif/webp/avif` до 10 МБ,
  видео `mp4/webm` до 200 МБ. Имя случайное, расширение из MIME.
- Раскладка на диске: `WIKI_MEDIA/YYYY/MM/<hash>.<ext>` (год и месяц берутся из SQLite `strftime`),
  в базе `media.filename` хранит относительный путь `YYYY/MM/<hash>.<ext>`.
- `GET /media/{*path}` отдаёт файл с `Cache-Control: immutable`, отсекает `..` и бэкслеши.
- В редакторе: кнопка «Картинка» (файл/drag&drop/вставка), кнопка «Видео»
  (файл или ссылка YouTube) — в текст пишутся `![alt](/media/…)`, `::video{src="…"}`,
  `::youtube{id="…"}`. Директивы раскрываются сервером уже после санитайза.
- SVG запрещён (XSS).

## Разметка в редакторе
- `::video{src="/media/…"}` и `::youtube{id="…"}` — видео.
- `:::note`, `:::tip`, `:::warning`, `:::caution` … `:::` — цветные врезки.
- Кнопки: «Ссылка» (`[текст](адрес)` с проверкой схемы), «Врезка» (тип + текст),
  «Таблица» (столбцы/строки), «Картинка», «Видео».
- Вставка из буфера: если там есть HTML (страница, Word, Google Docs), он конвертируется
  в Markdown через Turndown + GFM (таблицы, зачёркивание, чек-листы). Картинки становятся
  ссылками на исходные URL — на `/media/` они не загружаются.

## Поиск

- FTS5: `pages_fts` (external content = `pages`, токенизатор `unicode61`), триггеры держат индекс
  в актуальном состоянии, миграция бэкфиллит существующие страницы.
- `GET /api/wiki/search?q=…`: термины чистятся до букв/цифр, каждый ищется как префикс (`"табл"*`),
  сортировка `bm25`, лимит 20.
- Сниппет собирается в Rust: Markdown → HTML → текст, окно вокруг первого совпадения,
  совпадения помечаются `[[…]]`. Фронтенд режет маркеры и рисует `<mark>` — без `{@html}`.
- UI: поле поиска в шапке сайдбара и страница `/search?q=…`.

## История версий

- Таблица `revisions(page_id, title, body_md, created_at)`, удаляется каскадом вместе со страницей.
- Версия пишется при создании страницы и при каждом сохранении (PATCH).
- `GET /api/pages/{id}/revisions` — последние 50, с превью первых 90 символов текста.
- `GET /api/revisions/{id}` — полная версия.
- `POST /api/revisions/{id}/restore` — возвращает текст в страницу и создаёт ещё одну версию
  (история не перезаписывается, откат обратим).
- UI: кнопка «История» в редакторе → диалог со списком версий и кнопкой «Вернуть».

## API (дополнено)

| Метод | Путь | Что делает |
|---|---|---|
| GET | `/api/resolve?path=…` | страница или категория одним запросом |
| POST | `/api/preview` | Markdown → HTML для превью в редакторе |

## Подсветка кода- Код в ```-блоках подсвечивается на клиенте через `highlight.js/lib/common`
  (`src/lib/highlight.ts`, вызывается в превью редактора и на странице статьи).
- Цвета и фон — CSS-переменные `--code-bg`, `--code-fg`, `--code-keyword`, `--code-string`, …
  для светлой и тёмной темы; переключение вместе с сайтом.
- Языки берутся из класса `language-rust`/`language-typescript`, который ставит pulldown-cmark.

## Режим разработки
- `./scripts/dev-rust.sh` — только бэкенд: `cargo watch` пересобирает и перезапускает на изменения
  `src/` и `migrations/`. Фронтенд отдельно: `cd dashboard && bun run dev`.
- `./scripts/dev.sh` — бэкенд + Vite с HMR (UI на :5173, прокси `/api` и `/media` на :3000).
- Оба скрипта проверяют, что порт из `WIKI_ADDR` (по умолчанию 3000) свободен, и при выходе
  останавливают только своих детей — чужие процессы не трогают.
- `cargo-watch` ставится один раз: `cargo install cargo-watch --locked`.
- Тестовые прогоны и скриншоты: `scripts/tests/*.sh` (Playwright-сценарии в `e2e/*.mjs`),
  одноразовые патч-скрипты: `scripts/patches/*.py`. В корне репозитория — только код и доки.

## Пароль администратора

- Первый запуск: создаётся учётка `admin` / `admin` — в базе только хеш argon2id,
  в лог пишется предупреждение без самого пароля.
- Пока пароль стандартный, в админке сверху висит баннер со ссылкой на смену.
- Смена: `/admin/password` (текущий → новый, минимум 8 символов); остальные сессии
  сбрасываются, текущая остаётся.
- Плейнтекстовых переменных окружения нет. `WIKI_PASSWORD_HASH` нужен только чтобы задать
  пароль при первом запуске или после сброса.
- Сброс: `DELETE FROM settings WHERE key = 'password_hash';` → перезапуск (снова admin/admin).
- Хеш: `printf 'пароль' | ./wiki --hash-password`.

## Отмена и черновики

- Undo/redo: история CodeMirror (`Ctrl+Z` / `Ctrl+Shift+Z`) плюс кнопки ↺ ↻ в тулбаре редактора.
- Автосохранение: через 600 мс после правки текст (заголовок, slug, категория, статус, тело)
  пишется в `localStorage` под ключом `wiki-draft:<id>` (для новой страницы — `wiki-draft:new`).
- При открытии редактора черновик восстанавливается, если отличается от серверного текста;
  тост «Восстановлен несохранённый черновик» с кнопкой «Сбросить» возвращает серверную версию.
- После успешного сохранения или удаления черновик стирается.

## Якоря заголовков

- Сервер при рендере Markdown проставляет `id` каждому заголовку: транслит slug
  (`Первый раздел` → `pervyy-razdel`), дубликаты получают суффикс `-2`, `-3`.
- На странице и в превью рядом с заголовком появляется иконка ссылки (по наведению),
  клик ведёт на `#id`; `scroll-margin-top` учитывает липкую шапку.
