use std::path::PathBuf;

use crate::fileinfo::FileInfo;
use crate::pipeline::DuplicateGroup;
use crate::resolver::{select_keeper, KeepStrategy};

pub const STRATEGIES: [KeepStrategy; 6] = [
    KeepStrategy::Newest,
    KeepStrategy::Oldest,
    KeepStrategy::First,
    KeepStrategy::Last,
    KeepStrategy::Shortest,
    KeepStrategy::Shallowest,
];

pub fn strategy_label(strategy: KeepStrategy) -> &'static str {
    match strategy {
        KeepStrategy::Newest => "newest",
        KeepStrategy::Oldest => "oldest",
        KeepStrategy::First => "first",
        KeepStrategy::Last => "last",
        KeepStrategy::Shortest => "shortest",
        KeepStrategy::Shallowest => "shallowest",
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Groups,
    Files,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StrategyScope {
    CurrentGroup,
    AllGroups,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Popup {
    None,
    Strategy { scope: StrategyScope },
    ConfirmDelete,
}

pub enum Key {
    Up,
    Down,
    Tab,
    Space,
    Enter,
    Esc,
    Char(char),
}

pub enum Outcome {
    Continue,
    Quit,
    Delete,
    Open(PathBuf),
}

pub struct Group {
    pub files: Vec<FileInfo>,
    pub marked: Vec<bool>,
}

impl Group {
    pub fn size(&self) -> u64 {
        self.files.first().map(|f| f.size).unwrap_or(0)
    }
}

pub struct App {
    pub groups: Vec<Group>,
    pub group_cursor: usize,
    pub file_cursor: usize,
    pub focus: Focus,
    pub popup: Popup,
    pub strategy_cursor: usize,
    pub status: Option<String>,
    pub should_quit: bool,
}

fn clamp_add(cur: usize, delta: isize, max: usize) -> usize {
    (cur as isize + delta).clamp(0, max as isize) as usize
}

impl App {
    pub fn from_groups(groups: Vec<DuplicateGroup>) -> Self {
        let groups = groups
            .into_iter()
            .map(|g| Group {
                marked: vec![false; g.files.len()],
                files: g.files,
            })
            .collect();

        Self {
            groups,
            group_cursor: 0,
            file_cursor: 0,
            focus: Focus::Files,
            popup: Popup::None,
            strategy_cursor: 0,
            status: None,
            should_quit: false,
        }
    }

    pub fn move_group(&mut self, delta: isize) {
        if self.groups.is_empty() {
            return;
        }
        self.group_cursor = clamp_add(self.group_cursor, delta, self.groups.len() - 1);
        let len = self.groups[self.group_cursor].files.len();
        self.file_cursor = self.file_cursor.min(len.saturating_sub(1));
    }

    pub fn move_file(&mut self, delta: isize) {
        if let Some(g) = self.groups.get(self.group_cursor) {
            self.file_cursor = clamp_add(self.file_cursor, delta, g.files.len().saturating_sub(1));
        }
    }

    pub fn toggle_mark(&mut self) {
        let fc = self.file_cursor;
        if let Some(g) = self.groups.get_mut(self.group_cursor) {
            if fc >= g.marked.len() {
                return;
            }
            if !g.marked[fc] && g.marked.iter().filter(|m| !**m).count() <= 1 {
                return;
            }
            g.marked[fc] = !g.marked[fc];
        }
    }

    pub fn apply_strategy(&mut self, strategy: KeepStrategy, scope: StrategyScope) {
        let indices: Vec<usize> = match scope {
            StrategyScope::CurrentGroup => vec![self.group_cursor],
            StrategyScope::AllGroups => (0..self.groups.len()).collect(),
        };
        for gi in indices {
            if let Some(g) = self.groups.get_mut(gi) {
                if g.files.len() < 2 {
                    continue;
                }
                let keeper = select_keeper(&g.files, strategy);
                for i in 0..g.marked.len() {
                    g.marked[i] = i != keeper;
                }
            }
        }
    }

    pub fn marked_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|g| g.marked.iter())
            .filter(|m| **m)
            .count()
    }

    pub fn reclaimable_bytes(&self) -> u64 {
        self.groups
            .iter()
            .flat_map(|g| g.files.iter().zip(&g.marked))
            .filter(|(_, m)| **m)
            .map(|(f, _)| f.size)
            .sum()
    }

    pub fn marked_paths(&self) -> Vec<PathBuf> {
        self.groups
            .iter()
            .flat_map(|g| g.files.iter().zip(&g.marked))
            .filter(|(_, m)| **m)
            .map(|(f, _)| f.path.to_path_buf())
            .collect()
    }

    pub fn apply_deletion(&mut self, deleted: &[PathBuf]) {
        use std::collections::HashSet;
        let removed: HashSet<&std::path::Path> = deleted.iter().map(|p| p.as_path()).collect();

        for g in &mut self.groups {
            let mut files = Vec::new();
            let mut marked = Vec::new();
            for (i, f) in g.files.iter().enumerate() {
                if !removed.contains(&*f.path) {
                    files.push(f.clone());
                    marked.push(g.marked[i]);
                }
            }
            g.files = files;
            g.marked = marked;
        }

        self.groups.retain(|g| g.files.len() >= 2);

        if self.groups.is_empty() {
            self.should_quit = true;
            self.group_cursor = 0;
            self.file_cursor = 0;
            return;
        }

        self.group_cursor = self.group_cursor.min(self.groups.len() - 1);
        let len = self.groups[self.group_cursor].files.len();
        self.file_cursor = self.file_cursor.min(len.saturating_sub(1));
    }

    pub fn handle_key(&mut self, key: Key) -> Outcome {
        match self.popup {
            Popup::Strategy { scope } => self.handle_strategy_key(key, scope),
            Popup::ConfirmDelete => self.handle_confirm_key(key),
            Popup::None => self.handle_main_key(key),
        }
    }

    fn handle_main_key(&mut self, key: Key) -> Outcome {
        self.status = None;
        match key {
            Key::Char('q') | Key::Esc => {
                self.should_quit = true;
                Outcome::Quit
            }
            Key::Tab => {
                self.focus = match self.focus {
                    Focus::Groups => Focus::Files,
                    Focus::Files => Focus::Groups,
                };
                Outcome::Continue
            }
            Key::Up | Key::Char('k') => {
                self.move_in_focus(-1);
                Outcome::Continue
            }
            Key::Down | Key::Char('j') => {
                self.move_in_focus(1);
                Outcome::Continue
            }
            Key::Space => {
                if self.focus == Focus::Files {
                    self.toggle_mark();
                }
                Outcome::Continue
            }
            Key::Char('s') => {
                self.open_strategy(StrategyScope::CurrentGroup);
                Outcome::Continue
            }
            Key::Char('S') => {
                self.open_strategy(StrategyScope::AllGroups);
                Outcome::Continue
            }
            Key::Char('o') => self.open_current(),
            Key::Char('d') => {
                if self.marked_count() > 0 {
                    self.popup = Popup::ConfirmDelete;
                } else {
                    self.status = Some("nothing marked".to_string());
                }
                Outcome::Continue
            }
            _ => Outcome::Continue,
        }
    }

    fn handle_strategy_key(&mut self, key: Key, scope: StrategyScope) -> Outcome {
        match key {
            Key::Up | Key::Char('k') => {
                self.strategy_cursor = self.strategy_cursor.saturating_sub(1);
            }
            Key::Down | Key::Char('j') => {
                self.strategy_cursor = (self.strategy_cursor + 1).min(STRATEGIES.len() - 1);
            }
            Key::Enter => {
                let strategy = STRATEGIES[self.strategy_cursor];
                self.apply_strategy(strategy, scope);
                self.popup = Popup::None;
            }
            Key::Esc => self.popup = Popup::None,
            _ => {}
        }
        Outcome::Continue
    }

    fn handle_confirm_key(&mut self, key: Key) -> Outcome {
        match key {
            Key::Char('y') => {
                self.popup = Popup::None;
                Outcome::Delete
            }
            Key::Char('n') | Key::Esc => {
                self.popup = Popup::None;
                Outcome::Continue
            }
            _ => Outcome::Continue,
        }
    }

    fn move_in_focus(&mut self, delta: isize) {
        match self.focus {
            Focus::Groups => self.move_group(delta),
            Focus::Files => self.move_file(delta),
        }
    }

    fn open_strategy(&mut self, scope: StrategyScope) {
        self.strategy_cursor = 0;
        self.popup = Popup::Strategy { scope };
    }

    fn open_current(&self) -> Outcome {
        match self.groups.get(self.group_cursor).and_then(|g| g.files.get(self.file_cursor)) {
            Some(f) => Outcome::Open(f.path.to_path_buf()),
            None => Outcome::Continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileinfo::FileInfo;
    use crate::pipeline::DuplicateGroup;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    fn file(path: &str, size: u64, mtime_secs: u64) -> FileInfo {
        FileInfo {
            path: PathBuf::from(path).into_boxed_path(),
            size,
            modified: UNIX_EPOCH + Duration::from_secs(mtime_secs),
        }
    }

    fn group(files: Vec<FileInfo>) -> DuplicateGroup {
        DuplicateGroup { hash: 0, files }
    }

    fn sample() -> App {
        App::from_groups(vec![
            group(vec![file("/a/x", 10, 100), file("/a/y", 10, 200)]),
            group(vec![file("/b/p", 5, 50), file("/b/q", 5, 60), file("/b/r", 5, 70)]),
        ])
    }

    #[test]
    fn move_group_clamps_at_bounds() {
        let mut app = sample();
        app.move_group(-1);
        assert_eq!(app.group_cursor, 0);
        app.move_group(1);
        assert_eq!(app.group_cursor, 1);
        app.move_group(5);
        assert_eq!(app.group_cursor, 1);
    }

    #[test]
    fn move_group_reclamps_file_cursor() {
        let mut app = sample();
        app.group_cursor = 1;
        app.file_cursor = 2;
        app.move_group(-1);
        assert_eq!(app.group_cursor, 0);
        assert_eq!(app.file_cursor, 1);
    }

    #[test]
    fn move_file_clamps() {
        let mut app = sample();
        app.move_file(-1);
        assert_eq!(app.file_cursor, 0);
        app.move_file(10);
        assert_eq!(app.file_cursor, 1);
    }

    #[test]
    fn toggle_mark_blocks_marking_the_last_survivor() {
        let mut app = sample();
        app.group_cursor = 0;
        app.file_cursor = 0;
        app.toggle_mark();
        assert!(app.groups[0].marked[0]);
        app.file_cursor = 1;
        app.toggle_mark();
        assert!(!app.groups[0].marked[1]);
    }

    #[test]
    fn toggle_mark_allows_unmark() {
        let mut app = sample();
        app.file_cursor = 0;
        app.toggle_mark();
        app.toggle_mark();
        assert!(!app.groups[0].marked[0]);
    }

    #[test]
    fn apply_strategy_current_group_marks_all_but_keeper() {
        let mut app = sample();
        app.group_cursor = 0;
        app.apply_strategy(KeepStrategy::Newest, StrategyScope::CurrentGroup);
        assert_eq!(app.groups[0].marked, vec![true, false]);
        assert_eq!(app.groups[1].marked, vec![false, false, false]);
    }

    #[test]
    fn apply_strategy_all_groups() {
        let mut app = sample();
        app.apply_strategy(KeepStrategy::Oldest, StrategyScope::AllGroups);
        assert_eq!(app.groups[0].marked, vec![false, true]);
        assert_eq!(app.groups[1].marked, vec![false, true, true]);
    }

    #[test]
    fn totals_sum_marked_only() {
        let mut app = sample();
        app.apply_strategy(KeepStrategy::Newest, StrategyScope::AllGroups);
        assert_eq!(app.marked_count(), 3);
        assert_eq!(app.reclaimable_bytes(), 20);
    }

    #[test]
    fn marked_paths_lists_marked_files() {
        let mut app = sample();
        app.group_cursor = 0;
        app.file_cursor = 0;
        app.toggle_mark();
        assert_eq!(app.marked_paths(), vec![PathBuf::from("/a/x")]);
    }

    #[test]
    fn apply_deletion_removes_files_and_drops_small_groups() {
        let mut app = sample();
        app.apply_deletion(&[PathBuf::from("/a/x")]);
        assert_eq!(app.groups.len(), 1);
        assert_eq!(app.groups[0].files.len(), 3);
    }

    #[test]
    fn apply_deletion_emptying_everything_sets_quit() {
        let mut app = App::from_groups(vec![group(vec![file("/a/x", 1, 1), file("/a/y", 1, 2)])]);
        app.apply_deletion(&[PathBuf::from("/a/x")]);
        assert!(app.groups.is_empty());
        assert!(app.should_quit);
    }

    #[test]
    fn handle_key_quits_on_q() {
        let mut app = sample();
        assert!(matches!(app.handle_key(Key::Char('q')), Outcome::Quit));
        assert!(app.should_quit);
    }

    #[test]
    fn handle_key_space_marks_in_files_focus() {
        let mut app = sample();
        app.focus = Focus::Files;
        app.handle_key(Key::Space);
        assert!(app.groups[0].marked[0]);
    }

    #[test]
    fn handle_key_strategy_popup_apply_flow() {
        let mut app = sample();
        app.handle_key(Key::Char('S'));
        assert!(matches!(app.popup, Popup::Strategy { .. }));
        app.handle_key(Key::Enter);
        assert!(matches!(app.popup, Popup::None));
        assert_eq!(app.groups[0].marked, vec![true, false]);
    }

    #[test]
    fn handle_key_delete_requires_marks_then_confirms() {
        let mut app = sample();
        app.handle_key(Key::Char('d'));
        assert!(matches!(app.popup, Popup::None));
        app.focus = Focus::Files;
        app.handle_key(Key::Space);
        app.handle_key(Key::Char('d'));
        assert!(matches!(app.popup, Popup::ConfirmDelete));
        assert!(matches!(app.handle_key(Key::Char('y')), Outcome::Delete));
        assert!(matches!(app.popup, Popup::None));
    }

    #[test]
    fn handle_key_open_returns_path() {
        let mut app = sample();
        app.focus = Focus::Files;
        match app.handle_key(Key::Char('o')) {
            Outcome::Open(p) => assert_eq!(p, PathBuf::from("/a/x")),
            _ => panic!("expected Open"),
        }
    }
}
