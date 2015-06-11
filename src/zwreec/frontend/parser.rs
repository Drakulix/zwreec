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
    Makro,
    Makrof,
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
    lookahead: usize,
    is_in_else: u8,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            ast: ast::AST::new(),
            stack: Vec::new(),
            tokens: tokens,
            lookahead: 0,
            is_in_else: 0
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
                (S, &TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(Passage));
                    new_nodes.push(PNode::new_non_terminal(Sf));
                },
                (Sf, &TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(S));
                },
                (Passage, &TokPassageName(ref name)) => {
                    new_nodes.push(PNode::new_terminal(TokPassageName(name.clone())));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_passage(TokPassageName(name.clone()));
                },

                // PassageContent
                (PassageContent, &TokText(ref text)) => {
                    new_nodes.push(PNode::new_terminal(TokText(text.clone())));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_child(TokText(text.clone()));
                },
                (PassageContent, &TokFormatBoldStart) | 
                (PassageContent, &TokFormatItalicStart) |
                (PassageContent, &TokFormatMonoStart) => {
                    new_nodes.push(PNode::new_non_terminal(Formating));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, &TokPassageLink(_, _)) => {
                    new_nodes.push(PNode::new_non_terminal(Link));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, &TokNewLine) => {
                    new_nodes.push(PNode::new_terminal(TokNewLine));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));

                    // ast
                    self.ast.add_child(TokNewLine);
                },
                (PassageContent, &TokSet) |
                (PassageContent, &TokIf) |
                (PassageContent, &TokVariable(_)) |
                (PassageContent, &TokMakroVar(_)) => {
                    new_nodes.push(PNode::new_non_terminal(Makro));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                },
                (PassageContent, &TokEndIf) => {
                    // jump one ast-level higher
                    debug!("pop TokEndIf Passage; in else: {:?}", self.is_in_else);

                    // ast
                    // special handling for the else
                    if self.is_in_else > 0 {
                        self.is_in_else -= 1;
                        self.ast.one_child_up();
                        self.ast.add_child(TokEndIf);
                    }
                },
                (PassageContent, &TokFormatBoldEnd) => {
                    // jump one ast-level higher
                    self.ast.one_child_up();
                },
                (PassageContent, &TokFormatItalicEnd) => {
                    // jump one ast-level higher
                    self.ast.one_child_up();
                },
                (PassageContent, _) => {
                    // PassageContent -> ε
                },

                // Formating
                (Formating, &TokFormatBoldStart) => {
                    new_nodes.push(PNode::new_non_terminal(BoldFormatting));
                },
                (Formating, &TokFormatItalicStart) => {
                    new_nodes.push(PNode::new_non_terminal(ItalicFormatting));
                },
                (Formating, &TokFormatMonoStart) => {
                    new_nodes.push(PNode::new_non_terminal(MonoFormatting));
                },

                // BoldFormatting
                (BoldFormatting, &TokFormatBoldStart) => {
                    new_nodes.push(PNode::new_terminal(TokFormatBoldStart));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokFormatBoldEnd));

                    //ast
                    self.ast.add_child_and_go_down(TokFormatBoldStart);
                },

                // ItalicFormatting
                (ItalicFormatting, &TokFormatItalicStart) => {
                    new_nodes.push(PNode::new_terminal(TokFormatItalicStart));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokFormatItalicEnd));

                    //ast
                    self.ast.add_child_and_go_down(TokFormatItalicStart);
                },

                // MonoFormatting
                (MonoFormatting, &TokFormatMonoStart) => {
                    new_nodes.push(PNode::new_terminal(TokFormatMonoStart));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                    new_nodes.push(PNode::new_terminal(TokFormatMonoEnd));

                    //ast
                    self.ast.add_child_and_go_down(TokFormatMonoStart);
                },

                // MonoContent
                (MonoContent, &TokText(ref text)) => {
                    new_nodes.push(PNode::new_terminal(TokText(text.clone())));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                },
                (MonoContent, &TokNewLine) => {
                    new_nodes.push(PNode::new_terminal(TokNewLine));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                },

                // Link
                (Link, &TokPassageLink(ref text, ref name)) => {
                    new_nodes.push(PNode::new_terminal(TokPassageLink(text.clone(), name.clone())));

                    // ast
                    self.ast.add_child(TokPassageLink(text.clone(), name.clone()));
                },

                // Makro
                (Makro, &TokSet) => {
                    new_nodes.push(PNode::new_terminal(TokSet));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));
                },
                (Makro, &TokIf) => {
                    new_nodes.push(PNode::new_terminal(TokIf));
                    new_nodes.push(PNode::new_non_terminal(ExpressionList));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_non_terminal(Makrof));

                    // ast
                    self.ast.add_child_and_go_down(TokIf);

                    // pseudo_node for expression
                    self.ast.add_child_and_go_down(TokPseudo);
                },
                // means <<$var>>
                (Makro, &TokMakroVar(ref name)) => {
                    new_nodes.push(PNode::new_terminal(TokMakroVar(name.clone())));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));

                    // ast
                    self.ast.add_child(TokMakroVar(name.clone()));
                },

                // Makrof
                (Makrof, &TokElse) => {
                    new_nodes.push(PNode::new_terminal(TokElse));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));
                    new_nodes.push(PNode::new_non_terminal(PassageContent));
                    new_nodes.push(PNode::new_terminal(TokEndIf));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));

                    // ast
                    debug!("pop TokElse");
                    self.ast.one_child_up();
                    self.is_in_else += 1;

                    self.ast.add_child_and_go_down(TokElse);
                },
                (Makrof, &TokEndIf) => {
                    new_nodes.push(PNode::new_terminal(TokEndIf));
                    new_nodes.push(PNode::new_terminal(TokMakroEnd));

                    // ast
                    debug!("pop TokEndIf Macro");
                    self.ast.one_child_up();
                    self.ast.add_child(TokEndIf);
                }

                // ExpressionList
                (ExpressionList, &TokVariable(_)) |
                (ExpressionList, &TokInt(_)) |
                (ExpressionList, &TokString(_)) |
                (ExpressionList, &TokBoolean(_)) |
                (ExpressionList, &TokAssign(_, _)) => {
                    new_nodes.push(PNode::new_non_terminal(Expression));
                    new_nodes.push(PNode::new_non_terminal(ExpressionListf));
                },

                // ExpressionListf
                (ExpressionListf, &TokMakroEnd) => {
                    debug!("pop TokMakroEnd");
                    self.ast.one_child_up();
                    
                },
                (ExpressionListf, _) => {
                    // ExpressionListf -> ε
                },

                // Expression
                (Expression, &TokVariable(_)) |
                (Expression, &TokInt(_)) |
                (Expression, &TokString(_)) |
                (Expression, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(E));
                },
                (Expression, &TokAssign(_, _)) => {
                    new_nodes.push(PNode::new_non_terminal(AssignVariable));
                },

                // E
                (E, &TokVariable(_)) |
                (E, &TokInt(_)) |
                (E, &TokString(_)) |
                (E, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(T));
                    new_nodes.push(PNode::new_non_terminal(E2));
                },

                // E2
                (E2, _) => {
                    // E2 -> ε
                },

                // T
                (T, &TokVariable(_)) |
                (T, &TokInt(_)) |
                (T, &TokString(_)) |
                (T, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(B));
                    new_nodes.push(PNode::new_non_terminal(T2));
                },

                // T2
                (T2, _) => {
                    // T2 -> ε
                },

                // B
                (B, &TokVariable(_)) |
                (B, &TokInt(_)) |
                (B, &TokString(_)) |
                (B, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(F));
                    new_nodes.push(PNode::new_non_terminal(B2));
                },

                // B2
                (B2, &TokCompOp(ref op)) => {
                    new_nodes.push(PNode::new_terminal(TokCompOp(op.clone())));
                    new_nodes.push(PNode::new_non_terminal(F));
                    new_nodes.push(PNode::new_non_terminal(B2));

                    // ast
                    self.ast.add_child(TokCompOp(op.clone()));
                },
                (B2, _) => {
                    // B2 -> ε
                },

                // F
                (F, &TokVariable(_)) |
                (F, &TokInt(_)) |
                (F, &TokString(_)) |
                (F, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(G));
                    new_nodes.push(PNode::new_non_terminal(F2));
                },

                // F2
                (F2, _) => {
                    // F2 -> ε
                },

                // G
                (G, &TokVariable(_)) |
                (G, &TokInt(_)) |
                (G, &TokString(_)) |
                (G, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(H));
                    new_nodes.push(PNode::new_non_terminal(G2));
                },

                // G2
                (G2, _) => {
                    // G2 -> ε
                },

                // H
                (H, &TokInt(_)) |
                (H, &TokString(_)) |
                (H, &TokBoolean(_)) => {
                    new_nodes.push(PNode::new_non_terminal(DataType));
                },
                (H, &TokVariable(ref name)) => {
                    new_nodes.push(PNode::new_terminal(TokVariable(name.clone())));

                    // ast
                    self.ast.add_child(TokVariable(name.clone()));
                },

                // AssignVariable
                (AssignVariable, &TokAssign(ref name, ref assign)) => {
                    new_nodes.push(PNode::new_terminal(TokAssign(name.clone(), assign.clone())));
                    new_nodes.push(PNode::new_non_terminal(E));

                    //ast
                    self.ast.add_child_and_go_down(TokAssign(name.clone(), assign.clone()));
                },

                // DataType
                (DataType, &TokInt(ref value)) => {
                    new_nodes.push(PNode::new_terminal(TokInt(value.clone())));

                    // ast
                    self.ast.add_child(TokInt(value.clone()));
                },
                (DataType, &TokString(ref value)) => {
                    new_nodes.push(PNode::new_terminal(TokString(value.clone())));

                    // ast
                    self.ast.add_child(TokString(value.clone()));
                },
                (DataType, &TokBoolean(ref value)) => {
                    new_nodes.push(PNode::new_terminal(TokBoolean(value.clone())));

                    // ast
                    self.ast.add_child(TokBoolean(value.clone()));
                }

                
                _ => {
                    panic!("not supported grammar: {:?}", state_first);
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
