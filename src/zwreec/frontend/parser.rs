//! The `parser` module contains a lot of useful functionality
//! to parse tokens from the lexer (and creating the parsetree
//! and the ast)
//! its an predictiv parser for a LL(1) grammar
//! for more info about the parser: look in the Compiler Dragonbook,
//! Chapter 4.4.4, "Nonrecursive Predictive Parsing"

use config::Config;
use frontend::lexer::Token;
use frontend::lexer::Token::*;
use frontend::ast::ASTOperation;
use frontend::ast::ASTOperation::*;
use utils::extensions::{ParserExt, ParseResult};
use self::NonTerminalType::*;
use self::Elem::*;

//==============================
// grammar

#[derive(Debug, Copy, Clone)]
pub enum NonTerminalType {
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
    Macrof,
    Function,
    Functionf,
    Arguments,
    Argumentsf,
    ExpressionList,
    ExpressionListf,
    Expression,
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

enum Elem {
    NonTerminal(NonTerminalType),
    Terminal(Token)
}

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
                                    panic!("parser paniced, stack token does not match. Stack:{:?}, Token:{:?}", stack_token, token);
                                }
                            },
                            None => panic!("parser paniced, tokens left but stack is empty. First Token left at {:?}", token),
                        },
                        None => match state.stack.pop() {

                            Some(Elem::NonTerminal(non_terminal)) => (ParseResult::Continue, (state.grammar_func)(non_terminal, None, &mut state.stack)),
                            Some(Elem::Terminal(stack_token)) => panic!("parser paniced, no tokens left and terminal found. Stack:{:?}", stack_token),
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
                (PassageContent, &TokMacroPrint { .. } ) |
                (PassageContent, TokVariable { .. } ) |
                (PassageContent, TokMacroContentVar { .. } ) |
                (PassageContent, TokMacroContentPassageName { .. } ) => {
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
<<<<<<< HEAD
                (Macro, tok @ TokMacroSet { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok));

                    None
=======
                (Macro, tok @ &TokMacroDisplay { .. } ) |
                (Macro, tok @ &TokMacroSet { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));
>>>>>>> master
                },
                (Macro, tok @ TokMacroIf { .. } ) => {
                    stack.push(NonTerminal(Macrof));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(NonTerminal(ExpressionList));
                    stack.push(Terminal(tok.clone()));

                    Some(TwoChildsDown(tok, TokPseudo))
                },
                (Macro, tok @ &TokMacroPrint { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));

                    // ast
                    self.ast.child_down(tok.clone());
                }

                // means <<$var>>
                (Macro, tok @ TokMacroContentVar { .. }) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                // means <<passagename>>
                (Macro, tok @ TokMacroContentPassageName { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                // Macrof
                (Macrof, tok @ TokMacroElse { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(TokMacroEndIf {location: (0, 0)} ));
                    stack.push(NonTerminal(PassageContent));
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok.clone()));

                    Some(UpChildDown(tok))
                },
                (Macrof, tok @ TokMacroEndIf { .. } ) => {
                    stack.push(Terminal(TokMacroEnd {location: (0, 0)} ));
                    stack.push(Terminal(tok));

                    None
                }

                // ExpressionList
<<<<<<< HEAD
                (ExpressionList, TokVariable { .. } ) |
                (ExpressionList, TokInt      { .. } ) |
                (ExpressionList, TokString   { .. } ) |
                (ExpressionList, TokBoolean  { .. } ) |
                (ExpressionList, TokAssign   { .. } ) => {
                    stack.push(NonTerminal(ExpressionListf));
                    stack.push(NonTerminal(Expression));

                    None
=======
                (ExpressionList, &TokVariable { .. } ) |
                (ExpressionList, &TokInt      { .. } ) |
                (ExpressionList, &TokString   { .. } ) |
                (ExpressionList, &TokBoolean  { .. } ) |
                (ExpressionList, &TokAssign   { .. } ) |
                (ExpressionList, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Expression));
                    new_nodes.push(PNode::new_non_terminal(ExpressionListf));
>>>>>>> master
                },

                // ExpressionListf
                (ExpressionListf, TokMacroEnd { .. } ) => {
                    debug!("pop TokMacroEnd");
                    Some(Up)
                },
                (ExpressionListf, _) => {
                    // ExpressionListf -> ε
                    None
                },

                // Expression
<<<<<<< HEAD
                (Expression, TokVariable { .. } ) |
                (Expression, TokInt      { .. } ) |
                (Expression, TokString   { .. } ) |
                (Expression, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(E));

                    None
=======
                (Expression, &TokVariable { .. } ) |
                (Expression, &TokInt      { .. } ) |
                (Expression, &TokString   { .. } ) |
                (Expression, &TokBoolean  { .. } ) |
                (Expression, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(E));
>>>>>>> master
                },
                (Expression, TokAssign { .. } ) => {
                    stack.push(NonTerminal(AssignVariable));

                    None
                },

                // E
<<<<<<< HEAD
                (E, TokVariable { .. } ) |
                (E, TokInt      { .. } ) |
                (E, TokString   { .. } ) |
                (E, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(E2));
                    stack.push(NonTerminal(T));

                    None
=======
                (E, &TokVariable { .. } ) |
                (E, &TokInt      { .. } ) |
                (E, &TokString   { .. } ) |
                (E, &TokBoolean  { .. } ) |
                (E, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(T));
                    new_nodes.push(PNode::new_non_terminal(E2));
>>>>>>> master
                },

                // E2
                (E2, _) => {
                    // E2 -> ε
                    None
                },

                // T
<<<<<<< HEAD
                (T, TokVariable { .. } ) |
                (T, TokInt      { .. } ) |
                (T, TokString   { .. } ) |
                (T, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(T2));
                    stack.push(NonTerminal(B));

                    None
=======
                (T, &TokVariable { .. } ) |
                (T, &TokInt      { .. } ) |
                (T, &TokString   { .. } ) |
                (T, &TokBoolean  { .. } ) |
                (T, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(B));
                    new_nodes.push(PNode::new_non_terminal(T2));
>>>>>>> master
                },

                // T2
                (T2, _) => {
                    // T2 -> ε
                    None
                },

                // B
<<<<<<< HEAD
                (B, TokVariable { .. } ) |
                (B, TokInt      { .. } ) |
                (B, TokString   { .. } ) |
                (B, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(B2));
                    stack.push(NonTerminal(F));

                    None
=======
                (B, &TokVariable { .. } ) |
                (B, &TokInt      { .. } ) |
                (B, &TokString   { .. } ) |
                (B, &TokBoolean  { .. } ) |
                (B, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(F));
                    new_nodes.push(PNode::new_non_terminal(B2));
>>>>>>> master
                },

                // B2
                (B2, tok @ TokCompOp { .. } ) => {
                    stack.push(NonTerminal(B2));
                    stack.push(NonTerminal(F));
                    stack.push(Terminal(tok.clone()));

                    Some(AddChild(tok))
                },
                (B2, _) => {
                    // B2 -> ε
                    None
                },

                // F
<<<<<<< HEAD
                (F, TokVariable { .. } ) |
                (F, TokInt      { .. } ) |
                (F, TokString   { .. } ) |
                (F, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(F2));
                    stack.push(NonTerminal(G));

                    None
=======
                (F, &TokVariable { .. } ) |
                (F, &TokInt      { .. } ) |
                (F, &TokString   { .. } ) |
                (F, &TokBoolean  { .. } ) |
                (F, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(G));
                    new_nodes.push(PNode::new_non_terminal(F2));
>>>>>>> master
                },

                // F2
                (F2, _) => {
                    // F2 -> ε
                    None
                },

                // G
<<<<<<< HEAD
                (G, TokVariable { .. } ) |
                (G, TokInt      { .. } ) |
                (G, TokString   { .. } ) |
                (G, TokBoolean  { .. } ) => {
                    stack.push(NonTerminal(G2));
                    stack.push(NonTerminal(H));

                    None
=======
                (G, &TokVariable { .. } ) |
                (G, &TokInt      { .. } ) |
                (G, &TokString   { .. } ) |
                (G, &TokBoolean  { .. } ) |
                (G, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(H));
                    new_nodes.push(PNode::new_non_terminal(G2));
>>>>>>> master
                },

                // G2
                (G2, _) => {
                    // G2 -> ε
                    None
                },

                // H
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
                (H, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Function));
                },

                // Function
                (Function, tok @ &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(Functionf));

                    // ast
                    self.ast.child_down(tok.clone())
                },

                // Functionf
                (Functionf, tok @ &TokArgsEnd { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    // Get out of empty function
                    self.ast.up();
                },
                (Functionf, &TokVariable { .. } ) |
                (Functionf, &TokInt      { .. } ) |
                (Functionf, &TokString   { .. } ) |
                (Functionf, &TokBoolean  { .. } ) |
                (Functionf, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Arguments));
                    new_nodes.push(PNode::new_terminal(TokArgsEnd {location: (0, 0)} ));
                },

                // Arguments
                (Arguments, &TokVariable { .. } ) |
                (Arguments, &TokInt      { .. } ) |
                (Arguments, &TokString   { .. } ) |
                (Arguments, &TokBoolean  { .. } ) |
                (Arguments, &TokFunction { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Expression));
                    new_nodes.push(PNode::new_non_terminal(Argumentsf));

                    self.ast.child_down(TokPseudo);
                },

                // Argumentsf
                (Argumentsf, &TokArgsEnd { .. } ) => {
                    // Argumentsf -> ε
                    // TokArgsEnd is already on the stack

                    // We still need to get out of the expression
                    self.ast.up();

                    // And out of the function
                    self.ast.up();
                },
                (Argumentsf, tok @ &TokColon { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(Arguments));

                    self.ast.up();
                },
                (Argumentsf, _) => {
                    // Argumentsf -> ε
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
                (_, tok) => {
                    let (line, character) = tok.location();
                    panic!("Unexpected token at {}:{}", line, character);
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
                    panic!("Nonterminal '{:?}' is not an allowed end.", top);
                }
            }
        }
    }

}
