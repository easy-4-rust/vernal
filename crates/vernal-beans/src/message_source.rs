use std::collections::HashMap;
pub trait MessageSource: Send + Sync {
    fn get_message(&self, code: &str, args: &[&str], locale: &str) -> Option<String>;
}
pub fn interpolate_message(template: &str, args: &[&str]) -> String {
    args.iter()
        .enumerate()
        .fold(template.to_owned(), |s, (i, arg)| {
            s.replace(&format!("{{{i}}}"), arg)
        })
}
pub type MessageMap = HashMap<String, HashMap<String, String>>;
