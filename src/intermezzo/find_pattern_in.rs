fn find_pattern_in<'a>(pattern: &str, text: &'a str) -> Option<&'a str> {
    let n = pattern.len();
    if text.len() < n {
        return None;
    }
    let m = text.len() - n;
    for i in 0..m {
        let s = &text[i..i + n];
        if pattern == s {
            return Some(s);
        }
    }
    None
}

fn main() {
    let text: String = "Hello world!".into();
    let substring = {
        let pattern: String = "world".into();
        find_pattern_in(&pattern, &text)
    };
    println!("substring is {:?}", substring);
}
