use crate::error::{Result, SdlcError};
use crate::types::TaskStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub blocker: Option<String>,
    pub depends_on: Vec<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            blocker: None,
            depends_on: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Task list operations (operate on a mutable Vec<Task>)
// ---------------------------------------------------------------------------

pub fn add_task(tasks: &mut Vec<Task>, title: impl Into<String>) -> String {
    let id = format!("T{}", tasks.len() + 1);
    tasks.push(Task::new(id.clone(), title));
    id
}

pub fn start_task(tasks: &mut [Task], id: &str) -> Result<()> {
    let task = find_mut(tasks, id)?;
    task.status = TaskStatus::InProgress;
    task.started_at = Some(Utc::now());
    Ok(())
}

pub fn complete_task(tasks: &mut [Task], id: &str) -> Result<()> {
    let task = find_mut(tasks, id)?;
    task.status = TaskStatus::Completed;
    task.completed_at = Some(Utc::now());
    Ok(())
}

pub fn block_task(tasks: &mut [Task], id: &str, reason: impl Into<String>) -> Result<()> {
    let task = find_mut(tasks, id)?;
    task.status = TaskStatus::Blocked;
    task.blocker = Some(reason.into());
    Ok(())
}

/// Return the next pending or in-progress task that has no incomplete dependencies.
pub fn next_task(tasks: &[Task]) -> Option<&Task> {
    let completed_ids: std::collections::HashSet<&str> = tasks
        .iter()
        .filter(|t| matches!(t.status, TaskStatus::Completed))
        .map(|t| t.id.as_str())
        .collect();

    tasks.iter().find(|t| {
        matches!(t.status, TaskStatus::Pending | TaskStatus::InProgress)
            && t.depends_on.iter().all(|dep| completed_ids.contains(dep.as_str()))
    })
}

/// Human-readable summary: "3/5 tasks complete, 1 in progress, 1 blocked"
pub fn summarize(tasks: &[Task]) -> String {
    let total = tasks.len();
    let done = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
    let in_progress = tasks.iter().filter(|t| matches!(t.status, TaskStatus::InProgress)).count();
    let blocked = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Blocked)).count();
    format!("{done}/{total} completed, {in_progress} in progress, {blocked} blocked")
}

fn find_mut<'a>(tasks: &'a mut [Task], id: &str) -> Result<&'a mut Task> {
    tasks
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or_else(|| SdlcError::TaskNotFound(id.to_string()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_lifecycle() {
        let mut tasks: Vec<Task> = Vec::new();
        let id = add_task(&mut tasks, "Write tests");
        assert_eq!(tasks[0].status, TaskStatus::Pending);

        start_task(&mut tasks, &id).unwrap();
        assert_eq!(tasks[0].status, TaskStatus::InProgress);

        complete_task(&mut tasks, &id).unwrap();
        assert_eq!(tasks[0].status, TaskStatus::Completed);
        assert!(tasks[0].completed_at.is_some());
    }

    #[test]
    fn block_task_sets_reason() {
        let mut tasks: Vec<Task> = Vec::new();
        let id = add_task(&mut tasks, "Deploy");
        block_task(&mut tasks, &id, "waiting for infra").unwrap();
        assert_eq!(tasks[0].status, TaskStatus::Blocked);
        assert_eq!(tasks[0].blocker.as_deref(), Some("waiting for infra"));
    }

    #[test]
    fn task_not_found() {
        let mut tasks: Vec<Task> = Vec::new();
        assert!(start_task(&mut tasks, "T99").is_err());
    }

    #[test]
    fn next_task_respects_deps() {
        let mut tasks: Vec<Task> = Vec::new();
        let t1 = add_task(&mut tasks, "First");
        let t2 = add_task(&mut tasks, "Second");
        tasks[1].depends_on.push(t1.clone());

        let next = next_task(&tasks).unwrap();
        assert_eq!(next.id, t1);

        complete_task(&mut tasks, &t1).unwrap();
        let next = next_task(&tasks).unwrap();
        assert_eq!(next.id, t2);
    }
}
