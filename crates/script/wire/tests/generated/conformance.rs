pub mod test_conformance_shared {
    use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Green,
    Blue,
}
impl Wire for Color {
    fn put(&self, out: &mut Vec<u8>) { put_disc(out, *self as u32) }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> {
        Ok(match take_disc(input)? {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            d => return Err(bad(format!("unknown color case {d}"))),
        })
    }
}
/// A set of `perms` flags; bit `i` is the `i`th flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Perms(pub u64);
impl Perms {
    pub const A: Self = Self(1 << 0);
    pub const B: Self = Self(1 << 1);
    pub const C: Self = Self(1 << 2);
    pub const D: Self = Self(1 << 3);
    pub const E: Self = Self(1 << 4);
    pub const F: Self = Self(1 << 5);
    pub const G: Self = Self(1 << 6);
    pub const H: Self = Self(1 << 7);
    pub const I: Self = Self(1 << 8);
    pub const J: Self = Self(1 << 9);
    pub fn contains(self, other: Self) -> bool { self.0 & other.0 == other.0 }
}
impl Wire for Perms {
    fn put(&self, out: &mut Vec<u8>) { put_flags(out, self.0, 2) }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> { take_flags(input, 10).map(Self) }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Numbers {
    pub a: i8,
    pub b: i16,
    pub c: i32,
    pub d: i64,
    pub e: u16,
    pub f: f32,
    pub g: f64,
    pub h: char,
}
impl Wire for Numbers {
    fn put(&self, out: &mut Vec<u8>) {
        self.a.put(out);
        self.b.put(out);
        self.c.put(out);
        self.d.put(out);
        self.e.put(out);
        self.f.put(out);
        self.g.put(out);
        self.h.put(out);
    }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> {
        Ok(Self {
            a: Wire::take(input)?,
            b: Wire::take(input)?,
            c: Wire::take(input)?,
            d: Wire::take(input)?,
            e: Wire::take(input)?,
            f: Wire::take(input)?,
            g: Wire::take(input)?,
            h: Wire::take(input)?,
        })
    }
}
/// A `token` handle: opaque bytes the provider minted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(pub Vec<u8>);
impl Wire for Token {
    fn put(&self, out: &mut Vec<u8>) { self.0.put(out) }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> { Ok(Self(Wire::take(input)?)) }
}
}
pub type Color = test_conformance_shared::Color;
pub type Perms = test_conformance_shared::Perms;
pub type Numbers = test_conformance_shared::Numbers;
pub type Token = test_conformance_shared::Token;
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u64,
    pub r#type: String,
    pub r#match: bool,
}
impl Wire for Point {
    fn put(&self, out: &mut Vec<u8>) {
        self.x.put(out);
        self.y.put(out);
        self.r#type.put(out);
        self.r#match.put(out);
    }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> {
        Ok(Self {
            x: Wire::take(input)?,
            y: Wire::take(input)?,
            r#type: Wire::take(input)?,
            r#match: Wire::take(input)?,
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Empty,
    Dot(Point),
    Bytes(Vec<u8>),
    Many(Vec<Point>),
    Named(String),
    Nested(Option<Result<u64, String>>),
    Painted(Color),
}
impl Wire for Shape {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Empty => put_disc(out, 0),
            Self::Dot(v) => { put_disc(out, 1); v.put(out) },
            Self::Bytes(v) => { put_disc(out, 2); v.put(out) },
            Self::Many(v) => { put_disc(out, 3); v.put(out) },
            Self::Named(v) => { put_disc(out, 4); v.put(out) },
            Self::Nested(v) => { put_disc(out, 5); v.put(out) },
            Self::Painted(v) => { put_disc(out, 6); v.put(out) },
        }
    }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> {
        Ok(match take_disc(input)? {
            0 => Self::Empty,
            1 => Self::Dot(Wire::take(input)?),
            2 => Self::Bytes(Wire::take(input)?),
            3 => Self::Many(Wire::take(input)?),
            4 => Self::Named(Wire::take(input)?),
            5 => Self::Nested(Wire::take(input)?),
            6 => Self::Painted(Wire::take(input)?),
            d => return Err(bad(format!("unknown shape case {d}"))),
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Bag {
    pub flag: bool,
    pub small: u8,
    pub shapes: Vec<Shape>,
    pub pair: (u32, String,),
    pub triple: (u8, Vec<u8>, Option<u64>,),
    pub maybe: Option<Option<u8>>,
    pub outcome: Result<(), Shape>,
    pub ok_only: Result<Point, ()>,
    pub colors: Vec<Color>,
    pub perms: Perms,
    pub numbers: Vec<Numbers>,
    pub nine: (u8, u8, u8, u8, u8, u8, u8, u8, char,),
    pub token: Option<Token>,
}
impl Wire for Bag {
    fn put(&self, out: &mut Vec<u8>) {
        self.flag.put(out);
        self.small.put(out);
        self.shapes.put(out);
        self.pair.put(out);
        self.triple.put(out);
        self.maybe.put(out);
        self.outcome.put(out);
        self.ok_only.put(out);
        self.colors.put(out);
        self.perms.put(out);
        self.numbers.put(out);
        self.nine.put(out);
        self.token.put(out);
    }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> {
        Ok(Self {
            flag: Wire::take(input)?,
            small: Wire::take(input)?,
            shapes: Wire::take(input)?,
            pair: Wire::take(input)?,
            triple: Wire::take(input)?,
            maybe: Wire::take(input)?,
            outcome: Wire::take(input)?,
            ok_only: Wire::take(input)?,
            colors: Wire::take(input)?,
            perms: Wire::take(input)?,
            numbers: Wire::take(input)?,
            nine: Wire::take(input)?,
            token: Wire::take(input)?,
        })
    }
}
/// A `handle` handle: opaque bytes the provider minted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handle(pub Vec<u8>);
impl Wire for Handle {
    fn put(&self, out: &mut Vec<u8>) { self.0.put(out) }
    fn take(input: &mut &[u8]) -> std::io::Result<Self> { Ok(Self(Wire::take(input)?)) }
}
/// The provider side.
pub trait Handler {
    fn handle_new(&mut self, seed: u32) -> Handle;
    fn handle_make(&mut self, name: String) -> Handle;
    fn handle_type(&mut self, this: &Handle, r#match: u32) -> u32;
    fn echo(&mut self, value: Bag) -> Bag;
    fn r#type(&mut self, r#loop: u32) -> Result<Handle, Shape>;
    fn drop_handle(&mut self, this: Handle);
}
/// Answer one request frame: the call's name, then its arguments.
pub fn dispatch<H: Handler>(handler: &mut H, mut frame: &[u8]) -> std::io::Result<Vec<u8>> {
    let input = &mut frame;
    let mut out = Vec::new();
    match String::take(input)?.as_str() {
        "handle" => {
            let seed: u32 = Wire::take(input)?;
            handler.handle_new(seed, ).put(&mut out);
        },
        "handle.make" => {
            let name: String = Wire::take(input)?;
            handler.handle_make(name, ).put(&mut out);
        },
        "handle.type" => {
            let this: Handle = Wire::take(input)?;
            let r#match: u32 = Wire::take(input)?;
            handler.handle_type(&this, r#match, ).put(&mut out);
        },
        "echo" => {
            let value: Bag = Wire::take(input)?;
            handler.echo(value, ).put(&mut out);
        },
        "type" => {
            let r#loop: u32 = Wire::take(input)?;
            handler.r#type(r#loop, ).put(&mut out);
        },
        "[drop]handle" => handler.drop_handle(Wire::take(input)?),
        f => return Err(bad(format!("unknown function `{f}`"))),
    }
    if !input.is_empty() { return Err(bad("trailing bytes".into())); }
    Ok(out)
}
/// The consumer side: one blocking call per function.
pub struct Client<T: Transport>(pub T);
impl<T: Transport> Client<T> {
    pub fn handle_new(&mut self, seed: u32) -> std::io::Result<Handle> {
        let mut out = Vec::new();
        String::from("handle").put(&mut out);
        seed.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
    pub fn handle_make(&mut self, name: String) -> std::io::Result<Handle> {
        let mut out = Vec::new();
        String::from("handle.make").put(&mut out);
        name.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
    pub fn handle_type(&mut self, this: &Handle, r#match: u32) -> std::io::Result<u32> {
        let mut out = Vec::new();
        String::from("handle.type").put(&mut out);
        this.put(&mut out);
        r#match.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
    pub fn echo(&mut self, value: Bag) -> std::io::Result<Bag> {
        let mut out = Vec::new();
        String::from("echo").put(&mut out);
        value.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
    pub fn r#type(&mut self, r#loop: u32) -> std::io::Result<Result<Handle, Shape>> {
        let mut out = Vec::new();
        String::from("type").put(&mut out);
        r#loop.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
    pub fn drop_handle(&mut self, this: Handle) -> std::io::Result<()> {
        let mut out = Vec::new();
        String::from("[drop]handle").put(&mut out);
        this.put(&mut out);
        let reply = self.0.call(out)?;
        finish(&reply)
    }
}
