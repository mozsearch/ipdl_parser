enum State {
    Default,
    StartComment,
    InLineComment,
    InMultilineComment,
    EndingMultilineComment,
}

// I think this may produce different results for something like
// "/**/*/". I think the regex version will turn it into "      ",
// but the manual version will turn it into "    */".
pub fn uncomment(text: &str) -> String {
    let mut state = State::Default;
    let mut s = String::with_capacity(text.len());

    for c in text.chars() {
        match (state, c) {
            (State::Default, '/') =>
                state = State::StartComment,
            (State::Default, _) => {
                s.push(c);
                state = State::Default;
            },
            (State::StartComment, '/') => {
                s.push(' ');
                s.push(' ');
                state = State::InLineComment;
            },
            (State::StartComment, '*') => {
                s.push(' ');
                s.push(' ');
                state = State::InMultilineComment;
            },
            (State::StartComment, _) => {
                s.push('/');
                s.push(c);
                state = State::Default;
            },
            (State::InLineComment, '\n') => {
                s.push('\n');
                state = State::Default;
            },
            (State::InLineComment, _) => {
                s.push(' ');
                state = State::InLineComment;
            },
            (State::InMultilineComment, _) => {
                s.push(if c == '\n' { '\n' } else { ' ' });
                if c == '*' {
                    state = State::EndingMultilineComment;
                } else {
                    state = State::InMultilineComment;
                }
            },
            (State::EndingMultilineComment, _) => {
                s.push(if c == '\n' { '\n' } else { ' ' });
                if c == '/' {
                    state = State::Default;
                } else {
                    state = State::EndingMultilineComment;
                }
            },
        }
    };

    s
}

#[test]
fn basic_tests() {
    assert_eq!(uncomment("123"), "123");
    assert_eq!(uncomment("//123"), "     ");
    assert_eq!(uncomment("//123\n"), "     \n");
    assert_eq!(uncomment("//123\n45"), "     \n45");
    assert_eq!(uncomment("//123\n45"), "     \n45");
    assert_eq!(uncomment("//123\n//45\n6"), "     \n    \n6");

    assert_eq!(uncomment("0/*123*/0"), "0       0");
    assert_eq!(uncomment("0/*12\n3*/0"), "0    \n   0");

    // Newline right before fake end of multiline comment.
    assert_eq!(uncomment("/**\n*/"), "   \n  ");
}