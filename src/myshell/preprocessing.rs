use crate::string_utils::{find_all_start_end_symb, find_all_subshells};

use super::super::string_utils;
use super::MyShell;

impl MyShell {
    pub fn preprocess_comments(line: &str) -> &str {
        match line.find('#') {
            Some(idx) => &line[0..idx],
            None => line,
        }
    }

    pub fn split_command(line: &str) -> Result<Vec<&str>, &str> {
        let one_quote = find_all_start_end_symb(line, "'")?;
        let two_quote = find_all_start_end_symb(line, "\"")?;
        let subshells = find_all_subshells(line)?;

        let mut non_split_regions = one_quote;
        non_split_regions.extend(two_quote);
        non_split_regions.extend(subshells);
        
        let mut splitted: Vec<&str> = Vec::new();
        let mut next: usize = 0;
        let mut prev: usize = 0;
        for ci in 0..line.len() {
            let c = line.as_bytes()[ci] as char;
            let mut can_split = true;
            for (s, e) in &non_split_regions {
                if (*s <= ci) && (ci <= *e) {
                    can_split = false;
                }
            }
            if can_split && c == ' ' {
                if prev != next {
                    splitted.push(&line[prev..next].trim());
                }
                next += 1;
                prev = next;
                continue;
            }
            next += 1;
        }
        splitted.push(&line[prev..]);
        Ok(splitted)
    }
}
