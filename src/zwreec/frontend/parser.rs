//! The `parser` module contains a lot of useful functionality
//! to parse tokens from the lexer (and creating the parsetree
//! and the ast)
//! its an predictiv parser for a LL(1) grammar
//! for more info about the parser: look in the Compiler Dragonbook,
//! Chapter 4.4.4, "Nonrecursive Predictive Parsing"

use frontend::lexer::Token;
use frontend::ast;
use frontend::parsetree::{ParseTree, PNode};
use self::NonTerminalType::*;
use frontend::lexer::Token::{TokPassageName, TokText, TokNewLine,
    TokFormatItalicStart, TokFormatItalicEnd, TokFormatBoldStart, TokFormatBoldEnd,
    TokFormatMonoStart, TokFormatMonoEnd, TokPassageLink};

/*

----------------------------------------
Grammatik:

LL(1)
------------------------------------------------------------+-----------------------------------
S -> Passage S2                                             |
S2 -> S                                                     |
S2 -> ɛ                                                     |
Passage -> PassageName PassageContent                       | "add passage (name)", simple
PassageContent -> TextPassage PassageContent                | add text as child
PassageContent -> Formatting PassageContent                 |
PassageContent -> ɛ                                         |
Formating -> BoldFormatting                                 |
Formating -> ItalicFormatting                               |
BoldFormatting -> FormatBold BoldContent FormatBold         | add child bold
BoldContent -> TextPassage BoldContent                      | add text as child
BoldContent -> FormatItalic BoldItalicContent FormatItalic  | add child italic
BoldContent -> ɛ                                            |
ItalicFormatting -> FormatItalic ItalicContent FormatItalic | add child italic
ItalicContent -> TextPassage ItalicContent                  | add text as child
ItalicContent -> FormatBold BoldItalicContent FormatBold    | add child bold
ItalicContent -> ɛ                                          | add text as child
BoldItalicContent -> TextPassage                            |
------------------------------------------------------------+-----------------------------------

::Start
Hello World, ''Yeah''.
//Italic// is possible too.

*/

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
    PassageContentNewline,
    PassageContentNonNewline,
    Formating,
    BoldFormatting,
    ItalicFormatting,
    MonoFormatting,
    MonoContent,
    Link
}

//==============================
// parser

struct Parser {
    tree: ParseTree,
    ast: ast::AST,
    stack: Stack,
    ast_path: Vec<usize>,
    tokens: Vec<Token>,
    lookahead: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tree: ParseTree::new(),
            ast: ast::AST::new(),
            stack: Stack::new(),
            ast_path: Vec::new(),
            tokens: tokens,
            lookahead: 0
        }
    }

    /// the predictive stack ll(1) parsing routine
    pub fn parsing(&mut self) {
        self.stack.push_start();

        while let Some(top) = self.stack.pop() {
            if self.tree.is_terminal(top.to_vec()) {
                self.next_token();
            } else {
                self.apply_grammar(top.to_vec());
            }
        }

        self.tree.print();
    }

    /// apply the ll(1) grammar
    /// the match-statement simulates the parsing-table behavior
    ///
    /// it creates the parse tree (from the ll(1) grammar)
    /// and the ast from the sdd
    fn apply_grammar(&mut self, top_path: Vec<usize>) {
        if let Some(token) = self.tokens.get_mut(self.lookahead) {

            let to_add_path: Vec<usize> = top_path.to_vec();

            // the frst item in the tuple is the current state and
            // the snd is the current lookup-token
            let state_first: (NonTerminalType, &Token) = (self.tree.get_non_terminal(top_path).clone(), token);

            let mut new_nodes = Vec::new();

            println!("match {:?}", state_first);
            match state_first {
                (S, &TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(Passage));
                    new_nodes.push(PNode::new_non_terminal(Sf));
                },
                (Sf, &TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(S));
                },
                (Passage, &TokPassageName(ref name)) => {
                    let new_token: Token = TokPassageName(name.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNewline));

                    // ast
                    self.ast_path.clear();
                    let ast_count_passages = self.ast.count_childs(self.ast_path.to_vec());
                    let new_token2: Token = TokPassageName(name.clone());
                    self.ast.add_passage(new_token2);
                    self.ast_path.push(ast_count_passages);
                },

                // PassageContentNewline / PassageContentNonNewline
                (PassageContentNewline, &TokText(ref text)) |
                (PassageContentNonNewline, &TokText(ref text)) => {
                    let new_token: Token = TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNonNewline));

                    let new_token2: Token = TokText(text.clone());
                    self.ast.add_child(&self.ast_path, new_token2);
                },
                (PassageContentNewline, &TokFormatBoldStart) | 
                (PassageContentNewline, &TokFormatItalicStart) |
                (PassageContentNewline, &TokFormatMonoStart) |
                (PassageContentNonNewline, &TokFormatBoldStart) | 
                (PassageContentNonNewline, &TokFormatItalicStart) |
                (PassageContentNonNewline, &TokFormatMonoStart) => {
                    new_nodes.push(PNode::new_non_terminal(Formating));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNonNewline));
                },
                (PassageContentNewline, &TokPassageLink(_, _)) |
                (PassageContentNonNewline, &TokPassageLink(_, _)) => {
                    new_nodes.push(PNode::new_non_terminal(Link));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNewline));
                },
                (PassageContentNewline, &TokNewLine) |
                (PassageContentNonNewline, &TokNewLine) => {
                    new_nodes.push(PNode::new_terminal(TokNewLine));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNewline));

                    // ast
                    self.ast.add_child(&self.ast_path, TokNewLine);
                },
                (PassageContentNonNewline, &TokFormatBoldEnd) |
                (PassageContentNewline, &TokFormatBoldEnd) => {
                    // jump one ast-level higher
                    self.ast_path.pop();
                },
                (PassageContentNonNewline, &TokFormatItalicEnd) |
                (PassageContentNewline, &TokFormatItalicEnd) => {
                    // jump one ast-level higher
                    self.ast_path.pop();
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
                    new_nodes.push(PNode::new_non_terminal(PassageContentNonNewline));
                    new_nodes.push(PNode::new_terminal(TokFormatBoldEnd));

                    //ast
                    let ast_count_passages = self.ast.count_childs(self.ast_path.to_vec());
                    let ast_token: Token = TokFormatBoldStart;
                    self.ast.add_child(&self.ast_path, ast_token);
                    self.ast_path.push(ast_count_passages);
                },

                // ItalicFormatting
                (ItalicFormatting, &TokFormatItalicStart) => {
                    new_nodes.push(PNode::new_terminal(TokFormatItalicStart));
                    new_nodes.push(PNode::new_non_terminal(PassageContentNonNewline));
                    new_nodes.push(PNode::new_terminal(TokFormatItalicEnd));

                    //ast
                    let ast_count_passages = self.ast.count_childs(self.ast_path.to_vec());
                    let ast_token: Token = TokFormatItalicStart;
                    self.ast.add_child(&self.ast_path, ast_token);
                    self.ast_path.push(ast_count_passages);
                },

                // MonoFormatting
                (MonoFormatting, &TokFormatMonoStart) => {
                    new_nodes.push(PNode::new_terminal(TokFormatMonoStart));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                    new_nodes.push(PNode::new_terminal(TokFormatMonoEnd));
                },

                // MonoContent
                (MonoContent, &TokText(ref text)) => {
                    let new_token: Token = TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                },
                (MonoContent, &TokNewLine) => {
                    new_nodes.push(PNode::new_terminal(TokNewLine));
                    new_nodes.push(PNode::new_non_terminal(MonoContent));
                },

                // Link
                (Link, &TokPassageLink(ref text, ref name)) => {
                    let new_token: Token = TokPassageLink(text.clone(), name.clone());
                    new_nodes.push(PNode::new_terminal(new_token));

                    // ast
                    let new_token2: Token = TokPassageLink(text.clone(), name.clone());
                    self.ast.add_child(&self.ast_path, new_token2);
                },

                
                _ => {

                }
            }

            // adds the new nodes to the tree
            // and adds the path in the tree to the stack
            let length = new_nodes.len().clone();
            self.tree.add_nodes(new_nodes, &to_add_path);
            self.stack.push_path(length as u8, to_add_path);

        } else {
            // no token left
            // only ɛ-productions could be here
            // these productions will be poped of the stack
        }
    }

    /// sets the lookahead to the next token
    fn next_token(&mut self) {
        self.lookahead = self.lookahead + 1;
    }
}

//==============================
// stack of the parser
struct Stack {
    data: Vec<Vec<usize>>
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    /// pushs the address of the first startsymbol to the stack
    fn push_start(&mut self) {
        self.data.push(vec![]);
    }

    /// save the path of the nodes in the tree to the stack
    /// the right part of the production
    /// should be on the stack in reverse order
    fn push_path(&mut self, count_elements: u8, to_add_path: Vec<usize>) {
        for i in 0..count_elements {
            let mut path: Vec<usize> = to_add_path.to_vec();
            path.push((count_elements-i-1) as usize);
            self.data.push(path);
        }
    }

    fn pop(&mut self) -> Option<Vec<usize>> {
        self.data.pop()
    }
}
