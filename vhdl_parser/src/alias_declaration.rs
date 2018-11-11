// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com

use ast::{AliasDeclaration, AliasDesignator};
use message::ParseResult;
use names::parse_name;
use source::WithPos;
use subprogram::parse_signature;
use subtype_indication::parse_subtype_indication;
use tokenizer::Kind::*;
use tokenstream::TokenStream;

pub fn parse_alias_declaration(stream: &mut TokenStream) -> ParseResult<AliasDeclaration> {
    stream.expect_kind(Alias)?;

    let token = stream.expect()?;
    let designator = try_token_kind!(
        token,
        Identifier => token.expect_ident()?.map_into(AliasDesignator::Identifier),
        StringLiteral => WithPos::new(AliasDesignator::OperatorSymbol(token.expect_string()?), token),
        Character => WithPos::new(AliasDesignator::Character(token.expect_character()?), token)
    );

    let subtype_indication = {
        if stream.skip_if_kind(Colon)? {
            Some(parse_subtype_indication(stream)?)
        } else {
            None
        }
    };

    stream.expect_kind(Is)?;
    let name = parse_name(stream)?;

    let signature = {
        if stream.peek_kind()? == Some(LeftSquare) {
            Some(parse_signature(stream)?)
        } else {
            None
        }
    };

    stream.expect_kind(SemiColon)?;

    Ok(AliasDeclaration {
        designator,
        subtype_indication: subtype_indication,
        name: name,
        signature: signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Name;
    use test_util::Code;

    #[test]
    fn parse_simple_alias() {
        let code = Code::new("alias foo is name;");
        assert_eq!(
            code.with_stream(parse_alias_declaration),
            AliasDeclaration {
                designator: code.s1("foo").ident().map_into(AliasDesignator::Identifier),
                subtype_indication: None,
                name: code.s1("name").name(),
                signature: None
            }
        );
    }

    #[test]
    fn parse_alias_with_subtype_indication() {
        let code = Code::new("alias foo : vector(0 to 1) is name;");
        assert_eq!(
            code.with_stream(parse_alias_declaration),
            AliasDeclaration {
                designator: code.s1("foo").ident().map_into(AliasDesignator::Identifier),
                subtype_indication: Some(code.s1("vector(0 to 1)").subtype_indication()),
                name: code.s1("name").name(),
                signature: None
            }
        );
    }

    #[test]
    fn parse_alias_with_signature() {
        let code = Code::new("alias foo is name [return natural];");
        assert_eq!(
            code.with_stream(parse_alias_declaration),
            AliasDeclaration {
                designator: code.s1("foo").ident().map_into(AliasDesignator::Identifier),
                subtype_indication: None,
                name: code.s1("name").name(),
                signature: Some(code.s1("[return natural]").signature())
            }
        );
    }

    #[test]
    fn parse_alias_with_operator_symbol() {
        let code = Code::new("alias \"and\" is name;");

        let designator = code.s1("\"and\"").name().map_into(|name| match name {
            Name::OperatorSymbol(sym) => AliasDesignator::OperatorSymbol(sym),
            _ => {
                panic!("{:?}", name);
            }
        });

        assert_eq!(
            code.with_stream(parse_alias_declaration),
            AliasDeclaration {
                designator,
                subtype_indication: None,
                name: code.s1("name").name(),
                signature: None
            }
        );
    }

    #[test]
    fn parse_alias_with_character() {
        let code = Code::new("alias 'c' is 'b';");

        let designator = code.s1("'c'").name().map_into(|name| match name {
            Name::CharacterLiteral(sym) => AliasDesignator::Character(sym),
            _ => {
                panic!("{:?}", name);
            }
        });

        assert_eq!(
            code.with_stream(parse_alias_declaration),
            AliasDeclaration {
                designator,
                subtype_indication: None,
                name: code.s1("'b'").name(),
                signature: None
            }
        );
    }

}
