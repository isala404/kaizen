use std::collections::HashMap;

use crate::forge::{Task, TaskStatus};

pub const POSITION_STEP: i32 = 10_000;

pub fn trailing_position(tasks: &[Task]) -> i32 {
    tasks
        .last()
        .map(|task| task.position + POSITION_STEP)
        .unwrap_or(POSITION_STEP)
}

pub fn insertion_position(previous_position: Option<i32>, next_position: i32) -> i32 {
    previous_position.map_or(next_position - POSITION_STEP, |previous| {
        (previous + next_position) / 2
    })
}

pub fn max_position_for_statuses(tasks: &[Task], statuses: &[TaskStatus]) -> i32 {
    tasks
        .iter()
        .filter(|task| statuses.iter().any(|status| task.status == *status))
        .map(|task| task.position)
        .max()
        .unwrap_or_default()
}

pub fn appended_position(tasks: &[Task], statuses: &[TaskStatus]) -> i32 {
    max_position_for_statuses(tasks, statuses) + POSITION_STEP
}

pub fn drop_status(task_status: &TaskStatus) -> &'static str {
    match task_status {
        TaskStatus::Inbox => "inbox",
        TaskStatus::UpNext => "up_next",
        TaskStatus::Paused => "paused",
        TaskStatus::Done => "done",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Focused => "focused",
        TaskStatus::Archived => "archived",
    }
}

pub fn parse_drop_status(status: &str) -> Option<TaskStatus> {
    match status {
        "inbox" => Some(TaskStatus::Inbox),
        "up_next" => Some(TaskStatus::UpNext),
        "paused" => Some(TaskStatus::Paused),
        "done" => Some(TaskStatus::Done),
        "in_progress" => Some(TaskStatus::InProgress),
        "focused" => Some(TaskStatus::Focused),
        "archived" => Some(TaskStatus::Archived),
        _ => None,
    }
}

pub fn dock_target_status(task_status: Option<&TaskStatus>) -> TaskStatus {
    match task_status {
        Some(TaskStatus::Focused) => TaskStatus::Focused,
        Some(TaskStatus::InProgress) => TaskStatus::InProgress,
        _ => TaskStatus::InProgress,
    }
}

pub fn apply_pending_moves(
    mut tasks: Vec<Task>,
    pending_moves: &HashMap<String, (TaskStatus, i32)>,
) -> Vec<Task> {
    for task in &mut tasks {
        if let Some((status, position)) = pending_moves.get(&task.id) {
            task.status = status.clone();
            task.position = *position;
        }
    }

    tasks
}

pub fn dock_tasks(tasks: &[Task]) -> Vec<Task> {
    let mut dock_tasks = tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Focused || task.status == TaskStatus::InProgress)
        .cloned()
        .collect::<Vec<_>>();
    dock_tasks.sort_by_key(|task| task.position);
    dock_tasks
}
