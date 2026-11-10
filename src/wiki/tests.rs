use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::auth::Authenticated;
use crate::wiki::categories::{create_category, delete_category, move_category, tree};
use crate::wiki::markdown::{make_snippet, render_markdown, slugify};
use crate::wiki::models::{
    Category, CreateCategory, CreatePage, ListPagesQuery, MoveCategory, PathQuery, SearchQuery,
    UpdatePage,
};
use crate::wiki::pages::{create_page, list_pages, page_by_path, search, update_page};
use crate::wiki::repo::build_tree;
use crate::wiki::revisions::list_revisions;

// ---------- тесты ----------

#[tokio::test]
async fn move_without_parent_keeps_parent() {
    let pool = test_pool().await;

    let (_, Json(root)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: None,
            name: "Root".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(child)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: Some(root.id),
            name: "Child".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    // передаём только position: родитель должен остаться прежним
    let Json(moved) = move_category(
        State(pool.clone()),
        Path(child.id),
        Json(MoveCategory {
            parent_id: None,
            position: Some(0),
        }),
    )
    .await
    .unwrap();

    assert_eq!(moved.parent_id, Some(root.id));

    // явный null — перенос в корень
    let Json(moved) = move_category(
        State(pool.clone()),
        Path(child.id),
        Json(MoveCategory {
            parent_id: Some(None),
            position: Some(0),
        }),
    )
    .await
    .unwrap();

    assert_eq!(moved.parent_id, None);
}

#[tokio::test]
async fn move_reorders_siblings_without_position_collisions() {
    let pool = test_pool().await;

    let mut ids = Vec::new();
    for name in ["A", "B", "C"] {
        let (_, Json(cat)) = create_category(
            State(pool.clone()),
            Json(CreateCategory {
                parent_id: None,
                name: name.into(),
                slug: None,
            }),
        )
        .await
        .unwrap();
        ids.push(cat.id);
    }

    // C перед A: раньше position у всех совпадал и порядок держался на id.
    let _ = move_category(
        State(pool.clone()),
        Path(ids[2]),
        Json(MoveCategory {
            parent_id: Some(None),
            position: Some(0),
        }),
    )
    .await
    .unwrap();

    let Json(tree) = tree(State(pool.clone())).await.unwrap();
    let order: Vec<&str> = tree.iter().map(|node| node.category.name.as_str()).collect();
    assert_eq!(order, ["C", "A", "B"]);

    let mut positions: Vec<i64> = tree.iter().map(|node| node.category.position).collect();
    positions.sort();
    assert_eq!(positions, [0, 1, 2], "position должны идти подряд без повторов");
}

#[tokio::test]
async fn draft_is_hidden_from_anonymous() {
    let pool = test_pool().await;

    let (_, Json(cat)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: None,
            name: "Notes".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let _ = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: cat.id,
            title: "Черновик".into(),
            slug: Some("draft".into()),
            body_md: Some("секретный текст".into()),
            status: Some("draft".into()),
        }),
    )
    .await
    .unwrap();

    let path = PathQuery {
        path: "notes/draft".to_string(),
    };

    let anonymous = page_by_path(State(pool.clone()), Query(path.clone()), None).await;
    assert!(matches!(anonymous, Err(error) if error.status == StatusCode::NOT_FOUND));

    let admin = page_by_path(
        State(pool.clone()),
        Query(path),
        Some(Extension(Authenticated(true))),
    )
    .await;
    assert!(admin.is_ok());

    let hits = search(
        State(pool.clone()),
        Query(SearchQuery {
            q: "секретный".to_string(),
        }),
        None,
    )
    .await
    .unwrap();
    assert!(hits.0.is_empty(), "поиск не должен находить черновики");

    let listed = list_pages(
        State(pool.clone()),
        Query(ListPagesQuery { category: None }),
        None,
    )
    .await
    .unwrap();
    assert!(listed.0.is_empty(), "список не должен показывать черновики");
}

#[tokio::test]
async fn revisions_require_auth() {
    let pool = test_pool().await;

    let (_, Json(cat)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: None,
            name: "Notes".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(page)) = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: cat.id,
            title: "Page".into(),
            slug: None,
            body_md: Some("text".into()),
            status: None,
        }),
    )
    .await
    .unwrap();

    let anonymous = list_revisions(State(pool.clone()), Path(page.id), None).await;
    assert!(matches!(anonymous, Err(error) if error.status == StatusCode::UNAUTHORIZED));

    let admin = list_revisions(
        State(pool.clone()),
        Path(page.id),
        Some(Extension(Authenticated(true))),
    )
    .await;
    assert!(admin.is_ok());
}

#[tokio::test]
async fn titles_and_slugs_are_unique_in_category() {
    let pool = test_pool().await;

    let (_, Json(notes)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: None,
            name: "Notes".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(other)) = create_category(
        State(pool.clone()),
        Json(CreateCategory {
            parent_id: None,
            name: "Other".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(first)) = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: notes.id,
            title: "Введение".into(),
            slug: Some("intro".into()),
            body_md: Some("текст".into()),
            status: Some("published".into()),
        }),
    )
    .await
    .unwrap();

    let duplicate_title = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: notes.id,
            title: "введение".into(),
            slug: Some("intro-2".into()),
            body_md: Some("текст".into()),
            status: None,
        }),
    )
    .await;
    assert!(matches!(duplicate_title, Err(error) if error.status == StatusCode::CONFLICT));

    let duplicate_slug = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: notes.id,
            title: "Другое".into(),
            slug: Some("intro".into()),
            body_md: Some("текст".into()),
            status: None,
        }),
    )
    .await;
    assert!(matches!(duplicate_slug, Err(error) if error.status == StatusCode::CONFLICT));

    // в другой категории те же title и slug допустимы
    let same_in_other = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: other.id,
            title: "Введение".into(),
            slug: Some("intro".into()),
            body_md: Some("текст".into()),
            status: None,
        }),
    )
    .await;
    assert!(same_in_other.is_ok());

    // заводим страницу с заголовком «Другое», чтобы было с чем конфликтовать
    let (_, Json(_)) = create_page(
        State(pool.clone()),
        Json(CreatePage {
            category_id: notes.id,
            title: "Другое".into(),
            slug: Some("drugoe".into()),
            body_md: Some("текст".into()),
            status: None,
        }),
    )
    .await
    .unwrap();

    // переименование в занятое имя — конфликт
    let rename = update_page(
        State(pool.clone()),
        Path(first.id),
        Json(UpdatePage {
            category_id: None,
            title: Some("Другое".into()),
            slug: None,
            body_md: None,
            status: None,
        }),
    )
    .await;
    assert!(matches!(rename, Err(error) if error.status == StatusCode::CONFLICT));

    // сохранение без изменений проходит
    let same = update_page(
        State(pool.clone()),
        Path(first.id),
        Json(UpdatePage {
            category_id: None,
            title: Some("Введение".into()),
            slug: Some("intro".into()),
            body_md: None,
            status: None,
        }),
    )
    .await;
    assert!(same.is_ok());
}

async fn test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[test]
fn slugify_transliterates() {
    assert_eq!(slugify("Привет мир"), "privet-mir");
    assert_eq!(slugify("  Rust / Async  "), "rust-async");
    assert_eq!(slugify("Щука"), "schuka");
}

#[test]
fn build_tree_nests_and_sorts() {
    let rows = vec![
        Category {
            id: 1,
            parent_id: None,
            position: 0,
            name: "A".into(),
            slug: "a".into(),
        },
        Category {
            id: 2,
            parent_id: None,
            position: 1,
            name: "B".into(),
            slug: "b".into(),
        },
        Category {
            id: 3,
            parent_id: Some(1),
            position: 0,
            name: "A1".into(),
            slug: "a1".into(),
        },
    ];
    let tree = build_tree(rows);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree[0].category.name, "A");
    assert_eq!(tree[0].children[0].category.name, "A1");
}

#[test]
fn markdown_is_sanitized() {
    let html = render_markdown("<script>alert(1)</script>\n\n**bold**");
    assert!(!html.contains("<script"));
    assert!(html.contains("<strong>bold</strong>"));
}

#[tokio::test]
async fn create_move_and_reject_cycle() {
    let pool = test_pool().await;
    let state = State(pool.clone());

    let (_, Json(a)) = create_category(
        state.clone(),
        Json(CreateCategory {
            parent_id: None,
            name: "Rust".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(b)) = create_category(
        state.clone(),
        Json(CreateCategory {
            parent_id: Some(a.id),
            name: "Async".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    assert_eq!(b.parent_id, Some(a.id));

    let err = move_category(
        state.clone(),
        Path(a.id),
        Json(MoveCategory {
            parent_id: Some(Some(b.id)),
            position: None,
        }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.status, StatusCode::BAD_REQUEST);

    let err = delete_category(state.clone(), Path(a.id))
        .await
        .unwrap_err();
    assert_eq!(err.status, StatusCode::CONFLICT);
}

#[test]
fn snippet_marks_terms() {
    let snippet = make_snippet(
        "# Введение\n\nТекст про **Markdown** и таблицы.",
        &["markdown".to_string()],
    );

    assert!(snippet.contains("[[Markdown]]"), "{snippet}");
    assert!(!snippet.contains('<'), "{snippet}");
}

#[tokio::test]
async fn page_path_resolves_and_renders() {
    let pool = test_pool().await;
    let state = State(pool.clone());

    let (_, Json(rust)) = create_category(
        state.clone(),
        Json(CreateCategory {
            parent_id: None,
            name: "Rust".into(),
            slug: None,
        }),
    )
    .await
    .unwrap();

    let (_, Json(page)) = create_page(
        state.clone(),
        Json(CreatePage {
            category_id: rust.id,
            title: "Введение".into(),
            body_md: Some("# Привет".into()),
            slug: None,
            status: Some("published".into()),
        }),
    )
    .await
    .unwrap();

    assert_eq!(page.path, "/rust/vvedenie");
    assert!(page.html.contains("<h1"));

    let found = page_by_path(
        state.clone(),
        Query(PathQuery {
            path: "rust/vvedenie".into(),
        }),
        None,
    )
    .await
    .unwrap();

    assert_eq!(found.0.id, page.id);
}