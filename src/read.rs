//! A Parser of Scheme types.
//! `SExpr` is the AST type here that will be executed.


use combine::parser::char::{char, spaces};
use combine::{satisfy, between, choice, many1, parser, sep_by, Parser};
use combine::error::{ParseError};
use combine::stream::{Stream};

use crate::common::{SExpr};

fn expr_<Input>() -> impl Parser<Input, Output=SExpr>
                     where Input: Stream<Token = char>,
                           Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{

   let atom_char = || satisfy(|ch: char| match ch {
      '0'..='9' | '<'..='Z' | 'a'..='z'
            | '~' | '!' | '$' | '%' | '^' | '&'
            | '*' | '_' | '+' | ':' | '/' | '-' | 'λ'
        => true,
      _ => false,
   });

   // TODO better parsing of atom,
   // some characters only allowed at start like #
   // so have something like optional(char('#')).then(atom_body())
   // but that doesnt work cause fuck if i know.
   let atom_body = || many1(atom_char());
   let atom = atom_body();

   let skip_spaces = || spaces().silent();
   let lex_char = |c| char(c).skip(skip_spaces());

   let space_separated_exprs = sep_by(expr(), spaces());
   let list = between(lex_char('(') , lex_char(')'), space_separated_exprs);

   choice((atom.map(SExpr::Atom),
           list.map(SExpr::List),)).skip(skip_spaces())
}

parser!{
   pub fn expr[Input]()(Input) -> SExpr
   where [Input: Stream<Token = char>]
   {
      expr_()
   }
}
