// WARNING: Not utf-8 safe. But (hopefully) you get the idea.  To make
// it utf-8 safe, you'd want to rewrite to extract out characters
// rather than just indexing `str[pos]`.

extern mod std;

#[deriving(Eq)]
enum Re {
    ReChar(char),
    ReDot,
    ReStar(~Re),
    RePlus(~Re),
    ReSeq(~[Re]),
    ReGroup(~Re)
}

enum IntermediateResult {
    ParseOk(Re, uint),
    ParseErr(uint, ~str)
}

fn parse_regex(str: &str) -> Result<Re, ~str> {
    return match seq(str, 0) {
        ParseErr(pos, msg) => {
            Err(fmt!("at position %u, %s", pos, msg))
        }
        ParseOk(_, pos) if pos < str.len() => {
            // did not consume entire string
            Err(fmt!("at position %u, unexpected character '%c'", pos, str[pos] as char))
        }
        ParseOk(re, _) => {
            Ok(re)
        }
    };

    fn expect(str: &str, pos: uint, expect: char, ok: Re) -> IntermediateResult {
        if pos < str.len() && str[pos] as char == expect {
            ParseOk(ok, pos+1)
        } else if pos < str.len() {
            ParseErr(pos, fmt!("expected '%c', found '%c'", expect, str[pos] as char))
        } else {
            ParseErr(pos, fmt!("expected '%c', found EOF", expect))
        }
    }

    fn seq(str: &str, mut pos: uint) -> IntermediateResult {
        let mut vec = ~[];
        while pos < str.len() {
            match str[pos] as char {
                ')' => {
                    break;
                }
                _ => {
                    match rep(str, pos) {
                        ParseErr(pos, msg) => {
                            return ParseErr(pos, msg);
                        }
                        ParseOk(r, pos1) => {
                            vec.push(r);
                            pos = pos1;
                        }
                    }
                }
            }
        }
        ParseOk(ReSeq(vec), pos)
    }

    fn rep(str: &str, pos: uint) -> IntermediateResult {
        match base(str, pos) {
            ParseErr(pos, msg) => ParseErr(pos, msg),
            ParseOk(r, pos) => {
                match str[pos] as char {
                    '*' => {
                        ParseOk(ReStar(~r), pos+1)
                    }
                    '+' => {
                        ParseOk(RePlus(~r), pos+1)
                    }
                    _ => {
                        ParseOk(r, pos)
                    }
                }
            }
        }
    }

    fn base(str: &str, pos: uint) -> IntermediateResult {
        match str[pos] as char {
            '.' => {
                ParseOk(ReDot, pos+1)
            }
            '\\' => {
                if pos + 1 == str.len() {
                    ParseErr(pos+1, ~"EOF in escape")
                } else {
                    ParseOk(ReChar(str[pos+1] as char), pos + 2)
                }
            }
            '(' => {
                match seq(str, pos+1) {
                    ParseErr(pos, msg) => ParseErr(pos, msg),
                    ParseOk(r, pos) => {
                        expect(str, pos, ')', ReGroup(~r))
                    }
                }
            }
            ')' => {
                ParseErr(pos, ~"Unbalanced close paren")
            }
            c => {
                ParseOk(ReChar(c), pos+1)
            }
        }
    }
}

#[test]
fn parse_abc() {
    assert_eq!(
        parse_regex("abc*"),
        Ok(ReSeq(~[ReChar('a'), ReChar('b'), ReStar(~ReChar('c'))])));
}

#[test]
fn parse_group() {
    assert_eq!(
        parse_regex("a(bc)*"),
        Ok(ReSeq(~[ReChar('a'),
                   ReStar(~ReGroup(~ReSeq(~[ReChar('b'), ReChar('c')])))])));
}
