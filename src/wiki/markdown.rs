use std::collections::HashMap;

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};


// Рендер Markdown, директивы, якоря и слаги.

pub(crate) fn strip_duplicate_title(markdown: &str, title: &str) -> String {
    let mut lines = markdown.lines();

    if let Some(first) = lines.next() {
        let trimmed = first.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            if heading.trim().eq_ignore_ascii_case(title.trim()) {
                return lines
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim_start()
                    .to_string();
            }
        }
    }

    markdown.to_string()
}

/// Достаёт директивы ::video{...}, ::youtube{...} и блоки :::note ... ::: до рендера Markdown.
pub(crate) fn is_callout_kind(kind: &str) -> bool {
    matches!(kind, "note" | "tip" | "warning" | "caution")
}

pub(crate) fn extract_directives(markdown: &str) -> (String, Vec<(String, String)>) {
    let mut prepared = String::with_capacity(markdown.len());
    let mut inserts: Vec<(String, String)> = Vec::new();
    let mut lines = markdown.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if let Some(kind) = trimmed.strip_prefix(":::").map(str::trim) {
            if is_callout_kind(kind) {
                let mut inner = String::new();
                let mut closed = false;

                for next in lines.by_ref() {
                    if next.trim() == ":::" {
                        closed = true;
                        break;
                    }
                    inner.push_str(next);
                    inner.push('\n');
                }

                if closed {
                    let token = format!("DSHCALLOUT{}", inserts.len());
                    let html = format!(
                        "<div class=\"callout callout-{kind}\">{}</div>",
                        render_markdown(&inner)
                    );
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }

                prepared.push_str(line);
                prepared.push('\n');
                prepared.push_str(&inner);
                continue;
            }
        }

        if let Some(body) = trimmed
            .strip_prefix("::video{")
            .and_then(|rest| rest.strip_suffix('}'))
        {
            if let Some(src) = attr_value(body, "src") {
                if let Some(html) = video_html(&src) {
                    let token = format!("DSHVIDEO{}", inserts.len());
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }
            }
        }

        if let Some(body) = trimmed
            .strip_prefix("::youtube{")
            .and_then(|rest| rest.strip_suffix('}'))
        {
            if let Some(id) = attr_value(body, "id") {
                if let Some(html) = youtube_html(&id) {
                    let token = format!("DSHYOUTUBE{}", inserts.len());
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }
            }
        }

        prepared.push_str(line);
        prepared.push('\n');
    }

    (prepared, inserts)
}

pub(crate) fn attr_value(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");

    for part in body.split(',') {
        let part = part.trim();
        if let Some(raw) = part.strip_prefix(&prefix) {
            let value = raw.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

pub(crate) fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn video_html(src: &str) -> Option<String> {
    let allowed =
        src.starts_with("/media/") || src.starts_with("https://") || src.starts_with("http://");

    if !allowed {
        return None;
    }

    Some(format!(
        "<figure class=\"video\"><video controls preload=\"metadata\" src=\"{}\"></video></figure>",
        escape_attr(src)
    ))
}

pub(crate) fn youtube_html(id: &str) -> Option<String> {
    let valid = id.len() >= 6
        && id.len() <= 24
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if !valid {
        return None;
    }

    Some(format!(
        "<figure class=\"video\"><iframe src=\"https://www.youtube-nocookie.com/embed/{id}\" title=\"YouTube\" loading=\"lazy\" allow=\"encrypted-media; picture-in-picture\" allowfullscreen></iframe></figure>"
    ))
}

pub(crate) fn render_markdown(markdown: &str) -> String {
    let (prepared, inserts) = extract_directives(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let events: Vec<Event> = Parser::new_ext(&prepared, options).collect();
    let mut transformed: Vec<Event> = Vec::with_capacity(events.len());
    let mut used_ids: HashMap<String, usize> = HashMap::new();
    let mut index = 0;

    while index < events.len() {
        if let Event::Start(Tag::Heading {
            level,
            id: None,
            classes,
            attrs,
        }) = &events[index]
        {
            let mut text = String::new();
            let mut cursor = index + 1;

            while cursor < events.len() {
                match &events[cursor] {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(value) | Event::Code(value) => text.push_str(value),
                    _ => {}
                }
                cursor += 1;
            }

            let mut slug = slugify(&text);
            if slug.is_empty() {
                slug = "section".to_string();
            }

            let count = used_ids.entry(slug.clone()).or_insert(0);
            *count += 1;

            let anchor = if *count > 1 {
                format!("{slug}-{count}")
            } else {
                slug
            };

            transformed.push(Event::Start(Tag::Heading {
                level: *level,
                id: Some(anchor.into()),
                classes: classes.clone(),
                attrs: attrs.clone(),
            }));

            index += 1;
            continue;
        }

        transformed.push(events[index].clone());
        index += 1;
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed.into_iter());

    let mut cleaner = ammonia::Builder::default();
    cleaner.add_generic_attributes(["class", "id"]);
    let mut safe = cleaner.clean(&html_output).to_string();

    for (token, html) in inserts {
        let wrapped = format!("<p>{token}</p>");
        safe = safe.replace(&wrapped, &html);
        safe = safe.replace(&token, &html);
    }

    safe
}




pub(crate) fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut dashed = false;

    for ch in input.chars() {
        let ch = ch.to_lowercase().next().unwrap_or(ch);
        for c in translit(ch).chars() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
                dashed = false;
            } else if !dashed && !out.is_empty() {
                out.push('-');
                dashed = true;
            }
        }
    }

    out.trim_end_matches('-').to_string()
}

pub(crate) fn translit(ch: char) -> String {
    let mapped = match ch {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'е' => "e",
        'ё' => "e",
        'ж' => "zh",
        'з' => "z",
        'и' => "i",
        'й' => "y",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ф' => "f",
        'х' => "h",
        'ц' => "ts",
        'ч' => "ch",
        'ш' => "sh",
        'щ' => "sch",
        'ъ' => "",
        'ы' => "y",
        'ь' => "",
        'э' => "e",
        'ю' => "yu",
        'я' => "ya",
        _ => return ch.to_string(),
    };
    mapped.to_string()
}

/// Грубое превращение HTML в текст: теги выкидываем, сущности раскрываем.
pub(crate) fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            other if !in_tag => out.push(other),
            _ => {}
        }
    }

    let decoded = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Кусок текста вокруг первого совпадения, совпадения помечены [[ ]].
pub(crate) fn make_snippet(markdown: &str, terms: &[String]) -> String {
    let text = html_to_text(&render_markdown(markdown));
    let lower = text.to_lowercase();

    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for term in terms {
        let needle = term.to_lowercase();
        if needle.is_empty() {
            continue;
        }

        let mut offset = 0;
        while let Some(found) = lower[offset..].find(&needle) {
            let start = offset + found;
            ranges.push((start, start + needle.len()));
            offset = start + needle.len();
            if ranges.len() >= 32 {
                break;
            }
        }
    }

    if ranges.is_empty() {
        return text.chars().take(160).collect();
    }

    ranges.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }

    let first = merged[0].0;
    let mut window_start = first.saturating_sub(70);
    while window_start > 0 && !text.is_char_boundary(window_start) {
        window_start -= 1;
    }

    let mut window_end = (first + 160).min(text.len());
    while window_end < text.len() && !text.is_char_boundary(window_end) {
        window_end += 1;
    }

    let mut out = String::new();
    if window_start > 0 {
        out.push('…');
    }

    let mut cursor = window_start;
    for (start, end) in merged {
        if start < window_start || end > window_end {
            continue;
        }
        out.push_str(&text[cursor..start]);
        out.push_str("[[");
        out.push_str(&text[start..end]);
        out.push_str("]]");
        cursor = end;
    }

    out.push_str(&text[cursor..window_end]);
    if window_end < text.len() {
        out.push('…');
    }

    out
}
