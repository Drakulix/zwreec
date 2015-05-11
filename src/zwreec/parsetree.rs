
/*

ll(1)...
parse-table erstellen
damit man weiß, von wo nach wo man gehen muss...


übersetzerbau mitschrift, s54..

LL(1)

S -> Passage S'                         | S.i = new Node()
S' -> S                                 |
S' -> ɛ                                 |
Passage -> PassageName PassageContent   |
PassageContent -> TextPassage B         |
PassageContent -> Formatting B          |
 PassageContent -> Newline B            |
B -> PassageContent                     |
B -> ɛ                                  |



*/

enum NonTerminalType {
    S,
    Passage,
    PassageContent,
    B
}

enum TerminalType {
    PassageName,
    TextPassage,
    Newline
}

struct SyntaxTree<'a> {
    childs: Vec<NodeType<'a>>
}

enum NodeType<'a> {
    NonTerminal (NodeNonTerminal<'a>),
    Terminal (NodeTerminal<'a>)
}

// ================================
// node types

struct NodeNonTerminal<'a> {
    category: NonTerminalType,
    childs: Vec<NodeType<'a>>
}

struct NodeTerminal<'a> {
    category: TerminalType,
    value: &'a str
}

// to print enum
#[derive(Debug)]
enum SymbolType {
    NonTerminal,
    Terminal
}

impl<'a> NodeType<'a> {

}

impl<'a> SyntaxTree<'a> {

    pub fn new() -> SyntaxTree<'a> {
        SyntaxTree {
            childs: Vec::new()
        }
    }
}

pub fn temp_create_syntax_tree() {
    debug!("temp_create_syntax_tree");

    //let 
    let mut tree: SyntaxTree = SyntaxTree::new();

    // parsing begins...



    /*tree.add_passage("start");
    tree.add_type(TokenType::FormatBold);
    tree.add_text("123");
    tree.add_type(TokenType::Newline);
    tree.add_passage(&"andere passage");
    tree.add_text("in passage2");
    tree.add_text("other text in passage2");
    tree.print();*/

}

#[test]
fn it_works() {
}
