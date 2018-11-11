// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com

use ast::{ContextDeclaration, ContextItem, ContextReference, LibraryClause, Name, UseClause};
use common::error_on_end_identifier_mismatch;
use message::{error, push_some, MessageHandler, ParseResult};
use names::parse_name;
use source::WithPos;
use tokenizer::Kind::*;
use tokenstream::TokenStream;

/// LRM 13. Design units and their analysis
pub fn parse_library_clause(stream: &mut TokenStream) -> ParseResult<LibraryClause> {
    stream.expect_kind(Library)?;
    parse_library_clause_no_keyword(stream)
}

fn parse_library_clause_no_keyword(stream: &mut TokenStream) -> ParseResult<LibraryClause> {
    let mut name_list = Vec::with_capacity(1);
    loop {
        name_list.push(stream.expect_ident()?);
        if !stream.skip_if_kind(Comma)? {
            break;
        }
    }
    stream.expect_kind(SemiColon)?;
    Ok(LibraryClause { name_list })
}

/// LRM 12.4. Use clauses
fn parse_use_clause_no_keyword(stream: &mut TokenStream) -> ParseResult<UseClause> {
    let mut name_list = Vec::with_capacity(1);
    loop {
        name_list.push(parse_name(stream)?);
        if !stream.skip_if_kind(Comma)? {
            break;
        }
    }
    stream.expect_kind(SemiColon)?;
    Ok(UseClause { name_list })
}

pub fn parse_use_clause(stream: &mut TokenStream) -> ParseResult<UseClause> {
    stream.expect_kind(Use)?;
    parse_use_clause_no_keyword(stream)
}

#[derive(PartialEq, Debug)]
pub enum DeclarationOrReference {
    Declaration(ContextDeclaration),
    Reference(ContextReference),
}

fn parse_context_reference_no_keyword(stream: &mut TokenStream) -> ParseResult<ContextReference> {
    let name = parse_name(stream)?;
    let mut name_list = vec![name];
    loop {
        if !stream.skip_if_kind(Comma)? {
            break;
        }
        name_list.push(parse_name(stream)?);
    }
    stream.expect_kind(SemiColon)?;
    Ok(ContextReference { name_list })
}

/// LRM 13.4 Context clauses
pub fn parse_context(
    stream: &mut TokenStream,
    messages: &mut MessageHandler,
) -> ParseResult<DeclarationOrReference> {
    stream.expect_kind(Context)?;
    let name = parse_name(stream)?;
    if stream.skip_if_kind(Is)? {
        let mut items = Vec::with_capacity(16);
        let end_ident;
        loop {
            let token = stream.expect()?;
            try_token_kind!(
                token,
                Library => items.push(ContextItem::Library(parse_library_clause_no_keyword(stream)?)),
                Use => items.push(ContextItem::Use(parse_use_clause_no_keyword(stream)?)),
                Context => items.push(ContextItem::Context(parse_context_reference_no_keyword(stream)?)),
                End => {
                    stream.pop_if_kind(Context)?;
                    end_ident = stream.pop_optional_ident()?;
                    stream.expect_kind(SemiColon)?;
                    break;
                }
            )
        }

        let ident = {
            match name.item {
                Name::Simple(symbol) => WithPos {
                    item: symbol,
                    pos: name.pos,
                },
                _ => {
                    return Err(error(&name, "Expected simple name"));
                }
            }
        };

        push_some(
            messages,
            error_on_end_identifier_mismatch(&ident, &end_ident),
        );

        Ok(DeclarationOrReference::Declaration(ContextDeclaration {
            ident,
            items,
        }))
    } else {
        // Context reference
        let mut name_list = vec![name];
        loop {
            if !stream.skip_if_kind(Comma)? {
                break;
            }
            name_list.push(parse_name(stream)?);
        }
        stream.expect_kind(SemiColon)?;
        Ok(DeclarationOrReference::Reference(ContextReference {
            name_list,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_util::Code;

    #[test]
    fn test_library_clause_single_name() {
        let code = Code::new("library foo;");
        assert_eq!(
            code.with_stream(parse_library_clause),
            LibraryClause {
                name_list: vec![code.s1("foo").ident()]
            }
        )
    }

    #[test]
    fn test_library_clause_multiple_names() {
        let code = Code::new("library foo, bar;");
        assert_eq!(
            code.with_stream(parse_library_clause),
            LibraryClause {
                name_list: vec![code.s1("foo").ident(), code.s1("bar").ident()]
            }
        )
    }

    #[test]
    fn test_use_clause_single_name() {
        let code = Code::new("use lib.foo;");
        assert_eq!(
            code.with_stream(parse_use_clause),
            UseClause {
                name_list: vec![code.s1("lib.foo").name()]
            }
        )
    }

    #[test]
    fn test_use_clause_multiple_names() {
        let code = Code::new("use foo.'a', lib.bar.all;");
        assert_eq!(
            code.with_stream(parse_use_clause),
            UseClause {
                name_list: vec![code.s1("foo.'a'").name(), code.s1("lib.bar.all").name()]
            }
        )
    }

    #[test]
    fn test_context_reference_single_name() {
        let code = Code::new("context lib.foo;");
        assert_eq!(
            code.with_stream_no_messages(parse_context),
            DeclarationOrReference::Reference(ContextReference {
                name_list: vec![code.s1("lib.foo").name()]
            })
        )
    }

    #[test]
    fn test_context_reference_multiple_names() {
        let code = Code::new("context work.foo, lib.bar.all;");
        assert_eq!(
            code.with_stream_no_messages(parse_context),
            DeclarationOrReference::Reference(ContextReference {
                name_list: vec![code.s1("work.foo").name(), code.s1("lib.bar.all").name()]
            })
        )
    }

    #[test]
    fn test_context_clause() {
        let variants = vec![
            &"\
context ident is
end;
",
            &"\
context ident is
end context;
",
            &"\
context ident is
end ident;
",
            &"\
context ident is
end context ident;
",
        ];
        for variant in variants {
            let code = Code::new(variant);
            assert_eq!(
                code.with_stream_no_messages(parse_context),
                DeclarationOrReference::Declaration(ContextDeclaration {
                    ident: code.s1("ident").ident(),
                    items: vec![]
                })
            );
        }
    }

    #[test]
    fn test_context_clause_error_end_identifier_mismatch() {
        let code = Code::new(
            "\
context ident is
end context ident2;
",
        );
        let (context, messages) = code.with_stream_messages(parse_context);
        assert_eq!(
            messages,
            vec![error(
                code.s1("ident2"),
                "End identifier mismatch, expected ident"
            )]
        );
        assert_eq!(
            context,
            DeclarationOrReference::Declaration(ContextDeclaration {
                ident: code.s1("ident").ident(),
                items: vec![]
            })
        );
    }

    #[test]
    fn test_context_clause_items() {
        let code = Code::new(
            "\
context ident is
  library foo;
  use foo.bar;
  context foo.ctx;
end context;
",
        );
        assert_eq!(
            code.with_stream_no_messages(parse_context),
            DeclarationOrReference::Declaration(ContextDeclaration {
                ident: code.s1("ident").ident(),
                items: vec![
                    ContextItem::Library(LibraryClause {
                        name_list: vec![code.s1("foo").ident()]
                    }),
                    ContextItem::Use(UseClause {
                        name_list: vec![code.s1("foo.bar").name()]
                    }),
                    ContextItem::Context(ContextReference {
                        name_list: vec![code.s1("foo.ctx").name()]
                    }),
                ]
            })
        )
    }

}
