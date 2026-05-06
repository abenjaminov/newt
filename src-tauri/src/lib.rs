mod commands;

use commands::{
    file_index, fs_ops, git_ops, logo, processes, pty, shells, watcher, worktree,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(watcher::WatcherState::default())
        .invoke_handler(tauri::generate_handler![
            fs_ops::read_dir,
            fs_ops::read_file,
            fs_ops::write_file,
            fs_ops::read_image,
            fs_ops::rename_path,
            fs_ops::delete_path,
            fs_ops::create_file,
            fs_ops::create_dir,
            fs_ops::move_into,
            watcher::start_watch,
            watcher::stop_watch,
            pty::spawn_pty,
            pty::write_pty,
            pty::resize_pty,
            pty::kill_pty,
            pty::list_ptys,
            processes::list_descendants,
            processes::kill_process,
            processes::self_stats,
            processes::app_pid,
            shells::list_shells,
            git_ops::git_status,
            git_ops::git_ignored_paths,
            git_ops::git_file_at_head,
            git_ops::git_diff,
            git_ops::git_stage,
            git_ops::git_unstage,
            git_ops::git_discard,
            git_ops::git_commit,
            git_ops::git_branches,
            git_ops::git_checkout,
            git_ops::git_clear_index_lock,
            worktree::list_worktrees,
            worktree::add_worktree,
            worktree::remove_worktree,
            worktree::path_exists,
            logo::find_logo,
            file_index::list_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
