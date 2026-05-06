use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::WalkBuilder;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct SearchHit {
    pub path: String,
    pub rel: String,
    pub line: u64,
    pub col_start: u32,
    pub col_end: u32,
    pub preview: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub truncated: bool,
    pub files_scanned: u64,
}

const HARD_HIT_CAP: usize = 5000;
const HARD_FILE_CAP: u64 = 20_000;
const PREVIEW_MAX: usize = 400;

#[tauri::command]
pub fn search_in_files(
    root: String,
    query: String,
    case_sensitive: Option<bool>,
    whole_word: Option<bool>,
    is_regex: Option<bool>,
    include_hidden: Option<bool>,
    respect_gitignore: Option<bool>,
) -> Result<SearchResult, String> {
    if query.is_empty() {
        return Ok(SearchResult {
            hits: vec![],
            truncated: false,
            files_scanned: 0,
        });
    }
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {}", root));
    }

    let cs = case_sensitive.unwrap_or(false);
    let ww = whole_word.unwrap_or(false);
    let regex = is_regex.unwrap_or(false);
    let hidden = include_hidden.unwrap_or(false);
    let gitignore = respect_gitignore.unwrap_or(true);

    let pattern = if regex {
        query.clone()
    } else {
        regex::escape(&query)
    };
    let pattern = if ww {
        format!(r"\b(?:{})\b", pattern)
    } else {
        pattern
    };

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!cs)
        .build(&pattern)
        .map_err(|e| format!("bad pattern: {}", e))?;

    let mut walker = WalkBuilder::new(root_path);
    walker
        .hidden(!hidden)
        .git_ignore(gitignore)
        .git_global(gitignore)
        .git_exclude(gitignore)
        .ignore(gitignore)
        .parents(gitignore)
        .filter_entry(|e| e.file_name() != ".git");

    let mut searcher = SearcherBuilder::new()
        .line_number(true)
        .build();

    let mut hits: Vec<SearchHit> = Vec::new();
    let mut files_scanned: u64 = 0;
    let mut truncated = false;

    for entry in walker.build() {
        if hits.len() >= HARD_HIT_CAP || files_scanned >= HARD_FILE_CAP {
            truncated = true;
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let ft = match entry.file_type() {
            Some(t) => t,
            None => continue,
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.path();
        files_scanned += 1;
        let rel = path
            .strip_prefix(root_path)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let path_str = path.to_string_lossy().into_owned();

        let _ = searcher.search_path(
            &matcher,
            path,
            UTF8(|line_no, line| {
                if hits.len() >= HARD_HIT_CAP {
                    truncated = true;
                    return Ok(false);
                }
                if let Ok(Some(m)) = matcher.find(line.as_bytes()) {
                    let preview: String = if line.len() > PREVIEW_MAX {
                        let mut s = line[..PREVIEW_MAX.min(line.len())].to_string();
                        s.push('…');
                        s
                    } else {
                        line.trim_end_matches(['\r', '\n']).to_string()
                    };
                    hits.push(SearchHit {
                        path: path_str.clone(),
                        rel: rel.clone(),
                        line: line_no,
                        col_start: m.start() as u32,
                        col_end: m.end() as u32,
                        preview,
                    });
                }
                Ok(true)
            }),
        );
    }

    Ok(SearchResult {
        hits,
        truncated,
        files_scanned,
    })
}
