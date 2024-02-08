use std::{ops::{Deref, DerefMut, Range}, path::{Path, PathBuf}};

use ariadne::{sources, ColorGenerator, FileCache, Label, Report, Source};

pub fn merge(a: &Span, b: &Span) -> Span {
    a.start.min(b.start)..a.end.max(b.end)
}

pub type SpannedResult<T,E> = Result<Spanned<T>, Spanned<E>>;
pub struct ParserCursor<'a, T> {
    pub input: &'a [T],
    pub pos: usize,
}

impl <'a, T> ParserCursor<'a, T> {
    pub fn new(input: &'a [T]) -> Self {
        ParserCursor { input, pos: 0 }
    }

    pub fn print_report(&self, error: &Spanned<String>, func: impl FnOnce(&[T]) -> String, filename: &str) {
        let mut colors = ColorGenerator::new();
        let input = func(self.input);
        Report::build(ariadne::ReportKind::Error, filename, error.span.clone().start)
            .with_message("Can't parse input.")
            .with_label(Label::new((filename, error.span.clone())).with_message(error.node.clone()).with_color(colors.next()))
            .finish()
            .print((filename, Source::from(input))).unwrap();
    }

    pub fn peek(&self) -> Option<Spanned<&'a T>> {
        self.input.get(self.pos).map(|c| Spanned::new(c, self.pos..(self.pos+1)))
    }

    pub fn next(&mut self) -> Option<Spanned<&'a T>> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    pub fn take(&mut self, n: usize) -> Option<Spanned<&'a [T]>> {
        let start = self.pos;
        let end = start + n;
        if end <= self.input.len() {
            self.pos = end;
            Some(Spanned::new(&self.input[start..end], start..end))
        } else {
            None
        }
    }

    pub fn peek_until(&mut self, f: fn(&T, &[T]) -> bool) -> Spanned<&'a [T]> {
        let start = self.pos;
        let mut end = self.pos;
        while end < self.input.len() && !f(&self.input[end], &self.input[start..end]) {
            end += 1;
        }
        let span = start..end;
        Spanned::new(&self.input[span.clone()], span)
    }

    pub fn take_until(&mut self, f: fn(&T, &[T]) -> bool) -> Spanned<&'a [T]> {
        let a = self.peek_until(f);
        self.pos = a.span.end;
        a
    }

    pub fn peek_while(&mut self, f: fn(&T, &[T]) -> bool) -> Spanned<&'a [T]> {
        let start = self.pos;
        let mut end = self.pos;
        while end < self.input.len() && f(&self.input[end], &self.input[start..end]) {
            end += 1;
        }
        let span = start..end;
        Spanned::new(&self.input[span.clone()], span)
    }

    pub fn take_while(&mut self, f: fn(&T, &[T]) -> bool) -> Spanned<&'a [T]> {
        let a = self.peek_while(f);
        self.pos = a.span.end;
        a
    }

    pub fn try_parse<E>(&mut self, f: fn(&mut Self) -> Option<Spanned<E>>) -> Option<Spanned<E>> {
        let start = self.pos;
        let result = f(self);
        if result.is_none() {
            self.pos = start;
        }
        result
    }

    pub fn catch_parse<E, R>(&mut self, f: fn(&mut Self) -> SpannedResult<R,E>) -> SpannedResult<R,E> {
        let start = self.pos;
        let result = f(self);
        if result.is_err() {
            self.pos = start;
        }
        result
    }


    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn span(&self) -> Span {
        self.pos..(self.pos+1)
    }
}

impl Clone for ParserCursor<'_, char> {
    fn clone(&self) -> Self {
        ParserCursor {
            input: self.input,
            pos: self.pos,
        }
    }
}

pub type Span = Range<usize>;

pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl <T: std::fmt::Debug> std::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.node, self.span)
    }
} 

impl <T: PartialEq> PartialEq<T> for Spanned<T> {
    fn eq(&self, other: &T) -> bool {
        self.node.eq(other)
    }
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }

    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    pub fn map_span<F>(self, f: F) -> Spanned<T>
    where
        F: FnOnce(Span) -> Span,
    {
        Spanned {
            node: self.node,
            span: f(self.span),
        }
    }

    pub fn merge_with_span<F>(self, other: Span) -> Spanned<T> {
        let start = self.span.start.min(other.start);
        let end = self.span.end.max(other.end);
        Spanned {
            node: self.node,
            span: start..end,
        }
    }

    pub fn merge<E>(self, other: Spanned<E>) -> Spanned<(T, E)> {
        let start = self.span.start.min(other.span.start);
        let end = self.span.end.max(other.span.end);
        Spanned {
            node: (self.node, other.node),
            span: start..end,
        }
    }
}

// Deref trait
impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

// DerefMut trait
impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.node
    }
}

// AsRef
impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.node
    }
}
