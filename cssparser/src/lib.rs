use std::{
    path::Path, sync::RwLock,
};

mod parser;

use parser::{merge, ParserCursor, Spanned, SpannedResult};
use ratatui::style::{Color, Modifier, Style};

static STYLESHEETS: RwLock<Vec<Stylesheet>> = RwLock::new(Vec::new());

pub fn add_stylesheet(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let file = parse(path).unwrap();
    STYLESHEETS.write().unwrap().push(file);
}

pub fn get_style(identifier: &str, classes: &[impl AsRef<str>]) -> Style {
    let mut style = Style::default();
    let classes = classes.iter().map(|x| x.as_ref().to_string()).collect::<Vec<_>>();
    for sheet in STYLESHEETS.read().unwrap().iter() {
        sheet.get(identifier, &classes, &mut style);
    }
    style
}

fn parse(path: impl AsRef<Path>) -> Option<Stylesheet> {
    let path = path.as_ref();
    let file = std::fs::read_to_string(path).ok()?;
    let chars = file.chars().filter(|x| *x != '\r').collect::<Vec<_>>();
    let mut cursor = ParserCursor::new(&chars);
    let result = parse_stylesheet(&mut cursor);
    match result {
        Ok(style) => Some(style.node),
        Err(e) => {
            cursor.print_report(
                &e.map(|x| format!("{x:?}")),
                |e| e.iter().collect::<String>(),
                path.as_os_str().to_str().unwrap(),
            );
            None
        }
    }
}

fn literal(parser: &mut ParserCursor<char>) -> SpannedResult<String, ()> {
    let k = parser.take_while(|c, _| c.is_alphabetic() || c.is_digit(10) || *c == '_' || *c == '-');
    let string: Spanned<String> = k.map(|s| s.iter().collect());
    if string.is_empty() {
        Err(string.map(|_| ()))
    } else {
        Ok(string)
    }
}

fn expect(parser: &mut ParserCursor<char>, expected: char) -> SpannedResult<(), char> {
    match parser.next() {
        Some(e) => {
            if e == &expected {
                Ok(Spanned::new((), e.span))
            } else {
                Err(Spanned::new(*e.node, e.span))
            }
        }
        None => Err(Spanned::new(expected, parser.pos..parser.pos + 1)),
    }
}

fn consume_whitespace(parser: &mut ParserCursor<char>) {
    loop {
        parser.take_while(|c, _| c.is_whitespace());
        if !skip_comment(parser) {
            break;
        }
    }
}

fn class(parser: &mut ParserCursor<char>) -> SpannedResult<String, Error> {
    consume_whitespace(parser);
    expect(parser, '.').map_err(|e| e.map(|c| Error::ExpectedClassDot(c)))?;
    consume_whitespace(parser);
    literal(parser).map_err(|e| e.map(|_| Error::ExpectedClassName))
}

#[derive(Debug)]
enum Error {
    InvalidColorHex(String),
    InvalidColorName(String),
    InvalidModifier(String),
    InvalidHexType(String),
    InvalidProperty(String),
    ExpectedHexString,
    ExpectedHashTag(char),
    ExpectedClassName,
    ExpectedProperty,
    ExpectedAssignment(char), // :
    ExpectedModifier,
    ExpectedColorName,
    ExpectedSemicolon,
    ExpectedBracketOpen(char),
    ExpectedBracketClose(char),
    ExpectedClassDot(char),
    Union(Vec<Error>),
}

fn color_hex(parser: &mut ParserCursor<char>) -> SpannedResult<Color, Error> {
    consume_whitespace(parser);
    expect(parser, '#').map_err(|e| e.map(|c| Error::ExpectedHashTag(c)))?;
    consume_whitespace(parser);
    let k = literal(parser).map_err(|e| e.map(|_| Error::ExpectedHexString))?;
    let string = match k.len() {
        3 => k.map(|k| k.chars().flat_map(|c| [c, c]).collect()),
        6 => k,
        _ => return Err(Spanned::new(Error::InvalidColorHex(k.node), k.span)),
    };
    let color = u32::from_str_radix(&string.node, 16)
        .map_err(|_| Spanned::new(Error::InvalidHexType(string.node), string.span.clone()))?;
    Ok(Spanned::new(
        Color::Rgb(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        ),
        string.span,
    ))
}

fn color_name(parser: &mut ParserCursor<char>) -> SpannedResult<Option<Color>, Error> {
    consume_whitespace(parser);
    let k = literal(parser).map_err(|e| e.map(|_| Error::ExpectedColorName))?;
    let color = match k.node.as_str() {
        "black" => Color::Black,
        "gray" => Color::Gray,
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "transparent" | "reset" => Color::Reset,
        "light_red" | "lightred" => Color::LightRed,
        "light_green" | "lightgreen" => Color::LightGreen,
        "light_blue" | "lightblue" => Color::LightBlue,
        "light_yellow" | "lightyellow" => Color::LightYellow,
        "light_magenta" | "lightmagenta" => Color::LightMagenta,
        "light_cyan" | "lightcyan" => Color::LightCyan,
        "dark_gray" | "darkgray" => Color::DarkGray,
        "none" => return Ok(Spanned::new(None, k.span)),
        _ => return Err(Spanned::new(Error::InvalidColorName(k.node), k.span)),
    };
    Ok(Spanned::new(Some(color), k.span))
}

fn modifier(parser: &mut ParserCursor<char>) -> SpannedResult<Modifier, Error> {
    consume_whitespace(parser);
    let k = literal(parser).map_err(|e| e.map(|_| Error::ExpectedModifier))?;
    let modifier = match k.node.as_str() {
        "bold" => Modifier::BOLD,
        "dim" => Modifier::DIM,
        "italic" => Modifier::ITALIC,
        "underlined" => Modifier::UNDERLINED,
        "slow_blink" => Modifier::SLOW_BLINK,
        "rapid_blink" => Modifier::RAPID_BLINK,
        "reversed" => Modifier::REVERSED,
        "hidden" => Modifier::HIDDEN,
        "crossed_out" => Modifier::CROSSED_OUT,
        _ => return Err(Spanned::new(Error::InvalidModifier(k.node), k.span)),
    };
    Ok(Spanned::new(modifier, k.span))
}

#[derive(Debug)]
struct Stylesheet {
    rules: Vec<(Identifier, Style)>,
}

impl Stylesheet {
    pub fn get(&self, identifier: &str, classes: &[String], on: &mut Style) {
        for (id, style) in &self.rules {
            if let Some(element) = id.element.as_ref() {
                if element.node.as_str() != identifier {
                    continue;
                }
            }
            if id.classes.iter().all(|x| classes.contains(&x.node)) {
                *on = on.patch(*style);
            }
        }
    }
}

fn parse_stylesheet(parser: &mut ParserCursor<char>) -> SpannedResult<Stylesheet, Error> {
    let mut rules = Vec::new();
    loop {
        consume_whitespace(parser);
        if parser.is_eof() {
            break;
        }
        let identifier = parse_identifier(parser)?;
        consume_whitespace(parser);
        let style = parse_style(parser)?;
        rules.push((identifier.node, style.node));
    }
    Ok(Spanned::new(Stylesheet { rules }, parser.span()))
}

#[derive(Debug)]
struct Identifier {
    element: Option<Spanned<String>>,
    classes: Vec<Spanned<String>>,
}

fn parse_identifier(parser: &mut ParserCursor<char>) -> SpannedResult<Identifier, Error> {
    consume_whitespace(parser);
    let start = parser.pos;
    let element = literal(parser).ok();
    let mut classes = Vec::new();
    loop {
        consume_whitespace(parser);
        if parser.peek().map(|c| c == &'{').unwrap_or(false) {
            break;
        }
        if parser.peek().map(|c| c == &'.').unwrap_or(false) {
            classes.push(class(parser)?);
        } else {
            break;
        }
    }
    Ok(Spanned::new(
        Identifier {
            element: element,
            classes,
        },
        start..parser.pos,
    ))
}

/*
Parses:
{
    bg: #ff0000;
    fg: #00ff00;
    modifier: bold;
}
*/
fn parse_style(parser: &mut ParserCursor<char>) -> SpannedResult<Style, Error> {
    consume_whitespace(parser);
    let start = parser.pos;
    expect(parser, '{').map_err(|e| e.map(|c| Error::ExpectedBracketOpen(c)))?;
    let mut style = Style::default();
    loop {
        consume_whitespace(parser);
        if parser.peek().map(|c| c == &'}').unwrap_or(false) {
            parser.advance(1);
            break;
        }
        let literal = literal(parser).map_err(|e| e.map(|_| Error::ExpectedClassName))?;
        consume_whitespace(parser);
        expect(parser, ':').map_err(|e| e.map(|c| Error::ExpectedAssignment(c)))?;
        consume_whitespace(parser);
        match literal.node.as_str() {
            "bg" => {
                style.bg = color(parser)?.node;
            }
            "fg" => {
                style.fg = color(parser)?.node;
            }
            "add-modifier" => {
                style = style.add_modifier(modifier(parser)?.node);
            }
            "remove-modifier" => {
                style = style.remove_modifier(modifier(parser)?.node);
            }
            _ => {
                return Err(Spanned::new(
                    Error::InvalidProperty(literal.node),
                    literal.span,
                ));
            }
        }
        consume_whitespace(parser);
        if parser.peek().map(|c| c == &';').unwrap_or(false) {
            parser.advance(1);
        } else if parser.peek().map(|c| c == &'}').unwrap_or(false) {
            parser.advance(1);
            break;
        } else if parser.is_eof() {
            return Err(Spanned::new(
                Error::ExpectedBracketClose(' '),
                parser.pos..parser.pos + 1,
            ));
        } else {
            return Err(Spanned::new(Error::ExpectedSemicolon, parser.span()));
        }
    }
    Ok(Spanned::new(style, start..parser.pos))
}

fn color(parser: &mut ParserCursor<char>) -> SpannedResult<Option<Color>, Error> {
    parser
        .catch_parse(color_hex)
        .map(|x| x.map(Some))
        .or_else(|e| {
            color_name(parser).map_err(|n| {
                Spanned::new(Error::Union(vec![e.node, n.node]), merge(&e.span, &n.span))
            })
        })
}

fn skip_comment(parser: &mut ParserCursor<char>) -> bool {
    parser.catch_parse(comment).is_ok()
}

fn comment(parser: &mut ParserCursor<char>) -> SpannedResult<(), ()> {
    expect(parser, '/').map_err(|e| e.map(|_| ()))?;
    if let Some(c) = parser.next() {
        if c == &'/' {
            parser.take_while(|c, _| *c != '\n');
        } else if c == &'*' {
            loop {
                if let Some(c) = parser.next() {
                    if c == &'*' {
                        if let Some(c) = parser.next() {
                            if c == &'/' {
                                break;
                            }
                        }
                    }
                } else {
                    return Err(Spanned::new((), parser.pos..parser.pos + 1));
                }
            }
        } else {
            return Err(Spanned::new((), c.span));
        }
    }
    Ok(Spanned::new((), parser.span()))
}
