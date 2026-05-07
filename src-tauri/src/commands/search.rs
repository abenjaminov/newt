use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::{WalkBuilder, WalkState};
use parking_lot::Mutex;
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

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
    pub cancelled: bool,
}

const HARD_HIT_CAP: usize = 2000;
const HARD_FILE_CAP: u64 = 20_000;
const PREVIEW_MAX: usize = 400;

// Generation counter — every new search increments it. Workers periodically
// compare their own generation to the latest; if behind, they bail out.
// Avoids stacking N pending searches on the blocking pool when the user types.
static LATEST_GEN: AtomicU64 = AtomicU64::new(0);

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
            cancelled: false,
        });
    }
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {}", root));
    }

    let my_gen = LATEST_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let cancelled = Arc::new(AtomicBool::new(false));

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

    let root_owned = root_path.to_path_buf();
    let mut walker = WalkBuilder::new(&root_owned);
    walker
        .hidden(!hidden)
        .git_ignore(gitignore)
        .git_global(gitignore)
        .git_exclude(gitignore)
        .ignore(gitignore)
        .parents(gitignore)
        .filter_entry(|e| e.file_name() != ".git");

    // Parallel walk + parallel matching. Each worker streams hits into a shared
    // buffer guarded by a single mutex; lock contention is fine because hits
    // are rare relative to scanned files.
    let hits = Arc::new(Mutex::new(Vec::<SearchHit>::with_capacity(256)));
    let files_scanned = Arc::new(AtomicU64::new(0));
    let truncated = Arc::new(AtomicBool::new(false));

    walker.build_parallel().run(|| {
        let matcher = matcher.clone();
        let hits = Arc::clone(&hits);
        let files_scanned = Arc::clone(&files_scanned);
        let truncated = Arc::clone(&truncated);
        let cancelled = Arc::clone(&cancelled);
        let root_owned = root_owned.clone();
        let mut searcher = SearcherBuilder::new().line_number(true).build();

        Box::new(move |entry| {
            // Cheap, lock-free cancellation check on every file.
            if cancelled.load(Ordering::Relaxed)
                || LATEST_GEN.load(Ordering::Relaxed) != my_gen
            {
                cancelled.store(true, Ordering::Relaxed);
                return WalkState::Quit;
            }
            if truncated.load(Ordering::Relaxed) {
                return WalkState::Quit;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };
            let ft = match entry.file_type() {
                Some(t) => t,
                None => return WalkState::Continue,
            };
            if !ft.is_file() {
                return WalkState::Continue;
            }

            let scanned = files_scanned.fetch_add(1, Ordering::Relaxed) + 1;
            if scanned > HARD_FILE_CAP {
                truncated.store(true, Ordering::Relaxed);
                return WalkState::Quit;
            }

            let path = entry.path();
            let rel = path
                .strip_prefix(&root_owned)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            let path_str = path.to_string_lossy().into_owned();

            let _ = searcher.search_path(
                &matcher,
                path,
                UTF8(|line_no, line| {
                    // Cancellation also checked per-line so huge files don't
                    // delay shutdown.
                    if cancelled.load(Ordering::Relaxed) {
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
                        let hit = SearchHit {
                            path: path_str.clone(),
                            rel: rel.clone(),
                            line: line_no,
                            col_start: m.start() as u32,
                            col_end: m.end() as u32,
                            preview,
                        };
                        let mut lk = hits.lock();
                        if lk.len() >= HARD_HIT_CAP {
                            truncated.store(true, Ordering::Relaxed);
                            return Ok(false);
                        }
                        lk.push(hit);
                    }
                    Ok(true)
                }),
            );

            WalkState::Continue
        })
    });

    let was_cancelled = cancelled.load(Ordering::Relaxed)
        || LATEST_GEN.load(Ordering::Relaxed) != my_gen;
    let hits = Arc::try_unwrap(hits)
        .map(|m| m.into_inner())
        .unwrap_or_else(|arc| arc.lock().clone());

    Ok(SearchResult {
        hits,
        truncated: truncated.load(Ordering::Relaxed),
        files_scanned: files_scanned.load(Ordering::Relaxed),
        cancelled: was_cancelled,
    })
}
