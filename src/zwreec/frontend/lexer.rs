//! Builds a token iterator for twee input.
//!
//! This is lexical analyser using [rustlex](#about-rustlex) to analyse twee input
//! and create a lazy evaluating iterator over lexical tokens.
//!
//! # About Rustlex
//!
//! From rustlex' [readme](https://github.com/LeoTestard/rustlex/blob/master/README.md):
//!
//! > RustLex is a lexical analysers generator, i.e. a program that generate [lexical
//! analysers](http://en.wikipedia.org/wiki/Lexical_analysis) for use in compiler from a
//! description of the language using regular expressions.
//!
//! The Zwreec development team forked rustlex to enable support for some advanced
//! features like preserving accurate line and column numbers for tokens and fix
//! some limitations of the original project. You can find the fork on Github at
//! [Farthen/rustlex](https://github.com/Farthen/rustlex).
//!
//! For more information about Rustlex, visit the original repository by
//! [LeoTestard](https://github.com/LeoTestard/rustlex).
//!
//! # The `TweeLexer` Struct
//!
//! The `TweeLexer` Struct and its accompaning implementations are autogenerated by
//! rustlex' compiler plugin and cannot be documented using rustdoc. To get a better
//! impression of how it is supposed to work, take a look at the uncompiled [source
//! code](/src/zwreec/frontend/lexer.rs.html#308-819)

use std::io::{BufReader, Read};
use utils::extensions::{Peeking, PeekingExt, FilteringScan, FilteringScanExt};
use config::Config;

use self::Token::*;

pub enum LexerError {
    UnexpectedCharacter { character: char, location: (u64, u64) }
}

/// Stores the state for the custom iterator `scan_filter()`
///
/// The `zwreec::utils::extensions` module defines a new iterator `FilteringScan`.
/// This struct stores the state for this iterator.
pub struct ScanState<'a> {
    cfg: &'a Config,
    current_text: String,
    current_text_location: (u64, u64),
    skip_next: bool,
}

/// Builds a Token iterator for twee input.
///
/// This function takes a twee input and uses the `TweeLexer` generated by rustlex
/// to create a token iterator. The tokens are then cleaned using the `Peeking`
/// iterator to provide lookahead and the `FilteringScan` iterator to merge adjacent
/// text tokens and combine variable assignment.
///
/// # Example
///
/// ```
/// # extern crate zwreec;
/// use std::io::Cursor;
///
/// let cfg = zwreec::config::Config::default_config();
/// let mut input = Cursor::new("::Start\nHello World".to_string().into_bytes());
///
/// let tokens = zwreec::frontend::lexer::lex(&cfg, &mut input);
/// ```
#[allow(unused_variables)]
pub fn lex<'a, R: Read>(cfg: &'a Config, input: &'a mut R) -> FilteringScan<Peeking<TweeLexer<BufReader<&'a mut R>>, Token>, ScanState<'a>, fn(&mut ScanState, (Token, Option<Token>)) -> Option<Token>>  {

    let mut lexer = TweeLexer::new(BufReader::new(input));
    lexer.cfg = Some(cfg.clone());

    lexer.peeking().scan_filter(
        ScanState {
            cfg: cfg,
            current_text: String::new(),
            current_text_location: (0, 0),
            skip_next: false,
        },
        {
            fn scan_fn(state: &mut ScanState, elem: (Token, Option<Token>)) -> Option<Token> {
                if state.skip_next {
                    state.skip_next = false;
                    return None;
                }

                match elem {
                    (x @ TokError {..}, _) => {
                        error_panic!(state.cfg => x);
                        None
                    }
                    (TokText {location, text}, Some(TokText{ .. })) => {
                        if state.current_text.len() == 0 {
                            state.current_text_location = location;
                        }

                        state.current_text.push_str(&text);
                        None
                    }
                    (TokText {location, text}, _) => {
                        if state.current_text.len() == 0 {
                            state.current_text_location = location;
                        }

                        state.current_text.push_str(&text);
                        let val = TokText {location: state.current_text_location, text: state.current_text.clone()};
                        state.current_text.clear();
                        Some(val)
                    },
                    (TokVariable {location, name: var}, Some(TokAssign {op_name: op, ..} )) => {
                        state.skip_next = true;
                        Some(TokAssign {location: location, var_name: var, op_name: op} )
                    },
                    (x, _) => Some(x),
                }
            }
            scan_fn
        }
    )
}

/// The resulting Tokens that are returned by the `lex` function.
///
/// These Tokens are matched by our lexical analyser. Every Token but
/// `TokExpression` store their original location inside the analysed input.
/// Some Tokens also use other fields to store additional information.
#[derive(PartialEq,Debug,Clone)]
pub enum Token {
    TokPassage                {location: (u64, u64), name: String},
    TokTagStart               {location: (u64, u64)},
    TokTagEnd                 {location: (u64, u64)},
    TokVarSetStart            {location: (u64, u64)},
    TokVarSetEnd              {location: (u64, u64)},
    TokPassageLink            {location: (u64, u64), display_name: String, passage_name: String},
    TokTag                    {location: (u64, u64), tag_name: String},
    TokText                   {location: (u64, u64), text: String},
    TokFormatBoldStart        {location: (u64, u64)}, TokFormatBoldEnd   {location: (u64, u64)},
    TokFormatItalicStart      {location: (u64, u64)}, TokFormatItalicEnd {location: (u64, u64)},
    TokFormatUnderStart       {location: (u64, u64)}, TokFormatUnderEnd  {location: (u64, u64)},
    TokFormatStrikeStart      {location: (u64, u64)}, TokFormatStrikeEnd {location: (u64, u64)},
    TokFormatSubStart         {location: (u64, u64)}, TokFormatSubEnd    {location: (u64, u64)},
    TokFormatSupStart         {location: (u64, u64)}, TokFormatSupEnd    {location: (u64, u64)},
    TokFormatMonoStart        {location: (u64, u64)}, TokFormatMonoEnd   {location: (u64, u64)},
    TokFormatBulList          {location: (u64, u64)},
    TokFormatNumbList         {location: (u64, u64)},
    TokFormatIndentBlock      {location: (u64, u64)},
    TokFormatHorizontalLine   {location: (u64, u64)},
    TokFormatHeading          {location: (u64, u64), rank: u8, text: String},
    TokMacroStart             {location: (u64, u64)},
    TokMacroEnd               {location: (u64, u64)},
    TokMacroContentVar        {location: (u64, u64), var_name: String},
    TokMacroSet               {location: (u64, u64)},
    TokMacroIf                {location: (u64, u64)},
    TokMacroElse              {location: (u64, u64)},
    TokMacroElseIf            {location: (u64, u64)},
    TokMacroEndIf             {location: (u64, u64)},
    TokMacroPrint             {location: (u64, u64)},
    TokMacroDisplay           {location: (u64, u64), passage_name: String},
    TokMacroSilently          {location: (u64, u64)},
    TokMacroEndSilently       {location: (u64, u64)},
    TokMacroNoBr              {location: (u64, u64)},
    TokMacroEndNoBr           {location: (u64, u64)},
    TokParenOpen              {location: (u64, u64)},
    TokParenClose             {location: (u64, u64)},
    TokVariable               {location: (u64, u64), name: String},
    TokArrayAccess            {location: (u64, u64), name: String, index: String},
    TokArrayLength            {location: (u64, u64), name: String},
    TokInt                    {location: (u64, u64), value: i32},
    TokFloat                  {location: (u64, u64), value: f32},
    TokString                 {location: (u64, u64), value: String},
    TokBoolean                {location: (u64, u64), value: String},
    TokFunction               {location: (u64, u64), name: String},
    TokColon                  {location: (u64, u64)},
    TokArgsEnd                {location: (u64, u64)},
    TokArrayStart             {location: (u64, u64)},
    TokArrayEnd               {location: (u64, u64)},
    TokAssign                 {location: (u64, u64), var_name: String, op_name: String},
    TokNumOp                  {location: (u64, u64), op_name: String},
    TokCompOp                 {location: (u64, u64), op_name: String},
    TokLogOp                  {location: (u64, u64), op_name: String},
    TokSemiColon              {location: (u64, u64)},
    TokNewLine                {location: (u64, u64)},
    TokUnaryMinus             {location: (u64, u64)},
    TokExpression,
    TokError                  {location: (u64, u64), message: String},
}

impl Token {
    /// Returns the original location inside the twee input as `(line, column)`.
    pub fn location(&self) -> (u64, u64) {
        match self {
            &TokPassage{location, ..} |
            &TokTagStart{location} |
            &TokTagEnd{location} |
            &TokVarSetStart{location} |
            &TokVarSetEnd{location} |
            &TokPassageLink{location, ..} |
            &TokTag{location, ..} |
            &TokText{location, ..} |
            &TokFormatBoldStart{location} |
            &TokFormatBoldEnd{location} |
            &TokFormatItalicStart{location} |
            &TokFormatItalicEnd{location} |
            &TokFormatUnderStart {location} |
            &TokFormatUnderEnd{location} |
            &TokFormatStrikeStart{location} |
            &TokFormatStrikeEnd{location} |
            &TokFormatSubStart{location} |
            &TokFormatSubEnd{location} |
            &TokFormatSupStart{location} |
            &TokFormatSupEnd{location} |
            &TokFormatMonoStart{location} |
            &TokFormatMonoEnd{location} |
            &TokFormatBulList{location} |
            &TokFormatNumbList{location} |
            &TokFormatIndentBlock{location} |
            &TokFormatHorizontalLine{location} |
            &TokFormatHeading{location, ..} |
            &TokMacroStart{location} |
            &TokMacroEnd{location} |
            &TokMacroContentVar{location, ..} |
            &TokMacroSet{location} |
            &TokMacroIf{location} |
            &TokMacroElse{location} |
            &TokMacroElseIf{location} |
            &TokMacroEndIf{location} |
            &TokMacroPrint{location} |
            &TokMacroDisplay{location, ..} |
            &TokMacroSilently{location} |
            &TokMacroEndSilently{location} |
            &TokMacroNoBr{location} |
            &TokMacroEndNoBr{location} |
            &TokParenOpen{location} |
            &TokParenClose{location} |
            &TokVariable{location, ..} |
            &TokArrayLength{location, ..} |
            &TokArrayAccess{location, ..} |
            &TokInt{location, ..} |
            &TokFloat{location, ..} |
            &TokString{location, ..} |
            &TokBoolean{location, ..} |
            &TokFunction{location, ..} |
            &TokColon{location} |
            &TokArgsEnd{location} |
            &TokArrayStart{location} |
            &TokArrayEnd{location} |
            &TokAssign{location, ..} |
            &TokNumOp{location, ..} |
            &TokCompOp{location, ..} |
            &TokLogOp{location, ..} |
            &TokSemiColon{location} |
            &TokNewLine{location} |
            &TokUnaryMinus{location} |
            &TokError{location, ..}
                => location,
            &TokExpression => (0, 0)
        }
    }
}

//we re-create tokens with location {0,0} in the parser.
//so comparing also works this way
impl Token {
    pub fn is_same_token(&self, other: &Token) -> bool {
        match (self, other) {
            (&TokPassage{..}, &TokPassage{..}) => true,
            (&TokTagStart{..}, &TokTagStart{..}) => true,
            (&TokTagEnd{..}, &TokTagEnd{..}) => true,
            (&TokVarSetStart{..}, &TokVarSetStart{..}) => true,
            (&TokVarSetEnd{..}, &TokVarSetEnd{..}) => true,
            (&TokPassageLink{..}, &TokPassageLink{..}) => true,
            (&TokTag{..}, &TokTag{..}) => true,
            (&TokText{..}, &TokText{..}) => true,
            (&TokFormatBoldStart{..}, &TokFormatBoldStart{..}) => true,
            (&TokFormatBoldEnd{..}, &TokFormatBoldEnd{..}) => true,
            (&TokFormatItalicStart{..}, &TokFormatItalicStart{..}) => true,
            (&TokFormatItalicEnd{..}, &TokFormatItalicEnd{..}) => true,
            (&TokFormatUnderStart {..}, &TokFormatUnderStart {..}) => true,
            (&TokFormatUnderEnd{..}, &TokFormatUnderEnd{..}) => true,
            (&TokFormatStrikeStart{..}, &TokFormatStrikeStart{..}) => true,
            (&TokFormatStrikeEnd{..}, &TokFormatStrikeEnd{..}) => true,
            (&TokFormatSubStart{..}, &TokFormatSubStart{..}) => true,
            (&TokFormatSubEnd{..}, &TokFormatSubEnd{..}) => true,
            (&TokFormatSupStart{..}, &TokFormatSupStart{..}) => true,
            (&TokFormatSupEnd{..}, &TokFormatSupEnd{..}) => true,
            (&TokFormatMonoStart{..}, &TokFormatMonoStart{..}) => true,
            (&TokFormatMonoEnd{..}, &TokFormatMonoEnd{..}) => true,
            (&TokFormatBulList{..}, &TokFormatBulList{..}) => true,
            (&TokFormatNumbList{..}, &TokFormatNumbList{..}) => true,
            (&TokFormatIndentBlock{..}, &TokFormatIndentBlock{..}) => true,
            (&TokFormatHorizontalLine{..}, &TokFormatHorizontalLine{..}) => true,
            (&TokFormatHeading{..}, &TokFormatHeading{..}) => true,
            (&TokMacroStart{..}, &TokMacroStart{..}) => true,
            (&TokMacroEnd{..}, &TokMacroEnd{..}) => true,
            (&TokMacroContentVar{..}, &TokMacroContentVar{..}) => true,
            (&TokMacroSet{..}, &TokMacroSet{..}) => true,
            (&TokMacroIf{..}, &TokMacroIf{..}) => true,
            (&TokMacroElse{..}, &TokMacroElse{..}) => true,
            (&TokMacroElseIf{..}, &TokMacroElseIf{..}) => true,
            (&TokMacroEndIf{..}, &TokMacroEndIf{..}) => true,
            (&TokMacroPrint{..}, &TokMacroPrint{..}) => true,
            (&TokMacroDisplay{..}, &TokMacroDisplay{..}) => true,
            (&TokMacroSilently{..}, &TokMacroSilently{..}) => true,
            (&TokMacroEndNoBr{..}, &TokMacroEndNoBr{..}) => true,
            (&TokMacroNoBr{..}, &TokMacroNoBr{..}) => true,
            (&TokMacroEndSilently{..}, &TokMacroEndSilently{..}) => true,
            (&TokParenOpen{..}, &TokParenOpen{..}) => true,
            (&TokParenClose{..}, &TokParenClose{..}) => true,
            (&TokVariable{..}, &TokVariable{..}) => true,
            (&TokArrayLength{..}, &TokArrayLength{..}) => true,
            (&TokArrayAccess{..}, &TokArrayAccess{..}) => true,
            (&TokInt{..}, &TokInt{..}) => true,
            (&TokFloat{..}, &TokFloat{..}) => true,
            (&TokString{..}, &TokString{..}) => true,
            (&TokBoolean{..}, &TokBoolean{..}) => true,
            (&TokFunction{..}, &TokFunction{..}) => true,
            (&TokColon{..}, &TokColon{..}) => true,
            (&TokArgsEnd{..}, &TokArgsEnd{..}) => true,
            (&TokArrayStart{..}, &TokArrayStart{..}) => true,
            (&TokArrayEnd{..}, &TokArrayEnd{..}) => true,
            (&TokAssign{..}, &TokAssign{..}) => true,
            (&TokNumOp{..}, &TokNumOp{..}) => true,
            (&TokCompOp{..}, &TokCompOp{..}) => true,
            (&TokLogOp{..}, &TokLogOp{..}) => true,
            (&TokSemiColon{..}, &TokSemiColon{..}) => true,
            (&TokNewLine{..}, &TokNewLine{..}) => true,
            (&TokUnaryMinus{..}, &TokUnaryMinus{..}) => true,
            (&TokError{..}, &TokError{..}) => true,
            (&TokExpression, &TokExpression) => true,
            _ => false,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/rustlex.rs"));

fn unescape(s: String) -> String {

    // strip the quotes from strings
    let trimmed = &s[1 .. s.len() - 1];

    // unescape quotes
    let quote_type = s.chars().next().unwrap();
    let mut unescaped = String::new();

    for (c, peek) in trimmed.chars().peeking() {
        if let Some(nextc) = peek {
            if c == '\\' && nextc == quote_type {
                continue;
            }
        }

        unescaped.push(c);
    }

    unescaped
}


// ================================
// test functions
#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::fmt::Write;
    use config::Config;

    use super::*;
    use super::Token::*;

    fn test_lex(input: &str) -> Vec<Token> {
        let cfg = Config::default_config();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
        lex(&cfg, &mut cursor).collect()
    }

    fn assert_tok_eq(expected: Vec<Token>, tokens: Vec<Token>) {
        let mut panic_msg = String::new();
        if tokens.len() != expected.len() {
            write!(&mut panic_msg, "Got {} tokens, expected {}\n\n", tokens.len(), expected.len()).unwrap();
        }

        for i in 0..tokens.len() {
            if tokens[i] != expected[i] {
                write!(&mut panic_msg, "Unexpected token #{}:\n'{:?}', expected\n'{:?}'\n\n", i, tokens[i], expected[i]).unwrap();
            }
        }

        if panic_msg.len() != 0 {
            panic!(panic_msg);
        }
    }

    #[test]
    fn test_assert_tok_eq() {
        assert_tok_eq(vec![
            TokPassage { name: "Start".to_string(), location: (1, 3) },
            TokText { text: "TestText".to_string(), location: (2, 1) },
        ], vec![
            TokPassage { name: "Start".to_string(), location: (1, 3) },
            TokText { text: "TestText".to_string(), location: (2, 1) },
        ]);
    }

    #[test]
    #[should_panic]
    fn test_assert_tok_eq_fail() {
        assert_tok_eq(vec![
            TokPassage { name: "Start".to_string(), location: (1, 3) },
            TokText { text: "TestText".to_string(), location: (2, 1) },
        ], vec![
            TokPassage { name: "Start".to_string(), location: (1, 4) },
            TokText { text: "TestText".to_string(), location: (2, 1) },
        ]);
    }

    #[test]
    fn preprocessing_test() {
        // This should remove all comments
        let tokens = test_lex("/%\nVortest1\n%/\n   /% Vortest2 %/\n        bla\nLorem Ipsum doloris!\n\n::Start /% Test %/\nText1, der bleiben muss\n/% Test1 %/\nText2, der bleiben muss\n/% /% Test2 %/\nText3, der bleiben muss\n/%% Test3 %/\nText4, der bleiben muss\n/% Test4 %%/ ! TestHeading1\nText5, der bleiben muss\n/% /% Test5 %/ %/\nText6, der bleiben muss\n/%%%%%%\nTest6\n%%%%%%%/\nText7, der bleiben muss /%\n/%\nTest7\n%/ ! TestHeading2\n/%%/\nText Text /% Test8 %/ Text Text\n{{{ Monospace /% Kommentar %/ }}}\n<<if true>> Bla /% Test10 %/ <<endif>>");
        let expected = vec!(
            TokPassage {name: "Start /% Test %/".to_string(), location: (8, 3)},
            TokText { text: "Text1, der bleiben muss".to_string(), location: (9, 1) },
            TokNewLine { location: (9, 24) },
            TokNewLine { location: (10, 12) },
            TokText { text: "Text2, der bleiben muss".to_string(), location: (11, 1) },
            TokNewLine { location: (11, 24) },
            TokNewLine { location: (12, 15) },
            TokText { text: "Text3, der bleiben muss".to_string(), location: (13, 1) },
            TokNewLine { location: (13, 24) },
            TokNewLine { location: (14, 13) },
            TokText { text: "Text4, der bleiben muss".to_string(), location: (15, 1) },
            TokNewLine { location: (15, 24) },
            TokText { text: " ! TestHeading1".to_string(), location: (16, 13) },
            TokNewLine { location: (16, 28) },
            TokText { text: "Text5, der bleiben muss".to_string(), location: (17, 1) },
            TokNewLine { location: (17, 24) },
            TokText { text: " %/".to_string(), location: (18, 15) },
            TokNewLine { location: (18, 18) },
            TokText { text: "Text6, der bleiben muss".to_string(), location: (19, 1) },
            TokNewLine { location: (19, 24) },
            TokNewLine { location: (22, 9) },
            TokText { text: "Text7, der bleiben muss  ! TestHeading2".to_string(), location: (23, 1) },
            TokNewLine { location: (26, 18) },
            TokNewLine { location: (27, 5) },
            TokText { text: "Text Text  Text Text".to_string(), location: (28, 1) },
            TokNewLine { location: (28, 32) },
            TokFormatMonoStart { location: (29, 1) },
            TokText { text: " Monospace /% Kommentar %/ ".to_string(), location: (29, 4) },
            TokFormatMonoEnd { location: (29, 31) },
            TokNewLine { location: (29, 34) },
            TokMacroIf { location: (30, 3) },
            TokBoolean { value: "true".to_string(), location: (30, 6) },
            TokMacroEnd { location: (30, 10) },
            TokText { text: " Bla  ".to_string(), location: (30, 12) },
            TokMacroEndIf { location: (30, 32) },
            TokMacroEnd { location: (30, 37) }
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn passage_test() {
        // This should detect the ::Start passage
        let start_tokens = test_lex("::Start");
        let expected = vec!(
            TokPassage {name: "Start".to_string(), location: (1, 3)}
        );

        assert_tok_eq(expected, start_tokens);

        // This should not return any tokens
        let fail_tokens = test_lex(":fail");
        assert_eq!(0, fail_tokens.len());
    }

    #[test]
    fn text_test() {
        // This should return a passage with a body text
        let tokens = test_lex("::Passage\nTestText\nTestNextLine\n::NextPassage");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokText {location: (2, 1), text: "TestText".to_string()},
            TokNewLine {location: (2, 9)},
            TokText {location: (3, 1), text: "TestNextLine".to_string()},
            TokNewLine {location: (3, 13)} ,
            TokPassage {name: "NextPassage".to_string(), location: (4, 3)},
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn tag_test() {
        // This should return a passage with tags
        let tokens = test_lex("::TagPassage [tag1 tag2]\nContent");
        let expected = vec!(
            TokPassage {name: "TagPassage".to_string(), location: (1, 3)},
            TokTagStart {location: (1, 14)},
            TokTag {location: (1, 15), tag_name: "tag1".to_string()},
            TokTag {location: (1, 20), tag_name: "tag2".to_string()},
            TokTagEnd {location: (1, 24)},
            TokText {location: (2, 1), text: "Content".to_string()}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_set_test() {
        // This should return a passage with a set macro
        let tokens = test_lex("::Passage\n<<set $var = 1>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroSet {location: (2, 3)},
            TokAssign {location: (2, 7), var_name: "$var".to_string(), op_name: "=".to_string()},
            TokInt {location: (2, 14), value: 1},
            TokMacroEnd {location: (2, 15)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_if_test() {
        // This should return a passage with an if macro
        let tokens = test_lex("::Passage\n<<if $var1 == 1>>1<<else if $var2 is 2>>2<<else>>3<<endif>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroIf {location: (2, 3)},
            TokVariable {location: (2, 6), name: "$var1".to_string()},
            TokCompOp {location: (2, 12), op_name: "==".to_string()},
            TokInt {location: (2, 15), value: 1},
            TokMacroEnd {location: (2, 16)},
            TokText {text: "1".to_string(), location: (2, 18) },
            TokMacroElseIf {location: (2, 21)},
            TokVariable {name: "$var2".to_string(), location: (2, 29)},
            TokCompOp {op_name: "is".to_string(), location: (2, 35)},
            TokInt {location: (2, 38), value: 2},
            TokMacroEnd {location: (2, 39)},
            TokText {text: "2".to_string(), location: (2, 41)},
            TokMacroElse {location: (2, 44)},
            TokMacroEnd {location: (2, 48)},
            TokText {text: "3".to_string(), location: (2, 50)},
            TokMacroEndIf {location: (2, 53)},
            TokMacroEnd {location: (2, 58)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_print_test() {
        let tokens = test_lex("::Passage\n<<print \"Test with escaped \\\"Quotes\">>\n<<print $var>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroPrint {location: (2, 3)},
            TokString {location: (2, 9), value: "Test with escaped \"Quotes".to_string()},
            TokMacroEnd {location: (2, 37)},
            TokNewLine {location: (2, 39)},
            TokMacroPrint {location: (3, 3)},
            TokVariable {location: (3, 9), name: "$var".to_string()},
            TokMacroEnd {location: (3, 13)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_display_test() {
        let tokens = test_lex("::Passage\n<<display Passage>>\n<<display  Passage  >>\n<<display  Passage\n>>\n<<display \'Passage\'>>\n<<display  \'Passage\'  >>\n<<display  \'Passage\'\n>>\n<<display \"Passage\">>\n<<display  \"Passage\"  >>\n<<display  \"Passage\"\n>>\n<<display Passage Passage>>\n<<display  Passage Passage  >>\n<<display  Passage Passage\n>>\n<<display \'Passage Passage\'>>\n<<display  \'Passage Passage\'  >>\n<<display  \'Passage Passage\'\n>>\n<<display \"Passage Passage\">>\n<<display  \"Passage Passage\"  >>\n<<display  \"Passage Passage\"\n>>\n<<display \"Passage\" 0+1>>\n<<display \"Passage\" 5+6\"P\" assage>>\n<<display Passage >Passage>>");
        let expected = vec!(
            TokPassage { location: (1, 3), name: "Passage".to_string() },
            TokMacroDisplay { location: (2, 11), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (2, 18) },
            TokNewLine { location: (2, 20) },
            TokMacroDisplay { location: (3, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (3, 21) },
            TokNewLine { location: (3, 23) },
            TokMacroDisplay { location: (4, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (5, 1) },
            TokNewLine { location: (5, 3) },
            TokMacroDisplay { location: (6, 11), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (6, 20) },
            TokNewLine { location: (6, 22) },
            TokMacroDisplay { location: (7, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (7, 23) },
            TokNewLine { location: (7, 25) },
            TokMacroDisplay { location: (8, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (9, 1) },
            TokNewLine { location: (9, 3) },
            TokMacroDisplay { location: (10, 11), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (10, 20) },
            TokNewLine { location: (10, 22) },
            TokMacroDisplay { location: (11, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (11, 23) },
            TokNewLine { location: (11, 25) },
            TokMacroDisplay { location: (12, 12), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (13, 1) },
            TokNewLine { location: (13, 3) },
            TokMacroDisplay { location: (14, 11), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (14, 26) },
            TokNewLine { location: (14, 28) },
            TokMacroDisplay { location: (15, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (15, 29) },
            TokNewLine { location: (15, 31) },
            TokMacroDisplay { location: (16, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (17, 1) },
            TokNewLine { location: (17, 3) },
            TokMacroDisplay { location: (18, 11), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (18, 28) },
            TokNewLine { location: (18, 30) },
            TokMacroDisplay { location: (19, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (19, 31) },
            TokNewLine { location: (19, 33) },
            TokMacroDisplay { location: (20, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (21, 1) },
            TokNewLine { location: (21, 3) },
            TokMacroDisplay { location: (22, 11), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (22, 28) },
            TokNewLine { location: (22, 30) },
            TokMacroDisplay { location: (23, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (23, 31) },
            TokNewLine { location: (23, 33) },
            TokMacroDisplay { location: (24, 12), passage_name: "Passage Passage".to_string() },
            TokMacroEnd { location: (25, 1) },
            TokNewLine { location: (25, 3) },
            TokMacroDisplay { location: (26, 11), passage_name: "\"Passage\" 0+1".to_string() },
            TokMacroEnd { location: (26, 24) },
            TokNewLine { location: (26, 26) },
            TokMacroDisplay { location: (27, 11), passage_name: "\"Passage\" 5+6\"P\" assage".to_string() },
            TokMacroEnd { location: (27, 34) },
            TokNewLine { location: (27, 36) },
            TokMacroDisplay { location: (28, 11), passage_name: "Passage >Passage".to_string() },
            TokMacroEnd { location: (28, 27) }
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_display_short_test() {
        let tokens = test_lex("::Passage\n<<Passage>>\n<<Passage   >>\n<<Passage\n>>\n<<Passage 5>>\n<<Passage \"test\">>\n<<Passage \"test\"+5>>\n<<\"Passage\'>>");
        let expected = vec![
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroDisplay {location: (2, 3), passage_name: "Passage".to_string()},
            TokMacroEnd {location: (2, 10)},
            TokNewLine { location: (2, 12) },
            TokMacroDisplay { location: (3, 3), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (3, 13) },
            TokNewLine { location: (3, 15) },
            TokMacroDisplay { location: (4, 3), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (5, 1) },
            TokNewLine { location: (5, 3) },
            TokMacroDisplay { location: (6, 3), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (6, 12) },
            TokNewLine { location: (6, 14) },
            TokMacroDisplay { location: (7, 3), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (7, 17) },
            TokNewLine { location: (7, 19) },
            TokMacroDisplay { location: (8, 3), passage_name: "Passage".to_string() },
            TokMacroEnd { location: (8, 19) },
            TokNewLine { location: (8, 21) },
            TokMacroDisplay { location: (9, 3), passage_name: "\"Passage\'".to_string() },
            TokMacroEnd { location: (9, 12) }
        ];

        assert_tok_eq(expected, tokens);

        let fail_tokens = test_lex(":fail");
        assert_eq!(0, fail_tokens.len());
    }

    #[test]
    fn macro_print_function_test() {
        let tokens = test_lex("::Start\n<<print random(1, 100)>>");
        let expected = vec![
            TokPassage {name: "Start".to_string(), location: (1, 3)},
            TokMacroPrint {location: (2, 3)},
            TokFunction {name: "random".to_string(), location: (2, 9)},
            TokInt {value: 1, location: (2, 16)},
            TokColon {location: (2, 17)},
            TokInt {value: 100, location: (2, 19)},
            TokArgsEnd {location: (2, 22)},
            TokMacroEnd {location: (2, 23)}
        ];

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn url_test() {
        let tokens = test_lex("::Start\nhttp://foo.com/blah_blahhttp://foo.com/blah_blah/ http://foo.com/blah_blah_(wikipedia) http://foo.com/blah_blah_(wikipedia)_(again) http://www.example.com/wpstyle/?p=364 https://www.example.com/foo/?bar=baz&inga=42&quux http://✪df.ws/123 http://userid:password@example.com:8080 http://userid:password@example.com:8080/ http://userid@example.com http://userid@example.com/ http://userid@example.com:8080 http://userid@example.com:8080/ http://userid:password@example.com http://userid:password@example.com/ http://142.42.1.1/ http://142.42.1.1:8080/ http://➡.ws/䨹 http://⌘.ws http://⌘.ws/ http://foo.com/blah_(wikipedia)#cite-1 http://foo.com/blah_(wikipedia)_blah#cite-1 http://foo.com/unicode_(✪)_in_parens http://foo.com/(something)?after=parens http://☺.damowmow.com/ http://code.google.com/events/#&product=browser http://j.mp ftp://foo.bar/baz http://foo.bar/?q=Test%20URL-encoded%20stuff http://例子.测试 http://उदाहरण.परीक्षा http://-.~_!$&'()*+,;=:%40:80%2f::::::@example.com http://1337.net http://a.b-c.de http://223.255.255.254");
        let expected = vec![
            TokPassage {name: "Start".to_string(), location: (1, 3)},
            TokText {text: "http://foo.com/blah_blahhttp://foo.com/blah_blah/ http://foo.com/blah_blah_(wikipedia) http://foo.com/blah_blah_(wikipedia)_(again) http://www.example.com/wpstyle/?p=364 https://www.example.com/foo/?bar=baz&inga=42&quux http://\u{272a}df.ws/123 http://userid:password@example.com:8080 http://userid:password@example.com:8080/ http://userid@example.com http://userid@example.com/ http://userid@example.com:8080 http://userid@example.com:8080/ http://userid:password@example.com http://userid:password@example.com/ http://142.42.1.1/ http://142.42.1.1:8080/ http://➡.ws/䨹 http://⌘.ws http://⌘.ws/ http://foo.com/blah_(wikipedia)#cite-1 http://foo.com/blah_(wikipedia)_blah#cite-1 http://foo.com/unicode_(\u{272a})_in_parens http://foo.com/(something)?after=parens http://\u{263a}.damowmow.com/ http://code.google.com/events/#&product=browser http://j.mp ftp://foo.bar/baz http://foo.bar/?q=Test%20URL-encoded%20stuff http://\u{4f8b}\u{5b50}.\u{6d4b}\u{8bd5} http://\u{909}\u{926}\u{93e}\u{939}\u{930}\u{923}.\u{92a}\u{930}\u{940}\u{915}\u{94d}\u{937}\u{93e} http://-.~_!$&\'()*+,;=:%40:80%2f::::::@example.com http://1337.net http://a.b-c.de http://223.255.255.254".to_string(), location: (2, 1)},
        ];

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn html_filter_test() {
        let tokens = test_lex("Text\n\n::Start\n<!DOCTYPE html>\n<html lang=\"de\">\n\n<head>\n  <meta charset=\"UTF-8\"/>\n  <title>Example</title>\n  <img src=\"smiley.gif\" alt=\"Smiley face\" height=\"42\" width=\"42\">\n  <style type=\"text/css\">\n    <a href=\"http://www.w3schools.com\">Visit W3Schools.com!</a>\n  </style>\n\n</head>\n<body>\n\n</body>\n</html>\n\n::Passage1 [stylesheet]\ntext shouldn't be displayed\n\n::Passage2\ntext <html><b>should</b></html> be displayed\n\n::Passage3 [script]\ntext shouldn't be displayed\n");
        let expected = vec![
            TokPassage {name: "Start".to_string(), location: (3, 3)},
            TokNewLine { location: (4, 16) },
            TokNewLine { location: (5, 17) },
            TokNewLine { location: (6, 1) },
            TokNewLine { location: (7, 7) },
            TokText { location: (8, 1), text: "  ".to_string() },
            TokNewLine { location: (8, 26) },
            TokText { location: (9, 1), text: "  Example".to_string() },
            TokNewLine { location: (9, 25) },
            TokText { location: (10, 1), text: "  ".to_string() },
            TokNewLine { location: (10, 66) },
            TokText { location: (11, 1), text: "  ".to_string() },
            TokNewLine { location: (11, 26) },
            TokText { location: (12, 1), text: "    Visit W3Schools.com!".to_string() },
            TokNewLine { location: (12, 64) },
            TokText { location: (13, 1), text: "  ".to_string() },
            TokNewLine { location: (13, 11) },
            TokNewLine { location: (14, 1) },
            TokNewLine { location: (15, 8) },
            TokNewLine { location: (16, 7) },
            TokNewLine { location: (17, 1) },
            TokNewLine { location: (18, 8) },
            TokNewLine { location: (19, 8) },
            TokNewLine { location: (20, 1) },
            TokPassage { location: (21, 3), name: "Passage1".to_string() },
            TokTagStart { location: (21, 12) },
            TokTag { location: (21, 13), tag_name: "stylesheet".to_string() },
            TokTagEnd { location: (21, 23) },
            TokPassage { location: (24, 3), name: "Passage2".to_string() },
            TokText { location: (25, 1), text: "text should be displayed".to_string() },
            TokNewLine { location: (25, 45) },
            TokNewLine { location: (26, 1) },
            TokPassage { location: (27, 3), name: "Passage3".to_string() },
            TokTagStart { location: (27, 12) },
            TokTag { location: (27, 13), tag_name: "script".to_string() },
            TokTagEnd { location: (27, 19) },

        ];

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn escape_line_break_test() {
        let tokens = test_lex("::Start\nTest\\\n<<if true>>\\\nLi\\ne\n<<else>>\nBla\\\n<<endif>>\n\\\nTest");
        let expected = vec![
            TokPassage { name: "Start".to_string(), location: (1, 3) },
            TokText { text: "Test".to_string(), location: (2, 1) },
            TokMacroIf { location: (3, 3) },
            TokBoolean { location: (3, 6), value: "true".to_string() },
            TokMacroEnd { location: (3, 10) },
            TokText { text: "Li\\ne".to_string(), location: (4, 1) },
            TokNewLine { location: (4, 6) },
            TokMacroElse { location: (5, 3) },
            TokMacroEnd { location: (5, 7) },
            TokNewLine { location: (5, 9) },
            TokText { text: "Bla".to_string(), location: (6, 1) },
            TokMacroEndIf { location: (7, 3) },
            TokMacroEnd { location: (7, 8) },
            TokNewLine { location: (7, 10) },
            TokText { text: "Test".to_string(), location: (9, 1) },
        ];

        assert_tok_eq(expected, tokens);
    }
}
