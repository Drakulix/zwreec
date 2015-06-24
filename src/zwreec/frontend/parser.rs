//! Constructs a parsing iterator for a twee token iterator.
//!
//! This is a predictive parser for a twee token stream. It takes an `Token`
//! iterator (see [lexer](/zwreec/frontend/lexer/index.html)) and wraps it inside
//! a parsing iterator that returns operations to construct an abstract syntax
//! tree.
//!
//! # Parser
//!
//! The underlying parser is a predictive top-down parser for a LL(1) grammar.
//! For more information about the parser you should refer to *Compilers:
//! Principles, Techniques, and Tools* by A. V. Aho, M. S. Lam, R. Sethi, and J. D. Ullman,
//! Chapter 4.4.4: "Nonrecursive Predictive Parsing".
//!
//! # Grammar
//!
//! As mentioned, the parser operates on a LL(1) grammar. It is documented in [Zwreec's
//! Github Wiki](https://github.com/Drakulix/zwreec/wiki/Underlying-Twee-Grammar#grammar),
//! together with the resulting [parse
//! table](https://github.com/Drakulix/zwreec/wiki/Underlying-Parsetable)
use config::Config;
use frontend::lexer::Token;
use frontend::lexer::Token::*;
use frontend::ast::ASTOperation;
use frontend::ast::ASTOperation::*;
use utils::error::Error;
use utils::extensions::{ParserExt, ParseResult};
use self::NonTerminalType::*;
use self::Elem::*;

//=============================
// error handling

pub enum ParserError {
    TokenDoNotMatch { token: Option<Token>, stack: Token },
    StackIsEmpty { token: Token },
    NoProjection { token: Token, stack: NonTerminalType },
    NonTerminalEnd { stack: NonTerminalType },
}

/// The Type of nonterminal encountered by the parser.
///
/// These are the nonterminals of the underlying LL(1) grammar. For the full grammar,
/// take a look at Zwreec's [Wiki](https://github.com/Drakulix/zwreec/wiki/Underlying-Twee-Grammar#grammar).
#[derive(Debug, Copy, Clone)]
pub enum NonTerminalType {
    /// Start symbol
    S,
    Sf,
    Passage,
    Passagef,
    PassageContent,
    Formating,
    BoldFormatting,
    ItalicFormatting,
    MonoFormatting,
    MonoContent,
    Link,
    Macro,
    ElseIf,
    EndIf,
    Function,
    Functionf,
    Arguments,
    Argumentsf,
    ExpressionList,
    ExpressionListf,
    Expression,
    /// Start of the expression definition
    E,
    E2,
    T,
    T2,
    B,
    B2,
    F,
    F2,
    G,
    G2,
    H,
    DataType,
    AssignVariable,
}

/// The Type that represents an element of the grammar
///
/// This enum represents the lexical elements of a grammar. Nonterminals are defined by
/// `NonTerminalType`, terminals use the lexical token as specification.
///
/// For the full grammar, take a look at Zwreec's
/// [Wiki](https://github.com/Drakulix/zwreec/wiki/Underlying-Twee-Grammar#grammar).
pub enum Elem {
    NonTerminal(NonTerminalType),
    Terminal(Token)
}

/// Stores the stack for the custom iterator `parsing()`
///
/// The `zwreec::utils::extensions` module defines a new iterator `Parser`.
/// This struct stores the state for this iterator.
pub struct ParseState {
    stack: Vec<Elem>,
    grammar_func: Box<Fn(NonTerminalType, Option<Token>, &mut Vec<Elem>) -> Option<ASTOperation>>,
}

//==============================
// parser

#[allow(dead_code)]
pub struct Parser<'a> {
    cfg: &'a Config
}

impl<'a> Parser<'a> {
    pub fn new(cfg: &Config) -> Parser {
        Parser {
            cfg: cfg
        }
    }

    pub fn parse<I: Iterator<Item=Token>>(&self, tokens: I) ->
       ::utils::extensions::Parser<I, Token, ParseState, fn(&mut ParseState, Option<Token>) -> (ParseResult, Option<ASTOperation>)> {

        // prepare stack
        let mut stack: Vec<Elem> = Vec::new();
        stack.push(NonTerminal(S));

        //create Iterator
        tokens.parsing(
            ParseState {
                stack: stack,
                grammar_func: Box::new(Parser::apply_grammar),
            },
            {
                /// the predictive stack ll(1) parsing routine
                fn parse(state: &mut ParseState, token: Option<Token>) -> (ParseResult, Option<ASTOperation>) {
                    match token {
                        Some(token) => match state.stack.pop() {
                            Some(Elem::NonTerminal(non_terminal)) => (ParseResult::Halt, (state.grammar_func)(non_terminal, Some(token), &mut state.stack)),
                            Some(Elem::Terminal(stack_token)) => {
                                if stack_token == token {
                                    (ParseResult::Continue, None)
                                } else {
                                    ParserError::TokenDoNotMatch{token: Some(token), stack: stack_token}.raise()
                                }
                            },
                            None => ParserError::StackIsEmpty{token: token}.raise(),
                        },
                        None => match state.stack.pop() {
                            Some(Elem::NonTerminal(non_terminal)) => (ParseResult::Continue, (state.grammar_func)(non_terminal, None, &mut state.stack)),
                            Some(Elem::Terminal(stack_token)) => ParserError::TokenDoNotMatch{token: token, stack: stack_token}.raise(),
                            None => (ParseResult::End, None),
                        }
                    }
                }
                parse
            }
        )
    }

    /// apply the ll(1) grammar
    /// the match-statement simulates the parsing-table behavior
    ///
    fn apply_grammar(top: NonTerminalType, maybe_token: Option<Token>, stack: &mut Vec<Elem>) -> Option<ASTOperation> {
        if let Some(token) = maybe_token {

            let state = (top, token);

            debug!("match {:?}", state);
            match state {
                (S, TokPassage { .. } ) => {
                    stack.push(NonTerminal(Sf));
                    stack.push(NonTerminal(Passage));

                    None
                },
                (Sf, TokPassage { .. } ) => {
                    stack.push(NonTerminal(S));

                    None
                },
                (Passage, tok @ TokPassage { .. } ) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(tok.clone()));

                    Some(AddPassage(tok))
                },

                // PassageContent
                (PassageContent, tok @ TokText { .. } ) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (PassageContent, TokFormatBoldStart   { .. }) |
                (PassageContent, TokFormatItalicStart { .. }) |
                (PassageContent, TokFormatMonoStart   { .. }) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(NonTerminal(Formating));

                    None
                },
                (PassageContent, TokPassageLink { .. } ) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(NonTerminal(Link));

                    None
                },
                (PassageContent, tok @ TokNewLine { .. }) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (PassageContent, TokMacroDisplay { .. } ) |
                (PassageContent, TokMacroSet { .. } ) |
                (PassageContent, TokMacroIf  { .. } ) |
                (PassageContent, TokMacroPrint { .. } ) |
                (PassageContent, TokVariable { .. } ) |
                (PassageContent, TokMacroContentVar { .. } ) => {
                    stack.push(NonTerminal(PassageContent));
                    stack.push(NonTerminal(Macro));

                    None
                },
                (PassageContent, tok @ TokMacroEndIf { .. }) => {
                    debug!("pop TokMacroEndIf Passage;");

                    // jump one ast-level higher
                    Some(UpChild(tok))
                },
                (PassageContent, TokFormatBoldEnd { .. } ) => {
                    // jump one ast-level higher
                    Some(Up)
                },
                (PassageContent, TokFormatItalicEnd { .. } ) => {
                    // jump one ast-level higher
                    Some(Up)
                },
                (PassageContent, _) => {
                    // PassageContent -> ε
                    None
                },

                // Formating
                (Formating, TokFormatBoldStart { .. } ) => {
                    stack.push(NonTerminal(BoldFormatting));

                    None
                },
                (Formating, TokFormatItalicStart { .. } ) => {
                    stack.push(NonTerminal(ItalicFormatting));

                    None
                },
                (Formating, TokFormatMonoStart { .. } ) => {
                    stack.push(NonTerminal(MonoFormatting));

                    None
                },

                //BoldFormatting
                (BoldFormatting, tok @ TokFormatBoldStart { .. } ) => {
                    stack.push(Terminal(TokFormatBoldEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },

                // ItalicFormatting
                (ItalicFormatting, tok @ TokFormatItalicStart { .. } ) => {
                    stack.push(Terminal(TokFormatItalicEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },

                // MonoFormatting
                (MonoFormatting, tok @ TokFormatMonoStart { .. } ) => {
                    stack.push(Terminal(TokFormatMonoEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(MonoContent));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },

                // MonoContent
                (MonoContent, tok @ TokText { .. } ) => {
                    stack.push(NonTerminal(MonoContent));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (MonoContent, tok @ TokNewLine { .. } ) => {
                    stack.push(NonTerminal(MonoContent));
                    stack.push(Terminal(tok));

                    None
                },

                (MonoContent, TokFormatMonoEnd { .. } ) => {
                    // jump one ast-level higher
                    Some(Up)
                },

                // Link
                (Link, tok @ TokPassageLink { .. } ) => {
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },

                // Macro
                (Macro, tok @ TokMacroDisplay { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (Macro, tok @ TokMacroSet { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok));

                    None
                },
                (Macro, tok @ TokMacroIf { .. } ) => {
                    stack.push(NonTerminal(EndIf));
                    stack.push(NonTerminal(ElseIf));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },
                (Macro, tok @ TokMacroPrint { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                }

                // means <<$var>>
                (Macro, tok @ TokMacroContentVar { .. }) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },

                // ElseIf
                (ElseIf, tok @ TokMacroElseIf { .. } ) => {
                    stack.push(NonTerminal(ElseIf));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok.clone()));

                    Some(UpChildDown(tok))
                },
                (ElseIf, _) => {
                    // ElseIf -> ε
                    None
                },

                // EndIf
                (EndIf, tok @ TokMacroElse { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(TokMacroEndIf {location: (0, 0)} ));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(UpChildDown(tok))
                },
                (EndIf, tok @ TokMacroEndIf { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    None
                },

                // ExpressionList
                (ExpressionList, TokVariable { .. } ) |
                (ExpressionList, TokInt      { .. } ) |
                (ExpressionList, TokString   { .. } ) |
                (ExpressionList, TokBoolean  { .. } ) |
                (ExpressionList, TokAssign   { .. } ) |
                (ExpressionList, TokFunction { .. } ) |
                (ExpressionList, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(ExpressionListf));
                    stack.push(NonTerminal(Expression));

                    None
                },
                (ExpressionList, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(ExpressionListf));
                        stack.push(NonTerminal(Expression));

                        None
                    }
                    _ => None
                },

                // ExpressionListf
                (ExpressionListf, TokMacroEnd { .. } ) => {
                    debug!("pop TokMacroEnd");

                    Some(UpSpecial)
                },
                (ExpressionListf, _) => {
                    // ExpressionListf -> ε
                    debug!("pop ExpressionListf -> ε");
                    Some(Up)
                },

                // Expression
                (Expression, TokVariable { .. } ) |
                (Expression, TokInt      { .. } ) |
                (Expression, TokString   { .. } ) |
                (Expression, TokBoolean  { .. } ) |
                (Expression, TokFunction { .. } ) |
                (Expression, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(E));

                    None
                },
                (Expression, TokAssign { .. } ) => {
                    stack.push(NonTerminal(AssignVariable));

                    None
                },

                (Expression, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(E));

                        None
                    }
                    _ => None
                },

                // E
                (E, TokVariable { .. } ) |
                (E, TokInt      { .. } ) |
                (E, TokString   { .. } ) |
                (E, TokBoolean  { .. } ) |
                (E, TokFunction { .. } ) |
                (E, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(E2));
                    stack.push(NonTerminal(T));

                    //None
                    Some(ChildDown(TokExpression))
                },
                (E, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(E2));
                        stack.push(NonTerminal(T));

                        None
                    }
                    _ => None
                },

                // E2
                (E2, TokLogOp { location, op_name: op }) => match &*op {
                    "or" => {
                        stack.push(NonTerminal(E2));
                        stack.push(NonTerminal(T));
                        stack.push(Terminal(TokLogOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokLogOp{location: location, op_name: op}))
                    }
                    _ => None
                },
                (E2, _) => {
                    // E2 -> ε
                    debug!("pop E2 -> ε");
                    Some(Up)
                },

                // T
                (T, TokVariable { .. } ) |
                (T, TokInt      { .. } ) |
                (T, TokString   { .. } ) |
                (T, TokBoolean  { .. } ) |
                (T, TokFunction { .. } ) |
                (T, TokParenOpen{ .. } )=> {
                    stack.push(NonTerminal(T2));
                    stack.push(NonTerminal(B));

                    None
                },
                (T, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(T2));
                        stack.push(NonTerminal(B));

                        None
                    }
                    _ => None
                },

                // T2
                (T2, TokLogOp { location, op_name: op }) => match &*op {
                    "and" => {
                        stack.push(NonTerminal(T2));
                        stack.push(NonTerminal(B));
                        stack.push(Terminal(TokLogOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokLogOp{location: location, op_name: op}))
                    }
                    _ => None
                },
                (T2, _) => {
                    // T2 -> ε
                    None
                },

                // B
                (B, TokVariable { .. } ) |
                (B, TokInt      { .. } ) |
                (B, TokString   { .. } ) |
                (B, TokBoolean  { .. } ) |
                (B, TokFunction { .. } ) |
                (B, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(B2));
                    stack.push(NonTerminal(F));

                    None
                },
                (B, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(B2));
                        stack.push(NonTerminal(F));

                        None
                    }
                    _ => None
                },

                // B2
                (B2, TokCompOp { location, op_name: op }) => match &*op {
                    "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte" => {
                        stack.push(NonTerminal(B2));
                        stack.push(NonTerminal(F));
                        stack.push(Terminal(TokCompOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokCompOp{location: location, op_name: op}))
                    }
                    _ => None
                },
                (B2, _) => {
                    // B2 -> ε
                    None
                },

                // F
                (F, TokVariable { .. } ) |
                (F, TokInt      { .. } ) |
                (F, TokString   { .. } ) |
                (F, TokBoolean  { .. } ) |
                (F, TokFunction { .. } ) |
                (F, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(F2));
                    stack.push(NonTerminal(G));

                    None
                },
                (F, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(F2));
                        stack.push(NonTerminal(G));

                        None
                    }
                    _ => None
                },

                // F2
                (F2, TokNumOp { location, op_name: op }) =>  match &*op {
                    "+" | "-" => {
                        stack.push(NonTerminal(F2));
                        stack.push(NonTerminal(G));
                        stack.push(Terminal(TokNumOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokNumOp{location: location, op_name: op}))
                    }
                    _ => None
                },
                (F2, _) => {
                    // F2 -> ε
                    None
                },

                // G
                (G, TokVariable { .. } ) |
                (G, TokInt      { .. } ) |
                (G, TokString   { .. } ) |
                (G, TokBoolean  { .. } ) |
                (G, TokFunction { .. } ) |
                (G, TokParenOpen{ .. } ) => {
                    stack.push(NonTerminal(G2));
                    stack.push(NonTerminal(H));

                    None
                },
                (G, TokNumOp { op_name: op, .. }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(G2));
                        stack.push(NonTerminal(H));

                        None
                    }
                    _ => None
                },

                // G2
                (G2, TokNumOp { location, op_name: op }) => match &*op {
                    "*" | "/" | "%" => {
                        stack.push(NonTerminal(G2));
                        stack.push(NonTerminal(H));
                        stack.push(Terminal(TokNumOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokNumOp{location: location, op_name: op}))
                    }
                    _ => None
                },
                (G2, TokVarSetEnd  { .. } ) |
                (G2, TokMacroEnd   { .. } ) |
                (G2, TokSemiColon  { .. } ) |
                (G2, TokCompOp     { .. } ) |
                (G2, TokArgsEnd    { .. } ) |
                (G2, TokColon      { .. } ) => {
                    // G2 -> ε
                    None
                },
                (G2, TokLogOp { location, op_name: op }) => match &*op {
                    "and" | "or" => {
                        // G2 -> ε =>
                        None
                    }
                    _ => ParserError::NoProjection{token: TokLogOp{location: location.clone(), op_name: op.clone()}, stack: G2}.raise()
                },
                (G2, tok) => {
                    ParserError::NoProjection{token: tok, stack: G2}.raise()
                }

                // H
                (H, TokNumOp { location, op_name: op }) =>  match &*op {
                    "-" => {
                        stack.push(NonTerminal(H));
                        stack.push(Terminal(TokNumOp{location: location.clone(), op_name: op.clone()}));

                        Some(AddChild(TokNumOp{location: location, op_name: "_".to_string()}))
                    }
                    _ => None
                },
                (H, TokInt     { .. } ) |
                (H, TokString  { .. } ) |
                (H, TokBoolean { .. } ) => {
                    stack.push(NonTerminal(DataType));

                    None
                },
                (H, tok @ TokVariable { .. } ) => {
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (H, TokFunction { .. } ) => {
                    stack.push(NonTerminal(Function));

                    None
                },
                (H, tok @ TokParenOpen { .. } ) => {
                    stack.push(Terminal(TokParenClose{location: (0, 0)}));
                    stack.push(NonTerminal(Expression));
                    stack.push(Terminal(tok.clone()));

                    None
                },

                // Function
                (Function, tok @ TokFunction { .. } ) => {
                    stack.push(NonTerminal(Functionf));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },

                // Functionf
                (Functionf, tok @ TokArgsEnd { .. } ) => {
                    stack.push(Terminal(tok));

                    None
                },
                (Functionf, TokVariable { .. } ) |
                (Functionf, TokInt      { .. } ) |
                (Functionf, TokString   { .. } ) |
                (Functionf, TokBoolean  { .. } ) |
                (Functionf, TokFunction { .. } ) => {
                    stack.push(Terminal(TokArgsEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(Arguments));

                    None
                },

                // Arguments
                (Arguments, TokVariable { .. } ) |
                (Arguments, TokInt      { .. } ) |
                (Arguments, TokString   { .. } ) |
                (Arguments, TokBoolean  { .. } ) |
                (Arguments, TokFunction { .. } ) => {
                    stack.push(NonTerminal(Argumentsf));
                    stack.push(NonTerminal(Expression));

                    None
                },

                // Argumentsf
                (Argumentsf, TokArgsEnd { .. } ) => {
                    // Argumentsf -> ε

                    Some(Up)
                },
                (Argumentsf, tok @ TokColon { .. } ) => {
                    stack.push(NonTerminal(Arguments));
                    stack.push(Terminal(tok));

                    None
                },
                (Argumentsf, _) => {
                    // Argumentsf -> ε
                    None
                },

                // AssignVariable
                (AssignVariable, tok @ TokAssign { .. } ) => {
                    stack.push(NonTerminal(E));
                    stack.push(Terminal(tok.clone()));

                    Some(ChildDown(tok))
                },

                // DataType
                (DataType, tok @ TokInt { .. } ) => {
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (DataType, tok @ TokString { .. } ) => {
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (DataType, tok @ TokBoolean { .. } ) => {
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (x, tok) => {
                    ParserError::NoProjection{token: tok, stack: x}.raise()
                }
            }

        } else {
            // no token left

            // Sf, PassageContent, Linkf,

            match top {
                Sf | PassageContent => {
                    // ... -> ε
                    None
                },
                _ => {
                    ParserError::NonTerminalEnd{stack: top}.raise()
                }
            }
        }
    }

}
