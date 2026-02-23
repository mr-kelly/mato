use super::app::{App, Focus, JUMP_LABELS};

impl App {
    pub(super) fn jump_key_reserved_for_focus(&self, c: char) -> bool {
        let key = c.to_ascii_lowercase();
        match self.focus {
            Focus::Content => matches!(key, 'c' | 'r' | 'q'),
            Focus::Sidebar | Focus::Topbar => matches!(key, 'r' | 'q'),
        }
    }

    pub fn jump_labels(&self) -> Vec<char> {
        JUMP_LABELS
            .chars()
            .filter(|c| !self.jump_key_reserved_for_focus(*c))
            .collect()
    }

    pub(super) fn visible_desk_indices(&self) -> Vec<usize> {
        let desks_len = self.offices[self.current_office].desks.len();
        if desks_len == 0 {
            return vec![];
        }
        let visible_rows = self.sidebar_list_area.height.saturating_sub(2) as usize;
        if visible_rows == 0 {
            return (0..desks_len).collect();
        }
        let start = self.list_state.offset().min(desks_len.saturating_sub(1));
        let end = (start + visible_rows).min(desks_len);
        (start..end).collect()
    }

    pub fn jump_targets(&self) -> Vec<(char, usize, usize)> {
        let max_labels = self.jump_labels().len();
        let mut targets: Vec<(char, usize, usize)> = Vec::new();
        let desk_indices = self.visible_desk_indices();
        let tab_indices: Vec<usize> = self.tab_area_tab_indices.clone();
        let task_idx = self.selected();

        let push_tab = |targets: &mut Vec<(char, usize, usize)>, t: usize| {
            targets.push(('b', task_idx, t));
        };
        let push_desk = |targets: &mut Vec<(char, usize, usize)>, d: usize| {
            targets.push(('t', d, 0));
        };

        match self.focus {
            Focus::Topbar => {
                for &t in &tab_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_tab(&mut targets, t);
                }
                for &d in &desk_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_desk(&mut targets, d);
                }
            }
            Focus::Sidebar => {
                for &d in &desk_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_desk(&mut targets, d);
                }
                for &t in &tab_indices {
                    if targets.len() >= max_labels {
                        break;
                    }
                    push_tab(&mut targets, t);
                }
            }
            Focus::Content => {
                // Balanced allocation: interleave tab/desk as much as possible.
                let (mut ti, mut di) = (0usize, 0usize);
                let mut take_tab = true;
                while targets.len() < max_labels
                    && (ti < tab_indices.len() || di < desk_indices.len())
                {
                    if take_tab {
                        if ti < tab_indices.len() {
                            push_tab(&mut targets, tab_indices[ti]);
                            ti += 1;
                        } else if di < desk_indices.len() {
                            push_desk(&mut targets, desk_indices[di]);
                            di += 1;
                        }
                    } else if di < desk_indices.len() {
                        push_desk(&mut targets, desk_indices[di]);
                        di += 1;
                    } else if ti < tab_indices.len() {
                        push_tab(&mut targets, tab_indices[ti]);
                        ti += 1;
                    }
                    take_tab = !take_tab;
                }
            }
        }

        targets
    }
}
