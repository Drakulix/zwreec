//! The `parser` module contains a lot of useful functionality
//! to parse tokens from the lexer (and creating the parsetree
//! and the ast)
//! its an predictiv parser for a LL(1) grammar
//! for more info about the parser: look in the Compiler Dragonbook,
//! Chapter 4.4.4, "Nonrecursive Predictive Parsing"

use frontend::lexer::Token;
use frontend::ast;
use frontend::parsetree::{PNode};
use self::NonTerminalType::*;
use frontend::lexer::Token::*;
use frontend::lexer::InputFileLocation;

pub fn parse_tokens(tokens: Vec<Token>) -> ast::AST {
    let mut parser: Parser = Parser::new(tokens);
    parser.parsing();
    parser.ast
}

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

//==============================
// parser

struct Parser {
    ast: ast::AST,
    stack: Vec<PNode>,
    tokens: Vec<Token>,
    lookahead: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            ast: ast::AST::new(),
            stack: Vec::new(),
            tokens: tokens,
            lookahead: 0
        }
    }

    /// the predictive stack ll(1) parsing routine
    pub fn parsing(&mut self) {
        // push Start-Non-Terminal to the stack
        self.stack.push(PNode::new_non_terminal(S));

        while let Some(top) = self.stack.pop() {
            match top {
                PNode::NonTerminal(ref node) => {
                    self.apply_grammar(node.clone());
                }
                PNode::Terminal(_) => {
                    self.next_token();
                }
            }
        }
    }

    /// apply the ll(1) grammar
    /// the match-statement simulates the parsing-table behavior
    ///
    fn apply_grammar(&mut self, top: NonTerminalType) {
        if let Some(token) = self.tokens.get_mut(self.lookahead) {

            // the frst item in the tuple is the current state and
            // the snd is the current lookup-token
            let state_first: (NonTerminalType, &Token) = (top, token);

            let mut new_nodes = Vec::new();

            debug!("match {:?}", state_first);
            match state_first {
                (S, &TokPassage { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Passage));
                    new_nodes.push(PNode::new_non_terminal(Sf));
                },
                (Sf, &TokPassage { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(S));
                },
                (Passage, tok @ &TokPassage { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_passage(tok.clone());
                },

                // PassageContent
                (PassageContent, tok @ &TokText { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (PassageContent, &TokFormatBoldStart   { .. }) | 
                (PassageContent, &TokFormatItalicStart { .. }) |
                (PassageContent, &TokFormatMonoStart   { .. }) => {
                    new_nodes.push(PNode::new_non_terminal(Formating));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, &TokPassageLink { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Link));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, tok @ &TokNewLine { .. }) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (PassageContent, &TokMacroSet { .. } ) |
                (PassageContent, &TokMacroIf  { .. } ) |
                (PassageContent, &TokVariable { .. } ) |
                (PassageContent, &TokMacroContentVar { .. } ) |
                (PassageContent, &TokMacroContentPassageName { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Macro));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, tok @ &TokMacroEndIf { .. }) => {
                    // jump one ast-level higher
                    debug!("pop TokMacroEndIf Passage;");

                    self.ast.up_child(tok.clone(), true);
                },
                (PassageContent, &TokFormatBoldEnd { .. } ) => {
                    // jump one ast-level higher
                    self.ast.up();
                },
                (PassageContent, &TokFormatItalicEnd { .. } ) => {
                    // jump one ast-level higher
                    self.ast.up();
                },
                (PassageContent, _) => {
                    // PassageContent -> ε
                },

                // Formating
                (Formating, &TokFormatBoldStart { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(BoldFormatting));
                },
                (Formating, &TokFormatItalicStart { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(ItalicFormatting));
                },
                (Formating, &TokFormatMonoStart { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(MonoFormatting));
                },

                // BoldFormatting
                (BoldFormatting, tok @ &TokFormatBoldStart { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokFormatBoldEnd {location: (0, 0)} ));

                    //ast
                    self.ast.child_down(tok.clone());
                },

                // ItalicFormatting
                (ItalicFormatting, tok @ &TokFormatItalicStart { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokFormatItalicEnd {location: (0, 0)} ));

                    //ast
                    self.ast.child_down(tok.clone());
                },

                // MonoFormatting
                (MonoFormatting, tok @ &TokFormatMonoStart { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                    new_nodes.push(PNode::new_terminal(TokFormatMonoEnd {location: (0, 0)} ));

                    //ast
                    self.ast.child_down(tok.clone());
                },

                // MonoContent
                (MonoContent, tok @ &TokText { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (MonoContent, tok @ &TokNewLine { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                },

                (MonoContent, &TokFormatMonoEnd { .. } ) => {
                    // jump one ast-level higher
                    self.ast.up();
                },

                // Link
                (Link, tok @ &TokPassageLink { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    self.ast.add_child(tok.clone());
                },

                // Macro
                (Macro, tok @ &TokMacroSet { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));
                },
                (Macro, tok @ &TokMacroIf { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_non_terminal(Macrof));

                    // ast
                    self.ast.two_childs_down(tok.clone(), TokPseudo);
                },
                // means <<$var>>
                (Macro, tok @ &TokMacroContentVar { .. }) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                // means <<passagename>>
                (Macro, tok @ &TokMacroContentPassageName { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                // Macrof
                (Macrof, tok @ &TokMacroElse { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokMacroEndIf {location: (0, 0)} ));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));

                    // ast
                    self.ast.up_child_down(tok.clone());
                },
                (Macrof, tok @ &TokMacroEndIf { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_terminal(TokMacroEnd {location: (0, 0)} ));

                    // ast
                    self.ast.up_child(tok.clone(), false);
                }

                // ExpressionList
                (ExpressionList, &TokVariable { .. } ) |
                (ExpressionList, &TokInt      { .. } ) |
                (ExpressionList, &TokString   { .. } ) |
                (ExpressionList, &TokBoolean  { .. } ) |
                (ExpressionList, &TokAssign   { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(Expression));
                    new_nodes.push(PNode::new_non_terminal(ExpressionListf));
                },

                // ExpressionListf
                (ExpressionListf, &TokMacroEnd { .. } ) => {
                    debug!("pop TokMacroEnd");
                    self.ast.up();
                },
                (ExpressionListf, _) => {
                    // ExpressionListf -> ε
                },

                // Expression
                (Expression, &TokVariable { .. } ) |
                (Expression, &TokInt      { .. } ) |
                (Expression, &TokString   { .. } ) |
                (Expression, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(E));
                },
                (Expression, &TokAssign { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(AssignVariable));
                },

                // E
                (E, &TokVariable { .. } ) |
                (E, &TokInt      { .. } ) |
                (E, &TokString   { .. } ) |
                (E, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(T));
                    new_nodes.push(PNode::new_non_terminal(E2));
                },

                // E2
                (E2, _) => {
                    // E2 -> ε
                },

                // T
                (T, &TokVariable { .. } ) |
                (T, &TokInt      { .. } ) |
                (T, &TokString   { .. } ) |
                (T, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(B));
                    new_nodes.push(PNode::new_non_terminal(T2));
                },

                // T2
                (T2, _) => {
                    // T2 -> ε
                },

                // B
                (B, &TokVariable { .. } ) |
                (B, &TokInt      { .. } ) |
                (B, &TokString   { .. } ) |
                (B, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(F));
                    new_nodes.push(PNode::new_non_terminal(B2));
                },

                // B2
                (B2, tok @ &TokCompOp { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(F));
                    new_nodes.push(PNode::new_non_terminal(B2));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (B2, _) => {
                    // B2 -> ε
                },

                // F
                (F, &TokVariable { .. } ) |
                (F, &TokInt      { .. } ) |
                (F, &TokString   { .. } ) |
                (F, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(G));
                    new_nodes.push(PNode::new_non_terminal(F2));
                },

                // F2
                (F2, _) => {
                    // F2 -> ε
                },

                // G
                (G, &TokVariable { .. } ) |
                (G, &TokInt      { .. } ) |
                (G, &TokString   { .. } ) |
                (G, &TokBoolean  { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(H));
                    new_nodes.push(PNode::new_non_terminal(G2));
                },

                // G2
                (G2, _) => {
                    // G2 -> ε
                },

                // H
                (H, &TokInt     { .. } ) |
                (H, &TokString  { .. } ) |
                (H, &TokBoolean { .. } ) => {
                    new_nodes.push(PNode::new_non_terminal(DataType));
                },
                (H, tok @ &TokVariable { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    self.ast.add_child(tok.clone());
                },

                // AssignVariable
                (AssignVariable, tok @ &TokAssign { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));
                    new_nodes.push(PNode::new_non_terminal(E));

                    //ast
                    self.ast.child_down(tok.clone());
                },

                // DataType
                (DataType, tok @ &TokInt { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (DataType, tok @ &TokString { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (DataType, tok @ &TokBoolean { .. } ) => {
                    new_nodes.push(PNode::new_terminal(tok.clone()));

                    // ast
                    self.ast.add_child(tok.clone());
                },
                (_, tok) => {
                    let (line, character) = tok.location();
                    panic!("Unexpected token at {}:{}", line, character);
                }
            }

            // adds the new nodes to the stack (in reversed order)
            while let Some(child) = new_nodes.pop() {
                self.stack.push(child);
            }

        } else {
            // no token left

            // Sf, PassageContent, Linkf, 

            match top {
                Sf | PassageContent => {
                    // ... -> ε
                },
                _ => {
                    panic!("Nonterminal '{:?}' is not an allowed end.", top);
                }
            }
        }
    }

    /// sets the lookahead to the next token
    fn next_token(&mut self) {
        self.lookahead = self.lookahead + 1;
    }
}
