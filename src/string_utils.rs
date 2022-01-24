pub fn find_all_start_end_symb<'a>(
    string: &str,
    symb: &str,
) -> Result<Vec<(usize, usize)>, &'a str> {
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut start_i: usize;
    let mut end_i: usize;

    start_i = match string.find(symb) {
        Some(idx) => idx,
        None => return Ok(result),
    };

    loop {
        end_i = match string[start_i + 1..].find(symb) {
            Some(idx) => start_i + 1 + idx,
            None => return Err("myshell: parse error"),
        };
        result.push((start_i, end_i));
        start_i = match string[end_i + 1..].find(symb) {
            Some(idx) => end_i + 1 + idx,
            None => break,
        };
    }

    Ok(result)
}

pub fn find_all_subshells<'a>(line: &str) -> Result<Vec<(usize, usize)>, &'a str> {
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut depth: usize = 0;
    let mut start: usize = 0;

    // let line_bytes = line;
    for ci in 0..line.len() {
        if (ci != line.len() - 1) && &line[ci..ci + 2] == "$(" {
            if depth == 0 {
                start = ci;
            }
            depth += 1;
        }

        if depth > 0 && &line[ci..ci + 1] == ")" {
            depth -= 1;
            if depth == 0 {
                result.push((start, ci));
            }
        }
    }
    // not closed $()
    if depth > 0 {
        return Err("myshell: parse error");
    }

    Ok(result)
}
