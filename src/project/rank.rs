use crate::project::model::Project;

/// Sort projects for the picker: favorite, last launch, count, name.
pub fn rank_projects(mut projects: Vec<Project>) -> Vec<Project> {
    projects.sort_by(|a, b| {
        b.is_favorite
            .cmp(&a.is_favorite)
            .then_with(|| b.last_launched_at.cmp(&a.last_launched_at))
            .then_with(|| b.launch_count.cmp(&a.launch_count))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.path.cmp(&b.path))
    });
    projects
}

/// Deduplicate by normalized path string, keeping the richer record.
pub fn dedup_projects(projects: Vec<Project>) -> Vec<Project> {
    use std::collections::HashMap;
    let mut map: HashMap<String, Project> = HashMap::new();
    for p in projects {
        let key = p.path.to_string_lossy().to_string().to_lowercase();
        map.entry(key)
            .and_modify(|existing| {
                if p.launch_count > existing.launch_count
                    || p.last_launched_at > existing.last_launched_at
                {
                    *existing = p.clone();
                }
            })
            .or_insert(p);
    }
    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::path::PathBuf;

    fn proj(name: &str, fav: bool, launches: i64, last: Option<i64>) -> Project {
        Project {
            id: name.to_string(),
            path: PathBuf::from(format!("D:/{name}")),
            name: name.to_string(),
            is_favorite: fav,
            last_launched_at: last.map(|t| Utc.timestamp_opt(t, 0).unwrap()),
            launch_count: launches,
            last_git_activity_at: None,
            last_provider: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn favorite_first() {
        let ranked = rank_projects(vec![
            proj("a", false, 10, Some(100)),
            proj("b", true, 0, None),
        ]);
        assert_eq!(ranked[0].name, "b");
    }

    #[test]
    fn recent_launch_before_count() {
        let ranked = rank_projects(vec![
            proj("old", false, 99, Some(10)),
            proj("new", false, 1, Some(1000)),
        ]);
        assert_eq!(ranked[0].name, "new");
    }

    #[test]
    fn dedup_keeps_higher_count() {
        let mut a = proj("x", false, 1, None);
        a.path = PathBuf::from(r"D:\Code\App");
        let mut b = proj("x", false, 5, Some(50));
        b.path = PathBuf::from(r"D:\Code\App");
        let d = dedup_projects(vec![a, b]);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].launch_count, 5);
    }
}
