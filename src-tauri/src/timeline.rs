//! Global undo timeline across all overlay windows.
//!
//! The backend stores lightweight op records only (`{op_id, owner, kind}` — no
//! stroke data). Invariant: the globally-latest active op always sits on its
//! owner window's local undo-stack top, so undo is "pop the newest record,
//! tell that owner to run its plain local `undo()`" — no targeted entry lookup.
//!
//! Global clear folds every active op into ONE clear op (they survive in its
//! `frozen` list) so a single Ctrl+Z restores every display at once.

/// Backend → owner window: replay an undo locally (payload: op id).
pub const UNDO_EVENT: &str = "timeline-undo";
/// Backend → owner window: replay a redo locally (payload: op id).
pub const REDO_EVENT: &str = "timeline-redo";
/// A new commit invalidated every window's redo branch.
pub const REDO_CLEARED_EVENT: &str = "timeline-redo-cleared";
/// Authoritative global undo availability (toolbar buttons) — sent after any
/// timeline mutation. Overlays' per-window stacks are NOT authoritative.
pub const STATE_EVENT: &str = "timeline-state";

/// Global timeline availability, serialized for the toolbar window.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineState {
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Push the current global availability to every webview (toolbar consumes).
pub fn broadcast_state(app: &tauri::AppHandle, timeline: &std::sync::Mutex<Timeline>) {
    let payload = {
        let t = crate::config::lock_or_recover(timeline);
        TimelineState {
            can_undo: t.can_undo(),
            can_redo: t.can_redo(),
        }
    };
    use tauri::Emitter;
    if let Err(e) = app.emit(STATE_EVENT, payload) {
        tracing::warn!("Failed to emit timeline-state: {}", e);
    }
}

/// One user-visible mutation, attributed to the overlay window it happened on.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineOp {
    pub op_id: u64,
    /// Owning overlay window label; empty for a global clear op (its undo
    /// is replayed on every overlay).
    pub owner: String,
    /// Diagnostics only — mirrors the frontend UndoEntry kinds.
    pub kind: String,
    /// Ops folded into this op by a global clear (kept in original order).
    pub frozen: Vec<TimelineOp>,
}

#[derive(Debug, Default)]
pub struct Timeline {
    active: Vec<TimelineOp>,
    undone: Vec<TimelineOp>,
    next_op_id: u64,
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new op: appended after every active op; any redoable history
    /// is invalidated (mirrors the frontend `redoStack.length = 0`).
    pub fn commit(&mut self, owner: &str, kind: &str) -> u64 {
        let op_id = self.next_op_id;
        self.next_op_id += 1;
        self.active.push(TimelineOp {
            op_id,
            owner: owner.to_string(),
            kind: kind.to_string(),
            frozen: Vec::new(),
        });
        self.undone.clear();
        op_id
    }

    /// Globally-latest op, moved to the redo (undone) list. Unfolding a clear
    /// op releases its frozen ops back onto the active list in original order.
    pub fn undo(&mut self) -> Option<TimelineOp> {
        let op = self.active.pop()?;
        if !op.frozen.is_empty() {
            self.active.extend(op.frozen.iter().cloned());
        }
        self.undone.push(op.clone());
        Some(op)
    }

    /// Most-recently undone op, moved back onto the active list. Redoing a
    /// clear op re-folds the currently active ops (necessarily the released
    /// frozen set) back into it.
    pub fn redo(&mut self) -> Option<TimelineOp> {
        let op = self.undone.pop()?;
        if !op.frozen.is_empty() {
            self.active.clear();
        }
        self.active.push(op.clone());
        Some(op)
    }

    /// Fold every active op into a single global clear op so one undo
    /// restores all displays. Idempotent when there is nothing to clear —
    /// still records the clear op if the timeline has undone history.
    pub fn begin_global_clear(&mut self) -> Option<TimelineOp> {
        if self.active.is_empty() && self.undone.is_empty() {
            return None;
        }
        let op_id = self.next_op_id;
        self.next_op_id += 1;
        let op = TimelineOp {
            op_id,
            owner: String::new(),
            kind: "clear".to_string(),
            frozen: std::mem::take(&mut self.active),
        };
        self.undone.clear();
        self.active.push(op.clone());
        Some(op)
    }

    /// Drop all history (non-undoable hard reset).
    pub fn reset(&mut self) {
        self.active.clear();
        self.undone.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty() && self.undone.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_appends_and_invalidates_redo() {
        let mut t = Timeline::new();
        assert_eq!(t.commit("overlay", "add"), 0);
        assert_eq!(t.commit("overlay-2", "add"), 1);
        assert!(t.can_undo());
        let undone = t.undo().unwrap();
        assert_eq!(undone.owner, "overlay-2");
        assert!(t.can_redo());
        // A new op after undo kills the redoable history.
        t.commit("overlay", "add");
        assert!(!t.can_redo());
    }

    #[test]
    fn undo_pops_globally_latest_owner() {
        let mut t = Timeline::new();
        t.commit("overlay", "add"); // screen 1 stroke
        t.commit("overlay-2", "add"); // screen 2 stroke
        t.commit("overlay", "add"); // screen 1 again
                                    // Latest op belongs to the static overlay window.
        assert_eq!(t.undo().unwrap().owner, "overlay");
        // Then the screen-2 stroke; one op remains active.
        assert_eq!(t.undo().unwrap().owner, "overlay-2");
        assert!(t.can_undo());
        assert_eq!(t.undo().unwrap().owner, "overlay");
        assert!(!t.can_undo());
        // Redo is LIFO: the most recently undone op (overlay's last) returns
        // first, then the screen-2 stroke, then the first stroke.
        assert_eq!(t.redo().unwrap().owner, "overlay");
        assert_eq!(t.redo().unwrap().owner, "overlay-2");
        assert_eq!(t.redo().unwrap().owner, "overlay");
        assert!(!t.can_redo());
    }

    #[test]
    fn global_clear_freezes_ops_and_single_undo_restores_order() {
        let mut t = Timeline::new();
        t.commit("overlay", "add");
        t.commit("overlay-2", "add");
        t.commit("overlay", "erase");
        let clear = t.begin_global_clear().unwrap();
        // Everything folded into one op owned by nobody (broadcast on undo).
        assert_eq!(clear.owner, "");
        assert_eq!(clear.kind, "clear");
        assert_eq!(clear.frozen.len(), 3);
        assert!(t.can_undo());
        assert!(!t.can_redo());

        // Undo of the clear restores the frozen ops in their original order.
        let popped = t.undo().unwrap();
        assert_eq!(popped.op_id, clear.op_id);
        assert!(!t.can_undo() || t.active.len() == 3);
        assert_eq!(
            t.active
                .iter()
                .map(|op| op.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["add", "add", "erase"]
        );
        assert_eq!(
            t.active
                .iter()
                .map(|op| op.owner.as_str())
                .collect::<Vec<_>>(),
            vec!["overlay", "overlay-2", "overlay"]
        );

        // Redo re-freezes them.
        let redone = t.redo().unwrap();
        assert_eq!(redone.kind, "clear");
        assert_eq!(redone.frozen.len(), 3);
        assert_eq!(t.active.len(), 1);
    }

    #[test]
    fn global_clear_after_undo_only_keeps_active_history() {
        let mut t = Timeline::new();
        t.commit("overlay", "add");
        t.undo();
        t.commit("overlay-2", "add");
        // undone was invalidated by the second commit; clear folds only the
        // active op.
        let clear = t.begin_global_clear().unwrap();
        assert_eq!(clear.frozen.len(), 1);
        assert_eq!(clear.frozen[0].owner, "overlay-2");
    }

    #[test]
    fn global_clear_on_empty_timeline_is_noop() {
        let mut t = Timeline::new();
        assert!(t.begin_global_clear().is_none());
        assert!(t.is_empty());
    }

    #[test]
    fn undo_on_empty_timeline_returns_none() {
        let mut t = Timeline::new();
        assert!(t.undo().is_none());
        assert!(t.redo().is_none());
    }

    #[test]
    fn reset_drops_everything() {
        let mut t = Timeline::new();
        t.commit("overlay", "add");
        t.undo();
        t.reset();
        assert!(t.is_empty());
        assert!(!t.can_undo());
        assert!(!t.can_redo());
    }
}
