use public_items::diff::PublicItemsDiff;

pub fn print_diff(w: &mut impl std::io::Write, diff: &PublicItemsDiff) -> std::io::Result<()> {
    if diff.removed.is_empty() && diff.changed.is_empty() && diff.added.is_empty() {
        return writeln!(w, "(No changes to the public API)");
    }

    print_items_with_header(
        w,
        "## Removed items from the public API",
        &diff.removed,
        |w, item| writeln!(w, "* `{}`", item),
    )?;

    print_items_with_header(
        w,
        "## Changed items in the public API",
        &diff.changed,
        |w, changed_item| {
            writeln!(
                w,
                "* `{}` changed to\n  `{}`",
                changed_item.old, changed_item.new
            )
        },
    )?;

    print_items_with_header(
        w,
        "## Added items to the public API",
        &diff.added,
        |w, item| writeln!(w, "* {}", print_with_colour(item)),
    )?;

    Ok(())
}

fn print_items_with_header<W: std::io::Write, T>(
    w: &mut W,
    header: &str,
    items: &[T],
    print_fn: impl Fn(&mut W, &T) -> std::io::Result<()>,
) -> std::io::Result<()> {
    writeln!(w, "{}", header)?;
    if items.is_empty() {
        writeln!(w, "(none)")?;
    } else {
        for item in items {
            print_fn(w, item)?;
        }
    }
    writeln!(w)
}

use ansi_term::{ANSIString, ANSIStrings, Colour, Style};
use public_items::tokens::*;

fn print_with_colour(item: &public_items::PublicItem) -> String {
    if let Ok(tokens) = item.tokens() {
        let mut output = Vec::new();
        let space = Style::new().paint(" ");

        for qual in &tokens.qualifiers {
            output.push(Colour::Blue.paint(qual.to_string()));
            output.push(space.clone());
        }

        let mut path = Vec::new();
        for id in &tokens.path {
            path.push(Colour::Blue.paint(id));
            path.push(Style::new().paint("::"));
        }
        path.remove(path.len() - 1);

        match &tokens.kind {
            Kind::Function {
                generics,
                arguments,
                return_type,
            } => {
                output.push(Style::new().paint("fn "));
                output.extend(path);
                if !generics.params.is_empty() {
                    output.push(Style::new().paint("<"));
                    for param in &generics.params {
                        output.push(Colour::Blue.paint(param.name.clone()));
                        output.push(Style::new().paint(": "));
                        output.push(Colour::Red.paint("generic stuff")); // TODO: add better support
                        output.push(Style::new().paint(","));
                    }
                    output.remove(output.len() - 1);
                    output.push(Style::new().paint(">"));
                }
                output.push(Style::new().paint("("));
                for (name, ty) in arguments {
                    output.push(Style::new().dimmed().paint(name));
                    output.push(Style::new().paint(": "));
                    output.extend(colour_type(ty));
                    output.push(Style::new().paint(","));
                }
                if !arguments.is_empty() {
                    output.remove(output.len() - 1);
                }
                output.push(Style::new().paint(")"));
                if let Some(ty) = return_type {
                    output.push(Style::new().paint(" -> "));
                    output.extend(colour_type(ty));
                }
            }
            Kind::Enum => {
                output.push(Style::new().paint("enum "));
                output.extend(path);
            }
            Kind::EnumVariant(var) => {
                output.push(Style::new().paint("enum variant "));
                output.extend(path);

                match var {
                    Variant::Plain => {}
                    Variant::Tuple(types) => output.extend(colour_tuple(types)),
                    Variant::Struct(ids) => {
                        output.push(Style::new().paint("{"));
                        for id in ids {
                            output.push(Colour::Green.paint(id.0.clone()));
                            output.push(Style::new().paint(", "));
                        }
                        output.remove(output.len() - 1);
                        output.push(Style::new().paint("}"));
                    }
                }
            }
            Kind::Struct => {
                output.push(Style::new().paint("struct "));
                output.extend(path);
            }
            Kind::StructField(ty) => {
                output.push(Style::new().paint("struct field "));
                output.extend(path);
                output.push(Style::new().paint(": "));
                output.extend(colour_type(ty));
            }
        }

        ANSIStrings(&output).to_string()
    } else {
        format!("{}", item)
    }
}

fn colour_type(ty: &Type) -> Vec<ANSIString<'_>> {
    match ty {
        Type::ResolvedPath { name, .. } => vec![Colour::Yellow.paint(name)],
        Type::Generic(name) => vec![Colour::Green.bold().paint(name)],
        Type::Primitive(name) => vec![Colour::Green.paint(name)],
        Type::FunctionPointer(_) => vec![Colour::Red.paint("Function pointer")], // TODO: add something better
        Type::Tuple(types) => colour_tuple(types),
        Type::Slice(ty) => {
            let mut output = vec![Style::new().paint("[")];
            output.extend(colour_type(ty));
            output.push(Style::new().paint("]"));
            output
        }
        Type::Array { type_, len } => {
            let mut output = vec![Style::new().paint("[")];
            output.extend(colour_type(type_));
            output.push(Style::new().paint(": "));
            output.push(Colour::Green.bold().paint(len));
            output.push(Style::new().paint("]"));
            output
        }
        Type::ImplTrait(bounds) => {
            let mut output = vec![Style::new().paint("impl ")];
            for bound in bounds {
                if let GenericBound::TraitBound { trait_, .. } = bound {
                    output.extend(colour_type(trait_));
                    output.push(Style::new().paint(" + "));
                }
            }
            output.remove(output.len() - 1);
            output
        }
        Type::Infer => vec![Style::new().paint("_")],
        Type::RawPointer { mutable, type_ } => {
            let mut output = vec![Style::new().paint("*")];
            if *mutable {
                output.push(Colour::Blue.paint("mut"));
            }
            output.push(Style::new().paint(" "));
            output.extend(colour_type(type_));
            output
        }
        Type::BorrowedRef {
            lifetime,
            mutable,
            type_,
        } => {
            let mut output = vec![Style::new().paint("&")];
            if let Some(lt) = lifetime {
                output.push(Colour::Yellow.paint(lt));
                output.push(Style::new().paint(" "));
            }
            if *mutable {
                output.push(Colour::Blue.paint("mut"));
                output.push(Style::new().paint(" "));
            }
            output.extend(colour_type(type_));
            output
        }
        Type::QualifiedPath {
            name,
            args: _, // TODO: check if this output if correct
            self_type,
            trait_,
        } => {
            let mut output = vec![Style::new().paint("<")];
            output.extend(colour_type(self_type));
            output.push(Style::new().paint(" as "));
            output.extend(colour_type(trait_));
            output.push(Style::new().paint(">::"));
            output.push(Style::new().paint(name));
            output
        }
    }
}

fn colour_tuple(types: &[Type]) -> Vec<ANSIString<'_>> {
    let mut output = vec![Style::new().paint("(")];
    for ty in types {
        output.extend(colour_type(ty));
        output.push(Style::new().paint(", "));
    }
    if !types.is_empty() {
        output.remove(output.len() - 1);
    } else {
        output.push(Style::new().paint(","));
    }
    output.push(Style::new().paint(")"));
    output
}
