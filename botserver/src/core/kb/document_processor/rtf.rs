pub fn strip_rtf_commands(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0i32;

    while i < len {
        if chars[i] == '{' {
            depth += 1;
            i += 1;
        } else if chars[i] == '}' {
            depth -= 1;
            if depth < 0 {
                depth = 0;
            }
            i += 1;
        } else if chars[i] == '\\' && i + 1 < len {
            if chars[i + 1] == '\'' && i + 4 <= len {
                if let Ok(byte_val) = u8::from_str_radix(
                    &input[chars[..i + 2].iter().collect::<String>().len()..]
                        .chars()
                        .take(2)
                        .collect::<String>(),
                    16,
                ) {
                    if let Some(c) = char::from_u32(byte_val as u32) {
                        result.push(c);
                    }
                }
                i += 4;
            } else if chars[i + 1] == '\n' || chars[i + 1] == '\r' {
                result.push('\n');
                i += 2;
            } else {
                let mut j = i + 1;
                while j < len && chars[j].is_ascii_alphabetic() {
                    j += 1;
                }
                if j < len && (chars[j] == '-' || chars[j] == ' ') && chars[j].is_ascii_digit()
                    || (j > i + 1 && chars[j] == ' ')
                {
                    j += 1;
                    while j < len && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                }
                while j < len && chars[j] == ' ' {
                    j += 1;
                }
                i = j;
            }
        } else {
            if depth <= 1 {
                result.push(chars[i]);
            }
            i += 1;
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
