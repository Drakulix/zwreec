//! The `parser` module contains a lot of useful functionality
//! to parse tokens from the lexer and create an parse-tree
//! its an predictiv parser for a LL(1) grammar

/*


----------------------------------------
Grammatik:

LL(1)
----------------------------------------|
S -> Passage S2                         |
S2 -> S                                 |
S2 -> ɛ                                 |
Passage -> PassageName PassageContent   |
PassageContent -> TextPassage B         |
PassageContent -> Formatting B          |
PassageContent -> Newline B             |
B -> PassageContent                     |
B -> ɛ                                  |
----------------------------------------|


Hello World im LL(1)-Parser:
token: [TokPassageName("Start"), TokText("Hello World\n")]

input                   | stack                         | output                              | 
------------------------+-------------------------------+-------------------------------------+---------------------------------------------------
PassageName TextPassage | S                             | S->Passage S'                       | passage and s' as child of S
        --||--          | Passage S2                    | Passage->PassageName PassageContent | passageName and passagecontent as child of passage
        --||--          | PassageName PassageContent S2 |                                     | gehe bei den token 1 weiter
TextPassage             | PassageContent S2             | PassageContent->TextPassage B       | textpassage and b as child of PassageContent
        --||--          | TextPassage B S'              |                                     | gehe bei den token 1 weiter
empty                   | B S'                          | B->ɛ                                | 
empty                   | S'                            | S' -> ɛ                             |
empty                   | empty                         | ACCEPT                              |
------------------------+-------------------------------+-------------------------------------|---------------------------------------------------

*/

pub use frontend::lexer::Token;

pub fn parse_tokens(tokens: Vec<Token>) -> ParseTree {
    let mut parser: Parser = Parser::new(tokens);    
    parser.parsing();
    parser.tree
}

//==============================
// grammar

#[derive(Debug, Copy, Clone)]
pub enum NonTerminalType {
    S,
    S2,
    Passage,
    PassageContent,
    B
}

//==============================
// parser

struct Parser {
    tree: ParseTree,
    stack: Stack,
    tokens: Vec<Token>,
    lookahead: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tree: ParseTree::new(),
            stack: Stack::new(),
            tokens: tokens,
            lookahead: 0
        }
    }

    /// the predictive stack ll(1) parsing routine
    pub fn parsing(&mut self) {
        self.stack.push_start();
        
        while let Some(top) = self.stack.pop() {
            if self.tree.root.is_terminal(top.to_vec()) {
                self.next_token();
            } else {
                self.apply_grammar(top.to_vec());
            }
        }

        debug!("Parse Tree");
        self.tree.root.print(0);
    }

    /// apply the ll(1) grammar
    /// the match-statement simulates the parsing-table behavior
    /// 
    fn apply_grammar(&mut self, top_path: Vec<usize>) {
        //let current_token: &Token;
        if let Some(token) = self.tokens.get_mut(self.lookahead) {
            //current_token = token;

            let to_add_path: Vec<usize> = top_path.to_vec();

            // the frst item in the tuple is the current state and
            // the snd is the current lookup-token
            let state_first: (NonTerminalType, &Token) = (self.tree.root.get_non_terminal(top_path).clone(), token);

            let mut new_nodes = Vec::new();
            match state_first {
                (NonTerminalType::S, &Token::TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::Passage));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::S2));
                },
                (NonTerminalType::S2, &Token::TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::S));
                },
                (NonTerminalType::Passage, &Token::TokPassageName(ref name)) => {
                    let new_token: Token = Token::TokPassageName(name.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::PassageContent));
                },
                (NonTerminalType::PassageContent, &Token::TokText(ref text)) => {
                    let new_token: Token = Token::TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::B));
                },
                (NonTerminalType::B, _) => {
                    // not implemented
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

//==============================
// parsetree

pub struct ParseTree {
    root: PNode
}

impl ParseTree {
    pub fn new() -> ParseTree {
        ParseTree {
            root: PNode::new_non_terminal(NonTerminalType::S)
        }
    }

    /// adds nodes to the tree
    pub fn add_nodes(&mut self, childs: Vec<PNode>, to_add_path: &Vec<usize>) {
        for child in childs {
            self.root.add_child_at(to_add_path, child);
        }
    }
}

//==============================
// node of the paretree

pub struct NodeNonTerminal {
    category: NonTerminalType,
    childs: Vec<PNode>
}

pub struct NodeTerminal {
    category: Token
}

//#[derive(Debug)]
pub enum PNode {
    NonTerminal (NodeNonTerminal),
    Terminal (NodeTerminal)
}

impl PNode {
    pub fn new_terminal(terminal: Token) -> PNode {
        PNode::Terminal(NodeTerminal  { category: terminal })
    }

    pub fn new_non_terminal(non_terminal: NonTerminalType) -> PNode {
        PNode::NonTerminal(NodeNonTerminal { category: non_terminal, childs: Vec::new() })
    }

    /// prints a node
    pub fn print(&self, indent: usize) {
        let mut spaces = "".to_string();
        for _ in 0..indent { 
            spaces.push_str(" ");
        }

        match self {
            &PNode::NonTerminal(ref t) => {
                debug!("{}|- Node: {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            }
            &PNode::Terminal(ref t) => { debug!("{}|- Terminal: {:?}", spaces, t.category); }
        }
    }

    /// returns a non_terminal of the path
    pub fn get_non_terminal(&self, path: Vec<usize>) -> &NonTerminalType {
        match self {
            &PNode::NonTerminal(ref node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    return node.childs[*index].get_non_terminal(new_path);
                } else {
                    return &node.category;
                }
            },
            _ => panic!("error")
        }
    }

    /// adds a node to the childs in path
    pub fn add_child_at(&mut self, path: &[usize], child: PNode) {
        match self {
            &mut PNode::NonTerminal (ref mut node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    node.childs[*index].add_child_at(&new_path, child);
                } else {
                    node.childs.push(child);
                }
            },
            _ => panic!("error")
        }
    }

    /// checks if the node of the path is a terminal or not
    pub fn is_terminal(&self, path: Vec<usize>) -> bool {
        match self {
            &PNode::NonTerminal(ref node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    return node.childs[*index].is_terminal(new_path);
                }

                return false
            },
            &PNode::Terminal(_) => return true
        }

        false
    }
}

#[test]
fn it_works() {

}

