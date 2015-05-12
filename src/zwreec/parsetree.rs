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

pub use super::frontend::lexer::Token;


//==============================
// grammar

#[derive(Debug)]
enum NonTerminalType {
    S,
    S2,
    Passage,
    PassageContent,
    B
}

//==============================

struct SyntaxTree {
    root: NodeType
}

//#[derive(Debug)]
enum NodeType {
    NonTerminal (NodeNonTerminal),
    Terminal (NodeTerminal)
}

// ================================
// node types

struct NodeNonTerminal {
    category: NonTerminalType,
    childs: Vec<NodeType>
}

struct NodeTerminal {
    category: Token
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
            &NodeType::Terminal(ref t) => {
                debug!("{}|- Terminal: {:?}", spaces, t.category);
            }
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

impl SyntaxTree {
    pub fn new() -> SyntaxTree {
        SyntaxTree {
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

pub fn temp_create_parse_tree(tokens: Vec<Token>) {
    let mut tree: SyntaxTree = SyntaxTree::new();
    let mut stack: Vec<Vec<usize>> = Vec::new();

    stack.push(vec![]);
    let mut lookahead = 0;
    
    // stack parser...
    while !stack.is_empty() {
        let mut is_terminal;
        let top_path;

        // check if the top-element of the stack is a terminal or non terminal
        if let Some(top) = stack.last() {
            is_terminal = tree.root.is_terminal( top.to_vec() );
            top_path = top.to_vec();
        } else {
            panic!("error");
        }

        stack.pop();

        if is_terminal {
            // lookahead looks to the next token
            lookahead = lookahead + 1;

        } else {
            // parse table in code
            let to_add_path: Vec<usize> = top_path.to_vec();
            match tree.root.get_non_terminal(top_path) {
                &NonTerminalType::S => {
                    tree.add_two_nodes(NodeType::new_non_terminal(NonTerminalType::Passage), NodeType::new_non_terminal(NonTerminalType::S2), &to_add_path);
                    add_two_to_stack(&mut stack, to_add_path);

                },
                &NonTerminalType::S2 => {
                    // check follow...
                    if let Some(token) = tokens.get(lookahead) {
                        match token {
                            &Token::TokPassageName (_) => {
                                tree.add_one_node(NodeType::new_non_terminal(NonTerminalType::S), &to_add_path);
                                add_one_to_stack(&mut stack, to_add_path);
                            },
                            _ => { }
                        }
                    }
                },
                &NonTerminalType::Passage => {
                    if let Some(token) = tokens.get(lookahead) {
                        match token {
                            &Token::TokPassageName (ref name) => {
                                let new_token: Token = Token::TokPassageName(name.clone());
                                tree.add_two_nodes(NodeType::new_terminal(new_token), NodeType::new_non_terminal(NonTerminalType::PassageContent), &to_add_path);
                                add_two_to_stack(&mut stack, to_add_path);
                            },
                            _ => { }
                        }
                    }
                    

                    
                },
                &NonTerminalType::PassageContent => {
                     if let Some(token) = tokens.get(lookahead) {
                        match token {
                            &Token::TokText (ref text) => {
                                let new_token: Token = Token::TokText(text.clone());
                                tree.add_two_nodes(NodeType::new_terminal(new_token), NodeType::new_non_terminal(NonTerminalType::B), &to_add_path);
                                add_two_to_stack(&mut stack, to_add_path);
                            },
                            _ => { }
                        }
                    }
                },
                &NonTerminalType::B => {
                    // not implemented
                }
            }
        }
    }

    debug!("Parse Tree");
    tree.root.print(0);
}

// save the path of the nodes in the tree to the stack
// the right part of the production
// should be on the stack in reverse order
fn add_two_to_stack(stack: &mut Vec<Vec<usize>>, to_add_path: Vec<usize>) {
    let mut path1: Vec<usize> = to_add_path.to_vec();
    let mut path2: Vec<usize> = to_add_path.to_vec();

    path1.push(1);
    path2.push(0);

    stack.push(path1);
    stack.push(path2);
}

fn add_one_to_stack(stack: &mut Vec<Vec<usize>>, to_add_path: Vec<usize>) {
    let mut path1: Vec<usize> = to_add_path.to_vec();
    path1.push(0);
    stack.push(path1);
}


#[test]
fn it_works() {
}
