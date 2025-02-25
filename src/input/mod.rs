pub mod handler;
pub mod keybindings;

pub use keybindings::KeyBindings;

// This is a global instance for the default keybindings
pub static KEYBINDINGS: std::sync::LazyLock<KeyBindings> =
    std::sync::LazyLock::new(|| KeyBindings::default());
