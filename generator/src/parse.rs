use std::{borrow::Cow, str::Lines};

use crate::Category;

pub fn parse_header(lines: Lines) -> Result<Vec<Category>, ()> {
    let mut lines = lines
        // Skip all the license header info.
        .skip_while(|line| line.starts_with("#define _UAPI_INPUT_EVENT_CODES_H"))
        // Skip while will present the entry that returns false in the predicate.
        // We do not want that define since it has no value, so skip it.
        .skip(1)
        .peekable();

    let mut lines_with_defines = vec![];

    // This logic is required because some defines have end of line comments that span multiple lines such as
    // KEY_SWITCHVIDEOMODE and KEY_BRIGHTNESS_AUTO.
    loop {
        match lines.next() {
            Some(line) => {
                // Only consider defines
                if line.starts_with("#define") {
                    // Check if an end of line comment is present.
                    // And if the end of line comment isn't terminated on the same line.
                    let next_line_comment = line.find("/*").is_some() && line.find("*/").is_none();

                    let line = if next_line_comment {
                        let mut join_lines = Vec::with_capacity(3); // Current longest multi-line comment is 3.
                        join_lines.push(line);

                        // No, the end of line is not terminated, attach the next line and try to find a terminator.
                        loop {
                            // Take the next line.
                            match lines.next() {
                                Some(line) => {
                                    assert!(
                                        !line.starts_with("#define"),
                                        "Multi-line comment handling encountered define"
                                    );

                                    join_lines.push(line);

                                    // End of line terminator found, return the value.
                                    if line.contains("*/") {
                                        break;
                                    }
                                }

                                None => panic!("Unterminated end of line comment?"),
                            };
                        }

                        // Now combine the line contents.
                        // Replace newlines with a space to effectively join the lines.
                        let raw_line = join_lines.join("");

                        let (define, comment) = raw_line.split_once("/*").unwrap();

                        // Make sure we reinsert the `/*` we split on.
                        let comment = format!(
                            "/* {}",
                            comment
                                .replace('\n', " ")
                                .split_ascii_whitespace()
                                .collect::<Vec<_>>()
                                // Now join with proper spacing.
                                .join(" ")
                        );

                        Cow::Owned(String::from_iter([define, &comment]))
                    } else {
                        Cow::Borrowed(line)
                    };

                    lines_with_defines.push(line);
                };
            }

            None => break,
        }
    }

    todo!("Begin parsing normalized entries.")
}
