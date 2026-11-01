//! Route A (D12): a small generator of our own over an exactly pinned
//! `wit-parser`. It binds one WIT interface for native code: plain types, a
//! blocking client over any `Transport`, and a `Handler` trait with a
//! dispatcher. Values are written in wRPC's value encoding (D15) by the
//! runtime in `c0-route-a`; resources travel as opaque handle bytes.
//!
//! Covers what the projection session uses: records, variants, options,
//! results, tuples, lists, strings, integers, type aliases and one level of
//! resources with methods. Anything else is refused, not guessed.

use std::fmt::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use heck::{ToSnakeCase, ToUpperCamelCase};
use wit_parser::{Function, FunctionKind, Handle, InterfaceId, Resolve, Type, TypeDefKind, TypeId};

/// Generate Rust for `interface` from the WIT file at `wit`.
pub fn generate(wit: &Path, interface: &str) -> Result<String> {
    let mut resolve = Resolve::default();
    let package = resolve.push_file(wit)?;
    let id = *resolve.packages[package]
        .interfaces
        .get(interface)
        .with_context(|| format!("no interface `{interface}`"))?;
    Gen {
        resolve: &resolve,
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
                    return Ok(name.to_upper_camel_case());
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
            TypeDefKind::Tuple(t) => {
                let parts: Result<Vec<_>> = t.types.iter().map(|t| self.ty(t)).collect();
                format!("({},)", parts?.join(", "))
            },
            TypeDefKind::Handle(Handle::Own(r) | Handle::Borrow(r)) => self.resolve.types[*r]
                .name
                .as_ref()
                .unwrap()
                .to_upper_camel_case(),
            TypeDefKind::Type(t) => self.ty(t)?,
            other => bail!("unsupported anonymous type {other:?}"),
        })
    }

    fn opt(&self, ty: &Option<Type>) -> Result<String> {
        ty.as_ref().map_or(Ok("()".into()), |t| self.ty(t))
    }

    fn typedef(&mut self, name: &str, id: TypeId) -> Result<()> {
        let rust = name.to_upper_camel_case();
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
                    writeln!(
                        out,
                        "    pub {}: {},",
                        f.name.to_snake_case(),
                        self.ty(&f.ty)?
                    )?;
                }
                writeln!(
                    out,
                    "}}\nimpl Wire for {rust} {{\n    fn put(&self, out: &mut Vec<u8>) {{"
                )?;
                for f in &r.fields {
                    writeln!(out, "        self.{}.put(out);", f.name.to_snake_case())?;
                }
                writeln!(
                    out,
                    "    }}\n    fn take(input: &mut &[u8]) -> std::io::Result<Self> {{\n        Ok(Self {{"
                )?;
                for f in &r.fields {
                    writeln!(
                        out,
                        "            {}: Wire::take(input)?,",
                        f.name.to_snake_case()
                    )?;
                }
                writeln!(out, "        }})\n    }}\n}}")?;
            },
            TypeDefKind::Variant(v) => {
                writeln!(
                    out,
                    "#[derive(Clone, Debug, PartialEq, Eq)]\npub enum {rust} {{"
                )?;
                for c in &v.cases {
                    let case = c.name.to_upper_camel_case();
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
                    let case = c.name.to_upper_camel_case();
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
                    let case = c.name.to_upper_camel_case();
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
            FunctionKind::Freestanding => Ok((f.name.to_snake_case(), None, &f.params)),
            FunctionKind::Method(r) => {
                let res = self.resolve.types[*r].name.clone().unwrap();
                let method = f.name.rsplit('.').next().unwrap();
                Ok((
                    format!("{}_{}", res.to_snake_case(), method.to_snake_case()),
                    Some(res.to_upper_camel_case()),
                    &f.params[1..],
                ))
            },
            other => bail!("unsupported function kind {other:?} for `{}`", f.name),
        }
    }

    fn resources(&self) -> Vec<String> {
        self.resolve.interfaces[self.id]
            .types
            .iter()
            .filter(|(_, id)| matches!(self.resolve.types[**id].kind, TypeDefKind::Resource))
            .map(|(n, _)| n.to_upper_camel_case())
            .collect()
    }

    fn handler(&mut self, functions: &[&Function]) -> Result<()> {
        let mut trait_src = String::from("/// The provider side.\npub trait Handler {\n");
        let mut arms = String::new();
        for (index, f) in functions.iter().enumerate() {
            let (name, receiver, params) = self.shape(f)?;
            let ret = self.opt(&f.result)?;
            let mut args = String::new();
            let mut call = String::new();
            if let Some(r) = &receiver {
                args.push_str(&format!(", this: &{r}"));
                arms.push_str(&format!(
                    "        {index} => {{\n            let this: {r} = Wire::take(input)?;\n"
                ));
                call.push_str("&this, ");
            } else {
                arms.push_str(&format!("        {index} => {{\n"));
            }
            for p in params {
                let n = p.name.to_snake_case();
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
        for (k, r) in self.resources().iter().enumerate() {
            let index = functions.len() + k;
            let snake = r.to_snake_case();
            trait_src.push_str(&format!("    fn drop_{snake}(&mut self, this: {r});\n"));
            arms.push_str(&format!(
                "        {index} => handler.drop_{snake}(Wire::take(input)?),\n"
            ));
        }
        trait_src.push_str("}\n");
        self.out.push_str(&trait_src);
        writeln!(
            self.out,
            "/// Answer one request frame.\n\
             pub fn dispatch<H: Handler>(handler: &mut H, mut frame: &[u8]) -> std::io::Result<Vec<u8>> {{\n\
             \x20   let input = &mut frame;\n\
             \x20   let mut out = Vec::new();\n\
             \x20   match take_disc(input)? {{\n{arms}\
             \x20       f => return Err(bad(format!(\"unknown function {{f}}\"))),\n\
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
        for (index, f) in functions.iter().enumerate() {
            let (name, receiver, params) = self.shape(f)?;
            let ret = self.opt(&f.result)?;
            let mut args = String::new();
            let mut body = format!(
                "        let mut out = Vec::new();\n        put_disc(&mut out, {index});\n"
            );
            if let Some(r) = &receiver {
                args.push_str(&format!(", this: &{r}"));
                body.push_str("        this.put(&mut out);\n");
            }
            for p in params {
                let n = p.name.to_snake_case();
                args.push_str(&format!(", {n}: {}", self.ty(&p.ty)?));
                body.push_str(&format!("        {n}.put(&mut out);\n"));
            }
            src.push_str(&format!(
                "    pub fn {name}(&mut self{args}) -> std::io::Result<{ret}> {{\n{body}        \
                 let reply = self.0.call(out)?;\n        finish(&reply)\n    }}\n"
            ));
        }
        for (k, r) in self.resources().iter().enumerate() {
            let index = functions.len() + k;
            src.push_str(&format!(
                "    pub fn drop_{}(&mut self, this: {r}) -> std::io::Result<()> {{\n        \
                 let mut out = Vec::new();\n        put_disc(&mut out, {index});\n        \
                 this.put(&mut out);\n        let reply = self.0.call(out)?;\n        finish(&reply)\n    }}\n",
                r.to_snake_case()
            ));
        }
        src.push_str("}\n");
        self.out.push_str(&src);
        Ok(())
    }
}
