#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tail {
    Pending,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    NextFirst,
    PreviousKept,
    PreviousLast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Next,
    Prev,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChapterPos {
    pub index: usize,
    pub page: u32,
    pub pages: u32,
    pub pages_known: bool,
    pub tail: Tail,
    pub first_ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmPos {
    pub index: usize,
    pub goal: Goal,
    pub first_ready: bool,
    pub tail: Tail,
    pub seeking: bool,
    pub at_last: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadState {
    pub stamped: bool,
    pub shown: ChapterPos,
    pub warm: Option<WarmPos>,
    pub queued: Option<Edge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadEvent {
    ShownLayout { ready: bool, tail: Tail, width_ok: bool },
    ShownBox { page: u32, pages: u32 },
    Edge(Edge),
    WarmLayout { ready: bool, tail: Tail },
    SeekDone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Stamp,
    Mount { index: usize },
    Navigate { index: usize },
    SeekLast,
    SwapForward,
    SwapBack,
    SyncAnchor,
}

impl ReadState {
    pub fn fresh() -> Self {
        ReadState {
            stamped: false,
            shown: ChapterPos {
                index: 0,
                page: 1,
                pages: 1,
                pages_known: false,
                tail: Tail::Pending,
                first_ready: false,
            },
            warm: None,
            queued: None,
        }
    }
}

pub fn warm_order(spine_len: usize, state: &ReadState, event: ReadEvent) -> (ReadState, Vec<Effect>) {
    let mut state = state.clone();
    let mut effects = Vec::new();
    match event {
        ReadEvent::ShownLayout { ready, tail, width_ok } => {
            if ready {
                state.shown.first_ready = true;
            }
            state.shown.tail = tail;
            if !state.stamped && ready && width_ok {
                state.stamped = true;
                effects.push(Effect::Stamp);
            }
            catch_up(spine_len, &mut state, &mut effects);
        }
        ReadEvent::ShownBox { page, pages } => {
            state.shown.page = page;
            state.shown.pages = pages;
            state.shown.pages_known = true;
            catch_up(spine_len, &mut state, &mut effects);
        }
        ReadEvent::Edge(Edge::Next) => go_next(spine_len, &mut state, &mut effects),
        ReadEvent::Edge(Edge::Prev) => go_prev(spine_len, &mut state, &mut effects),
        ReadEvent::WarmLayout { ready, tail } => {
            if let Some(warm) = state.warm.as_mut() {
                if ready {
                    warm.first_ready = true;
                }
                warm.tail = tail;
            }
            release_queue(&mut state, &mut effects);
        }
        ReadEvent::SeekDone => {
            if let Some(warm) = state.warm.as_mut() {
                warm.seeking = false;
                warm.at_last = true;
            }
            release_queue(&mut state, &mut effects);
        }
    }
    (state, effects)
}

fn after(spine_len: usize, index: usize) -> Option<usize> {
    let next = index + 1;
    (next < spine_len).then_some(next)
}

fn before(index: usize) -> Option<usize> {
    index.checked_sub(1)
}

fn loading(index: usize, goal: Goal) -> WarmPos {
    WarmPos {
        index,
        goal,
        first_ready: false,
        tail: Tail::Pending,
        seeking: false,
        at_last: false,
    }
}

fn catch_up(spine_len: usize, state: &mut ReadState, effects: &mut Vec<Effect>) {
    if state.stamped && state.shown.tail == Tail::Done && state.warm.is_none() {
        if let Some(next) = after(spine_len, state.shown.index) {
            state.warm = Some(loading(next, Goal::NextFirst));
            effects.push(Effect::Mount { index: next });
        }
    }
    let leave = state.shown.page >= 2
        || (state.shown.pages_known && state.shown.tail == Tail::Done && state.shown.pages <= 1);
    if leave {
        if let Some(warm) = state.warm.as_ref() {
            if warm.goal == Goal::PreviousKept {
                if let Some(next) = after(spine_len, state.shown.index) {
                    state.warm = Some(loading(next, Goal::NextFirst));
                    effects.push(Effect::Navigate { index: next });
                }
            }
        }
    }
    release_queue(state, effects);
}

fn go_next(spine_len: usize, state: &mut ReadState, effects: &mut Vec<Effect>) {
    let Some(next) = after(spine_len, state.shown.index) else {
        return;
    };
    let ready = state.warm.as_ref().is_some_and(|warm| {
        warm.index == next && warm.goal == Goal::NextFirst && warm.first_ready && !warm.seeking
    });
    if ready {
        swap_forward(state, effects);
        return;
    }
    let same = state.warm.as_ref().is_some_and(|warm| warm.index == next);
    if same {
        state.queued = Some(Edge::Next);
        return;
    }
    state.queued = Some(Edge::Next);
    if state.stamped && state.shown.tail == Tail::Done {
        let mounted = state.warm.is_some();
        state.warm = Some(loading(next, Goal::NextFirst));
        effects.push(if mounted {
            Effect::Navigate { index: next }
        } else {
            Effect::Mount { index: next }
        });
    }
}

fn go_prev(spine_len: usize, state: &mut ReadState, effects: &mut Vec<Effect>) {
    let _ = spine_len;
    let Some(prev) = before(state.shown.index) else {
        return;
    };
    let kept = state
        .warm
        .as_ref()
        .is_some_and(|warm| warm.index == prev && warm.goal == Goal::PreviousKept);
    let at_last = state.warm.as_ref().is_some_and(|warm| {
        warm.index == prev && warm.goal == Goal::PreviousLast && warm.at_last
    });
    if kept || at_last {
        swap_back(state, effects);
        return;
    }
    if !state.stamped {
        state.queued = Some(Edge::Prev);
        return;
    }
    state.queued = Some(Edge::Prev);
    let mounted = state.warm.is_some();
    state.warm = Some(loading(prev, Goal::PreviousLast));
    effects.push(if mounted {
        Effect::Navigate { index: prev }
    } else {
        Effect::Mount { index: prev }
    });
}

fn release_queue(state: &mut ReadState, effects: &mut Vec<Effect>) {
    let Some(warm) = state.warm.as_ref() else {
        return;
    };
    match state.queued {
        Some(Edge::Next) if warm.goal == Goal::NextFirst && warm.first_ready && !warm.seeking => {
            swap_forward(state, effects);
        }
        Some(Edge::Prev) if warm.goal == Goal::PreviousKept => swap_back(state, effects),
        Some(Edge::Prev) if warm.goal == Goal::PreviousLast && warm.at_last => swap_back(state, effects),
        Some(Edge::Prev)
            if warm.goal == Goal::PreviousLast && warm.tail == Tail::Done && !warm.seeking && !warm.at_last =>
        {
            if let Some(warm) = state.warm.as_mut() {
                warm.seeking = true;
            }
            effects.push(Effect::SeekLast);
        }
        _ => {}
    }
}

fn swap_forward(state: &mut ReadState, effects: &mut Vec<Effect>) {
    let Some(warm) = state.warm.clone() else {
        return;
    };
    let old = state.shown.index;
    state.queued = None;
    state.shown = ChapterPos {
        index: warm.index,
        page: 1,
        pages: 1,
        pages_known: false,
        tail: warm.tail,
        first_ready: true,
    };
    state.warm = Some(WarmPos {
        index: old,
        goal: Goal::PreviousKept,
        first_ready: true,
        tail: Tail::Done,
        seeking: false,
        at_last: true,
    });
    effects.push(Effect::SwapForward);
    effects.push(Effect::SyncAnchor);
}

fn swap_back(state: &mut ReadState, effects: &mut Vec<Effect>) {
    let Some(warm) = state.warm.clone() else {
        return;
    };
    let old = state.shown.index;
    let old_tail = state.shown.tail;
    state.queued = None;
    state.shown = ChapterPos {
        index: warm.index,
        page: 1,
        pages: 1,
        pages_known: false,
        tail: warm.tail,
        first_ready: true,
    };
    state.warm = Some(WarmPos {
        index: old,
        goal: Goal::NextFirst,
        first_ready: true,
        tail: old_tail,
        seeking: false,
        at_last: false,
    });
    effects.push(Effect::SwapBack);
    effects.push(Effect::SyncAnchor);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_two() -> ReadState {
        let (state, effects) = warm_order(
            3,
            &ReadState::fresh(),
            ReadEvent::ShownLayout {
                ready: true,
                tail: Tail::Pending,
                width_ok: true,
            },
        );
        assert_eq!(effects, vec![Effect::Stamp]);
        assert!(state.warm.is_none());
        let (state, effects) = warm_order(
            3,
            &state,
            ReadEvent::ShownLayout {
                ready: true,
                tail: Tail::Done,
                width_ok: true,
            },
        );
        assert_eq!(effects, vec![Effect::Mount { index: 1 }]);
        state
    }

    #[test]
    fn the_second_view_mounts_only_after_the_tail_and_a_ready_next_swaps() {
        let state = open_two();
        let (state, effects) = warm_order(
            3,
            &state,
            ReadEvent::Edge(Edge::Next),
        );
        assert!(effects.is_empty());
        assert_eq!(state.queued, Some(Edge::Next));
        let (state, effects) = warm_order(
            3,
            &state,
            ReadEvent::WarmLayout {
                ready: true,
                tail: Tail::Pending,
            },
        );
        assert_eq!(effects, vec![Effect::SwapForward, Effect::SyncAnchor]);
        assert_eq!(state.shown.index, 1);
        assert_eq!(state.warm.as_ref().map(|warm| warm.goal), Some(Goal::PreviousKept));
        let (state, effects) = warm_order(3, &state, ReadEvent::Edge(Edge::Prev));
        assert_eq!(effects, vec![Effect::SwapBack, Effect::SyncAnchor]);
        assert_eq!(state.shown.index, 0);
    }

    #[test]
    fn page_two_gives_the_slot_to_the_following_chapter() {
        let state = open_two();
        let (state, _) = warm_order(
            3,
            &state,
            ReadEvent::WarmLayout {
                ready: true,
                tail: Tail::Done,
            },
        );
        let (state, _) = warm_order(
            3,
            &state,
            ReadEvent::Edge(Edge::Next),
        );
        let (state, effects) = warm_order(3, &state, ReadEvent::ShownBox { page: 2, pages: 8 });
        assert_eq!(effects, vec![Effect::Navigate { index: 2 }]);
        assert_eq!(state.warm.as_ref().map(|warm| warm.index), Some(2));
    }

    #[test]
    fn a_back_edge_with_no_kept_chapter_loads_the_previous_one() {
        let mut state = open_two();
        state.shown.index = 1;
        state.warm = None;
        let (state, effects) = warm_order(3, &state, ReadEvent::Edge(Edge::Prev));
        assert_eq!(effects, vec![Effect::Mount { index: 0 }]);
        assert_eq!(state.warm.as_ref().map(|warm| warm.goal), Some(Goal::PreviousLast));
        let (state, effects) = warm_order(
            3,
            &state,
            ReadEvent::WarmLayout {
                ready: true,
                tail: Tail::Done,
            },
        );
        assert_eq!(effects, vec![Effect::SeekLast]);
        let (state, effects) = warm_order(3, &state, ReadEvent::SeekDone);
        assert_eq!(effects, vec![Effect::SwapBack, Effect::SyncAnchor]);
        assert_eq!(state.shown.index, 0);
    }

    #[test]
    fn the_first_chapter_ignores_back_and_the_last_ignores_next() {
        let state = ReadState::fresh();
        let (state, effects) = warm_order(2, &state, ReadEvent::Edge(Edge::Prev));
        assert!(effects.is_empty());
        assert!(state.queued.is_none());
        let mut last = state;
        last.stamped = true;
        last.shown.index = 1;
        last.shown.tail = Tail::Done;
        let (last, effects) = warm_order(2, &last, ReadEvent::Edge(Edge::Next));
        assert!(effects.is_empty());
        assert!(last.warm.is_none());
    }
}
