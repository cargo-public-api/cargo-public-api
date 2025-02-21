#![allow(clippy::unused_self)]
use crate::intermediate_public_item::IntermediatePublicItem;
use crate::nameable_item::NameableItem;
use crate::path_component::PathComponent;
use crate::tokens::Token;
use crate::BuilderOptions as Options;
use std::{cmp::Ordering, collections::HashMap, vec};

use rustdoc_types::{
    Abi, AssocItemConstraint, AssocItemConstraintKind, Constant, Crate, FunctionHeader,
    FunctionPointer, FunctionSignature, GenericArg, GenericArgs, GenericBound, GenericParamDef,
    GenericParamDefKind, Generics, Id, Impl, Item, ItemEnum, MacroKind, Path, PolyTrait,
    StructKind, Term, Trait, TraitBoundModifier, Type, VariantKind, WherePredicate,
};

/// A simple macro to write `Token::Whitespace` in less characters.
macro_rules! ws {
    () => {
        Token::Whitespace
    };
}

/// When we render an item, it might contain references to other parts of the
/// public API. For such cases, the rendering code can use the fields in this
/// struct.
pub struct RenderingContext<'c> {
    /// The original and unmodified rustdoc JSON, in deserialized form.
    pub crate_: &'c Crate,

    /// Given a rustdoc JSON ID, keeps track of what public items that have this Id.
    pub id_to_items: HashMap<&'c Id, Vec<&'c IntermediatePublicItem<'c>>>,

    pub(crate) options: Options,
}

impl<'c> RenderingContext<'c> {
    pub fn token_stream(&self, public_item: &IntermediatePublicItem<'c>) -> Vec<Token> {
        let item = public_item.item();
        let item_path = public_item.path();

        let mut tokens = vec![];

        for attr in &item.attrs {
            if attr_relevant_for_public_apis(attr) {
                tokens.push(Token::Annotation(attr.clone()));
                tokens.push(ws!());
            }
        }

        let inner_tokens = match &item.inner {
            ItemEnum::Module(_) => self.render_simple(&["mod"], item_path),
            ItemEnum::ExternCrate { .. } => self.render_simple(&["extern", "crate"], item_path),
            ItemEnum::Use(_) => self.render_simple(&["use"], item_path),
            ItemEnum::Union(_) => self.render_simple(&["union"], item_path),
            ItemEnum::Struct(s) => {
                let mut output = self.render_simple(&["struct"], item_path);
                output.extend(self.render_generics(&s.generics));
                if let StructKind::Tuple(fields) = &s.kind {
                    output.extend(
                        self.render_option_tuple(&self.resolve_tuple_fields(fields), Some(&pub_())),
                    );
                }
                output
            }
            ItemEnum::StructField(inner) => {
                let mut output = self.render_simple(&[], item_path);
                output.extend(colon());
                output.extend(self.render_type(inner));
                output
            }
            ItemEnum::Enum(e) => {
                let mut output = self.render_simple(&["enum"], item_path);
                output.extend(self.render_generics(&e.generics));
                output
            }
            ItemEnum::Variant(inner) => {
                let mut output = self.render_simple(&[], item_path);
                match &inner.kind {
                    VariantKind::Struct { .. } => {} // Each struct field is printed individually
                    VariantKind::Plain => {
                        if let Some(discriminant) = &inner.discriminant {
                            output.extend(equals());
                            output.push(Token::identifier(&discriminant.value));
                        }
                    }
                    VariantKind::Tuple(fields) => {
                        output.extend(
                            self.render_option_tuple(&self.resolve_tuple_fields(fields), None),
                        );
                    }
                }
                output
            }
            ItemEnum::Function(inner) => self.render_function(
                self.render_path(item_path),
                &inner.sig,
                &inner.generics,
                &inner.header,
            ),
            ItemEnum::Trait(trait_) => self.render_trait(trait_, item_path),
            ItemEnum::TraitAlias(_) => self.render_simple(&["trait", "alias"], item_path),
            ItemEnum::Impl(impl_) => {
                self.render_impl(impl_, item_path, false /* disregard_negativity */)
            }
            ItemEnum::TypeAlias(inner) => {
                let mut output = self.render_simple(&["type"], item_path);
                output.extend(self.render_generics(&inner.generics));
                output.extend(equals());
                output.extend(self.render_type(&inner.type_));
                output
            }
            ItemEnum::AssocType {
                generics,
                bounds,
                type_,
            } => {
                let mut output = self.render_simple(&["type"], item_path);
                output.extend(self.render_generics(generics));
                output.extend(self.render_generic_bounds_with_colon(bounds));
                if let Some(ty) = type_ {
                    output.extend(equals());
                    output.extend(self.render_type(ty));
                }
                output
            }
            ItemEnum::Constant { const_, type_ } => {
                let mut output = self.render_simple(&["const"], item_path);
                output.extend(colon());
                output.extend(self.render_constant(const_, Some(type_)));
                output
            }
            ItemEnum::AssocConst { type_, .. } => {
                let mut output = self.render_simple(&["const"], item_path);
                output.extend(colon());
                output.extend(self.render_type(type_));
                output
            }
            ItemEnum::Static(inner) => {
                let tags = if inner.is_mutable {
                    vec!["mut", "static"]
                } else {
                    vec!["static"]
                };
                let mut output = self.render_simple(&tags, item_path);
                output.extend(colon());
                output.extend(self.render_type(&inner.type_));
                output
            }
            ItemEnum::ExternType => self.render_simple(&["type"], item_path),
            ItemEnum::Macro(_definition) => {
                // TODO: _definition contains the whole definition, it would be really neat to get out all possible ways to invoke it
                let mut output = self.render_simple(&["macro"], item_path);
                output.push(Token::symbol("!"));
                output
            }
            ItemEnum::ProcMacro(inner) => {
                let mut output = self.render_simple(&["proc", "macro"], item_path);
                output.pop(); // Remove name of macro to possibly wrap it in `#[]`
                let name = Token::identifier(item.name.as_deref().unwrap_or(""));
                match inner.kind {
                    MacroKind::Bang => output.extend(vec![name, Token::symbol("!()")]),
                    MacroKind::Attr => {
                        output.extend(vec![Token::symbol("#["), name, Token::symbol("]")]);
                    }
                    MacroKind::Derive => {
                        output.extend(vec![Token::symbol("#[derive("), name, Token::symbol(")]")]);
                    }
                }
                output
            }
            ItemEnum::Primitive(primitive) => {
                // This is hard to write tests for since only Rust `core` is
                // allowed to define primitives. So you have to test this code
                // using the pre-built rustdoc JSON for core:
                //
                //   rustup component add rust-docs-json --toolchain nightly
                //   cargo run -- --rustdoc-json ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/share/doc/rust/json/core.json
                let mut output = pub_();
                output.extend([
                    Token::kind("type"),
                    ws!(),
                    Token::primitive(&primitive.name),
                ]);
                output
            }
        };

        tokens.extend(inner_tokens);

        tokens
    }

    /// Tuple fields are referenced by ID in JSON, but we need to look up the
    /// actual types that the IDs correspond to, in order to render the fields.
    /// This helper does that for a slice of fields.
    fn resolve_tuple_fields(&self, fields: &[Option<Id>]) -> Vec<Option<&'c Type>> {
        let mut resolved_fields: Vec<Option<&Type>> = vec![];

        for id in fields {
            resolved_fields.push(
                if let Some(Item {
                    inner: ItemEnum::StructField(type_),
                    ..
                }) = id.as_ref().and_then(|id| self.crate_.index.get(id))
                {
                    Some(type_)
                } else {
                    None
                },
            );
        }

        resolved_fields
    }

    fn render_simple(&self, tags: &[&str], path: &[PathComponent]) -> Vec<Token> {
        let mut output = pub_();
        output.extend(
            tags.iter()
                .flat_map(|t| [Token::kind(*t), ws!()])
                .collect::<Vec<Token>>(),
        );
        output.extend(self.render_path(path));
        output
    }

    fn render_path(&self, path: &[PathComponent]) -> Vec<Token> {
        let mut output = vec![];
        for component in path {
            if component.hide {
                continue;
            }

            let (tokens, push_a_separator) = component.type_.map_or_else(
                || self.render_nameable_item(&component.item),
                |ty| self.render_type_and_separator(ty),
            );

            output.extend(tokens);

            if push_a_separator {
                output.push(Token::symbol("::"));
            }
        }
        if !path.is_empty() {
            output.pop(); // Remove last "::" so "a::b::c::" becomes "a::b::c"
        }
        output
    }

    fn render_nameable_item(&self, item: &NameableItem) -> (Vec<Token>, bool) {
        let mut push_a_separator = false;
        let mut output = vec![];
        let token_fn = if matches!(item.item.inner, ItemEnum::Function(_)) {
            Token::function
        } else if matches!(
            item.item.inner,
            ItemEnum::Trait(_)
                | ItemEnum::Struct(_)
                | ItemEnum::Union(_)
                | ItemEnum::Enum(_)
                | ItemEnum::TypeAlias(_)
        ) {
            Token::type_
        } else {
            Token::identifier
        };

        if self.options.debug_sorting {
            // There is always a sortable name, so we can push the name
            // unconditionally
            output.push(token_fn(item.sortable_name(self)));
            push_a_separator = true;
        } else if let Some(name) = item.name() {
            // If we are not debugging, some items (read: impls) do not have
            // a name, so only push a name if it exists
            output.push(token_fn(name.to_string()));
            push_a_separator = true;
        }
        (output, push_a_separator)
    }

    fn render_sequence<T>(
        &self,
        start: Vec<Token>,
        end: Vec<Token>,
        between: Vec<Token>,
        sequence: &[T],
        render: impl Fn(&T) -> Vec<Token>,
    ) -> Vec<Token> {
        self.render_sequence_impl(start, end, between, false, sequence, render)
    }

    fn render_sequence_if_not_empty<T>(
        &self,
        start: Vec<Token>,
        end: Vec<Token>,
        between: Vec<Token>,
        sequence: &[T],
        render: impl Fn(&T) -> Vec<Token>,
    ) -> Vec<Token> {
        self.render_sequence_impl(start, end, between, true, sequence, render)
    }

    fn render_sequence_impl<T>(
        &self,
        start: Vec<Token>,
        end: Vec<Token>,
        between: Vec<Token>,
        return_nothing_if_empty: bool,
        sequence: &[T],
        render: impl Fn(&T) -> Vec<Token>,
    ) -> Vec<Token> {
        if return_nothing_if_empty && sequence.is_empty() {
            return vec![];
        }
        let mut output = start;
        for (index, seq) in sequence.iter().enumerate() {
            output.extend(render(seq));
            if index < sequence.len() - 1 {
                output.extend(between.clone());
            }
        }
        output.extend(end);
        output
    }

    fn render_type(&self, ty: &Type) -> Vec<Token> {
        self.render_option_type(&Some(ty))
    }

    fn render_type_and_separator(&self, ty: &Type) -> (Vec<Token>, bool) {
        (self.render_type(ty), true)
    }

    fn render_option_type(&self, ty: &Option<&Type>) -> Vec<Token> {
        let Some(ty) = ty else {
            return vec![Token::symbol("_")];
        }; // The `_` in `EnumWithStrippedTupleVariants::DoubleFirstHidden(_, bool)`
        match ty {
            Type::ResolvedPath(path) => self.render_resolved_path(path),
            Type::DynTrait(dyn_trait) => self.render_dyn_trait(dyn_trait),
            Type::Generic(name) => vec![Token::generic(name)],
            Type::Primitive(name) => vec![Token::primitive(name)],
            Type::FunctionPointer(ptr) => self.render_function_pointer(ptr),
            Type::Tuple(types) => self.render_tuple(types),
            Type::Slice(ty) => self.render_slice(ty),
            Type::Array { type_, len } => self.render_array(type_, len),
            Type::ImplTrait(bounds) => self.render_impl_trait(bounds),
            Type::Infer => vec![Token::symbol("_")],
            Type::RawPointer { is_mutable, type_ } => self.render_raw_pointer(*is_mutable, type_),
            Type::BorrowedRef {
                lifetime,
                is_mutable,
                type_,
            } => self.render_borrowed_ref(lifetime.as_deref(), *is_mutable, type_),
            Type::QualifiedPath {
                name,
                args: _,
                self_type,
                trait_,
            } => self.render_qualified_path(self_type, trait_.as_ref(), name),
            Type::Pat { .. } => vec![Token::symbol(
                "https://github.com/rust-lang/rust/issues/123646 is unstable and not supported",
            )],
        }
    }

    fn render_trait(&self, trait_: &Trait, path: &[PathComponent]) -> Vec<Token> {
        let mut output = pub_();
        if trait_.is_unsafe {
            output.extend(vec![Token::qualifier("unsafe"), ws!()]);
        };
        output.extend([Token::kind("trait"), ws!()]);
        output.extend(self.render_path(path));
        output.extend(self.render_generics(&trait_.generics));
        output.extend(self.render_generic_bounds_with_colon(&trait_.bounds));
        output
    }

    fn render_dyn_trait(&self, dyn_trait: &rustdoc_types::DynTrait) -> Vec<Token> {
        let mut output = vec![];

        let more_than_one = dyn_trait.traits.len() > 1 || dyn_trait.lifetime.is_some();
        if more_than_one {
            output.push(Token::symbol("("));
        }

        output.extend(self.render_sequence_if_not_empty(
            vec![Token::keyword("dyn"), ws!()],
            vec![],
            plus(),
            &dyn_trait.traits,
            |p| self.render_poly_trait(p),
        ));

        if let Some(lt) = &dyn_trait.lifetime {
            output.extend(plus());
            output.extend(vec![Token::lifetime(lt)]);
        }

        if more_than_one {
            output.push(Token::symbol(")"));
        }

        output
    }

    fn render_function(
        &self,
        name: Vec<Token>,
        sig: &FunctionSignature,
        generics: &Generics,
        header: &FunctionHeader,
    ) -> Vec<Token> {
        let mut output = pub_();
        if header.is_unsafe {
            output.extend(vec![Token::qualifier("unsafe"), ws!()]);
        };
        if header.is_const {
            output.extend(vec![Token::qualifier("const"), ws!()]);
        };
        if header.is_async {
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
                Abi::Rust => unreachable!(),
            });
            output.push(ws!());
        }

        output.extend(vec![Token::kind("fn"), ws!()]);
        output.extend(name);

        // Generic parameters
        output.extend(self.render_generic_param_defs(&generics.params));

        // Regular parameters and return type
        output.extend(self.render_fn_decl(sig, true));

        // Where predicates
        output.extend(self.render_where_predicates(&generics.where_predicates));

        output
    }

    fn render_fn_decl(&self, sig: &FunctionSignature, include_underscores: bool) -> Vec<Token> {
        let mut output = vec![];
        // Main arguments
        output.extend(self.render_sequence(
            vec![Token::symbol("(")],
            vec![Token::symbol(")")],
            comma(),
            &sig.inputs,
            |(name, ty)| {
                self.simplified_self(name, ty).unwrap_or_else(|| {
                    let mut output = vec![];
                    let ignore_name = name.is_empty() || (name == "_" && !include_underscores);
                    if !ignore_name {
                        output.extend(vec![Token::identifier(name), Token::symbol(":"), ws!()]);
                    }
                    output.extend(self.render_type(ty));
                    output
                })
            },
        ));
        // Return type
        if let Some(ty) = &sig.output {
            output.extend(arrow());
            output.extend(self.render_type(ty));
        }
        output
    }

    fn simplified_self(&self, name: &str, ty: &Type) -> Option<Vec<Token>> {
        if name == "self" {
            match ty {
                Type::Generic(name) if name == "Self" => Some(vec![Token::self_("self")]),
                Type::BorrowedRef {
                    lifetime,
                    is_mutable,
                    type_,
                } => match type_.as_ref() {
                    Type::Generic(name) if name == "Self" => {
                        let mut output = vec![Token::symbol("&")];
                        if let Some(lt) = lifetime {
                            output.extend(vec![Token::lifetime(lt), ws!()]);
                        }
                        if *is_mutable {
                            output.extend(vec![Token::keyword("mut"), ws!()]);
                        }
                        output.push(Token::self_("self"));
                        Some(output)
                    }
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn render_resolved_path(&self, path: &Path) -> Vec<Token> {
        let mut output = vec![];
        if let Some(item) = self.best_item_for_id(&path.id) {
            output.extend(self.render_path(item.path()));
        } else if let Some(item) = self.crate_.paths.get(&path.id) {
            output.extend(self.render_path_components(item.path.iter()));
        } else if !path.path.is_empty() {
            // If we get here it means there was no item for this Path in the
            // rustdoc JSON. Examples of when this happens:
            //
            // * The resolved path is for a public item inside a private mod
            //   (and thus effectively the item is not public)
            //
            // In these cases we simply use the `name` verbatim, which typically
            // is equal to how it appears in the source text. It might not be
            // ideal and end up identical to the corresponding rustdoc HTML, but
            // it is good enough given the edge-case nature of this code path.
            output.extend(self.render_path_name(&path.path));
        }
        if let Some(args) = &path.args {
            output.extend(self.render_generic_args(args));
        }
        output
    }

    fn render_path_name(&self, name: &str) -> Vec<Token> {
        self.render_path_components(name.split("::"))
    }

    fn render_path_components(
        &self,
        path_iter: impl Iterator<Item = impl AsRef<str>>,
    ) -> Vec<Token> {
        let mut output = vec![];
        let path: Vec<_> = path_iter.collect();
        let len = path.len();
        for (index, part) in path.into_iter().enumerate() {
            if index == len - 1 {
                output.push(Token::type_(part.as_ref()));
            } else {
                output.push(Token::identifier(part.as_ref()));
            }
            output.push(Token::symbol("::"));
        }
        if len > 0 {
            output.pop();
        }
        output
    }

    fn render_function_pointer(&self, ptr: &FunctionPointer) -> Vec<Token> {
        let mut output = self.render_higher_rank_trait_bounds(&ptr.generic_params);
        output.push(Token::kind("fn"));
        output.extend(self.render_fn_decl(&ptr.sig, false));
        output
    }

    fn render_tuple(&self, types: &[Type]) -> Vec<Token> {
        let option_tuple: Vec<Option<&Type>> = types.iter().map(Some).collect();
        self.render_option_tuple(&option_tuple, None)
    }

    /// `prefix` is to handle the difference  between tuple structs and enum variant
    /// tuple structs. The former marks public fields as `pub ` whereas all fields
    /// of enum tuple structs are always implicitly `pub`.
    fn render_option_tuple(&self, types: &[Option<&Type>], prefix: Option<&[Token]>) -> Vec<Token> {
        self.render_sequence(
            vec![Token::symbol("(")],
            vec![Token::symbol(")")],
            comma(),
            types,
            |type_| {
                let mut output: Vec<Token> = vec![];
                if let (Some(prefix), Some(_)) = (prefix, type_) {
                    output.extend(prefix.to_owned());
                }
                output.extend(self.render_option_type(type_));
                output
            },
        )
    }

    fn render_slice(&self, ty: &Type) -> Vec<Token> {
        let mut output = vec![Token::symbol("[")];
        output.extend(self.render_type(ty));
        output.push(Token::symbol("]"));
        output
    }

    fn render_array(&self, type_: &Type, len: &str) -> Vec<Token> {
        let mut output = vec![Token::symbol("[")];
        output.extend(self.render_type(type_));
        output.extend(vec![
            Token::symbol(";"),
            ws!(),
            Token::primitive(len),
            Token::symbol("]"),
        ]);
        output
    }

    pub(crate) fn render_impl(
        &self,
        impl_: &Impl,
        path: &[PathComponent],
        disregard_negativity: bool,
    ) -> Vec<Token> {
        let mut output = vec![];

        if self.options.debug_sorting {
            output.extend(self.render_path(path));
            output.push(ws!());
        }

        if impl_.is_unsafe {
            output.extend(vec![Token::keyword("unsafe"), ws!()]);
        }

        output.push(Token::keyword("impl"));

        output.extend(self.render_generic_param_defs(&impl_.generics.params));

        output.push(ws!());

        if let Some(trait_) = &impl_.trait_ {
            if !disregard_negativity && impl_.is_negative {
                output.push(Token::symbol("!"));
            }
            output.extend(self.render_resolved_path(trait_));
            output.extend(vec![ws!(), Token::keyword("for"), ws!()]);
            output.extend(self.render_type(&impl_.for_));
        } else {
            output.extend(self.render_type(&impl_.for_));
        }

        output.extend(self.render_where_predicates(&impl_.generics.where_predicates));

        output
    }

    fn render_impl_trait(&self, bounds: &[GenericBound]) -> Vec<Token> {
        let mut output = vec![Token::keyword("impl")];
        output.push(ws!());
        output.extend(self.render_generic_bounds(bounds));
        output
    }

    fn render_raw_pointer(&self, is_mutable: bool, type_: &Type) -> Vec<Token> {
        let mut output = vec![Token::symbol("*")];
        output.push(Token::keyword(if is_mutable { "mut" } else { "const" }));
        output.push(ws!());
        output.extend(self.render_type(type_));
        output
    }

    fn render_borrowed_ref(
        &self,
        lifetime: Option<&str>,
        is_mutable: bool,
        type_: &Type,
    ) -> Vec<Token> {
        let mut output = vec![Token::symbol("&")];
        if let Some(lt) = lifetime {
            output.extend(vec![Token::lifetime(lt), ws!()]);
        }
        if is_mutable {
            output.extend(vec![Token::keyword("mut"), ws!()]);
        }
        output.extend(self.render_type(type_));
        output
    }

    fn render_qualified_path(&self, type_: &Type, trait_: Option<&Path>, name: &str) -> Vec<Token> {
        let mut output = vec![];
        match (type_, trait_) {
            (Type::Generic(name), Some(trait_)) if name == "Self" && trait_.path.is_empty() => {
                output.push(Token::keyword("Self"));
            }
            (_, trait_) => {
                if trait_.is_some() {
                    output.push(Token::symbol("<"));
                }
                output.extend(self.render_type(type_));
                if let Some(trait_) = trait_ {
                    output.extend(vec![ws!(), Token::keyword("as"), ws!()]);
                    output.extend(self.render_resolved_path(trait_));
                    output.push(Token::symbol(">"));
                }
            }
        }
        output.push(Token::symbol("::"));
        output.push(Token::identifier(name));
        output
    }

    fn render_generic_args(&self, args: &GenericArgs) -> Vec<Token> {
        match args {
            GenericArgs::AngleBracketed { args, constraints } => {
                self.render_angle_bracketed(args, constraints)
            }
            GenericArgs::Parenthesized { inputs, output } => {
                self.render_parenthesized(inputs, output)
            }
        }
    }

    fn render_parenthesized(&self, inputs: &[Type], return_ty: &Option<Type>) -> Vec<Token> {
        let mut output = self.render_sequence(
            vec![Token::symbol("(")],
            vec![Token::symbol(")")],
            comma(),
            inputs,
            |type_| self.render_type(type_),
        );
        if let Some(return_ty) = return_ty {
            output.extend(arrow());
            output.extend(self.render_type(return_ty));
        }
        output
    }

    fn render_angle_bracketed(
        &self,
        args: &[GenericArg],
        constraints: &[AssocItemConstraint],
    ) -> Vec<Token> {
        enum Arg<'c> {
            GenericArg(&'c GenericArg),
            AssocItemConstraint(&'c AssocItemConstraint),
        }
        self.render_sequence_if_not_empty(
            vec![Token::symbol("<")],
            vec![Token::symbol(">")],
            comma(),
            &args
                .iter()
                .map(Arg::GenericArg)
                .chain(constraints.iter().map(Arg::AssocItemConstraint))
                .collect::<Vec<_>>(),
            |arg| match arg {
                Arg::GenericArg(arg) => self.render_generic_arg(arg),
                Arg::AssocItemConstraint(constraints) => {
                    self.render_assoc_item_constraint(constraints)
                }
            },
        )
    }

    fn render_term(&self, term: &Term) -> Vec<Token> {
        match term {
            Term::Type(ty) => self.render_type(ty),
            Term::Constant(c) => self.render_constant(c, None),
        }
    }

    fn render_poly_trait(&self, poly_trait: &PolyTrait) -> Vec<Token> {
        let mut output = self.render_higher_rank_trait_bounds(&poly_trait.generic_params);
        output.extend(self.render_resolved_path(&poly_trait.trait_));
        output
    }

    fn render_generic_arg(&self, arg: &GenericArg) -> Vec<Token> {
        match arg {
            GenericArg::Lifetime(name) => vec![Token::lifetime(name)],
            GenericArg::Type(ty) => self.render_type(ty),
            GenericArg::Const(c) => self.render_constant(c, None),
            GenericArg::Infer => vec![Token::symbol("_")],
        }
    }

    fn render_assoc_item_constraint(&self, constraints: &AssocItemConstraint) -> Vec<Token> {
        let mut output = vec![Token::identifier(&constraints.name)];
        output.extend(self.render_generic_args(&constraints.args));
        match &constraints.binding {
            AssocItemConstraintKind::Equality(term) => {
                output.extend(equals());
                output.extend(self.render_term(term));
            }
            AssocItemConstraintKind::Constraint(bounds) => {
                output.extend(self.render_generic_bounds(bounds));
            }
        }
        output
    }

    fn render_constant(&self, constant: &Constant, type_: Option<&Type>) -> Vec<Token> {
        let mut output = vec![];
        // In general we do not want to include values of e.g. public
        // constants, since the values themselves do not typically
        // constitute the public API surface. It is the constant identifier
        // itself that forms the public API surface. So if we have a type,
        // we only render the type.
        //
        // However, sometimes we do not have any type (e.g. for const
        // generic args), and it that case it looks weird to not show
        // anything. So in that case we show the value.
        if let Some(type_) = type_ {
            output.extend(self.render_type(type_));
        } else if let Some(value) = &constant.value {
            if constant.is_literal {
                output.push(Token::primitive(value));
            } else {
                output.push(Token::identifier(value));
            }
        } else {
            output.push(Token::identifier(&constant.expr));
        }
        output
    }

    fn render_generics(&self, generics: &Generics) -> Vec<Token> {
        let mut output = vec![];
        output.extend(self.render_generic_param_defs(&generics.params));
        output.extend(self.render_where_predicates(&generics.where_predicates));
        output
    }

    fn render_generic_param_defs(&self, params: &[GenericParamDef]) -> Vec<Token> {
        let params_without_synthetics: Vec<_> = params
            .iter()
            .filter(|p| {
                if let GenericParamDefKind::Type { is_synthetic, .. } = p.kind {
                    !is_synthetic
                } else {
                    true
                }
            })
            .collect();

        self.render_sequence_if_not_empty(
            vec![Token::symbol("<")],
            vec![Token::symbol(">")],
            comma(),
            &params_without_synthetics,
            |param| self.render_generic_param_def(param),
        )
    }

    fn render_generic_param_def(&self, generic_param_def: &GenericParamDef) -> Vec<Token> {
        let mut output = vec![];
        match &generic_param_def.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                output.push(Token::lifetime(&generic_param_def.name));
                if !outlives.is_empty() {
                    output.extend(colon());
                    output.extend(self.render_sequence(vec![], vec![], plus(), outlives, |s| {
                        vec![Token::lifetime(s)]
                    }));
                }
            }
            GenericParamDefKind::Type { bounds, .. } => {
                output.push(Token::generic(&generic_param_def.name));
                output.extend(self.render_generic_bounds_with_colon(bounds));
            }
            GenericParamDefKind::Const { type_, .. } => {
                output.push(Token::qualifier("const"));
                output.push(ws!());
                output.push(Token::identifier(&generic_param_def.name));
                output.extend(colon());
                output.extend(self.render_type(type_));
            }
        }
        output
    }

    fn render_where_predicates(&self, where_predicates: &[WherePredicate]) -> Vec<Token> {
        let mut output = vec![];
        if !where_predicates.is_empty() {
            output.push(ws!());
            output.push(Token::Keyword("where".to_owned()));
            output.push(ws!());
            output.extend(
                self.render_sequence(vec![], vec![], comma(), where_predicates, |p| {
                    self.render_where_predicate(p)
                }),
            );
        }
        output
    }

    fn render_where_predicate(&self, where_predicate: &WherePredicate) -> Vec<Token> {
        let mut output = vec![];
        match where_predicate {
            WherePredicate::BoundPredicate {
                type_,
                bounds,
                generic_params,
            } => {
                output.extend(self.render_higher_rank_trait_bounds(generic_params));
                output.extend(self.render_type(type_));
                output.extend(self.render_generic_bounds_with_colon(bounds));
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                output.push(Token::Lifetime(lifetime.clone()));
                output.extend(self.render_sequence_if_not_empty(
                    colon(),
                    vec![],
                    plus(),
                    outlives,
                    |s| vec![Token::Lifetime(s.clone())],
                ));
            }
            WherePredicate::EqPredicate { lhs, rhs } => {
                output.extend(self.render_type(lhs));
                output.extend(equals());
                output.extend(self.render_term(rhs));
            }
        }
        output
    }

    fn render_generic_bounds_with_colon(&self, bounds: &[GenericBound]) -> Vec<Token> {
        let mut output = vec![];
        if !bounds.is_empty() {
            output.extend(colon());
            output.extend(self.render_generic_bounds(bounds));
        }
        output
    }

    fn render_generic_bounds(&self, bounds: &[GenericBound]) -> Vec<Token> {
        self.render_sequence_if_not_empty(vec![], vec![], plus(), bounds, |bound| match bound {
            GenericBound::TraitBound {
                trait_,
                generic_params,
                modifier,
            } => {
                let mut output = vec![];
                output.extend(self.render_higher_rank_trait_bounds(generic_params));
                match modifier {
                    TraitBoundModifier::None | TraitBoundModifier::MaybeConst=> {}
                    TraitBoundModifier::Maybe => {
                        output.push(Token::symbol("?"));
                    }
                }
                output.extend(self.render_resolved_path(trait_));
                output
            }
            GenericBound::Outlives(id) => vec![Token::lifetime(id)],
            GenericBound::Use(params) => {
                let mut output = vec![Token::keyword("use"), Token::symbol("<")];

                for i in 0..params.len() {
                    let param = &params[i];

                    // Rustdoc doesn't tell us if the parameter is a lifetime or a generic, so we
                    // check if the string starts with `'` to see if it is a lifetime.
                    if param.starts_with('\'') {
                        output.push(Token::lifetime(param));
                    } else {
                        output.push(Token::generic(param));
                    }

                    // Insert a ", " in between parameters, but not after the final one.
                    if i < params.len() - 1 {
                        output.extend_from_slice(&[Token::symbol(","), Token::Whitespace]);
                    }
                }

                output.push(Token::symbol(">"));

                output
            }
        })
    }

    fn render_higher_rank_trait_bounds(&self, generic_params: &[GenericParamDef]) -> Vec<Token> {
        let mut output = vec![];
        if !generic_params.is_empty() {
            output.push(Token::keyword("for"));
            output.extend(self.render_generic_param_defs(generic_params));
            output.push(ws!());
        }
        output
    }

    fn best_item_for_id(&self, id: &'c Id) -> Option<&'c IntermediatePublicItem<'c>> {
        match self.id_to_items.get(&id) {
            None => None,
            Some(items) => {
                items
                    .iter()
                    .max_by(|a, b| {
                        // If there is any item in the path that has been
                        // renamed/re-exported, i.e. that is not the original
                        // path, prefer that less than an item with a path where
                        // all items are original.
                        let mut ordering = match (
                            a.path_contains_renamed_item(),
                            b.path_contains_renamed_item(),
                        ) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => Ordering::Equal,
                        };

                        // If we still can't make up our mind, go with the shortest path
                        if ordering == Ordering::Equal {
                            ordering = b.path().len().cmp(&a.path().len());
                        }

                        ordering
                    })
                    .copied()
            }
        }
    }
}

/// Our list of allowed attributes comes from
/// <https://github.com/rust-lang/rust/blob/68d0b29098/src/librustdoc/html/render/mod.rs#L941-L942>
fn attr_relevant_for_public_apis<S: AsRef<str>>(attr: S) -> bool {
    let prefixes = [
        "#[export_name",
        "#[link_section",
        "#[no_mangle",
        "#[non_exhaustive",
        "#[repr",
    ];

    for prefix in prefixes {
        if attr.as_ref().starts_with(prefix) {
            return true;
        }
    }

    false
}

fn pub_() -> Vec<Token> {
    vec![Token::qualifier("pub"), ws!()]
}

fn plus() -> Vec<Token> {
    vec![ws!(), Token::symbol("+"), ws!()]
}

fn colon() -> Vec<Token> {
    vec![Token::symbol(":"), ws!()]
}

fn comma() -> Vec<Token> {
    vec![Token::symbol(","), ws!()]
}

fn equals() -> Vec<Token> {
    vec![ws!(), Token::symbol("="), ws!()]
}

fn arrow() -> Vec<Token> {
    vec![ws!(), Token::symbol("->"), ws!()]
}
