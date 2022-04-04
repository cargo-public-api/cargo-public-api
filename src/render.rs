use crate::intermediate_public_item::IntermediatePublicItem;
use std::rc::Rc;

use rustdoc_types::{
    Abi, Constant, Crate, FnDecl, GenericArg, GenericArgs, GenericBound, GenericParamDef,
    GenericParamDefKind, Header, Id, ItemEnum, MacroKind, Term, Type, TypeBinding, TypeBindingKind,
    Variant,
};

use crate::tokens::{Token, TokenStream};
use crate::ws;

pub fn token_stream(item: &IntermediatePublicItem) -> TokenStream {
    match &item.item.inner {
        ItemEnum::Module(_) => render_simple(&["mod"], item.path()),
        ItemEnum::ExternCrate { .. } => render_simple(&["extern", "crate"], item.path()),
        ItemEnum::Import(_) => render_simple(&["import"], item.path()),
        ItemEnum::Union(_) => render_simple(&["union"], item.path()),
        ItemEnum::Struct(_) => render_simple(&["struct"], item.path()),
        ItemEnum::StructField(inner) => {
            let mut output = render_simple(&["struct", "field"], item.path());
            output.push(Token::symbol(":"));
            output.push(ws!());
            output.extend(render_type(item.root, inner));
            output
        }
        ItemEnum::Enum(_) => render_simple(&["enum"], item.path()),
        ItemEnum::Variant(inner) => {
            let mut output = render_simple(&["enum", "variant"], item.path());
            match inner {
                Variant::Plain => {}
                Variant::Tuple(types) => output.extend(render_sequence(
                    Token::symbol("("),
                    Token::symbol(")"),
                    vec![Token::symbol(","), ws!()],
                    false,
                    types,
                    |ty| render_type(item.root, ty),
                )),
                Variant::Struct(ids) => output.extend(render_sequence(
                    Token::symbol("{"),
                    Token::symbol("}"),
                    vec![Token::symbol(","), ws!()],
                    false,
                    ids,
                    |id| render_id(item.root, id),
                )),
            }
            output
        }
        ItemEnum::Function(inner) => render_function(
            item.root,
            render_path(&item.path()),
            &inner.decl,
            &inner.generics.params,
            &inner.header,
        ),
        ItemEnum::Method(inner) => render_function(
            item.root,
            render_path(&item.path()),
            &inner.decl,
            &inner.generics.params,
            &inner.header,
        ),
        ItemEnum::Trait(inner) => {
            let tags = if inner.is_unsafe {
                vec!["unsafe", "trait"]
            } else {
                vec!["trait"]
            };
            let mut output = render_simple(&tags, item.path());
            output.extend(render_path(&item.path()));
            output.extend(render_generics(item.root, &inner.generics.params));
            output
        }
        ItemEnum::TraitAlias(_) => render_simple(&["trait", "alias"], item.path()),
        ItemEnum::Impl(_) => render_simple(&["impl"], item.path()),
        ItemEnum::Typedef(inner) => {
            let mut output = render_simple(&["type"], item.path());
            output.extend(render_generics(item.root, &inner.generics.params));
            output.extend(vec![ws!(), Token::symbol("="), ws!()]);
            output.extend(render_type(item.root, &inner.type_));
            output
        }
        ItemEnum::AssocType {
            generics,
            bounds,
            default,
        } => {
            let mut output = render_simple(&["type"], item.path());
            output.extend(render_generics(item.root, &generics.params));
            output.extend(render_generic_bounds(item.root, bounds));
            if let Some(ty) = default {
                output.extend(vec![ws!(), Token::symbol("="), ws!()]);
                output.extend(render_type(item.root, ty));
            }
            output
        }
        ItemEnum::OpaqueTy(_) => render_simple(&["opaque", "type"], item.path()),
        ItemEnum::Constant(_) => render_simple(&["const"], item.path()),
        ItemEnum::AssocConst { .. } => render_simple(&["const"], item.path()),
        ItemEnum::Static(inner) => {
            let tags = if inner.mutable {
                vec!["mut", "static"]
            } else {
                vec!["static"]
            };
            let mut output = render_simple(&tags, item.path());
            output.extend(vec![Token::symbol(":"), ws!()]);
            output.extend(render_type(item.root, &inner.type_));
            output
        }
        ItemEnum::ForeignType => render_simple(&["type"], item.path()),
        ItemEnum::Macro(_definition) => {
            // TODO: _definition contains the whole definition, it would be really neat to get out all possible ways to invoke it
            let mut output = render_simple(&["macro"], item.path());
            output.push(Token::symbol("!"));
            output
        }
        ItemEnum::ProcMacro(inner) => {
            let mut output = render_simple(&["macro"], item.path());
            output.remove_from_back(1); // Remove name of macro\
            let name = Token::identifier(item.item.name.as_ref().unwrap_or(&"".to_string()));
            match inner.kind {
                MacroKind::Bang => output.extend(vec![name, Token::symbol("!()")]),
                MacroKind::Attr => {
                    output.extend(vec![Token::symbol("#["), name, Token::symbol("]")])
                }
                MacroKind::Derive => {
                    output.extend(vec![Token::symbol("#[derive("), name, Token::symbol(")]")])
                }
            }
            output.push(Token::symbol("!"));
            output
        }
        ItemEnum::PrimitiveType(_) => render_simple(&["primitive", "type"], item.path()),
    }
}

fn render_simple(tags: &[&str], path: Vec<Rc<IntermediatePublicItem<'_>>>) -> TokenStream {
    let mut output: TokenStream = vec![Token::qualifier("pub"), ws!()].into();
    output.extend(
        tags.iter()
            .flat_map(|t| [Token::kind(*t), ws!()])
            .collect::<Vec<Token>>(),
    );
    output.extend(render_path(&path));
    output
}

fn render_id(root: &Crate, id: &Id) -> TokenStream {
    root.index[id]
        .name
        .as_ref()
        .map_or_else(TokenStream::default, |name| Token::identifier(name).into())
}

fn render_path(path: &[Rc<IntermediatePublicItem<'_>>]) -> TokenStream {
    let mut output = TokenStream::default();
    for item in path {
        if matches!(item.item.inner, ItemEnum::Function(_) | ItemEnum::Method(_)) {
            output.push(Token::function(item.get_effective_name()));
        } else {
            output.push(Token::identifier(item.get_effective_name()));
        }
        output.push(Token::symbol("::"));
    }
    if !path.is_empty() {
        output.remove_from_back(1);
    }
    output
}

fn render_sequence<T>(
    start: impl Into<TokenStream>,
    end: impl Into<TokenStream>,
    between: impl Into<TokenStream>,
    return_nothing_if_empty: bool,
    sequence: &[T],
    render: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let start = start.into();
    let between = between.into();
    let mut output = start.clone();
    for seq in sequence {
        output.extend(render(seq));
        output.extend(between.clone());
    }
    if !sequence.is_empty() {
        output.remove_from_back(between.len());
    } else if return_nothing_if_empty {
        return TokenStream::default();
    }
    if output.len() == start.len() && return_nothing_if_empty {
        return TokenStream::default();
    }
    output.extend(end);
    output
}

fn render_type(root: &Crate, ty: &Type) -> TokenStream {
    match ty {
        Type::ResolvedPath { name, args, id, .. } => {
            let mut output = TokenStream::default();
            if name.is_empty() {
                output.extend(render_id(root, id));
            } else {
                let len = name.split("::").count();
                for (index, part) in name.split("::").enumerate() {
                    if index == 0 && part == "$crate" {
                        output.push(Token::keyword("crate"));
                    } else if index == len - 1 {
                        output.push(Token::type_(part));
                    } else {
                        output.push(Token::identifier(part));
                    }
                    output.push(Token::symbol("::"));
                }
                if len > 0 {
                    output.remove_from_back(1);
                }
                if let Some(args) = args {
                    output.extend(render_generic_args(root, args));
                }
            }
            output
        } //  _serde::__private::Result | standard type
        Type::Generic(name) => Token::generic(name).into(),
        Type::Primitive(name) => Token::primitive(name).into(),
        Type::FunctionPointer(ptr) => render_function(
            root,
            TokenStream::default(),
            &ptr.decl,
            &ptr.generic_params,
            &ptr.header,
        ),
        Type::Tuple(types) => render_sequence(
            Token::symbol("("),
            Token::symbol(")"),
            vec![Token::symbol(","), ws!()],
            false,
            types,
            |ty| render_type(root, ty),
        ),
        Type::Slice(ty) => {
            let mut output: TokenStream = Token::symbol("[").into();
            output.extend(render_type(root, ty));
            output.push(Token::symbol("]"));
            output
        }
        Type::Array { type_, len } => {
            let mut output: TokenStream = Token::symbol("[").into();
            output.extend(render_type(root, type_));
            output.push(Token::symbol(";"));
            output.push(ws!());
            output.push(Token::primitive(len));
            output.push(Token::symbol("]"));
            output
        }
        Type::ImplTrait(bounds) => render_generic_bounds(root, bounds),
        Type::Infer => Token::symbol("_").into(),
        Type::RawPointer { mutable, type_ } => {
            let mut output: TokenStream = Token::symbol("*").into();
            if *mutable {
                output.push(Token::keyword("mut"));
                output.push(ws!());
            }
            output.extend(render_type(root, type_));
            output
        }
        Type::BorrowedRef {
            lifetime,
            mutable,
            type_,
        } => {
            let mut output: TokenStream = Token::symbol("&").into();
            if let Some(lt) = lifetime {
                output.extend(vec![Token::lifetime(lt), ws!()]);
            }
            if *mutable {
                output.extend(vec![Token::keyword("mut"), ws!()]);
            }
            output.extend(render_type(root, type_));
            output
        }
        Type::QualifiedPath {
            name,
            args: _,
            self_type,
            trait_,
        } => {
            let mut output: TokenStream = Token::symbol("<").into();
            output.extend(render_type(root, self_type));
            output.extend(vec![ws!(), Token::keyword("as"), ws!()]);
            output.extend(render_type(root, trait_));
            output.push(Token::symbol(">::"));
            output.push(Token::identifier(name));
            output
        }
    }
}

fn render_function(
    root: &Crate,
    name: TokenStream,
    decl: &FnDecl,
    generics: &[GenericParamDef],
    header: &Header,
) -> TokenStream {
    let mut output: TokenStream = vec![Token::qualifier("pub"), ws!()].into();
    if header.unsafe_ {
        output.extend(vec![Token::qualifier("unsafe"), ws!()]);
    };
    if header.const_ {
        output.extend(vec![Token::qualifier("const"), ws!()]);
    };
    if header.async_ {
        output.extend(vec![Token::qualifier("async"), ws!()]);
    };
    if header.abi != Abi::Rust {
        output.push(match &header.abi {
            Abi::C { .. } => Token::qualifier("c"),
            Abi::Cdecl { .. } => Token::qualifier("cdecl"),
            Abi::Stdcall { .. } => Token::qualifier("stdcall"),
            Abi::Fastcall { .. } => Token::qualifier("fastcall"),
            Abi::Aapcs { .. } => Token::qualifier("aapcs"),
            Abi::Win64 { .. } => Token::qualifier("win64"),
            Abi::SysV64 { .. } => Token::qualifier("sysV64"),
            Abi::System { .. } => Token::qualifier("system"),
            Abi::Other(text) => Token::qualifier(text),
            _ => unreachable!(),
        });
        output.push(ws!());
    }

    output.extend(vec![Token::kind("fn"), ws!()]);
    output.extend(name);

    // Generic
    output.extend(render_generics(root, generics));
    // Main arguments
    output.extend(render_sequence(
        Token::symbol("("),
        Token::symbol(")"),
        vec![Token::symbol(","), ws!()],
        false,
        &decl.inputs,
        |(name, ty)| {
            let simplified_self: Option<TokenStream> = if name == "self" {
                match ty {
                    Type::Generic(name) if name == "Self" => Some(Token::self_("Self").into()),
                    Type::BorrowedRef {
                        lifetime,
                        mutable,
                        type_,
                    } => match type_.as_ref() {
                        Type::Generic(name) if name == "Self" => {
                            let mut output: TokenStream = Token::symbol("&").into();
                            if let Some(lt) = lifetime {
                                output.extend(vec![Token::lifetime(lt), ws!()]);
                            }
                            if *mutable {
                                output.extend(vec![Token::keyword("mut"), ws!()]);
                            }
                            output.extend(Token::self_("self"));
                            Some(output)
                        }
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                None
            };
            simplified_self.unwrap_or_else(|| {
                let mut output: TokenStream =
                    vec![Token::identifier(name), Token::symbol(":"), ws!()].into();
                output.extend(render_type(root, ty));
                output
            })
        },
    ));
    // Return type
    if let Some(ty) = &decl.output {
        output.extend(vec![ws!(), Token::symbol("->"), ws!()]);
        output.extend(render_type(root, ty));
    }
    output
}

enum Binding<'a> {
    GenericArg(&'a GenericArg),
    TypeBinding(&'a TypeBinding),
}

fn render_generic_args(root: &Crate, args: &GenericArgs) -> TokenStream {
    match args {
        GenericArgs::AngleBracketed { args, bindings } => render_sequence(
            Token::symbol("<"),
            Token::symbol(">"),
            vec![Token::symbol(","), ws!()],
            true,
            &args
                .iter()
                .map(Binding::GenericArg)
                .chain(bindings.iter().map(Binding::TypeBinding))
                .collect::<Vec<_>>(),
            |arg| match arg {
                Binding::GenericArg(arg) => render_generic_arg(root, arg),
                Binding::TypeBinding(TypeBinding {
                    name,
                    args,
                    binding,
                }) => {
                    let mut output: TokenStream = Token::identifier(name).into();
                    output.extend(render_generic_args(root, args));
                    match binding {
                        TypeBindingKind::Equality(term) => {
                            output.extend(vec![ws!(), Token::symbol("="), ws!()]);
                            output.extend(match term {
                                Term::Type(ty) => render_type(root, ty),
                                Term::Constant(c) => render_constant(root, c),
                            });
                        }
                        TypeBindingKind::Constraint(bounds) => {
                            output.extend(render_generic_bounds(root, bounds));
                        }
                    }
                    output
                }
            },
        ),
        GenericArgs::Parenthesized {
            inputs,
            output: return_ty,
        } => {
            let mut output: TokenStream = render_sequence(
                Token::symbol("("),
                Token::symbol(")"),
                vec![Token::symbol(","), ws!()],
                false,
                inputs,
                |ty| render_type(root, ty),
            );
            if let Some(return_ty) = return_ty {
                output.extend(vec![ws!(), Token::symbol("->"), ws!()]);
                output.extend(render_type(root, return_ty));
            }
            output
        }
    }
}

fn render_generic_arg(root: &Crate, arg: &GenericArg) -> TokenStream {
    match arg {
        GenericArg::Lifetime(name) => Token::lifetime(name).into(),
        GenericArg::Type(ty) => render_type(root, ty),
        GenericArg::Const(c) => render_constant(root, c),
        GenericArg::Infer => Token::symbol("_").into(),
    }
}

fn render_constant(root: &Crate, constant: &Constant) -> TokenStream {
    let mut output = render_type(root, &constant.type_);
    if let Some(value) = &constant.value {
        output.extend(vec![ws!(), Token::symbol("="), ws!()]);
        if constant.is_literal {
            output.push(Token::primitive(value));
        } else {
            output.push(Token::identifier(value));
        }
    }
    output
}

fn render_generics(root: &Crate, generics: &[GenericParamDef]) -> TokenStream {
    let mut output = TokenStream::default();
    if !generics.is_empty() {
        output.extend(render_sequence(
            Token::symbol("<"),
            Token::symbol(">"),
            vec![Token::symbol(","), ws!()],
            true,
            generics,
            |param| match &param.kind {
                // See if this is an empty definition (for a human reader)
                GenericParamDefKind::Type {
                    bounds, synthetic, ..
                } if bounds.is_empty() || *synthetic => TokenStream::default(),
                _ => {
                    if let GenericParamDefKind::Lifetime { .. } = param.kind {
                        Token::lifetime(param.name.clone()).into()
                    } else {
                        let mut output: TokenStream = vec![
                            Token::identifier(param.name.clone()),
                            Token::symbol(":"),
                            ws!(),
                        ]
                        .into();
                        output.extend(render_generic(root, &param.kind));
                        output
                    }
                }
            },
        ));
    }
    output
}

fn render_generic(root: &Crate, generic: &GenericParamDefKind) -> TokenStream {
    match generic {
        GenericParamDefKind::Lifetime { outlives } => outlives
            .iter()
            .map(Token::lifetime)
            .collect::<Vec<_>>()
            .into(),
        GenericParamDefKind::Type { bounds, .. } => render_generic_bounds(root, bounds),
        GenericParamDefKind::Const { type_, .. } => render_type(root, type_),
    }
}

fn render_generic_bounds(root: &Crate, bounds: &[GenericBound]) -> TokenStream {
    if bounds.is_empty() {
        TokenStream::default()
    } else {
        render_sequence(
            vec![Token::keyword("impl"), ws!()],
            Vec::new(),
            vec![ws!(), Token::symbol("+"), ws!()],
            true,
            bounds,
            |bound| match bound {
                GenericBound::TraitBound {
                    trait_,
                    generic_params,
                    ..
                } => {
                    let mut output = render_type(root, trait_);
                    if output == Token::type_("Iterator").into() {
                        dbg!(trait_);
                    }
                    output.extend(render_generics(root, generic_params));
                    output
                }
                GenericBound::Outlives(id) => Token::lifetime(id).into(),
            },
        )
    }
}
