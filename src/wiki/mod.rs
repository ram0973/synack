//! Wiki: дерево категорий, страницы, поиск, история версий.
//!
//! Логика разложена по назначению:
//! - [`models`] — модели БД и DTO
//! - [`categories`] — категории и дерево
//! - [`pages`] — страницы и поиск
//! - [`revisions`] — история версий
//! - [`repo`] — общие запросы к БД
//! - [`markdown`] — рендер Markdown
//! - [`router`] — сборка маршрутов
//!
//! Тесты живут здесь: они проверяют модули вместе.

pub(crate) mod categories;
pub(crate) mod markdown;
pub(crate) mod models;
pub(crate) mod pages;
pub(crate) mod repo;
pub(crate) mod revisions;
pub(crate) mod router;

pub use router::router;

#[cfg(test)]
mod tests;
