

/// Creates a temporary environment where the
/// path is changed to the ``path``'s parent.
#[macro_export]
macro_rules! with_parent_path {
    ($path:expr, $block:tt) => {
        {
            let root = env::current_dir().unwrap();
            if let Some(parent_path) = $path.parent() {
                if parent_path.exists() {  // Check if path is not empty
                    std::env::set_current_dir(parent_path).unwrap();
                }
            }
            let result = $block;
            std::env::set_current_dir(root).unwrap();
            result
        }
    };
}
