use regex::Regex;
use std::{ops::Range, sync::LazyLock};
use uuid::Uuid;

static CHECKBOX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([ \t]*)(?:[-*+] |\d+[.)] )\[([^\]])\](?:[ \t]+|$)(.*)$").unwrap()
});
static IDENTIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*<!--\s*tray-tasks:([a-zA-Z0-9-]+)\s*-->").unwrap());
static HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+?)(?:\s+#+)?\s*$").unwrap());

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub deferred: bool,
    pub indent: usize,
    pub parent_id: Option<String>,
    pub line: Range<usize>,
    pub block: Range<usize>,
    pub marker: usize,
}
fn indentation(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace() && *c != '\r' && *c != '\n')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}
pub fn newline(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}
fn section(text: &str, heading: &str) -> Range<usize> {
    if heading.trim().is_empty() {
        return 0..text.len();
    }
    let mut start = None;
    let mut level = 0;
    let mut offset = 0;
    let mut fence: Option<char> = None;
    for line in text.split_inclusive('\n') {
        let clean = line.trim_end_matches(['\r', '\n']);
        let trim = clean.trim_start();
        if trim.starts_with("```") || trim.starts_with("~~~") {
            let ch = trim.chars().next().unwrap();
            if fence == Some(ch) {
                fence = None;
            } else if fence.is_none() {
                fence = Some(ch);
            }
        } else if fence.is_none() {
            if let Some(c) = HEADING.captures(clean) {
                let this_level = c[1].len();
                if let Some(section_start) = start {
                    if this_level <= level {
                        return section_start..offset;
                    }
                }
                if start.is_none() && c[2].trim() == heading.trim().trim_start_matches('#').trim() {
                    start = Some(offset + line.len());
                    level = this_level;
                }
            }
        }
        offset += line.len();
    }
    start
        .map(|s| s..text.len())
        .unwrap_or(usize::MAX..usize::MAX)
}
pub fn parse(text: &str, heading: &str) -> Vec<Task> {
    let scope = section(text, heading);
    let lines: Vec<(&str, usize)> = text
        .split_inclusive('\n')
        .scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((line, start))
        })
        .collect();
    let mut tasks = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut frontmatter = false;
    for (idx, &(line, offset)) in lines.iter().enumerate() {
        let clean = line.trim_end_matches(['\r', '\n']);
        if idx == 0 && clean.trim_start_matches('\u{feff}') == "---" {
            frontmatter = true;
            continue;
        }
        if frontmatter {
            if clean == "---" || clean == "..." {
                frontmatter = false;
            }
            continue;
        }
        let trim = clean.trim_start();
        if trim.starts_with("```") || trim.starts_with("~~~") {
            let ch = trim.chars().next().unwrap();
            let len = trim.chars().take_while(|c| *c == ch).count();
            if let Some((old_ch, old_len)) = fence {
                if old_ch == ch && len >= old_len {
                    fence = None;
                }
            } else {
                fence = Some((ch, len));
            }
            continue;
        }
        if fence.is_some() || offset < scope.start || offset >= scope.end {
            continue;
        }
        let Some(c) = CHECKBOX.captures(clean) else {
            continue;
        };
        let status = c[2].chars().next().unwrap();
        if ![' ', 'x', 'X', '>'].contains(&status) {
            continue;
        }
        let body = c.get(3).unwrap().as_str();
        let id = IDENTIFIER
            .captures(body)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let title = IDENTIFIER.replace_all(body, "").trim().to_string();
        let indent = indentation(clean);
        let mut end = offset + line.len();
        for &(next, next_offset) in lines.iter().skip(idx + 1) {
            let clean_next = next.trim_end_matches(['\r', '\n']);
            if clean_next.trim().is_empty() {
                continue;
            }
            if indentation(clean_next) <= indent {
                break;
            }
            end = next_offset + next.len();
        }
        let parent_id = tasks
            .iter()
            .rev()
            .find(|t: &&Task| t.indent < indent && t.block.end > offset)
            .map(|t| t.id.clone())
            .filter(|s| !s.is_empty());
        tasks.push(Task {
            id,
            title,
            done: matches!(status, 'x' | 'X'),
            deferred: status == '>',
            indent,
            parent_id,
            line: offset..offset + line.len(),
            block: offset..end,
            marker: offset + c.get(2).unwrap().start(),
        });
    }
    tasks
}
pub fn ensure_ids(text: &str, heading: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut output = text.to_string();
    for task in parse(text, heading).into_iter().rev() {
        if !task.id.is_empty() && seen.insert(task.id.clone()) {
            continue;
        }
        let line = &text[task.line.clone()];
        let ending = if line.ends_with("\r\n") {
            "\r\n"
        } else if line.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let body = IDENTIFIER.replace_all(line.trim_end_matches(['\r', '\n']), "");
        let updated = format!(
            "{} <!-- tray-tasks:{} -->{}",
            body.trim_end(),
            Uuid::new_v4(),
            ending
        );
        output.replace_range(task.line, &updated);
    }
    output
}
pub fn find(text: &str, id: &str) -> Result<Task, String> {
    let matches: Vec<_> = parse(text, "").into_iter().filter(|t| t.id == id).collect();
    if matches.len() != 1 {
        return Err("Task changed or its identifier is duplicated; refresh the list.".into());
    }
    Ok(matches[0].clone())
}
pub fn set_done(text: &str, id: &str, done: bool) -> Result<String, String> {
    let task = find(text, id)?;
    let mut out = text.to_string();
    out.replace_range(task.marker..task.marker + 1, if done { "x" } else { " " });
    Ok(out)
}
pub fn rename(text: &str, id: &str, name: &str) -> Result<String, String> {
    if name.trim().is_empty() || name.contains(['\r', '\n']) {
        return Err("Enter a task name on one line.".into());
    }
    let task = find(text, id)?;
    let old = &text[task.line.clone()];
    let ending = if old.ends_with("\r\n") {
        "\r\n"
    } else if old.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    let mut out = text.to_string();
    out.replace_range(
        task.line.clone(),
        &format!(
            "{} {} <!-- tray-tasks:{} -->{}",
            &text[task.line.start..task.marker + 2],
            name.trim(),
            id,
            ending
        ),
    );
    Ok(out)
}
pub fn remove(text: &str, id: &str) -> Result<String, String> {
    let task = find(text, id)?;
    let mut out = text.to_string();
    out.replace_range(task.block, "");
    Ok(out)
}
pub fn append(text: &str, block: &str, heading: &str) -> String {
    let nl = newline(text);
    let block = block.replace("\r\n", "\n").replace('\n', nl);
    let scope = section(text, heading);
    let mut output = text.to_string();
    if !heading.trim().is_empty() && scope.start == usize::MAX {
        if !output.ends_with('\n') && !output.is_empty() {
            output.push_str(nl);
        }
        output.push_str(&format!(
            "{}## {}{}",
            nl,
            heading.trim().trim_start_matches('#').trim(),
            nl
        ));
        output.push_str(block.trim_end_matches(['\r', '\n']));
        output.push_str(nl);
        return output;
    }
    let insert = if heading.trim().is_empty() {
        text.len()
    } else {
        scope.end
    };
    let prefix = if insert > 0 && !text[..insert].ends_with('\n') {
        nl
    } else {
        ""
    };
    output.insert_str(
        insert,
        &format!("{}{}{}", prefix, block.trim_end_matches(['\r', '\n']), nl),
    );
    output
}
pub type MovedBlock = (String, String, String, Vec<(String, String)>);
pub fn moved(
    text: &str,
    id: &str,
    destination: &str,
    rollover: bool,
) -> Result<MovedBlock, String> {
    let task = find(text, id)?;
    let original_block = &text[task.block.clone()];
    let descendants = parse(original_block, "");
    let mut mapping = Vec::new();
    let mut copied = original_block.to_string();
    for child in descendants.iter().rev() {
        let new_id = Uuid::new_v4().to_string();
        copied = copied.replace(
            &format!("tray-tasks:{}", child.id),
            &format!("tray-tasks:{}", new_id),
        );
        mapping.push((child.id.clone(), new_id));
    }
    // If moving a nested item alone, promote its block to a top-level item.
    let prefix = &text[task.line.start
        ..task.line.start + text[task.line.clone()].len()
            - text[task.line.clone()].trim_start().len()];
    if !prefix.is_empty() {
        copied = copied
            .split_inclusive('\n')
            .map(|l| l.strip_prefix(prefix).unwrap_or(l))
            .collect();
    }
    let new_id = mapping
        .iter()
        .find(|(old, _)| old == id)
        .map(|(_, new)| new.clone())
        .ok_or("Missing task identifier")?;
    let mut retained = original_block.to_string();
    if rollover {
        for child in descendants.iter().rev() {
            retained.replace_range(child.marker..child.marker + 1, "x");
        }
    } else {
        for child in descendants.iter().rev() {
            if !child.done {
                retained.replace_range(child.marker..child.marker + 1, ">");
            }
        }
    }
    let first_end = retained.find('\n').unwrap_or(retained.len());
    let insert = if retained[..first_end].ends_with('\r') {
        first_end - 1
    } else {
        first_end
    };
    retained.insert_str(insert, &format!(" → [[{}]]", destination));
    let mut source = text.to_string();
    source.replace_range(task.block, &retained);
    Ok((source, copied, new_id, mapping))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_fences_frontmatter_and_other_sections() {
        let text = "---\nfoo: '- [ ] yaml'\n---\n## Tasks\n- [ ] real\n```md\n- [ ] example\n```\n## Journal\n- [ ] other\n";
        let tasks = parse(text, "Tasks");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "real");
    }
    #[test]
    fn duplicate_ids_are_repaired_and_normalizing_is_idempotent() {
        let text = "- [ ] a <!-- tray-tasks:one -->\n- [ ] b <!-- tray-tasks:one -->\n";
        let output = ensure_ids(text, "");
        let tasks = parse(&output, "");
        assert_ne!(tasks[0].id, tasks[1].id);
        assert_eq!(ensure_ids(&output, ""), output);
    }
    #[test]
    fn nested_move_preserves_prose_and_source_crlf() {
        let text = "- [ ] parent <!-- tray-tasks:one -->\r\n  details\r\n  - [ ] child <!-- tray-tasks:two -->\r\n\r\n## Other\r\nuntouched\r\n";
        let (source, copied, new_id, mapping) = moved(text, "one", "2026-10-12", false).unwrap();
        assert!(source.contains("- [>] parent"));
        assert!(source.contains("  - [>] child"));
        assert!(source.ends_with("## Other\r\nuntouched\r\n"));
        assert!(copied.contains("details"));
        assert_ne!(new_id, "one");
        assert_eq!(mapping.len(), 2);
        assert!(copied.contains("- [ ] child"));
    }
    #[test]
    fn completion_and_rename_do_not_touch_unrelated_content() {
        let text = "# 日誌\r\n- [ ] 할 일 <!-- tray-tasks:one -->\r\n끝\r\n";
        let done = set_done(text, "one", true).unwrap();
        let renamed = rename(&done, "one", "새 이름").unwrap();
        assert_eq!(
            renamed,
            "# 日誌\r\n- [x] 새 이름 <!-- tray-tasks:one -->\r\n끝\r\n"
        );
    }
    #[test]
    fn append_creates_missing_section_without_overwriting() {
        let output = append("# Journal\nhello", "- [ ] a", "Tasks");
        assert_eq!(output, "# Journal\nhello\n\n## Tasks\n- [ ] a\n");
    }
    #[test]
    fn remove_parent_removes_children_but_not_following_heading() {
        let text =
            "- [ ] a <!-- tray-tasks:one -->\n  - [ ] b <!-- tray-tasks:two -->\n# Keep\nbody";
        assert_eq!(remove(text, "one").unwrap(), "# Keep\nbody");
    }
}
