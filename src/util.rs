pub fn unescape(s: &str) -> String {
    let mut escaping = false;
    let mut output = String::new();

    for c in s.chars() {
        if !escaping && c == '\\' {
            escaping = true;
            continue
        }

        let escaped = if escaping {
            match c {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => c,
            }
        } else {
            c
        };

        output.push(escaped);
        escaping = false;
        continue
    }

    output
}
