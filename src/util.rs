pub fn rm_windows_new_lines(s: &str) -> String {
    s.chars().filter(|&c| c != '\r').collect()
}
