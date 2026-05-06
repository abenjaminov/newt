use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct ProcInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub cmd: Vec<String>,
    pub cpu: f32,
    pub memory: u64, // RSS bytes
    pub run_time_secs: u64,
}

#[derive(Serialize, Clone)]
pub struct ProcTree {
    pub roots: Vec<ProcInfo>,
    pub all: Vec<ProcInfo>, // flat list (the frontend builds the tree by parent_pid)
}

/// Returns the descendant tree under each provided root PID.
/// Roots that are no longer alive are skipped.
#[tauri::command]
pub fn list_descendants(roots: Vec<u32>) -> ProcTree {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_processes(ProcessRefreshKind::new().with_cpu().with_memory()),
    );
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    // sysinfo's CPU% needs two ticks. Sleep briefly and refresh once more.
    std::thread::sleep(std::time::Duration::from_millis(150));
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    let mut by_parent: HashMap<u32, Vec<u32>> = HashMap::new();
    for (pid, p) in sys.processes() {
        if let Some(parent) = p.parent() {
            by_parent.entry(parent.as_u32()).or_default().push(pid.as_u32());
        }
    }

    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut all: Vec<ProcInfo> = Vec::new();
    let mut roots_out: Vec<ProcInfo> = Vec::new();

    for root_pid in roots {
        if !visited.insert(root_pid) {
            continue;
        }
        // Root proc info — skip if not alive.
        if let Some(p) = sys.process(Pid::from_u32(root_pid)) {
            roots_out.push(make_info(root_pid, p));
            all.push(make_info(root_pid, p));
        } else {
            continue;
        }
        // BFS to collect descendants.
        let mut stack: Vec<u32> = by_parent.get(&root_pid).cloned().unwrap_or_default();
        while let Some(pid) = stack.pop() {
            if !visited.insert(pid) {
                continue;
            }
            if let Some(p) = sys.process(Pid::from_u32(pid)) {
                all.push(make_info(pid, p));
            }
            if let Some(kids) = by_parent.get(&pid) {
                stack.extend(kids.iter().copied());
            }
        }
    }

    ProcTree { roots: roots_out, all }
}

fn make_info(pid: u32, p: &sysinfo::Process) -> ProcInfo {
    let cmd: Vec<String> = p
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    ProcInfo {
        pid,
        parent_pid: p.parent().map(|x| x.as_u32()),
        name: p.name().to_string_lossy().into_owned(),
        cmd,
        cpu: p.cpu_usage(),
        memory: p.memory(),
        run_time_secs: p.run_time(),
    }
}

#[derive(Serialize, Clone)]
pub struct SelfStats {
    /// Normalized 0–100% CPU across all cores (Task-Manager style), summed over
    /// newt.exe and its descendants.
    pub cpu: f32,
    pub memory: u64,
    pub process_count: u32,
    pub root_pid: u32,
    pub num_cpus: u32,
}

/// Aggregate CPU% and memory across newt.exe and all of its descendants.
/// Cheap enough to call on a 1.5s poll from the footer.
#[tauri::command]
pub fn self_stats() -> SelfStats {
    let root_pid = std::process::id();
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_processes(ProcessRefreshKind::new().with_cpu().with_memory()),
    );
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );
    // sysinfo computes CPU% as the delta between two refreshes. A longer window
    // means less noise / less likely to catch a brief tick at 100% of a core.
    std::thread::sleep(std::time::Duration::from_millis(250));
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    let num_cpus = sys.cpus().len().max(1) as f32;

    let mut by_parent: HashMap<u32, Vec<u32>> = HashMap::new();
    for (pid, p) in sys.processes() {
        if let Some(parent) = p.parent() {
            by_parent.entry(parent.as_u32()).or_default().push(pid.as_u32());
        }
    }

    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut stack: Vec<u32> = vec![root_pid];
    let mut total_cpu_per_core = 0.0f32;
    let mut total_mem = 0u64;
    let mut count = 0u32;
    while let Some(p) = stack.pop() {
        if !visited.insert(p) {
            continue;
        }
        if let Some(proc) = sys.process(Pid::from_u32(p)) {
            // sysinfo returns CPU as "% of a single core" — a process pegging
            // one core on an 8-core machine reports 100%, which is misleading
            // when shown alongside 16 cores' worth of capacity.
            total_cpu_per_core += proc.cpu_usage();
            total_mem += proc.memory();
            count += 1;
        }
        if let Some(kids) = by_parent.get(&p) {
            stack.extend(kids.iter().copied());
        }
    }
    // Normalize: divide by core count, clamp to [0, 100] for display sanity.
    let cpu = (total_cpu_per_core / num_cpus).clamp(0.0, 100.0);
    SelfStats {
        cpu,
        memory: total_mem,
        process_count: count,
        root_pid,
        num_cpus: num_cpus as u32,
    }
}

#[tauri::command]
pub fn app_pid() -> u32 {
    std::process::id()
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::new(),
    );
    if let Some(p) = sys.process(Pid::from_u32(pid)) {
        if p.kill() {
            Ok(())
        } else {
            Err(format!("kill {} failed", pid))
        }
    } else {
        Err(format!("pid {} not found", pid))
    }
}
