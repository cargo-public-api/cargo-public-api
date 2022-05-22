#[macro_export]
macro_rules! simple_macro {
    ($($arg:tt)*) => ({
        println!("simple_macro with {}", format!($($arg)*));
    })
}
