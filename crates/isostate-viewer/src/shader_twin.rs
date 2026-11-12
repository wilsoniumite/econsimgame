//! A guard on the field system written twice (K26 of the sweep of 11
//! November). The ground shader, terrain.wesl, repeats line for line what
//! fields.rs and world.rs work out on the CPU: the hashes and the value
//! noise, the cutting of blocks into parcels and fields, the warp and the
//! fraying of the edges of wood and heath, the glades, the summer snow line
//! and how a wood deepens from its edge. The two are kept in step by hand.
//! The only check that they agree had been `ISOSTATE_COMPARE_CLASSES`, which
//! needs pictures taken on a graphics adapter.
//!
//! These tests need none. They read the shader's own source, run the
//! functions it repeats on the CPU with a small reader of the part of WGSL
//! those functions are written in (`Program`), and set what each gives
//! beside what the Rust gives, at a few thousand spots over and round an
//! island. A number or a step changed on one side and not the other fails a
//! test and names the function. What stays unguarded is listed in the notes
//! of `night/o-land-checks` and in the test at the end of this file.
//!
//! The reader knows only what those functions use: `fn`, `let`, `var`,
//! `if`, `for`, `loop`, `break`, `return`, structs, two- to four-wide
//! vectors and a dozen built-in functions. It does arithmetic in f32, i32
//! and u32 as the GPU does (whole numbers wrap), in the order the source
//! gives, so the hashes agree bit for bit and the noise to the last place.

use std::collections::HashMap;
use std::rc::Rc;

const SOURCE: &str = include_str!("terrain.wesl");

// ------------------------------------------------------------------ reading the source

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(String),
    Id(String),
    P(&'static str),
}

const PUNCT: [&str; 33] = [
    "->", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "+=", "-=", "*=", "/=", "++", "--", "(", ")", "{", "}", "[", "]", ",", ";", ":", ".", "+", "-", "*", "/", "%", "<", ">", "=",
];
const PUNCT1: [&str; 5] = ["!", "&", "|", "^", "@"];

fn lex(src: &str) -> Vec<Tok> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    'next: while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if src[i..].starts_with("//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let s = i;
            while i < b.len() && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'.') {
                let e = b[i] == b'e' || b[i] == b'E';
                i += 1;
                if e && i < b.len() && (b[i] == b'-' || b[i] == b'+') {
                    i += 1;
                }
            }
            out.push(Tok::Num(src[s..i].to_string()));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let s = i;
            while i < b.len() && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Tok::Id(src[s..i].to_string()));
            continue;
        }
        for p in PUNCT.iter().chain(PUNCT1.iter()) {
            if src[i..].starts_with(p) {
                out.push(Tok::P(p));
                i += p.len();
                continue 'next;
            }
        }
        panic!("the shader's twin cannot read {:?}", &src[i..(i + 20).min(src.len())]);
    }
    out
}

#[derive(Clone, Debug)]
enum Ex {
    Lit(V),
    Var(String),
    Call(String, Vec<Ex>),
    Member(Box<Ex>, String),
    Neg(Box<Ex>),
    Not(Box<Ex>),
    Bin(&'static str, Box<Ex>, Box<Ex>),
}

#[derive(Clone, Debug)]
enum St {
    Let(String, Option<String>, Option<Ex>),
    Assign(Ex, &'static str, Ex),
    If(Ex, Vec<St>, Vec<St>),
    For(Box<St>, Ex, Box<St>, Vec<St>),
    Loop(Vec<St>),
    Break,
    Continue,
    Return(Option<Ex>),
    Block(Vec<St>),
}

struct Func {
    params: Vec<(String, String)>,
    ret: Option<String>,
    body: Vec<St>,
}

struct Parser<'a> {
    t: &'a [Tok],
    at: usize,
}

/// The binary operators from the loosest to the tightest.
const LEVELS: [&[&str]; 10] = [&["||"], &["&&"], &["|"], &["^"], &["&"], &["==", "!="], &["<", ">", "<=", ">="], &["<<", ">>"], &["+", "-"], &["*", "/", "%"]];

impl Parser<'_> {
    fn peek(&self) -> &Tok {
        self.t.get(self.at).unwrap_or(&Tok::P(";"))
    }
    fn is(&self, p: &str) -> bool {
        matches!(self.peek(), Tok::P(q) if *q == p)
    }
    fn is_id(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Id(q) if q == w)
    }
    fn eat(&mut self, p: &str) -> bool {
        let yes = self.is(p);
        self.at += yes as usize;
        yes
    }
    fn want(&mut self, p: &str) {
        assert!(self.eat(p), "the shader's twin wanted {p:?} and found {:?} (token {})", self.peek(), self.at);
    }
    fn id(&mut self) -> String {
        match self.peek().clone() {
            Tok::Id(s) => {
                self.at += 1;
                s
            }
            t => panic!("the shader's twin wanted a name and found {t:?}"),
        }
    }
    fn attributes(&mut self) {
        while self.eat("@") {
            self.id();
            if self.eat("(") {
                let mut depth = 1;
                while depth > 0 {
                    depth += self.is("(") as i32 - self.is(")") as i32;
                    self.at += 1;
                }
            }
        }
    }
    fn ty(&mut self) -> String {
        let mut s = self.id();
        if self.eat("<") {
            s.push('<');
            s.push_str(&self.ty());
            while self.eat(",") {
                s.push(',');
                s.push_str(&self.ty());
            }
            self.want(">");
            s.push('>');
        }
        s
    }
    fn func(&mut self) -> Func {
        self.attributes();
        assert!(self.is_id("fn"));
        self.at += 1;
        self.id();
        self.want("(");
        let mut params = Vec::new();
        while !self.eat(")") {
            self.attributes();
            let name = self.id();
            self.want(":");
            params.push((name, self.ty()));
            self.eat(",");
        }
        let ret = if self.eat("->") {
            self.attributes();
            Some(self.ty())
        } else {
            None
        };
        Func { params, ret, body: self.block() }
    }
    fn block(&mut self) -> Vec<St> {
        self.want("{");
        let mut out = Vec::new();
        while !self.eat("}") {
            out.push(self.statement());
        }
        out
    }
    fn statement(&mut self) -> St {
        if self.is("{") {
            return St::Block(self.block());
        }
        if self.is_id("if") {
            self.at += 1;
            let cond = self.expr();
            let yes = self.block();
            let no = if self.is_id("else") {
                self.at += 1;
                if self.is_id("if") { vec![self.statement()] } else { self.block() }
            } else {
                Vec::new()
            };
            return St::If(cond, yes, no);
        }
        if self.is_id("for") {
            self.at += 1;
            self.want("(");
            let init = self.simple();
            self.want(";");
            let cond = self.expr();
            self.want(";");
            let step = self.simple();
            self.want(")");
            return St::For(Box::new(init), cond, Box::new(step), self.block());
        }
        if self.is_id("loop") {
            self.at += 1;
            return St::Loop(self.block());
        }
        let s = if self.is_id("break") {
            self.at += 1;
            St::Break
        } else if self.is_id("continue") {
            self.at += 1;
            St::Continue
        } else if self.is_id("return") {
            self.at += 1;
            St::Return(if self.is(";") { None } else { Some(self.expr()) })
        } else {
            self.simple()
        };
        self.want(";");
        s
    }
    fn simple(&mut self) -> St {
        if self.is_id("let") || self.is_id("var") {
            self.at += 1;
            let name = self.id();
            let ty = if self.eat(":") { Some(self.ty()) } else { None };
            let init = if self.eat("=") { Some(self.expr()) } else { None };
            return St::Let(name, ty, init);
        }
        let place = self.postfix();
        for op in ["=", "+=", "-=", "*=", "/="] {
            if self.eat(op) {
                let op = PUNCT.iter().find(|p| **p == op).unwrap();
                return St::Assign(place, op, self.expr());
            }
        }
        if self.eat("++") {
            return St::Assign(place, "+=", Ex::Lit(V::AI(1)));
        }
        if self.eat("--") {
            return St::Assign(place, "-=", Ex::Lit(V::AI(1)));
        }
        panic!("the shader's twin cannot read a statement at {:?}", self.peek());
    }
    fn expr(&mut self) -> Ex {
        self.level(0)
    }
    fn level(&mut self, k: usize) -> Ex {
        if k == LEVELS.len() {
            return self.unary();
        }
        let mut a = self.level(k + 1);
        'more: loop {
            for op in LEVELS[k] {
                if self.eat(op) {
                    let b = self.level(k + 1);
                    a = Ex::Bin(op, Box::new(a), Box::new(b));
                    continue 'more;
                }
            }
            return a;
        }
    }
    fn unary(&mut self) -> Ex {
        if self.eat("-") {
            return Ex::Neg(Box::new(self.unary()));
        }
        if self.eat("!") {
            return Ex::Not(Box::new(self.unary()));
        }
        self.postfix()
    }
    fn postfix(&mut self) -> Ex {
        let mut e = self.primary();
        while self.eat(".") {
            e = Ex::Member(Box::new(e), self.id());
        }
        e
    }
    fn primary(&mut self) -> Ex {
        match self.peek().clone() {
            Tok::Num(s) => {
                self.at += 1;
                Ex::Lit(literal(&s))
            }
            Tok::P("(") => {
                self.at += 1;
                let e = self.expr();
                self.want(")");
                e
            }
            Tok::Id(mut name) => {
                self.at += 1;
                // vec2<f32>(...), bitcast<u32>(...): a type in angle brackets
                if matches!(name.as_str(), "vec2" | "vec3" | "vec4" | "bitcast") && self.is("<") {
                    self.at += 1;
                    name = format!("{name}<{}>", self.ty());
                    self.want(">");
                }
                if self.eat("(") {
                    let mut args = Vec::new();
                    while !self.eat(")") {
                        args.push(self.expr());
                        self.eat(",");
                    }
                    Ex::Call(name, args)
                } else {
                    Ex::Var(name)
                }
            }
            t => panic!("the shader's twin cannot read an expression at {t:?}"),
        }
    }
}

fn literal(s: &str) -> V {
    if let Some(n) = s.strip_suffix('u') {
        V::U(n.parse().unwrap())
    } else if let Some(n) = s.strip_suffix('i') {
        V::I(n.parse().unwrap())
    } else if let Some(n) = s.strip_suffix('f') {
        V::F(n.parse().unwrap())
    } else if s.contains(['.', 'e', 'E']) {
        V::AF(s.parse().unwrap())
    } else {
        V::AI(s.parse().unwrap())
    }
}

// ------------------------------------------------------------------ values

/// A value: the GPU's three kinds of number and truth, WGSL's abstract
/// numbers (a literal before it meets a typed value), vectors and structs.
#[derive(Clone, Debug, PartialEq)]
pub enum V {
    F(f32),
    I(i32),
    U(u32),
    B(bool),
    AI(i64),
    AF(f64),
    Vec(Vec<V>),
    S(String, Vec<(String, V)>),
}

impl V {
    pub fn f(&self) -> f32 {
        match *self {
            V::F(x) => x,
            V::AF(x) => x as f32,
            V::AI(x) => x as f32,
            ref v => panic!("{v:?} is not a number"),
        }
    }
    pub fn u(&self) -> u32 {
        match *self {
            V::U(x) => x,
            ref v => panic!("{v:?} is not a u32"),
        }
    }
    pub fn i(&self) -> i32 {
        match *self {
            V::I(x) => x,
            ref v => panic!("{v:?} is not an i32"),
        }
    }
    pub fn v2(&self) -> [f32; 2] {
        match self {
            V::Vec(c) if c.len() == 2 => [c[0].f(), c[1].f()],
            v => panic!("{v:?} is not a vec2"),
        }
    }
}

/// Two numbers brought to one kind: an abstract number takes the kind of a
/// typed one, as WGSL does.
fn unify(a: V, b: V) -> (V, V) {
    use V::*;
    match (a, b) {
        (AI(x), F(y)) => (F(x as f32), F(y)),
        (F(x), AI(y)) => (F(x), F(y as f32)),
        (AF(x), F(y)) => (F(x as f32), F(y)),
        (F(x), AF(y)) => (F(x), F(y as f32)),
        (AI(x), I(y)) => (I(x as i32), I(y)),
        (I(x), AI(y)) => (I(x), I(y as i32)),
        (AI(x), U(y)) => (U(x as u32), U(y)),
        (U(x), AI(y)) => (U(x), U(y as u32)),
        (AI(x), AF(y)) => (AF(x as f64), AF(y)),
        (AF(x), AI(y)) => (AF(x), AF(y as f64)),
        (a, b) => {
            assert!(std::mem::discriminant(&a) == std::mem::discriminant(&b), "the shader's twin cannot mix {a:?} and {b:?}");
            (a, b)
        }
    }
}

/// A number made the kind of a type named in the source: an abstract one as
/// it is given to a typed name, a typed one as by a constructor
/// (`vec2<i32>(textureDimensions(t))`).
fn coerce(v: V, ty: &str) -> V {
    use V::*;
    match (v, ty) {
        (AI(x), "f32") => F(x as f32),
        (AF(x), "f32") => F(x as f32),
        (I(x), "f32") => F(x as f32),
        (U(x), "f32") => F(x as f32),
        (AI(x), "i32") => I(x as i32),
        (U(x), "i32") => I(x as i32),
        (F(x), "i32") => I(x as i32),
        (AI(x), "u32") => U(x as u32),
        (I(x), "u32") => U(x as u32),
        (F(x), "u32") => U(x as u32),
        (Vec(c), t) if t.starts_with("vec") => {
            let el = t.split_once('<').map_or("", |(_, e)| e.trim_end_matches('>'));
            Vec(c.into_iter().map(|x| if el.is_empty() { concrete(x) } else { coerce(x, el) }).collect())
        }
        (v, _) => v,
    }
}

/// An abstract number as WGSL makes it when nothing says otherwise.
fn concrete(v: V) -> V {
    match v {
        V::AI(x) => V::I(x as i32),
        V::AF(x) => V::F(x as f32),
        V::Vec(c) => V::Vec(c.into_iter().map(concrete).collect()),
        v => v,
    }
}

/// An abstract number made the kind of what it is put into.
fn like(target: &V, v: V) -> V {
    match (target, v) {
        (V::Vec(t), V::Vec(c)) => V::Vec(t.iter().zip(c).map(|(t, x)| like(t, x)).collect()),
        (V::F(_), v) => coerce(v, "f32"),
        (V::I(_), v) => coerce(v, "i32"),
        (V::U(_), v) => coerce(v, "u32"),
        (_, v) => v,
    }
}

fn bin(op: &str, a: V, b: V) -> V {
    match (a, b) {
        (V::Vec(x), V::Vec(y)) => V::Vec(x.into_iter().zip(y).map(|(p, q)| bin(op, p, q)).collect()),
        (V::Vec(x), b) => V::Vec(x.into_iter().map(|p| bin(op, p, b.clone())).collect()),
        (a, V::Vec(y)) => V::Vec(y.into_iter().map(|q| bin(op, a.clone(), q)).collect()),
        (a, b) => scalar(op, a, b),
    }
}

fn scalar(op: &str, a: V, b: V) -> V {
    use V::*;
    macro_rules! cmp {
        ($x:expr, $y:expr) => {
            match op {
                "<" => Some(B($x < $y)),
                ">" => Some(B($x > $y)),
                "<=" => Some(B($x <= $y)),
                ">=" => Some(B($x >= $y)),
                "==" => Some(B($x == $y)),
                "!=" => Some(B($x != $y)),
                _ => None,
            }
        };
    }
    let (a, b) = unify(a, b);
    let out = match (&a, &b) {
        (F(x), F(y)) => cmp!(x, y).or(match op {
            "+" => Some(F(x + y)),
            "-" => Some(F(x - y)),
            "*" => Some(F(x * y)),
            "/" => Some(F(x / y)),
            "%" => Some(F(x % y)),
            _ => None,
        }),
        (AF(x), AF(y)) => cmp!(x, y).or(match op {
            "+" => Some(AF(x + y)),
            "-" => Some(AF(x - y)),
            "*" => Some(AF(x * y)),
            "/" => Some(AF(x / y)),
            _ => None,
        }),
        (I(x), I(y)) => cmp!(x, y).or(match op {
            "+" => Some(I(x.wrapping_add(*y))),
            "-" => Some(I(x.wrapping_sub(*y))),
            "*" => Some(I(x.wrapping_mul(*y))),
            "/" => Some(I(x.wrapping_div(*y))),
            "%" => Some(I(x.wrapping_rem(*y))),
            "^" => Some(I(x ^ y)),
            "&" => Some(I(x & y)),
            "|" => Some(I(x | y)),
            "<<" => Some(I(x.wrapping_shl(*y as u32))),
            ">>" => Some(I(x.wrapping_shr(*y as u32))),
            _ => None,
        }),
        (U(x), U(y)) => cmp!(x, y).or(match op {
            "+" => Some(U(x.wrapping_add(*y))),
            "-" => Some(U(x.wrapping_sub(*y))),
            "*" => Some(U(x.wrapping_mul(*y))),
            "/" => Some(U(x / y)),
            "%" => Some(U(x % y)),
            "^" => Some(U(x ^ y)),
            "&" => Some(U(x & y)),
            "|" => Some(U(x | y)),
            "<<" => Some(U(x.wrapping_shl(*y))),
            ">>" => Some(U(x.wrapping_shr(*y))),
            _ => None,
        }),
        (AI(x), AI(y)) => cmp!(x, y).or(match op {
            "+" => Some(AI(x + y)),
            "-" => Some(AI(x - y)),
            "*" => Some(AI(x * y)),
            "/" => Some(AI(x / y)),
            "^" => Some(AI(x ^ y)),
            _ => None,
        }),
        (B(x), B(y)) => match op {
            "&&" | "&" => Some(B(*x && *y)),
            "||" | "|" => Some(B(*x || *y)),
            "==" => Some(B(x == y)),
            "!=" => Some(B(x != y)),
            _ => None,
        },
        _ => None,
    };
    out.unwrap_or_else(|| panic!("the shader's twin cannot do {a:?} {op} {b:?}"))
}

/// A built-in function of one or more numbers, applied part by part to
/// vectors, in the kind of number its arguments come to.
fn each(args: &[V], f: &dyn Fn(&[V]) -> V) -> V {
    if let Some(n) = args.iter().find_map(|a| if let V::Vec(c) = a { Some(c.len()) } else { None }) {
        return V::Vec(
            (0..n)
                .map(|k| {
                    let part: Vec<V> = args.iter().map(|a| if let V::Vec(c) = a { c[k].clone() } else { a.clone() }).collect();
                    each(&part, f)
                })
                .collect(),
        );
    }
    // bring every argument to the kind of the first typed one
    let kind = args.iter().find(|a| !matches!(a, V::AI(_) | V::AF(_))).cloned();
    let args: Vec<V> = match kind {
        Some(k) => args.iter().map(|a| unify(a.clone(), k.clone()).0).collect(),
        None => args.iter().map(|a| if let V::AI(x) = a { V::AF(*x as f64) } else { a.clone() }).collect(),
    };
    f(&args)
}

fn f32s(a: &[V]) -> Vec<f32> {
    a.iter().map(V::f).collect()
}

fn back(like: &V, x: f32) -> V {
    if matches!(like, V::AF(_)) { V::AF(x as f64) } else { V::F(x) }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn builtin(name: &str, args: Vec<V>) -> Option<V> {
    let comps = |v: &V| -> Vec<f32> {
        match v {
            V::Vec(c) => c.iter().map(V::f).collect(),
            v => vec![v.f()],
        }
    };
    Some(match name {
        "floor" => each(&args, &|a| back(&a[0], a[0].f().floor())),
        "fract" => each(&args, &|a| back(&a[0], a[0].f() - a[0].f().floor())),
        "sqrt" => each(&args, &|a| back(&a[0], a[0].f().sqrt())),
        "abs" | "max" | "min" | "clamp" => each(&args, &|a| match (&a[0], name) {
            (V::I(_), _) => {
                let x: Vec<i32> = a.iter().map(V::i).collect();
                V::I(match name {
                    "abs" => x[0].wrapping_abs(),
                    "max" => x[0].max(x[1]),
                    "min" => x[0].min(x[1]),
                    _ => x[0].max(x[1]).min(x[2]),
                })
            }
            (V::U(_), _) => {
                let x: Vec<u32> = a.iter().map(V::u).collect();
                V::U(match name {
                    "abs" => x[0],
                    "max" => x[0].max(x[1]),
                    "min" => x[0].min(x[1]),
                    _ => x[0].max(x[1]).min(x[2]),
                })
            }
            _ => {
                let x = f32s(a);
                back(&a[0], match name {
                    "abs" => x[0].abs(),
                    "max" => x[0].max(x[1]),
                    "min" => x[0].min(x[1]),
                    _ => x[0].max(x[1]).min(x[2]),
                })
            }
        }),
        "smoothstep" => each(&args, &|a| {
            let x = f32s(a);
            back(&a[2], smoothstep(x[0], x[1], x[2]))
        }),
        "mix" => each(&args, &|a| {
            let x = f32s(a);
            back(&a[0], x[0] * (1.0 - x[2]) + x[1] * x[2])
        }),
        "select" => match &args[2] {
            V::B(c) => if *c { args[1].clone() } else { args[0].clone() },
            V::Vec(c) => V::Vec(c.iter().enumerate().map(|(k, c)| builtin("select", vec![part(&args[0], k), part(&args[1], k), c.clone()]).unwrap()).collect()),
            v => panic!("select on {v:?}"),
        },
        "length" => {
            let c = comps(&args[0]);
            V::F(c.iter().fold(0.0, |s, x| s + x * x).sqrt())
        }
        "dot" => {
            let (a, b) = (comps(&args[0]), comps(&args[1]));
            V::F(a.iter().zip(&b).fold(0.0, |s, (x, y)| s + x * y))
        }
        "normalize" => {
            let c = comps(&args[0]);
            let r = 1.0 / c.iter().fold(0.0, |s, x| s + x * x).sqrt();
            V::Vec(c.iter().map(|x| V::F(x * r)).collect())
        }
        "bitcast<u32>" => match args[0] {
            V::I(x) => V::U(x as u32),
            V::U(x) => V::U(x),
            V::F(x) => V::U(x.to_bits()),
            ref v => panic!("bitcast<u32> of {v:?}"),
        },
        "bitcast<i32>" => match args[0] {
            V::U(x) => V::I(x as i32),
            V::I(x) => V::I(x),
            V::F(x) => V::I(x.to_bits() as i32),
            ref v => panic!("bitcast<i32> of {v:?}"),
        },
        "f32" => V::F(match args[0] {
            V::I(x) => x as f32,
            V::U(x) => x as f32,
            ref v => v.f(),
        }),
        "i32" => V::I(match args[0] {
            V::U(x) => x as i32,
            V::I(x) => x,
            V::AI(x) => x as i32,
            ref v => v.f() as i32,
        }),
        "u32" => V::U(match args[0] {
            V::I(x) => x as u32,
            V::U(x) => x,
            V::AI(x) => x as u32,
            V::B(x) => x as u32,
            ref v => v.f() as u32,
        }),
        _ if name.starts_with("vec") => {
            let n: usize = name[3..4].parse().unwrap();
            let el = name.split_once('<').map(|(_, e)| e.trim_end_matches('>').to_string());
            let mut c: Vec<V> = args.into_iter().flat_map(|a| if let V::Vec(c) = a { c } else { vec![a] }).collect();
            if c.len() == 1 {
                c = vec![c[0].clone(); n];
            }
            assert_eq!(c.len(), n, "{name} of {c:?}");
            coerce(V::Vec(c), &el.map_or(format!("vec{n}"), |e| format!("vec{n}<{e}>")))
        }
        _ => return None,
    })
}

fn part(v: &V, k: usize) -> V {
    if let V::Vec(c) = v { c[k].clone() } else { v.clone() }
}

fn swizzle(k: char) -> usize {
    match k {
        'x' | 'r' => 0,
        'y' | 'g' => 1,
        'z' | 'b' => 2,
        'w' | 'a' => 3,
        _ => panic!("no part {k} of a vector"),
    }
}

// ------------------------------------------------------------------ running it

/// The shader's source, read: its structs and constants, and its functions,
/// each read the first time it is called.
pub struct Program {
    toks: Vec<Tok>,
    fns: HashMap<String, (usize, std::cell::OnceCell<Rc<Func>>)>,
    structs: HashMap<String, Vec<(String, String)>>,
    pub consts: HashMap<String, V>,
    /// stand-ins for the shader's uniforms, by name
    pub globals: HashMap<String, V>,
    /// stand-ins for the size of the shader's textures, by name
    pub textures: HashMap<String, [i32; 2]>,
}

enum Flow {
    Next,
    Break,
    Continue,
    Return(V),
}

impl Program {
    pub fn read(src: &str) -> Program {
        let toks = lex(src);
        let mut p = Program { toks: Vec::new(), fns: HashMap::new(), structs: HashMap::new(), consts: HashMap::new(), globals: HashMap::new(), textures: HashMap::new() };
        let mut consts = Vec::new();
        let (mut depth, mut k) = (0i32, 0usize);
        while k < toks.len() {
            match &toks[k] {
                Tok::P("{") => depth += 1,
                Tok::P("}") => depth -= 1,
                Tok::Id(w) if depth == 0 && w == "fn" => {
                    if let Some(Tok::Id(name)) = toks.get(k + 1) {
                        p.fns.insert(name.clone(), (k, Default::default()));
                    }
                }
                Tok::Id(w) if depth == 0 && w == "struct" => {
                    let mut ps = Parser { t: &toks, at: k + 1 };
                    let name = ps.id();
                    ps.want("{");
                    let mut fields = Vec::new();
                    while !ps.eat("}") {
                        ps.attributes();
                        let f = ps.id();
                        ps.want(":");
                        fields.push((f, ps.ty()));
                        ps.eat(",");
                    }
                    p.structs.insert(name, fields);
                    k = ps.at;
                    continue;
                }
                Tok::Id(w) if depth == 0 && w == "const" => {
                    let mut ps = Parser { t: &toks, at: k + 1 };
                    let name = ps.id();
                    ps.want(":");
                    let ty = ps.ty();
                    ps.want("=");
                    consts.push((name, ty, ps.expr()));
                    k = ps.at;
                    continue;
                }
                _ => {}
            }
            k += 1;
        }
        p.toks = toks;
        for (name, ty, e) in consts {
            let v = coerce(p.eval(&e, &mut vec![HashMap::new()]), &ty);
            p.consts.insert(name, v);
        }
        p
    }

    fn func(&self, name: &str) -> Option<Rc<Func>> {
        let (at, cell) = self.fns.get(name)?;
        Some(cell.get_or_init(|| Rc::new(Parser { t: &self.toks, at: *at }.func())).clone())
    }

    /// Call a function of the shader with these arguments.
    pub fn call(&self, name: &str, args: Vec<V>) -> V {
        let f = self.func(name).unwrap_or_else(|| panic!("the shader has no function {name}"));
        assert_eq!(f.params.len(), args.len(), "{name} takes {} arguments", f.params.len());
        let mut scope = HashMap::new();
        for ((p, ty), a) in f.params.iter().zip(args) {
            scope.insert(p.clone(), coerce(a, ty));
        }
        let mut scopes = vec![scope];
        match self.run(&f.body, &mut scopes) {
            Flow::Return(v) => f.ret.as_deref().map_or(v.clone(), |t| coerce(v, t)),
            _ => V::B(false),
        }
    }

    fn zero(&self, ty: &str) -> V {
        match ty {
            "f32" => V::F(0.0),
            "i32" => V::I(0),
            "u32" => V::U(0),
            "bool" => V::B(false),
            t if t.starts_with("vec") => {
                let n: usize = t[3..4].parse().unwrap();
                let el = t.split_once('<').map_or("f32", |(_, e)| e.trim_end_matches('>'));
                V::Vec(vec![self.zero(el); n])
            }
            t => {
                let fields = self.structs.get(t).unwrap_or_else(|| panic!("no struct {t}"));
                V::S(t.to_string(), fields.iter().map(|(n, ty)| (n.clone(), self.zero(ty))).collect())
            }
        }
    }

    fn run(&self, body: &[St], scopes: &mut Vec<HashMap<String, V>>) -> Flow {
        scopes.push(HashMap::new());
        let mut flow = Flow::Next;
        for s in body {
            flow = self.step(s, scopes);
            if !matches!(flow, Flow::Next) {
                break;
            }
        }
        scopes.pop();
        flow
    }

    fn step(&self, s: &St, scopes: &mut Vec<HashMap<String, V>>) -> Flow {
        match s {
            St::Let(name, ty, init) => {
                let v = match (ty, init) {
                    (Some(t), Some(e)) => coerce(self.eval(e, scopes), t),
                    (None, Some(e)) => concrete(self.eval(e, scopes)),
                    (Some(t), None) => self.zero(t),
                    (None, None) => panic!("var {name} with neither type nor value"),
                };
                scopes.last_mut().unwrap().insert(name.clone(), v);
            }
            St::Assign(place, op, e) => {
                let v = self.eval(e, scopes);
                let slot = Self::place(place, scopes);
                let v = if *op == "=" { v } else { bin(&op[..1], slot.clone(), v) };
                *slot = like(slot, v);
            }
            St::If(c, yes, no) => {
                return match self.eval(c, scopes) {
                    V::B(true) => self.run(yes, scopes),
                    V::B(false) => self.run(no, scopes),
                    v => panic!("if on {v:?}"),
                };
            }
            St::For(init, cond, update, body) => {
                scopes.push(HashMap::new());
                self.step(init, scopes);
                let out = loop {
                    if self.eval(cond, scopes) != V::B(true) {
                        break Flow::Next;
                    }
                    match self.run(body, scopes) {
                        Flow::Break => break Flow::Next,
                        Flow::Return(v) => break Flow::Return(v),
                        _ => {}
                    }
                    self.step(update, scopes);
                };
                scopes.pop();
                return out;
            }
            St::Loop(body) => loop {
                match self.run(body, scopes) {
                    Flow::Break => return Flow::Next,
                    Flow::Return(v) => return Flow::Return(v),
                    _ => {}
                }
            },
            St::Break => return Flow::Break,
            St::Continue => return Flow::Continue,
            St::Return(e) => return Flow::Return(e.as_ref().map_or(V::B(false), |e| self.eval(e, scopes))),
            St::Block(b) => return self.run(b, scopes),
        }
        Flow::Next
    }

    fn place<'a>(e: &Ex, scopes: &'a mut [HashMap<String, V>]) -> &'a mut V {
        match e {
            Ex::Var(name) => scopes.iter_mut().rev().find_map(|s| s.get_mut(name)).unwrap_or_else(|| panic!("no variable {name}")),
            Ex::Member(base, field) => match Self::place(base, scopes) {
                V::S(_, fs) => &mut fs.iter_mut().find(|(n, _)| n == field).unwrap_or_else(|| panic!("no field {field}")).1,
                V::Vec(c) => &mut c[swizzle(field.chars().next().unwrap())],
                v => panic!("{v:?} has no {field}"),
            },
            e => panic!("cannot assign to {e:?}"),
        }
    }

    fn eval(&self, e: &Ex, scopes: &mut Vec<HashMap<String, V>>) -> V {
        match e {
            Ex::Lit(v) => v.clone(),
            Ex::Var(name) => scopes
                .iter()
                .rev()
                .find_map(|s| s.get(name))
                .or_else(|| self.consts.get(name))
                .or_else(|| self.globals.get(name))
                .cloned()
                .unwrap_or_else(|| panic!("the shader's twin knows no {name}")),
            Ex::Neg(a) => match self.eval(a, scopes) {
                V::F(x) => V::F(-x),
                V::I(x) => V::I(x.wrapping_neg()),
                V::AI(x) => V::AI(-x),
                V::AF(x) => V::AF(-x),
                V::Vec(c) => V::Vec(c.into_iter().map(|x| bin("-", V::AI(0), x)).collect()),
                v => panic!("-{v:?}"),
            },
            Ex::Not(a) => match self.eval(a, scopes) {
                V::B(x) => V::B(!x),
                v => panic!("!{v:?}"),
            },
            Ex::Bin(op, a, b) => {
                let a = self.eval(a, scopes);
                // WGSL's && and || do not look at the second when the first decides
                match (*op, &a) {
                    ("&&", V::B(false)) => return V::B(false),
                    ("||", V::B(true)) => return V::B(true),
                    _ => {}
                }
                let b = self.eval(b, scopes);
                bin(op, a, b)
            }
            Ex::Member(base, field) => match self.eval(base, scopes) {
                V::S(_, fs) => fs.into_iter().find(|(n, _)| n == field).unwrap_or_else(|| panic!("no field {field}")).1,
                V::Vec(c) => {
                    let mut out: Vec<V> = field.chars().map(|k| c[swizzle(k)].clone()).collect();
                    if out.len() == 1 { out.pop().unwrap() } else { V::Vec(out) }
                }
                v => panic!("{v:?} has no {field}"),
            },
            Ex::Call(name, args) => {
                if name == "textureDimensions" {
                    let Ex::Var(t) = &args[0] else { panic!("textureDimensions of {:?}", args[0]) };
                    let d = self.textures.get(t).unwrap_or_else(|| panic!("no stand-in for the texture {t}"));
                    return V::Vec(vec![V::U(d[0] as u32), V::U(d[1] as u32)]);
                }
                let vals: Vec<V> = args.iter().map(|a| self.eval(a, scopes)).collect();
                if let Some(fields) = self.structs.get(name) {
                    return V::S(name.clone(), fields.iter().zip(vals).map(|((n, ty), v)| (n.clone(), coerce(v, ty))).collect());
                }
                if self.fns.contains_key(name) {
                    return self.call(name, vals);
                }
                builtin(name, vals).unwrap_or_else(|| panic!("the shader's twin knows no function {name}"))
            }
        }
    }
}

// ------------------------------------------------------------------ the tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields;
    use crate::world;
    use bevy::math::Vec2;

    fn shader() -> Program {
        Program::read(SOURCE)
    }

    fn v2(p: Vec2) -> V {
        V::Vec(vec![V::F(p.x), V::F(p.y)])
    }

    fn block(b: &fields::Block) -> V {
        V::S("Block".into(), vec![("i".into(), V::I(b.i)), ("j".into(), V::I(b.j)), ("uv".into(), v2(b.uv)), ("a".into(), v2(b.a)), ("b".into(), v2(b.b)), ("c".into(), v2(b.c)), ("d".into(), v2(b.d))])
    }

    fn cut(c: &fields::Cut) -> V {
        V::S("Cut".into(), vec![("lo".into(), v2(c.lo)), ("hi".into(), v2(c.hi)), ("id".into(), V::U(c.id))])
    }

    /// Spots over an island of 64 cells by 64 and the sea round it, the same
    /// every run: a few on whole and half cells, where floors and edges
    /// fall, and the rest scattered.
    fn spots(n: usize) -> Vec<Vec2> {
        let mut out: Vec<Vec2> = (0..n / 10).map(|k| Vec2::new((k % 9) as f32 * 7.5 - 4.0, (k / 9) as f32 * 0.5 + (k % 3) as f32 * 9.0)).collect();
        out.extend((0..n - out.len()).map(|k| Vec2::new(fields::hash3(k as u32, 1, 9) * 76.0 - 6.0, fields::hash3(k as u32, 2, 9) * 76.0 - 6.0)));
        out
    }

    fn close(what: &str, at: Vec2, gpu: f32, cpu: f32, tol: f32) {
        assert!((gpu - cpu).abs() <= tol, "{what} at ({}, {}): the shader gives {gpu} and the Rust {cpu}: the two have drifted apart", at.x, at.y);
    }

    #[test]
    fn the_shaders_constants_are_the_rusts() {
        let p = shader();
        let k = |n: &str| p.consts.get(n).unwrap_or_else(|| panic!("the shader has no const {n}")).clone();
        for (name, class) in [
            ("SEA", world::SEA),
            ("FARM", world::FARM),
            ("PASTURE", world::PASTURE),
            ("WOOD", world::WOOD),
            ("HEATH", world::HEATH),
            ("QUARRY", world::QUARRY),
            ("CLAYPIT", world::CLAYPIT),
            ("IRONMINE", world::IRONMINE),
            ("COALMINE", world::COALMINE),
            ("WORKS", world::WORKS),
            ("URBAN", world::URBAN),
            ("DERELICT", world::DERELICT),
        ] {
            assert_eq!(k(name), V::U(class as u32), "the class {name}");
        }
        assert_eq!(k("BLOCK"), V::F(fields::BLOCK));
        assert_eq!(k("TURN_C"), V::F(fields::TURN_C));
        assert_eq!(k("TURN_S"), V::F(fields::TURN_S));
        assert_eq!(k("PARCEL_LEVEL"), V::U(fields::PARCEL_LEVEL));
    }

    #[test]
    fn the_hashes_and_the_noise_agree_bit_for_bit() {
        let p = shader();
        for k in 0..600u32 {
            let (a, b, s) = ((fields::hash3(k, 3, 1) * 4000.0) as i32 - 2000, (fields::hash3(k, 4, 1) * 4000.0) as i32 - 2000, (fields::hash3(k, 5, 1) * 600.0) as i32);
            assert_eq!(p.call("hash2", vec![V::I(a), V::I(b), V::I(s)]), V::F(world::hash2(a, b, s)), "hash2({a}, {b}, {s})");
            let (ua, ub, uc) = (a as u32, b.wrapping_mul(7919) as u32, s as u32 + 150);
            assert_eq!(p.call("hash3", vec![V::U(ua), V::U(ub), V::U(uc)]), V::F(fields::hash3(ua, ub, uc)), "hash3({ua}, {ub}, {uc})");
        }
        for (k, at) in spots(600).into_iter().enumerate() {
            let s = [5, 31, 141, 461, 465, 435][k % 6];
            let x = at * (1.0 + (k % 7) as f32);
            assert_eq!(p.call("vnoise", vec![V::F(x.x), V::F(x.y), V::I(s)]), V::F(world::vnoise(x.x, x.y, s)), "vnoise({}, {}, {s})", x.x, x.y);
        }
    }

    /// The field system: the warp into block space, the block a spot lies
    /// in, its parcel and its fields at every depth, the distance to a
    /// field's side and the field's number.
    #[test]
    fn the_shader_cuts_the_fields_where_the_rust_does() {
        let p = shader();
        for at in spots(1500) {
            let q = fields::to_block(at);
            let gq = p.call("to_block", vec![v2(at)]).v2();
            assert_eq!(gq, [q.x, q.y], "to_block at ({}, {}): the shader and the Rust have drifted apart", at.x, at.y);
            let back = p.call("from_block", vec![v2(q)]).v2();
            let rb = fields::from_block(q);
            assert_eq!(back, [rb.x, rb.y], "from_block at ({}, {})", at.x, at.y);
            let bl = fields::locate(q);
            let gb = p.call("locate", vec![v2(q)]);
            assert_eq!(gb, block(&bl), "locate at ({}, {}): the shader puts the spot in another block or elsewhere in it", at.x, at.y);
            assert_eq!(p.call("corner", vec![V::I(bl.i), V::I(bl.j)]).v2(), fields::corner(bl.i, bl.j).to_array(), "corner({}, {})", bl.i, bl.j);
            let whole = fields::Cut { lo: Vec2::ZERO, hi: Vec2::ONE, id: 1 };
            let pc = fields::cut(&bl, whole, 0, fields::PARCEL_LEVEL);
            assert_eq!(p.call("cut", vec![block(&bl), cut(&whole), V::U(0), V::U(fields::PARCEL_LEVEL)]), cut(&pc), "the parcel at ({}, {})", at.x, at.y);
            for depth in fields::PARCEL_LEVEL..=fields::MAX_DEPTH {
                let f = fields::field(&bl, pc, depth);
                let g = p.call("cut", vec![block(&bl), cut(&pc), V::U(fields::PARCEL_LEVEL), V::U(depth)]);
                assert_eq!(g, cut(&f.cut), "the field of depth {depth} at ({}, {})", at.x, at.y);
                close("the distance to a field's side", at, p.call("cut_edge", vec![block(&bl), g]).f(), f.edge, 1e-6);
                assert_eq!(p.call("field_key", vec![block(&bl), cut(&f.cut)]), V::U(fields::field_key(&f)), "field_key at ({}, {})", at.x, at.y);
            }
        }
    }

    #[test]
    fn the_quads_are_inverted_alike() {
        let p = shader();
        for (k, at) in spots(400).into_iter().enumerate() {
            let (i, j) = ((k % 20) as i32 - 10, (k / 20) as i32 - 10);
            let (a, b, c, d) = (fields::corner(i, j), fields::corner(i + 1, j), fields::corner(i + 1, j + 1), fields::corner(i, j + 1));
            let q = Vec2::new(i as f32, j as f32) + (at - at.floor()) * 1.2 - Vec2::splat(0.1);
            let uv = fields::inv_bilinear(q, a, b, c, d);
            assert_eq!(p.call("inv_bilinear", vec![v2(q), v2(a), v2(b), v2(c), v2(d)]).v2(), uv.to_array(), "inv_bilinear in block ({i}, {j})");
            assert_eq!(p.call("bilinear", vec![v2(a), v2(b), v2(c), v2(d), v2(uv)]).v2(), fields::bilinear(a, b, c, d, uv).to_array(), "bilinear in block ({i}, {j})");
        }
    }

    /// Where wood and heath meet the farmed land: the warp and the fraying
    /// of their edges, the summer snow line, how a wood deepens from its
    /// edge and the glades in it.
    #[test]
    fn the_edges_of_wood_and_heath_are_drawn_where_the_trees_stand() {
        let p = shader();
        for (k, at) in spots(1200).into_iter().enumerate() {
            let (wx, wz) = world::edge_warp(at.x, at.y);
            assert_eq!(p.call("edge_warp", vec![v2(at)]).v2(), [wx, wz], "edge_warp at ({}, {}): the shader and the Rust have drifted apart", at.x, at.y);
            close("edge_fray", at, p.call("edge_fray", vec![v2(at)]).f(), world::edge_fray(at.x, at.y), 1e-6);
            close("summer_snow_line", at, p.call("summer_snow_line", vec![v2(at)]).f(), world::summer_snow_line(at.x, at.y), 1e-6);
            let w = (k % 41) as f32 / 30.0 - 0.15;
            close("wood_depth", at, p.call("wood_depth", vec![V::F(w)]).f(), world::wood_depth(w), 1e-6);
            let deep = (k % 11) as f32 / 10.0;
            // the shader's mix and the Rust's sum round differently in the last place
            close("glade", at, p.call("glade", vec![v2(at), V::F(deep)]).f(), world::glade(at.x, at.y, deep), 1e-4);
        }
    }

    /// The shader finds a parcel's texel where `Land::index` numbers it, for
    /// every block of the grid. (Beyond the grid the two part: `Land::index`
    /// keeps the parcel's number in the nearest block and the shader takes
    /// the nearest texel. Those blocks lie off the map in the sea.)
    #[test]
    fn the_shader_reads_each_parcel_where_the_rust_writes_it() {
        let mut p = shader();
        let (bi0, bj0, bw, bh) = (-7, -4, 23usize, 19usize);
        let land = world::Land { bi0, bj0, bw, bh, ..Default::default() };
        p.textures.insert("year_parcels".into(), [(bw * 8) as i32, bh as i32]);
        let mut ground = p.zero("PatchworkParams");
        if let V::S(_, fs) = &mut ground {
            for (n, v) in fs.iter_mut() {
                match n.as_str() {
                    "block_i0" => *v = V::I(bi0),
                    "block_j0" => *v = V::I(bj0),
                    _ => {}
                }
            }
        }
        p.globals.insert("ground".into(), ground);
        for bj in bj0..bj0 + bh as i32 {
            for bi in bi0..bi0 + bw as i32 {
                for id in 8..16u32 {
                    let t = p.call("parcel_texel", vec![V::I(bi), V::I(bj), V::U(id)]);
                    let [x, y] = [t.v2i()[0] as usize, t.v2i()[1] as usize];
                    assert_eq!(y * bw * 8 + x, land.index(bi, bj, id), "parcel {id} of block ({bi}, {bj})");
                }
            }
        }
    }

    /// The reader itself: changes to the shader that it is meant to catch are
    /// caught. In a copy of the source a number of the field system's warp is
    /// changed, and a step of its cutting, and each copy no longer agrees with
    /// the Rust. (In the planned country the warp is nothing, so not every
    /// spot moves.)
    #[test]
    fn a_change_on_one_side_is_caught() {
        for (was, now) in [("vnoise(q.x * 1.1, q.y * 1.1, 143)", "vnoise(q.x * 1.2, q.y * 1.1, 143)"), ("if (k < 0.12) {", "if (k < 0.13) {"), ("c.id = c.id * 2u + 1u;", "c.id = c.id * 2u;")] {
            let changed = SOURCE.replacen(was, now, 1);
            assert_ne!(changed, SOURCE, "the shader no longer has {was:?}, which this test changes");
            let p = Program::read(&changed);
            let differ = spots(200)
                .into_iter()
                .filter(|&at| {
                    let bl = fields::locate(fields::to_block(at));
                    let whole = fields::Cut { lo: Vec2::ZERO, hi: Vec2::ONE, id: 1 };
                    let f = fields::field(&bl, fields::cut(&bl, whole, 0, fields::PARCEL_LEVEL), fields::MAX_DEPTH);
                    let g = p.call("cut", vec![p.call("locate", vec![p.call("to_block", vec![v2(at)])]), cut(&whole), V::U(0), V::U(fields::MAX_DEPTH)]);
                    g != cut(&f.cut)
                })
                .count();
            assert!(differ > 0, "with {now:?} in place of {was:?} the shader still cuts every field where the Rust does");
        }
    }

    impl V {
        fn v2i(&self) -> [i32; 2] {
            match self {
                V::Vec(c) if c.len() == 2 => [c[0].i(), c[1].i()],
                v => panic!("{v:?} is not a vec2<i32>"),
            }
        }
    }
}
