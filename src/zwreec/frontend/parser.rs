/*


TODO
-parser in eigenes struct packen. zumindest sollten die functionen raus aus dem globalen

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

//==============================
// grammar

//#[derive(Debug)]
#[derive(Debug, Copy, Clone)]
enum NonTerminalType {
    S,
    S2,
    Passage,
    PassageContent,
    B
}
/*impl Copy for NonTerminalType {}
impl Clone for NonTerminalType {
    fn clone(&self) -> NonTerminalType { *self }
}*/
//impl Clone for NonTerminalType {}


pub fn temp_create_parse_tree(tokens: Vec<Token>) {

    let mut parser: Parser = Parser::new(tokens);    
    parser.start();
}

//==============================

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

    pub fn start(&mut self) {
        self.stack.push_start();
        //let mut lookahead = 0;
        
        // stack parser...
        while !self.stack.is_empty() {
            let mut is_terminal;
            let top_path;

            // check if the top-element of the stack is a terminal or non terminal
            if let Some(top) = self.stack.last() {
                is_terminal = self.tree.root.is_terminal( top.to_vec() );
                top_path = top.to_vec();
            } else {
                panic!("error (but can't happen, becouse the stack can't be empty becouse of the while loop)");
            }

            self.stack.pop();

            if is_terminal {
                self.next_token();

            } else {
                let to_add_path: Vec<usize> = top_path.to_vec();

                let current_token;
                if let Some(token) = self.tokens.get(self.lookahead) {
                    current_token = token
                } else {
                    // no token left
                    // only ɛ-productions could be here
                    // these productions should be poped of the stack
                    continue;
                }
                
                // parse table in code
                let from_to: (NonTerminalType, &Token) = (self.tree.root.get_non_terminal(top_path).clone(), current_token);

                match from_to {
                    (NonTerminalType::S, &Token::TokPassageName (_)) => {
                        self.tree.add_two_nodes(NodeType::new_non_terminal(NonTerminalType::Passage), NodeType::new_non_terminal(NonTerminalType::S2), &to_add_path);
                        self.stack.push_path(2, to_add_path);
                    },
                    (NonTerminalType::S2, &Token::TokPassageName (_)) => {
                        self.tree.add_one_node(NodeType::new_non_terminal(NonTerminalType::S), &to_add_path);
                        self.stack.push_path(1, to_add_path);
                    },
                    (NonTerminalType::Passage, &Token::TokPassageName (ref name)) => {
                        let new_token: Token = Token::TokPassageName(name.clone());
                        self.tree.add_two_nodes(NodeType::new_terminal(new_token), NodeType::new_non_terminal(NonTerminalType::PassageContent), &to_add_path);
                        self.stack.push_path(2, to_add_path);
                    },
                    (NonTerminalType::PassageContent, &Token::TokText (ref text)) => {
                        let new_token: Token = Token::TokText(text.clone());
                        self.tree.add_two_nodes(NodeType::new_terminal(new_token), NodeType::new_non_terminal(NonTerminalType::B), &to_add_path);
                        self.stack.push_path(2, to_add_path);
                    },
                    (NonTerminalType::B, _) => {
                        // not implemented
                    }
                    _ => {

                    }
                }
            }
        }

        debug!("Parse Tree");
        self.tree.root.print(0);
    }

    fn next_token(&mut self) {
        self.lookahead = self.lookahead + 1;
    }
}

//==============================

struct Stack {
    data: Vec<Vec<usize>>
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    fn push_start(&mut self) {
        self.data.push(vec![]);
    }

    // save the path of the nodes in the tree to the stack
    // the right part of the production
    // should be on the stack in reverse order
    fn push_path(&mut self, count_elements: u8, to_add_path: Vec<usize>) {
        for i in 0..count_elements {
            let mut path: Vec<usize> = to_add_path.to_vec();
            path.push((count_elements-i-1) as usize);
            self.data.push(path);
        }
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn last(&self) -> Option<&Vec<usize>> {
        self.data.last()
    }

    fn pop(&mut self) -> Option<Vec<usize>> {
        self.data.pop()
    }
}

//==============================

struct ParseTree {
    root: NodeType
}

impl ParseTree {
    pub fn new() -> ParseTree {
        ParseTree {
            root: NodeType::new_non_terminal(NonTerminalType::S)
        }
    }

    // adds one nodes to childs at the path
    pub fn add_one_node(&mut self, child: NodeType, to_add_path: &Vec<usize>) {
        self.root.add_child_at(&to_add_path, child);
    }

    // adds two nodes to childs at the path
    pub fn add_two_nodes(&mut self, child1: NodeType, child2: NodeType, to_add_path: &Vec<usize>) {
        self.root.add_child_at(&to_add_path, child1);
        self.root.add_child_at(&to_add_path, child2);
    }
}

//==============================

struct NodeNonTerminal {
    category: NonTerminalType,
    childs: Vec<NodeType>
}

struct NodeTerminal {
    category: Token
}

//#[derive(Debug)]
enum NodeType {
    NonTerminal (NodeNonTerminal),
    Terminal (NodeTerminal)
}

impl NodeType {

    pub fn new_terminal(terminal: Token) -> NodeType {
        NodeType::Terminal(NodeTerminal  { category: terminal })
    }

    pub fn new_non_terminal(non_terminal: NonTerminalType) -> NodeType {
        NodeType::NonTerminal(NodeNonTerminal { category: non_terminal, childs: Vec::new() })
    }

    // prints a node
    pub fn print(&self, indent: usize) {
        let mut spaces = "".to_string();
        for _ in 0..indent { 
            spaces.push_str(" ");
        }

        match self {
            &NodeType::NonTerminal(ref t) => {
                debug!("{}|- Node: {:?}", spaces, t.category);
                for child in &t.childs {
                    child.print(indent+2);
                }
            }
            &NodeType::Terminal(ref t) => { debug!("{}|- Terminal: {:?}", spaces, t.category); }
        }
    }

    // returns a non_terminal of the path
    pub fn get_non_terminal(&self, path: Vec<usize>) -> &NonTerminalType {
        match self {
            &NodeType::NonTerminal(ref node) => {
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

    // adds a node to the childs in path
    pub fn add_child_at(&mut self, path: &[usize], child: NodeType) {
        match self {
            &mut NodeType::NonTerminal (ref mut node) => {
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

    // checks if the node of the path is a terminal or not
    pub fn is_terminal(&self, path: Vec<usize>) -> bool {
        match self {
            &NodeType::NonTerminal(ref node) => {
                if let Some(index) = path.first() {
                    let mut new_path: Vec<usize> = path.to_vec();
                    new_path.remove(0);
                    return node.childs[*index].is_terminal(new_path);
                }

                return false
            },
            &NodeType::Terminal(_) => return true
        }

        false
    }
}

#[test]
fn it_works() {
}
