use serde::{Serialize, Deserialize};


// TODO: Perf, i get it
#[derive(Debug, Clone)]
pub struct Tokenizer {
    text: String,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HxToken {
    Ident(String), //
    Equal,
    Colon,
    DoubleQuote,
    SingleQuote,
    OtherSymbol(String),
}

impl Tokenizer {
    pub fn new(text: &str) -> Self {
        return Tokenizer {
            text: text.to_string(),
            offset: text.len(),
        }
    }

    pub fn next_token(&mut self) -> Option<HxToken> {
        self.read_whitespace();

        match self.read_next_char()? {
            '=' => return Some(HxToken::Equal),
            ':' => return Some(HxToken::Colon),
            '"' => return Some(HxToken::DoubleQuote),
            '\'' => return Some(HxToken::SingleQuote),
            x => {
                if self.is_ident_char(x) {
                    return self.read_ident(x);
                }
                return Some(HxToken::OtherSymbol(x.to_string()))
            }
        }
    }

    fn is_ident_char(&self, char: char) -> bool {
        return char.is_alphanumeric() || char == '_' || char == '-';
    }

    fn read_ident(&mut self, char: char) -> Option<HxToken> {
        let mut ident = char.to_string();

        while let Some(c) = self.peek() {
            if self.is_ident_char(c) {
                ident.push(c);
                self.read_next_char();
            } else {
                break;
            }
        }

        let ident = ident.chars().rev().collect::<String>();
        return Some(HxToken::Ident(ident));
    }

    fn read_whitespace(&mut self) -> Option<()> {

        while let Some(char) = self.peek() {
            if char.is_whitespace() {
                self.read_next_char();
            } else {
                break;
            }
        }

        return None;
    }

    fn peek(&self) -> Option<char> {
        if self.offset == 0 {
            return None;
        }

        return self.text[self.offset - 1..self.offset].chars().next();
    }

    fn read_next_char(&mut self) -> Option<char> {
        if self.offset == 0 {
            return None;
        }

        let offset = self.offset;
        self.offset -= 1;

        return self.text[offset - 1..offset].chars().next();
    }

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HxPosition {
    InAttribute,
    InAttributeValue { hx_key: &'static str, current_value: String },
}

// TODO: upgrade to TreeSitter
pub fn hx_parse(str: &str) -> Option<HxPosition> {
    let mut tokenizer = Tokenizer::new(str);
    let first_token = tokenizer.next_token()?;

    match first_token {
        HxToken::Ident(x) => {
            if x == "hx-" {
                return Some(HxPosition::InAttribute);
            }
        }
        HxToken::DoubleQuote => {
            let tokens = read_until(&mut tokenizer, |x| {
                match x {
                    HxToken::Ident(_) => true,
                    _ => false,
                }
            });

            if let Some(HxToken::Ident(x)) = tokens.last() {
                if x == "hx-boost" {
                    return Some(HxPosition::InAttributeValue {
                        hx_key: "hx-boost",
                        current_value: "".to_string(),
                    });
                }
            }
        }

        _ => {},
    }

    return None;
}

fn read_until(tokenizer: &mut Tokenizer, until: fn(x: HxToken) -> bool) -> Vec<HxToken> {
    let mut tokens = Vec::new();

    while let Some(token) = tokenizer.next_token() {
        tokens.push(token.clone());
        if until(token) {
            break;
        }
    }

    return tokens;
}


#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::htmx::tokenizer::HxToken;

    use super::hx_parse;

    #[test]
    fn hx_parse_attribute() -> Result<()> {
        let attr = hx_parse("foo\" hx-");
        assert_eq!(attr, Some(super::HxPosition::InAttribute));
        return Ok(());
    }

    #[test]
    fn hx_parse_boost() -> Result<()> {
        let attr = hx_parse("foo\" hx-boost=\"");
        assert_eq!(attr, Some(super::HxPosition::InAttribute));
        return Ok(());
    }

    #[test]
    fn test_parsing() -> Result<()> {
        let string = "(<!'\"-:=foo bar-bar bar_bar\tfoo";
        let mut tokenizer = super::Tokenizer::new(string);

        assert_eq!(tokenizer.next_token(), Some(HxToken::Ident("foo".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Ident("bar_bar".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Ident("bar-bar".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Ident("foo".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Equal));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Colon));
        assert_eq!(tokenizer.next_token(), Some(HxToken::Ident("-".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::DoubleQuote));
        assert_eq!(tokenizer.next_token(), Some(HxToken::SingleQuote));
        assert_eq!(tokenizer.next_token(), Some(HxToken::OtherSymbol("!".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::OtherSymbol("<".into())));
        assert_eq!(tokenizer.next_token(), Some(HxToken::OtherSymbol("(".into())));

        return Ok(());
    }
}
