use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use rustdoc_types::{
    Constant, Crate, FnDecl, GenericArg, GenericArgs, GenericBound, GenericParamDef,
    GenericParamDefKind, Generics, Header, Id, Item, ItemEnum, Term, Type, TypeBinding,
    TypeBindingKind, Variant, WherePredicate,
};

use std::fmt::Result;

use crate::tokens::{Token, TokenStream};

/// This struct represents one public item of a crate, but in intermediate form.
/// It wraps a single [Item] but adds additional calculated values to make it
/// easier to work with. Later, one [`Self`] will be converted to exactly one
/// [`crate::PublicItem`].
#[derive(Clone)]
pub struct IntermediatePublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,
    root: &'a Crate,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    parent: Option<Rc<IntermediatePublicItem<'a>>>,
}

impl<'a> IntermediatePublicItem<'a> {
    #[must_use]
    pub fn new(
        item: &'a Item,
        root: &'a Crate,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) -> Self {
        Self { item, root, parent }
    }

    #[must_use]
    pub fn path(&'a self) -> Vec<Rc<IntermediatePublicItem<'a>>> {
        let mut path = vec![];

        let rc_self = Rc::new(self.clone());

        path.insert(0, rc_self.clone());

        let mut current_item = rc_self.clone();
        while let Some(parent) = current_item.parent.clone() {
            path.insert(0, parent.clone());
            current_item = parent.clone();
        }

        path
    }

    #[must_use]
    pub fn prefix(&'a self) -> String {
        format!("pub {} ", self.type_string_for_item())
    }

    #[must_use]
    pub fn suffix(&self) -> String {
        format!("{}", ItemSuffix(self))
    }

    fn type_string_for_item(&self) -> &str {
        match &self.item.inner {
            ItemEnum::Module(_) => "mod",
            ItemEnum::ExternCrate { .. } => "extern crate",
            ItemEnum::Import(_) => "use",
            ItemEnum::Union(_) => "union",
            ItemEnum::Struct(_) => "struct",
            ItemEnum::StructField(_) => "struct field",
            ItemEnum::Enum(_) => "enum",
            ItemEnum::Variant(_) => "enum variant",
            ItemEnum::Function(_) | ItemEnum::Method(_) => "fn",
            ItemEnum::Trait(_) => "trait",
            ItemEnum::TraitAlias(_) => "trait alias",
            ItemEnum::Impl(_) => "impl",
            ItemEnum::Typedef(_) | ItemEnum::AssocType { .. } => "type",
            ItemEnum::OpaqueTy(_) => "opaque ty",
            ItemEnum::Constant(_) | ItemEnum::AssocConst { .. } => "const",
            ItemEnum::Static(_) => "static",
            ItemEnum::ForeignType => "foreign type",
            ItemEnum::Macro(_) => "macro",
            ItemEnum::ProcMacro(_) => "proc macro",
            ItemEnum::PrimitiveType(name) => name,
        }
    }

    /// Some items do not use item.name. Handle that.
    #[must_use]
    pub fn get_effective_name(&'a self) -> String {
        match &self.item.inner {
            // An import uses its own name (which can be different from the name of
            // the imported item)
            ItemEnum::Import(i) => &i.name,

            _ => self.item.name.as_deref().unwrap_or("<<no_name>>"),
        }
        .to_owned()
    }

    // TODO: Maybe more tokens need/can be used?
    pub fn render_token_stream(&self) -> Option<TokenStream> {
        match &self.item.inner {
            ItemEnum::Module(_) => None,
            ItemEnum::ExternCrate { .. } => None,
            ItemEnum::Import(_) => None,
            ItemEnum::Union(_) => {
                let mut output: TokenStream = vec![
                    Token::qualifier("pub"),
                    Token::Whitespace,
                    Token::kind("union"),
                    Token::Whitespace,
                ]
                .into();
                output.extend(render_path(self.path()));
                Some(output)
            }
            ItemEnum::Struct(_) => {
                let mut output: TokenStream = vec![
                    Token::qualifier("pub"),
                    Token::Whitespace,
                    Token::kind("struct"),
                    Token::Whitespace,
                ]
                .into();
                output.extend(render_path(self.path()));

                Some(output)
            }
            ItemEnum::StructField(inner) => {
                let mut output: TokenStream = vec![
                    Token::qualifier("pub"),
                    Token::Whitespace,
                    Token::kind("struct"),
                    Token::Whitespace,
                    Token::kind("field"),
                    Token::Whitespace,
                ]
                .into();
                output.extend(render_path(self.path()));
                output.push(Token::symbol(":"));
                output.push(Token::Whitespace);
                output.extend(render_type(self.root, inner));

                Some(output)
            }
            ItemEnum::Enum(_) => {
                let mut output: TokenStream = vec![
                    Token::qualifier("pub"),
                    Token::Whitespace,
                    Token::kind("enum"),
                    Token::Whitespace,
                ]
                .into();
                output.extend(render_path(self.path()));
                Some(output)
            }
            ItemEnum::Variant(inner) => {
                let mut output: TokenStream = vec![
                    Token::qualifier("pub"),
                    Token::Whitespace,
                    Token::kind("enum"),
                    Token::Whitespace,
                    Token::kind("variant"),
                    Token::Whitespace,
                ]
                .into();
                output.extend(render_path(self.path()));
                match inner {
                    Variant::Plain => {}
                    Variant::Tuple(types) => output.extend(render_sequence(
                        Token::symbol("("),
                        Token::symbol(")"),
                        vec![Token::symbol(","), Token::Whitespace],
                        false,
                        types,
                        |ty| render_type(self.root, ty),
                    )),
                    Variant::Struct(ids) => output.extend(render_sequence(
                        Token::symbol("{"),
                        Token::symbol("}"),
                        vec![Token::symbol(","), Token::Whitespace],
                        false,
                        ids,
                        |id| render_id(self.root, id),
                    )),
                }
                Some(output)
            }
            ItemEnum::Function(inner) => Some(render_function(
                self.root,
                render_path(self.path()),
                &inner.decl,
                &inner.generics.params,
                &inner.header,
            )),
            ItemEnum::Method(inner) => Some(render_function(
                self.root,
                render_path(self.path()),
                &inner.decl,
                &inner.generics.params,
                &inner.header,
            )),
            ItemEnum::Trait(inner) => {
                let mut output: TokenStream =
                    vec![Token::qualifier("pub"), Token::Whitespace].into();

                if inner.is_unsafe {
                    output.extend(vec![Token::qualifier("unsafe"), Token::Whitespace]);
                }
                output.extend(vec![Token::kind("trait"), Token::Whitespace]);
                output.extend(render_path(self.path()));
                output.extend(render_generics(self.root, &inner.generics.params));
                Some(output)
            }
            ItemEnum::TraitAlias(_) => None,
            ItemEnum::Impl(_) => None,
            ItemEnum::Typedef(_) | ItemEnum::AssocType { .. } => None,
            ItemEnum::OpaqueTy(_) => None,
            ItemEnum::Constant(_) | ItemEnum::AssocConst { .. } => None,
            ItemEnum::Static(_) => None,
            ItemEnum::ForeignType => None,
            ItemEnum::Macro(_) => None,
            ItemEnum::ProcMacro(_) => None,
            ItemEnum::PrimitiveType(_) => None,
        }
    }
}

fn render_id(root: &Crate, id: &Id) -> TokenStream {
    if let Some(name) = &root.index[id].name {
        Token::identifier(name).into()
    } else {
        TokenStream::default()
    }
}

fn render_path(path: Vec<Rc<IntermediatePublicItem<'_>>>) -> TokenStream {
    let mut output = TokenStream::default();
    for item in &path {
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
                output.extend(render_id(root, id))
            } else {
                let len = name.split("::").count();
                for (index, part) in name.split("::").enumerate() {
                    if index == 0 && part == "$crate" {
                        output.push(Token::keyword("crate"));
                    } else if index == len - 1 {
                        output.push(Token::type_(part))
                    } else {
                        output.push(Token::identifier(part))
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
        ), // TODO: add something better?
        Type::Tuple(types) => render_sequence(
            Token::symbol("("),
            Token::symbol(")"),
            vec![Token::symbol(","), Token::Whitespace],
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
            output.push(Token::symbol(":"));
            output.push(Token::Whitespace);
            output.push(Token::primitive(len));
            output.push(Token::symbol("]"));
            output
        }
        Type::ImplTrait(bounds) => render_generic_bounds(root, bounds),
        Type::Infer => Token::symbol("_").into(),
        Type::RawPointer { mutable, type_ } => {
            let mut output: TokenStream = Token::symbol("*").into();
            if *mutable {
                output.push(Token::keyword("mut"))
            }
            output.push(Token::Whitespace);
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
                output.extend(vec![
                    Token::symbol("'"),
                    Token::identifier(lt.trim_start_matches('\'')),
                    Token::Whitespace,
                ]);
            }
            if *mutable {
                output.extend(vec![Token::keyword("mut"), Token::Whitespace]);
            }
            output.extend(render_type(root, type_));
            output
        }
        Type::QualifiedPath {
            name,
            args: _, // TODO: check if this output if correct
            self_type,
            trait_,
        } => {
            let mut output: TokenStream = Token::symbol("<").into();
            output.extend(render_type(root, self_type));
            output.extend(vec![
                Token::Whitespace,
                Token::keyword("as"),
                Token::Whitespace,
            ]);
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
    let mut output: TokenStream = vec![Token::qualifier("pub"), Token::Whitespace].into();
    if header.const_ {
        output.extend(vec![Token::qualifier("const"), Token::Whitespace])
    };
    if header.async_ {
        output.extend(vec![Token::qualifier("async"), Token::Whitespace])
    };
    // TODO: Do something with ABI?
    output.extend(vec![Token::kind("fn"), Token::Whitespace]);
    output.extend(name);

    // Generic
    output.extend(render_generics(root, generics));
    // Main arguments
    output.extend(render_sequence(
        Token::symbol("("),
        Token::symbol(")"),
        vec![Token::symbol(","), Token::Whitespace],
        false,
        &decl.inputs,
        |(name, ty)| {
            let mut output: TokenStream = vec![
                Token::identifier(name),
                Token::symbol(":"),
                Token::Whitespace,
            ]
            .into();
            output.extend(render_type(root, ty));
            output
        },
    ));
    // Return type
    if let Some(ty) = &decl.output {
        output.extend(vec![
            Token::Whitespace,
            Token::symbol("->"),
            Token::Whitespace,
        ]);
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
            vec![Token::symbol(","), Token::Whitespace],
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
                            output.extend(vec![
                                Token::Whitespace,
                                Token::symbol("="),
                                Token::Whitespace,
                            ]);
                            output.extend(match term {
                                Term::Type(ty) => render_type(root, ty),
                                Term::Constant(c) => render_constant(root, c),
                            });
                        }
                        TypeBindingKind::Constraint(bounds) => {
                            output.extend(render_generic_bounds(root, bounds))
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
            let mut output: TokenStream = Token::kind("Fn").into();
            output.extend(render_sequence(
                Token::symbol("("),
                Token::symbol(")"),
                vec![Token::symbol(","), Token::Whitespace],
                false,
                inputs,
                |ty| render_type(root, ty),
            ));
            if let Some(return_ty) = return_ty {
                output.extend(vec![
                    Token::Whitespace,
                    Token::symbol("->"),
                    Token::Whitespace,
                ]);
                output.extend(render_type(root, return_ty));
            }
            output
        }
    }
}

fn render_generic_arg(root: &Crate, arg: &GenericArg) -> TokenStream {
    match arg {
        GenericArg::Lifetime(name) => vec![
            Token::symbol("'"),
            Token::identifier(name.trim_start_matches('\'')),
        ]
        .into(),
        GenericArg::Type(ty) => render_type(root, ty),
        GenericArg::Const(c) => render_constant(root, c),
        GenericArg::Infer => Token::symbol("_").into(),
    }
}

fn render_constant(root: &Crate, constant: &Constant) -> TokenStream {
    let mut output = render_type(root, &constant.type_);
    if let Some(value) = &constant.value {
        output.extend(vec![
            Token::Whitespace,
            Token::symbol("="),
            Token::Whitespace,
        ]);
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
            vec![Token::symbol(","), Token::Whitespace],
            true,
            generics,
            |param| match &param.kind {
                // See if this is an empty definition (for a human reader)
                GenericParamDefKind::Type {
                    bounds, synthetic, ..
                } if bounds.is_empty() || *synthetic => TokenStream::default(),
                _ => {
                    let mut output: TokenStream = vec![
                        Token::identifier(param.name.clone()),
                        Token::symbol(":"),
                        Token::Whitespace,
                    ]
                    .into();
                    output.extend(render_generic(root, &param.kind));
                    output
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
            .flat_map(|lt| {
                vec![
                    Token::symbol("'"),
                    Token::identifier(lt.trim_start_matches('\'')),
                ]
            })
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
            vec![Token::keyword("impl"), Token::Whitespace],
            Vec::new(),
            vec![Token::Whitespace, Token::symbol("+"), Token::Whitespace],
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
                GenericBound::Outlives(id) => {
                    vec![Token::symbol("'"), Token::identifier(id)].into()
                }
            },
        )
    }
}
/// Decides what should be shown at the end of each item, i.e. item-specific
/// type information.
struct ItemSuffix<'a>(&'a IntermediatePublicItem<'a>);
impl Display for ItemSuffix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.0.item.inner {
            ItemEnum::Variant(v) => write!(f, "{}", D(v)),
            ItemEnum::Union(u) => write!(f, "{}", D(&u.generics)),
            ItemEnum::Struct(s) => write!(f, "{}", D(&s.generics)),
            ItemEnum::Enum(e) => write!(f, "{}", D(&e.generics)),
            ItemEnum::Trait(t) => write!(f, "{}", D(&t.generics)),
            ItemEnum::Typedef(t) => write!(f, "{} = {}", D(&t.generics), D(&t.type_)),
            ItemEnum::Constant(c) => write!(f, ": {}", D(&c.type_)),
            ItemEnum::StructField(type_) => write!(f, ": {}", D(type_)),
            ItemEnum::Function(n) => write!(
                f,
                "{}",
                FnDeclaration {
                    decl: &n.decl,
                    generics: &n.generics
                }
            ),
            ItemEnum::Method(m) => write!(
                f,
                "{}",
                FnDeclaration {
                    decl: &m.decl,
                    generics: &m.generics
                }
            ),
            ItemEnum::Static(s) => write!(f, ": {}", D(&s.type_)),
            ItemEnum::AssocConst { type_, .. } => {
                // Skip the `default` value for now because it can be multi-line
                write!(f, ": {}", D(type_))
            }
            ItemEnum::AssocType {
                bounds, default, ..
            } => {
                write!(f, "{}{:?}", Optional("= ", default.as_ref().map(D)), bounds)
            }
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => write!(f, "!"),
            _ => Ok(()),
        }
    }
}

struct FnDeclaration<'a> {
    decl: &'a FnDecl,
    generics: &'a Generics,
}
impl Display for FnDeclaration<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}{}{}",
            D(&self.generics.params),
            D(self.decl),
            D(&self.generics.where_predicates),
        )
    }
}

/// Formats a fn param like `param: Type`, but simplifies `self: Self` to
/// `self`, `self: &Self` to `&self`, and `self: &mut Self` to `&mut self`.
fn fmt_fn_param(name_and_arg: &(String, Type)) -> String {
    let simplified_self = if name_and_arg.0.as_str() == "self" {
        match &name_and_arg.1 {
            Type::Generic(name) if name == "Self" => Some(String::from("self")),
            Type::BorrowedRef {
                lifetime,
                mutable,
                type_,
            } => match type_.as_ref() {
                Type::Generic(name) if name == "Self" => {
                    Some(format!("&{}{}self", Lifetime(lifetime), Mutable(*mutable)))
                }
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    simplified_self.unwrap_or_else(|| format!("{}: {}", name_and_arg.0, D(&name_and_arg.1)))
}

struct Optional<T: Display>(&'static str, Option<T>);
impl<T: Display> Display for Optional<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(value) = &self.1 {
            write!(f, "{}{}", self.0, value)?;
        }

        Ok(())
    }
}

struct Mutable(bool);
impl Display for Mutable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", if self.0 { "mut " } else { "" })
    }
}

struct Lifetime<'a>(&'a Option<String>);
impl Display for Lifetime<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(lifetime) = self.0 {
            write!(f, "{} ", lifetime)
        } else {
            Ok(())
        }
    }
}

/// Helper to join items with a separator.
struct Joiner<'a, T, D: Display>(&'a Vec<T>, &'static str, fn(&'a T) -> D);
impl<'a, T, D: Display> Display for Joiner<'a, T, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|t| format!("{}", self.2(t)))
                .collect::<Vec<_>>()
                .join(self.1)
        )
    }
}

/// A simple wrapper for types so we can implement [Display] on them. Mostly
/// used for types in the `rustdoc-types` crate since we can't implement
/// [Display] on types defined in other crates.
struct D<T>(T);

impl Display for D<&Type> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.0 {
            Type::ResolvedPath {
                name,
                args,
                param_names,
                ..
            } => {
                write!(f, "{}", name)?;
                if let Some(args) = args {
                    write!(f, "{}", D(args.as_ref()))?;
                }
                if !param_names.is_empty() {
                    write!(f, " + {}", Joiner(param_names, " + ", D))?;
                }

                Ok(())
            }
            Type::Generic(s) => write!(f, "{}", s),
            Type::Primitive(p) => write!(f, "{}", p),
            Type::FunctionPointer(fp) => write!(f, "fn{}", D(&fp.decl)),
            Type::Tuple(types_) => {
                write!(f, "({})", Joiner(types_, ", ", D))
            }
            Type::Slice(t) => write!(f, "[{}]", D(t.as_ref())),
            Type::Array { type_, len } => write!(f, "[{};{}]", D(type_.as_ref()), len),
            Type::ImplTrait(bounds) => write!(f, "impl {}", Joiner(bounds, " + ", D)),
            Type::Infer => write!(f, "_"),
            Type::RawPointer { mutable, type_ } => {
                write!(f, "*{}{}", Mutable(*mutable), D(type_.as_ref()))
            }
            Type::BorrowedRef {
                lifetime,
                mutable,
                type_,
            } => {
                write!(
                    f,
                    "&{}{}{}",
                    Lifetime(lifetime),
                    Mutable(*mutable),
                    D(type_.as_ref()),
                )
            }
            Type::QualifiedPath {
                name,
                self_type,
                trait_,
                ..
            } => write!(
                f,
                "<{} as {}>::{}",
                D(self_type.as_ref()),
                D(trait_.as_ref()),
                name
            ),
        }
    }
}

impl Display for D<&Generics> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}", D(&self.0.params), D(&self.0.where_predicates))
    }
}

impl Display for D<&FnDecl> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "({}){}",
            Joiner(&self.0.inputs, ", ", fmt_fn_param),
            Optional(" -> ", self.0.output.as_ref().map(D)),
        )
    }
}

impl Display for D<&Vec<GenericParamDef>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let params_without_synthetics: Vec<_> = self
            .0
            .iter()
            .filter(|p| {
                if let GenericParamDefKind::Type { synthetic, .. } = p.kind {
                    !synthetic
                } else {
                    true
                }
            })
            .collect();
        if !&params_without_synthetics.is_empty() {
            write!(
                f,
                "<{}>",
                Joiner(&params_without_synthetics, ", ", |x| D(*x))
            )?;
        }

        Ok(())
    }
}

impl Display for D<&Vec<WherePredicate>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if !self.0.is_empty() {
            write!(f, " where {}", Joiner(self.0, ", ", D))?;
        }

        Ok(())
    }
}

impl Display for D<&Variant> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            Variant::Tuple(types) => write!(f, "({})", Joiner(types, ",", D)),
            Variant::Struct(_) | // Each struct field is printed individually
            Variant::Plain => Ok(()),
        }
    }
}

impl Display for D<&GenericParamDef> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}", self.0.name, D(&self.0.kind))
    }
}

impl Display for D<&WherePredicate> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            WherePredicate::BoundPredicate { type_, bounds } => {
                write!(f, "{}: {}", D(type_), Joiner(bounds, " + ", D))
            }
            WherePredicate::RegionPredicate { lifetime, bounds } => {
                write!(f, "{}{:?}", lifetime, bounds)
            }
            WherePredicate::EqPredicate { lhs, rhs } => write!(f, "{} = {}", D(lhs), D(rhs)),
        }
    }
}

impl Display for D<&GenericParamDefKind> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            GenericParamDefKind::Lifetime { outlives } => {
                if !outlives.is_empty() {
                    write!(f, ": {}", outlives.join(", "))?;
                }
            }
            GenericParamDefKind::Type {
                bounds, default, ..
            } => {
                if !bounds.is_empty() {
                    write!(
                        f,
                        ": {}{}",
                        Joiner(bounds, ", ", D),
                        Optional(" = ", default.as_ref().map(D))
                    )?;
                }
            }
            GenericParamDefKind::Const { type_, default } => write!(
                f,
                "GenericParamDefKind::Const{}{}",
                D(type_),
                Optional(" = ", default.as_ref())
            )?,
        }

        Ok(())
    }
}

impl Display for D<&GenericBound> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            GenericBound::TraitBound {
                trait_,
                generic_params,
                ..
            } => write!(f, "{}{}", D(trait_), Joiner(generic_params, " + ", D)),
            GenericBound::Outlives(s) => write!(f, "{}", s),
        }
    }
}

impl Display for D<&GenericArgs> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            GenericArgs::AngleBracketed { args, bindings } => {
                if !args.is_empty() {
                    write!(f, "<{}>", Joiner(args, ", ", D))?;
                }
                if !bindings.is_empty() {
                    write!(f, "<{}>", Joiner(bindings, ", ", D))?;
                }
            }
            GenericArgs::Parenthesized { inputs, output } => {
                if !inputs.is_empty() {
                    write!(
                        f,
                        "({}){}",
                        Joiner(inputs, ", ", D),
                        Optional(" -> ", output.as_ref().map(D))
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Display for D<&GenericArg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.0 {
            GenericArg::Lifetime(l) => write!(f, "{}", l),
            GenericArg::Type(t) => write!(f, "{}", D(t)),
            GenericArg::Const(c) => write!(f, "{}", D(c)),
            GenericArg::Infer => write!(f, "_"),
        }
    }
}

impl Display for D<&TypeBinding> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}{}", self.0.name, D(&self.0.binding))
    }
}

impl Display for D<&TypeBindingKind> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            TypeBindingKind::Equality(e) => write!(f, " = {}", D(e)),
            TypeBindingKind::Constraint(c) => write!(f, ": {}", Joiner(c, " + ", D)),
        }
    }
}

impl Display for D<&Term> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            Term::Type(t) => write!(f, "{}", D(t)),
            Term::Constant(c) => write!(f, " = {}", D(c)),
        }
    }
}

impl Display for D<&Constant> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}
