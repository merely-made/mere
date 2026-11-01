//! Route A (D12, D17): a small generator of our own over an exactly pinned
//! `wit-parser`. It binds one WIT interface for native code: plain types, a
//! blocking client over any `Transport`, and a `Handler` trait with a
//! dispatcher. Values are written in wRPC's value encoding (D15) by the
//! runtime in `c0-route-a`; calls travel under wRPC's names (`open`,
//! `session.snapshot`, `[drop]session`); resources travel as opaque handle
//! bytes.
//!
//! Covers what the projection session uses: records, variants, options,
//! results, tuples, lists, strings, integers, type aliases and one level of
//! resources with methods. Anything else is refused, not guessed.

use std::fmt::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use heck::{ToSnakeCase, ToUpperCamelCase};
use wit_parser::{Function, FunctionKind, Handle, InterfaceId, Resolve, Type, TypeDefKind, TypeId};

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "static", "struct", "trait", "true", "try", "type", "unsafe", "use", "where",
    "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
    "unsized", "virtual", "yield",
];

/// A snake-case Rust identifier, escaped where it is a keyword. `self`,
/// `super` and `crate` cannot be raw, so they take a trailing `_`.
fn ident(name: &str) -> String {
    let snake = name.to_snake_case();
    match snake.as_str() {
        "self" | "super" | "crate" => format!("{snake}_"),
        s if KEYWORDS.contains(&s) => format!("r#{snake}"),
        _ => snake,
    }
}

/// An upper-camel Rust type name; `Self` takes a trailing `_`.
fn type_name(name: &str) -> String {
    let camel = name.to_upper_camel_case();
    if camel == "Self" {
        format!("{camel}_")
    } else {
        camel
    }
}

/// Generate Rust for `interface` from the WIT file at `wit`.
pub fn generate(wit: &Path, interface: &str) -> Result<String> {
    let mut resolve = Resolve::default();
    let package = resolve.push_file(wit)?;
    let id = *resolve.packages[package]
        .interfaces
        .get(interface)
        .with_context(|| format!("no interface `{interface}`"))?;
    generate_interface(&resolve, id)
}

/// Generate Rust for one interface of an already-resolved package.
pub fn generate_interface(resolve: &Resolve, id: InterfaceId) -> Result<String> {
    Gen {
        resolve,
        id,
        out: String::new(),
    }
    .run()
}

struct Gen<'a> {
    resolve: &'a Resolve,
    id: InterfaceId,
    out: String,
}

impl Gen<'_> {
    fn run(mut self) -> Result<String> {
        let iface = &self.resolve.interfaces[self.id];
        for (name, &ty) in &iface.types {
            self.typedef(name, ty)?;
        }
        let functions: Vec<&Function> = iface.functions.values().collect();
        self.handler(&functions)?;
        self.client(&functions)?;
        Ok(self.out)
    }

    fn ty(&self, ty: &Type) -> Result<String> {
        Ok(match ty {
            Type::Bool => "bool".into(),
            Type::U8 => "u8".into(),
            Type::U32 => "u32".into(),
            Type::U64 => "u64".into(),
            Type::String => "String".into(),
            Type::Id(id) => {
                let def = &self.resolve.types[*id];
                if let Some(name) = &def.name {
                    if def.owner != wit_parser::TypeOwner::Interface(self.id) {
                        bail!("unsupported type used from another interface");
                    }
                    return Ok(type_name(name));
                }
                self.anon(&def.kind)?
            },
            other => bail!("unsupported type {other:?}"),
        })
    }

    fn anon(&self, kind: &TypeDefKind) -> Result<String> {
        Ok(match kind {
            TypeDefKind::List(Type::U8) => "Vec<u8>".into(),
            TypeDefKind::List(t) => format!("Vec<{}>", self.ty(t)?),
            TypeDefKind::Option(t) => format!("Option<{}>", self.ty(t)?),
            TypeDefKind::Result(r) => {
                format!("Result<{}, {}>", self.opt(&r.ok)?, self.opt(&r.err)?)
            },
            TypeDefKind::Tuple(t) if !(1..=6).contains(&t.types.len()) => {
                bail!("unsupported tuple of {}", t.types.len())
            },
            TypeDefKind::Tuple(t) => {
                let parts: Result<Vec<_>> = t.types.iter().map(|t| self.ty(t)).collect();
                format!("({},)", parts?.join(", "))
            },
            TypeDefKind::Handle(Handle::Own(r) | Handle::Borrow(r)) => self.resolve.types[*r]
                .name
                .as_deref()
                .map(type_name)
                .context("unnamed resource")?,
            TypeDefKind::Type(t) => self.ty(t)?,
            other => bail!("unsupported anonymous type {other:?}"),
        })
    }

    fn opt(&self, ty: &Option<Type>) -> Result<String> {
        ty.as_ref().map_or(Ok("()".into()), |t| self.ty(t))
    }

    fn typedef(&mut self, name: &str, id: TypeId) -> Result<()> {
        let rust = type_name(name);
        let out = &mut String::new();
        match &self.resolve.types[id].kind {
            kind @ (TypeDefKind::Type(_)
            | TypeDefKind::List(_)
            | TypeDefKind::Option(_)
            | TypeDefKind::Result(_)
            | TypeDefKind::Tuple(_)) => writeln!(out, "pub type {rust} = {};", self.anon(kind)?)?,
            TypeDefKind::Resource => {
                writeln!(
                    out,
                    "/// A `{name}` handle: opaque bytes the provider minted.\n\
                     #[derive(Clone, Debug, PartialEq, Eq)]\n\
                     pub struct {rust}(pub Vec<u8>);\n\
                     impl Wire for {rust} {{\n\
                     \x20   fn put(&self, out: &mut Vec<u8>) {{ self.0.put(out) }}\n\
                     \x20   fn take(input: &mut &[u8]) -> std::io::Result<Self> {{ Ok(Self(Wire::take(input)?)) }}\n\
                     }}"
                )?;
            },
            TypeDefKind::Record(r) => {
                writeln!(
                    out,
                    "#[derive(Clone, Debug, PartialEq, Eq)]\npub struct {rust} {{"
                )?;
                for f in &r.fields {
                    writeln!(out, "    pub {}: {},", ident(&f.name), self.ty(&f.ty)?)?;
                }
                writeln!(
                    out,
                    "}}\nimpl Wire for {rust} {{\n    fn put(&self, out: &mut Vec<u8>) {{"
                )?;
                for f in &r.fields {
                    writeln!(out, "        self.{}.put(out);", ident(&f.name))?;
                }
                writeln!(
                    out,
                    "    }}\n    fn take(input: &mut &[u8]) -> std::io::Result<Self> {{\n        Ok(Self {{"
                )?;
                for f in &r.fields {
                    writeln!(out, "            {}: Wire::take(input)?,", ident(&f.name))?;
                }
                writeln!(out, "        }})\n    }}\n}}")?;
            },
            TypeDefKind::Variant(v) => {
                writeln!(
                    out,
                    "#[derive(Clone, Debug, PartialEq, Eq)]\npub enum {rust} {{"
                )?;
                for c in &v.cases {
                    let case = type_name(&c.name);
                    match &c.ty {
                        Some(t) => writeln!(out, "    {case}({}),", self.ty(t)?)?,
                        None => writeln!(out, "    {case},")?,
                    }
                }
                writeln!(
                    out,
                    "}}\nimpl Wire for {rust} {{\n    fn put(&self, out: &mut Vec<u8>) {{\n        match self {{"
                )?;
                for (i, c) in v.cases.iter().enumerate() {
                    let case = type_name(&c.name);
                    match &c.ty {
                        Some(_) => writeln!(
                            out,
                            "            Self::{case}(v) => {{ put_disc(out, {i}); v.put(out) }},"
                        )?,
                        None => writeln!(out, "            Self::{case} => put_disc(out, {i}),")?,
                    }
                }
                writeln!(
                    out,
                    "        }}\n    }}\n    fn take(input: &mut &[u8]) -> std::io::Result<Self> {{\n        Ok(match take_disc(input)? {{"
                )?;
                for (i, c) in v.cases.iter().enumerate() {
                    let case = type_name(&c.name);
                    match &c.ty {
                        Some(_) => {
                            writeln!(out, "            {i} => Self::{case}(Wire::take(input)?),")?
                        },
                        None => writeln!(out, "            {i} => Self::{case},")?,
                    }
                }
                writeln!(
                    out,
                    "            d => return Err(bad(format!(\"unknown {name} case {{d}}\"))),\n        }})\n    }}\n}}"
                )?;
            },
            other => bail!("unsupported type definition `{name}`: {other:?}"),
        }
        self.out.push_str(out);
        Ok(())
    }

    /// `(rust name, receiver resource, params after the receiver)`.
    fn shape<'f>(
        &self,
        f: &'f Function,
    ) -> Result<(String, Option<String>, &'f [wit_parser::Param])> {
        match &f.kind {
            FunctionKind::Freestanding => Ok((ident(&f.name), None, &f.params)),
            FunctionKind::Method(r) => {
                let res = self.resolve.types[*r]
                    .name
                    .clone()
                    .context("unnamed resource")?;
                let method = f.name.rsplit('.').next().unwrap();
                Ok((
                    ident(&format!("{res}-{method}")),
                    Some(type_name(&res)),
                    &f.params[1..],
                ))
            },
            other => bail!("unsupported function kind {other:?} for `{}`", f.name),
        }
    }

    /// The name a call travels under, as wRPC names it.
    fn key(f: &Function) -> &str {
        f.name.strip_prefix("[method]").unwrap_or(&f.name)
    }

    /// `(rust type, wire key)` for each resource.
    fn resources(&self) -> Vec<(String, String)> {
        self.resolve.interfaces[self.id]
            .types
            .iter()
            .filter(|(_, id)| matches!(self.resolve.types[**id].kind, TypeDefKind::Resource))
            .map(|(n, _)| (type_name(n), format!("[drop]{n}")))
            .collect()
    }

    fn handler(&mut self, functions: &[&Function]) -> Result<()> {
        let mut trait_src = String::from("/// The provider side.\npub trait Handler {\n");
        let mut arms = String::new();
        for f in functions {
            let key = Self::key(f);
            let (name, receiver, params) = self.shape(f)?;
            let ret = self.opt(&f.result)?;
            let mut args = String::new();
            let mut call = String::new();
            arms.push_str(&format!("        {key:?} => {{\n"));
            if let Some(r) = &receiver {
                args.push_str(&format!(", this: &{r}"));
                arms.push_str(&format!(
                    "            let this: {r} = Wire::take(input)?;\n"
                ));
                call.push_str("&this, ");
            }
            for p in params {
                let n = ident(&p.name);
                let t = self.ty(&p.ty)?;
                args.push_str(&format!(", {n}: {t}"));
                arms.push_str(&format!("            let {n}: {t} = Wire::take(input)?;\n"));
                call.push_str(&format!("{n}, "));
            }
            trait_src.push_str(&format!("    fn {name}(&mut self{args}) -> {ret};\n"));
            arms.push_str(&format!(
                "            handler.{name}({call}).put(&mut out);\n        }},\n"
            ));
        }
        for (r, key) in self.resources() {
            let snake = ident(&r);
            trait_src.push_str(&format!("    fn drop_{snake}(&mut self, this: {r});\n"));
            arms.push_str(&format!(
                "        {key:?} => handler.drop_{snake}(Wire::take(input)?),\n"
            ));
        }
        trait_src.push_str("}\n");
        self.out.push_str(&trait_src);
        writeln!(
            self.out,
            "/// Answer one request frame: the call's name, then its arguments.\n\
             pub fn dispatch<H: Handler>(handler: &mut H, mut frame: &[u8]) -> std::io::Result<Vec<u8>> {{\n\
             \x20   let input = &mut frame;\n\
             \x20   let mut out = Vec::new();\n\
             \x20   match String::take(input)?.as_str() {{\n{arms}\
             \x20       f => return Err(bad(format!(\"unknown function `{{f}}`\"))),\n\
             \x20   }}\n\
             \x20   if !input.is_empty() {{ return Err(bad(\"trailing bytes\".into())); }}\n\
             \x20   Ok(out)\n\
             }}"
        )?;
        Ok(())
    }

    fn client(&mut self, functions: &[&Function]) -> Result<()> {
        let mut src = String::from(
            "/// The consumer side: one blocking call per function.\n\
             pub struct Client<T: Transport>(pub T);\n\
             impl<T: Transport> Client<T> {\n",
        );
        for f in functions {
            let key = Self::key(f);
            let (name, receiver, params) = self.shape(f)?;
            let ret = self.opt(&f.result)?;
            let mut args = String::new();
            let mut body = format!(
                "        let mut out = Vec::new();\n        String::from({key:?}).put(&mut out);\n"
            );
            if let Some(r) = &receiver {
                args.push_str(&format!(", this: &{r}"));
                body.push_str("        this.put(&mut out);\n");
            }
            for p in params {
                let n = ident(&p.name);
                args.push_str(&format!(", {n}: {}", self.ty(&p.ty)?));
                body.push_str(&format!("        {n}.put(&mut out);\n"));
            }
            src.push_str(&format!(
                "    pub fn {name}(&mut self{args}) -> std::io::Result<{ret}> {{\n{body}        \
                 let reply = self.0.call(out)?;\n        finish(&reply)\n    }}\n"
            ));
        }
        for (r, key) in self.resources() {
            src.push_str(&format!(
                "    pub fn drop_{}(&mut self, this: {r}) -> std::io::Result<()> {{\n        \
                 let mut out = Vec::new();\n        String::from({key:?}).put(&mut out);\n        \
                 this.put(&mut out);\n        let reply = self.0.call(out)?;\n        finish(&reply)\n    }}\n",
                ident(&r)
            ));
        }
        src.push_str("}\n");
        self.out.push_str(&src);
        Ok(())
    }
}
