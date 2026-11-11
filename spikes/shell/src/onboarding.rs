//! The bare shelf screen is gone. Preview remains so an old `SPIKE_ONBOARDING`
//! launch does not grow a second window.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Empty,
    Importing,
    Done,
    Denied,
}

pub fn preview() -> Option<Preview> {
    match std::env::var("SPIKE_ONBOARDING")
        .ok()
        .as_deref()
        .map(str::trim)
    {
        Some("empty") => Some(Preview::Empty),
        Some("importing") => Some(Preview::Importing),
        Some("done") => Some(Preview::Done),
        Some("denied") => Some(Preview::Denied),
        _ => None,
    }
}
