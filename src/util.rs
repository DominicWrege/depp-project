pub fn trim_new_lines(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\r')
        .collect::<String>()
        .lines()
        .map(|line| {
            let mut n_line = line.trim_end().to_string();
            n_line.push('\n');
            n_line
        })
        .collect::<String>()
}
