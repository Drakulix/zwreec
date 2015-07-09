rustlex! TweeLexer {

    // Manages callbacks from rustlex.
    //
    // Every unmatched character leads to a callback. Usually a callback causes a
    // lexer error, but in some specific cases there's no need for an error and
    // callbacks are ignored.
    callback => |lexer:&TweeLexer<R>, ch: char, location: (u64, u64)| {
        if !lexer.ignore_callback {
            let cfg = lexer.cfg.clone();
            error_panic!(&cfg.unwrap() => LexerError::UnexpectedCharacter { character: ch.clone(), location: location.clone() } );
        }
    }

    // Additional information to describe the lexer state.
    property cfg: Option<Config> = None;
    property ignore_callback:bool = true;
    property ignore_this_passage:bool = true;
    property format_bold_open:bool = false;
    property format_italic_open:bool = false;
    property format_under_open:bool = false;
    property format_strike_open:bool = false;
    property format_sub_open:bool = false;
    property format_sup_open:bool = false;
    property in_link:bool = false;
    property function_parens:usize = 0;
    property heading_rank:u8 = 0;

    // In the following regular expressions (regex) used by rustlex are listed.
    //
    // These regular expressions are used to control the current state of the
    // lexer as well as to identify Tokens that are matched by our lexical analyser.

    let WHITESPACE = ' ' | '\t';
    let NEWLINE = '\n';
    let UNDERSCORE = '_';
    let COLON = ',';
    let SEMI_COLON = ';';
    let PAREN_OPEN = '(';
    let PAREN_CLOSE = ')';
    let DIGIT = ['0'-'9'];
    let LETTER = ['a'-'z''A'-'Z'];
    let HTTP = ("http"("s")?|"ftp")"://"[^"/$.?# "'\n''\t']+"."[^" "'\n''\t']+;

    let COMMENT = "/%" ([^"%"]*(("%")*[^"%/"])?)* ("%")* "%/";
    let HTML_START = "<html" (" "[^">"]*)? ">";
    let HTML_END = "</html>";
    let HTML_DOCTYPE = "<!DOCTYPE" (" "[^">"]*)? ">";
    let HTML_TAGNAME = "a" | "abbr" | "acronym" | "address" | "applet" | "area" | "article" | "aside" | "audio" | "b" | "base" | "basefont" | "bdi" | "bdo" | "big" | "blockquote" | "body" | "br" | "button" | "canvas" | "caption" | "center" | "cite" |  "code" | "col" | "colgroup" | "datalist" | "dd" | "del" | "details" | "dfn" | "dialog" | "dir" | "div" | "dl" | "dt" | "em" | "embed" | "fieldset" | "figcaption" | "figure" | "font" | "footer" | "form" | "frame" | "frameset" | "h"("1"|"2"|"3"|"4"|"5"|"6") | "head" | "header" | "hr" | "i" | "iframe" | "img" | "input" | "ins" | "kbd" | "keygen" | "label" | "legend" | "li" | "link" | "main" | "map" | "mark" | "menu" | "menuitem" | "meta" | "meter" | "nav" | "noframes" | "noscript" | "object" | "ol" | "optgroup" | "option" | "output" | "p" | "param" | "pre" | "progress" | "q" | "rp" | "rt" | "ruby" | "s" | "samp" | "script" | "section" | "select" | "small" | "source" | "span" | "strike" | "strong" | "style" | "sub" | "summary" | "sup" | "table" | "tbody" | "td" | "textarea" | "tfoot" | "th" | "thead" | "time" | "title" | "tr" | "track" | "tt" | "u" | "ul" | "var" | "video" | "wbr";
    let HTML_TAG = "<" HTML_TAGNAME (" "[^">"]*)? ">" | "</" HTML_TAGNAME ">";
    let HTML_COMMENT_START = "<!--";
    let HTML_COMMENT_END = "-->";
    let HTML_TEXT = .;
    let ESCAPED_NEWLINE = '\\' NEWLINE;

    let PASSAGE_START = "::" ':'*;

    let PASSAGENAME_CHAR_START = [^"[]$<>:|" '\n'];
    let PASSAGENAME_CHAR = ":"? PASSAGENAME_CHAR_START;
    let PASSAGENAME = PASSAGENAME_CHAR_START PASSAGENAME_CHAR* ':'?;

    let TAG_START = '[';
    let TAG_END = ']';
    let TAG = ['a'-'z''A'-'Z''0'-'9''.''_']+;

    let TEXT_CHAR_START = [^"!#"'\n''\\'] | '\\'[^'\n'] | HTTP;
    let TEXT_CHAR = [^"/'_=~^{@<[" '\n''\\'] | '\\'[^'\n'] | HTTP;
    let TEXT = TEXT_CHAR+ | ["/'_=~^{@<["];
    let TEXT_HEADING = [^'\n']+;

    let VARIABLE_CHAR = LETTER | DIGIT | UNDERSCORE;
    let VARIABLE = '$' (LETTER | UNDERSCORE) VARIABLE_CHAR*;
    let VARIABLE_LENGTH = VARIABLE ".length";
    let ARRAY_ACCESS = VARIABLE '[' WHITESPACE* VARIABLE WHITESPACE* ']';

    let FORMAT_ITALIC = "//";
    let FORMAT_BOLD = "''";
    let FORMAT_UNDER = "__";
    let FORMAT_STRIKE = "==";
    let FORMAT_SUB = "~~";
    let FORMAT_SUP = "^^";
    let FORMAT_HEADING = ("!" | "!!" | "!!!" | "!!!!" | "!!!!!" );
    let FORMAT_NUMB_LIST = "#" WHITESPACE*;
    let FORMAT_INDENT_BLOCK = "<<<" NEWLINE;
    let FORMAT_HORIZONTAL_LINE = "----" NEWLINE;
    let FORMAT_INLINE = "@@"; //TODO ignore content

    let FORMAT_MONO_START = "{{{";
    let FORMAT_MONO_END = "}}}";
    let MONOSPACE_CHAR = [^"}"'\n'];
    let TEXT_MONOSPACE = MONOSPACE_CHAR+ | "}" | "}}";

    let LINK_OPEN = '[';
    let LINK_CLOSE = ']';
    let LINK_TEXT = [^'\n'"|[]"]+;
    let LINK_SIMPLE = "[[" (PASSAGENAME | VARIABLE) "]";
    let LINK_LABELED = "[[" LINK_TEXT "|" (PASSAGENAME | VARIABLE) "]";

    let MACRO_START = "<<";
    let MACRO_END = ">>";
    let MACRONAME = [^" >"'\n']* ( WHITESPACE+ "if")?;
    let MACRO_DISPLAY_PASSAGENAME = [^'"''>'' ''\t''\n'] ([^">"]*(">"[^">"])?)* [^'"''>'' ''\t''\n'] | [^"'>"' ''\t''\n'] ([^">"]*(">"[^">"])?)* [^"'>"' ''\t''\n'];

    let INT = DIGIT+;
    let FLOAT = (DIGIT+ "." DIGIT*) | (DIGIT* "." DIGIT+) | "Infinity";
    let STRING = '"' ([^'\\''"']|'\\'.)* '"' | "'" ([^'\\'"'"]|'\\'.)* "'";
    let BOOL = "true" | "false";
    let ASSIGN = "=" | "to" | "+=" | "-=" | "*=" | "/=";
    let NUM_OP = ["+-*/%"];
    let COMP_OP = "is" | "==" | "eq" | "!=" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
    let LOG_OP = "and" | "&&" | "or" | "||" | "not" | "!";

    let FUNCTION_NAME = (LETTER | UNDERSCORE) VARIABLE_CHAR*;
    let FUNCTION = FUNCTION_NAME '(';

    // In the following lexer states used by rustlex are described.
    //
    // Depending on the current state of the lexer tokens are identified and returned
    // when matched by a certain regex. If needed, matched regex' switch the state
    // of the lexer. Collections are used to bundle related regex.

    // Collection of regex, that manage most of passage content.
    I_PASSAGE_CONTENT {
        LINK_SIMPLE => |lexer:&mut TweeLexer<R>| {
            lexer.in_link = true;
            lexer.PASSAGE_CONTENT_LINK_VARIABLE_CHECK();
            let s =  lexer.yystr();
            let trimmed = &s[2 .. s.len()-1];
            let name = &trimmed.to_string();
            Some(TokPassageLink {location: lexer.yylloc(), display_name: name.clone(), passage_name: name.clone()} )
        }
        LINK_LABELED
                    => |lexer:&mut TweeLexer<R>| {
            lexer.in_link = true;
            lexer.PASSAGE_CONTENT_LINK_VARIABLE_CHECK();
            let s =  lexer.yystr();
            let trimmed = &s[2 .. s.len()-1];
            let matches = &trimmed.split("|").collect::<Vec<&str>>();
            assert_eq!(matches.len(), 2);
            let text = matches[0].to_string();
            let name = matches[1].to_string();
            Some(TokPassageLink {location: lexer.yylloc(), display_name: text, passage_name: name} )
        }
        MACRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.PASSAGE_CONTENT_MACRO();
            None
        }
        FORMAT_ITALIC
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_italic_open = !lexer.format_italic_open;
            if lexer.format_italic_open { Some(TokFormatItalicStart {location: lexer.yylloc()}) }
            else { Some(TokFormatItalicEnd {location: lexer.yylloc()}) }
        }
        FORMAT_BOLD => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_bold_open = !lexer.format_bold_open;
            if lexer.format_bold_open { Some(TokFormatBoldStart {location: lexer.yylloc()}) }
            else { Some(TokFormatBoldEnd {location: lexer.yylloc()}) }
        }
        FORMAT_UNDER
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_under_open = !lexer.format_under_open;
            if lexer.format_under_open { Some(TokFormatUnderStart {location: lexer.yylloc()}) }
            else { Some(TokFormatUnderEnd {location: lexer.yylloc()}) }
        }
        FORMAT_STRIKE
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_strike_open = !lexer.format_strike_open;
            if lexer.format_strike_open { Some(TokFormatStrikeStart {location: lexer.yylloc()}) }
            else { Some(TokFormatStrikeEnd {location: lexer.yylloc()}) }
        }
        FORMAT_SUB  => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_sub_open = !lexer.format_sub_open;
            if lexer.format_sub_open { Some(TokFormatSubStart {location: lexer.yylloc()}) }
            else { Some(TokFormatSubEnd {location: lexer.yylloc()}) }
        }
        FORMAT_SUP  => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            lexer.format_sup_open = !lexer.format_sup_open;
            if lexer.format_sup_open { Some(TokFormatSupStart {location: lexer.yylloc()}) }
            else { Some(TokFormatSupEnd {location: lexer.yylloc()}) }
        }
        FORMAT_MONO_START
                    => |lexer:&mut TweeLexer<R>| {
            lexer.PASSAGE_CONTENT_FORMAT_MONOSPACE();
            Some(TokFormatMonoStart {location: lexer.yylloc()})
        }
        NEWLINE     => |lexer:&mut TweeLexer<R>| {
            lexer.NEWLINE_PASSAGE_CONTENT();
            Some(TokNewLine {location: lexer.yylloc()} )
        }
        // The following matched regex are ignored in this state.
        COMMENT     => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            None
        }
        HTML_DOCTYPE
                    => |_    :&mut TweeLexer<R>| -> Option<Token> { None }
        HTML_START  => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.HTML();
            None
        }
        ESCAPED_NEWLINE
                    => |_    :&mut TweeLexer<R>| -> Option<Token> { None }
    }

    // Collection of regex, that ignore newlines and whitespace.
    I_IGNORE_NEWLINE {
        NEWLINE     => |_    :&mut TweeLexer<R>| -> Option<Token> { None }
    }
    I_IGNORE_WHITESPACE {
        WHITESPACE  => |_    :&mut TweeLexer<R>| -> Option<Token> { None }
    }

    // Collection of regex, that manage expressions. Functions are part of expressions.
    I_OPERANDS {
        VARIABLE    => |lexer:&mut TweeLexer<R>| Some(TokVariable{location: lexer.yylloc(), name: lexer.yystr()})
        ARRAY_ACCESS
                    => |lexer:&mut TweeLexer<R>| Some(TokArrayAccess{location: lexer.yylloc(), name: lexer.yystr()[..].split('[').next().unwrap().to_string(), index: lexer.yystr()[..].split('[').nth(1).unwrap().split(']').next().unwrap().trim().to_string() } )
        VARIABLE_LENGTH
                    => |lexer:&mut TweeLexer<R>| Some(TokArrayLength{location: lexer.yylloc(), name: lexer.yystr()[..].split('.').next().unwrap().to_string()} )
        INT         => |lexer:&mut TweeLexer<R>| Some(TokInt     {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()})
        FLOAT       => |lexer:&mut TweeLexer<R>| Some(TokFloat   {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()})
        STRING      => |lexer:&mut TweeLexer<R>| Some(TokString  {location: lexer.yylloc(), value: unescape(lexer.yystr())})
        BOOL        => |lexer:&mut TweeLexer<R>| Some(TokBoolean {location: lexer.yylloc(), value: lexer.yystr()})
    }
    I_OPERATORS {
        NUM_OP      => |lexer:&mut TweeLexer<R>| Some(TokNumOp   {location: lexer.yylloc(), op_name: lexer.yystr()})
        COMP_OP     => |lexer:&mut TweeLexer<R>| Some(TokCompOp  {location: lexer.yylloc(), op_name: lexer.yystr()})
        LOG_OP      => |lexer:&mut TweeLexer<R>| Some(TokLogOp   {location: lexer.yylloc(), op_name: lexer.yystr()})
    }
    I_EXPRESSION {
        :I_OPERANDS
        :I_OPERATORS
        FUNCTION =>  |lexer:&mut TweeLexer<R>| {
            let s =  lexer.yystr();
            let trimmed = &s[0 .. s.len()-1];
            let name = &trimmed.to_string();
            lexer.function_parens = 1;
            lexer.FUNCTION_ARGS();
            Some(TokFunction {location: lexer.yylloc(), name: name.clone()} )
        }
        PAREN_OPEN  => |lexer:&mut TweeLexer<R>| Some(TokParenOpen {location: lexer.yylloc()})
        PAREN_CLOSE => |lexer:&mut TweeLexer<R>| Some(TokParenClose{location: lexer.yylloc()})
        SEMI_COLON  => |lexer:&mut TweeLexer<R>| Some(TokSemiColon {location: lexer.yylloc()})
        ASSIGN      => |lexer:&mut TweeLexer<R>| {
            Some(TokAssign    {location: lexer.yylloc(), var_name: "".to_string(), op_name: lexer.yystr()})
        }
    }
    FUNCTION_ARGS {
        :I_OPERANDS
        :I_OPERATORS
        PAREN_OPEN  => |lexer:&mut TweeLexer<R>| {
            lexer.function_parens += 1;
            Some(TokParenOpen {location: lexer.yylloc()})
        }
        PAREN_CLOSE => |lexer:&mut TweeLexer<R>| {
            lexer.function_parens -= 1;
            if lexer.function_parens == 0 {
                if lexer.in_link {
                    lexer.PASSAGE_CONTENT_LINK_VARIABLE_SET();
                } else {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                }
                Some(TokArgsEnd {location: lexer.yylloc()})
            } else {
                Some(TokParenClose {location: lexer.yylloc()})
            }
        }
        COLON       => |lexer:&mut TweeLexer<R>| Some(TokColon {location: lexer.yylloc()})
        // The following matched regex are ignored in this state.
        :I_IGNORE_WHITESPACE
    }

    // This state is the initial state of our lexical analyser. It is left when matching
    // matching a PASSAGE_START regex. Unmatched characters will lead to a callback.
    // In this state callbacks are ignored.
    INITIAL {
        PASSAGE_START
                    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.ignore_callback = false;
            lexer.ignore_this_passage = false;
            lexer.PASSAGE();
            None
        }
    }

    // This state recognizes a passage declaration. Everything until a newline
    // or tag is considered as passagename. It is entered when matching a
    // PASSAGE_START regex and left when matching a NEWLINE regex.
    PASSAGE {
        PASSAGENAME => |lexer:&mut TweeLexer<R>| {
            Some(TokPassage {name: lexer.yystr().trim().to_string(), location: lexer.yylloc()} )
        }
        TAG_START   => |lexer:&mut TweeLexer<R>| {
            lexer.TAG_CONTENT();
            Some(TokTagStart {location: lexer.yylloc()})
        }
        NEWLINE     => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.NEWLINE_PASSAGE_CONTENT();
            None
        }
    }

    // This state recognizes tags. It is entered when matching a TAG_START regex
    // and left when matching a TAG_END regex. Unmatched characters will lead to
    // a callback. There are some specific tags that make ignore the current passage.
    TAG_CONTENT {
        TAG         => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            match lexer.yystr().as_ref() {
                "stylesheet" | "script" => { lexer.ignore_this_passage = true; }
                _                       => { }
            }
            Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr().to_string()})
        }
        TAG_END     => |lexer:&mut TweeLexer<R>| {
            lexer.ignore_callback = true;
            lexer.TAG_END_WAIT_FOR_NEWLINE();
            Some(TokTagEnd {location: lexer.yylloc()})
        }
        // The following matched regex are ignored in this state.
        :I_IGNORE_WHITESPACE
    }

    // This state waits for a newline after one or several tags to finish a passage
    // declaration. It is entered when matching a TAG_END regex and left when
    // matching a NEWLINE regex. Unmatched characters will lead to a callback.
    // In this state callbacks are ignored.
    TAG_END_WAIT_FOR_NEWLINE {
        NEWLINE     => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            if !lexer.ignore_this_passage {
                lexer.ignore_callback = false;
                lexer.NEWLINE_PASSAGE_CONTENT();
            } else {
                lexer.INITIAL();
            }
            None
        }
    }

    // This state manages passage content while looking at the first character in a
    // newline. It is entered after matching a newline within passage content or after
    // matching a passage declaration. It is left when matching any character.
    // Matching a PASSAGE_START leads to a new passage declaration. Everything else
    // belongs to the content of the current passage.
    NEWLINE_PASSAGE_CONTENT {
        PASSAGE_START
                    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.PASSAGE();
            None
        }
        :I_PASSAGE_CONTENT
        FORMAT_HORIZONTAL_LINE
                    => |lexer:&mut TweeLexer<R>| Some(TokFormatHorizontalLine {location: lexer.yylloc()})
        FORMAT_INDENT_BLOCK
                    => |lexer:&mut TweeLexer<R>| Some(TokFormatIndentBlock {location: lexer.yylloc()})
        FORMAT_NUMB_LIST
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokFormatNumbList {location: lexer.yylloc()})
        }
        FORMAT_HEADING
                    => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.heading_rank = lexer.yystr().trim().len() as u8;
            lexer.PASSAGE_CONTENT_FORMAT_HEADING();
            None
        }
        TEXT_CHAR_START
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokText {location: lexer.yylloc(), text: lexer.yystr()})
        }
    }

    // This state manages passage content unlike the first character in a line.
    // It is entered after matching a character within a passage content. It is left
    // when matching a NEWLINE regex or some special passage content constructs
    // like headings, links and macros. Matching a PASSAGE_START leads to a new
    // passage declaration.
    NON_NEWLINE_PASSAGE_CONTENT {
        :I_PASSAGE_CONTENT
        TEXT        => |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()})
    }

    // This state recognizes a heading. Everything until a newline is matched as
    // a heading text. It is entered when matching a FORMAT_HEADING regex and left
    // when matching a NEWLINE regex.
    PASSAGE_CONTENT_FORMAT_HEADING {
        TEXT_HEADING
                    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            Some(TokFormatHeading {location: lexer.yylloc(), text: lexer.yystr().trim().to_string(), rank: lexer.heading_rank})
        }
        NEWLINE     => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.heading_rank = 0;
            lexer.NEWLINE_PASSAGE_CONTENT();
            Some(TokNewLine {location: lexer.yylloc()})
        }
    }

    // This state recognizes monospace. Everything between `{{{` and `}}}` is
    // matched as monospaced text. It is entered when matching a MONOSPACE_START
    // regex and left when matching a MONOSPACE_END regex. Monospaced newlines
    // are ignored.
    PASSAGE_CONTENT_FORMAT_MONOSPACE {
        FORMAT_MONO_END
                    => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokFormatMonoEnd {location: lexer.yylloc()} )
        }
        TEXT_MONOSPACE
                    => |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()})
        NEWLINE     => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            Some(TokText {location: lexer.yylloc(), text: " ".to_string()} )
        }
    }

    // This state checks if a link is a setter-link: a link with a variable declaration.
    // It is entered when matching a LINK_CLOSE regex first time and left when
    // matching a second LINK_CLOSE regex or another LINK_OPEN regex. Everything else
    // will lead to a callback.
    PASSAGE_CONTENT_LINK_VARIABLE_CHECK {
        LINK_CLOSE  => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.in_link = false;
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            None
        }
        LINK_OPEN   => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.PASSAGE_CONTENT_LINK_VARIABLE_SET();
            Some(TokVarSetStart {location: lexer.yylloc()} )
        }
    }

    // This state recognizes an expression within a link. It is entered when
    // matching a LINK_OPEN regex and left when matching a LINK_CLOSE regex. Every
    // non valid expression will lead to a callback.
    PASSAGE_CONTENT_LINK_VARIABLE_SET {
        :I_EXPRESSION
        LINK_CLOSE  => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.PASSAGE_CONTENT_LINK_WAIT_FOR_CLOSE();
            Some(TokVarSetEnd {location: lexer.yylloc()} )
        }
        // The following matched regex are ignored in this state.
        :I_IGNORE_WHITESPACE
    }

    // This state waits for a final `]` after a variable declaration within a link.
    // It is entered when matching a LINK_CLOSE regex and left when matching
    // another LINK_CLOSE regex. Everything else will lead to a callback.
    PASSAGE_CONTENT_LINK_WAIT_FOR_CLOSE {
        LINK_CLOSE  => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.in_link = false;
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            None
        }
    }

    // This state recognizes a macro. It is entered when matching a MACRO_START
    // regex and left when matching a MACRONAME, VARIABLE or WHITESPACE regex.
    //       There are several built-in macros. Any matched not built-in macro indicates a short diplay macro.
    //       A matching variable indicates a short print.
    // Whitespace after an opening `<<` aborts. Unmatched characters will lead to a callback.
    PASSAGE_CONTENT_MACRO {
        MACRONAME   => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            let replaced_string = str::replace(lexer.yystr().trim(),  " ", "");
            match replaced_string.as_ref() {
                "set" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroSet {location: lexer.yylloc()} )
                },
                "if" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroIf {location: lexer.yylloc()} )
                },
                "else" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroElse {location: lexer.yylloc()} )
                },
                "elseif" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroElseIf {location: lexer.yylloc()} )
                },
                "endif" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroEndIf {location: lexer.yylloc()} )
                },
                "print" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroPrint {location: lexer.yylloc()} )
                },
                "display" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT_DISPLAY();
                    None
                },
                "silently" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroSilently {location: lexer.yylloc()} )
                },
                "endsilently" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroEndSilently {location: lexer.yylloc()} )
                },
                "nobr" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroNoBr {location: lexer.yylloc()} )
                },
                "endnobr" => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT();
                    Some(TokMacroEndNoBr {location: lexer.yylloc()} )
                },
                _ => {
                    lexer.PASSAGE_CONTENT_MACRO_CONTENT_SHORT_DISPLAY();
                    Some(TokMacroDisplay {location: lexer.yylloc(), passage_name: replaced_string.to_string()} )
                }
            }
        }
        VARIABLE    => |lexer:&mut TweeLexer<R>| {
            lexer.PASSAGE_CONTENT_MACRO_CONTENT_SHORT_PRINT();
            Some(TokMacroContentVar {location: lexer.yylloc(), var_name: lexer.yystr()} )
        }
        WHITESPACE  => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            None
        }
    }

    // This state recognizes a expression within a macro. It is entered when
    // matching a MACRONAME regex and left when matching a MACRO_END regex.
    // Every non valid expression will lead to a callback.
    PASSAGE_CONTENT_MACRO_CONTENT {
        MACRO_END   => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
        :I_EXPRESSION
        // The following matched regex are ignored in this state.
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE
    }

    // This state recognizes a passagename whithin a display macro. A
    // passagename can be represented as string or plain text. It is entered
    // when matching a MACRONAME regex and left when matching a MACRO_END
    // regex. Unmatched characters will lead to a callback.
    PASSAGE_CONTENT_MACRO_CONTENT_DISPLAY {
        MACRO_END   => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
        MACRO_DISPLAY_PASSAGENAME
                    => |lexer:&mut TweeLexer<R>| {
            Some(TokMacroDisplay {location: lexer.yylloc(), passage_name: lexer.yystr().trim().to_string()} )
        }
        STRING      => |lexer:&mut TweeLexer<R>| {
            Some(TokMacroDisplay {passage_name: unescape(lexer.yystr()), location: lexer.yylloc()})
        }
        // The following matched regex are ignored in this state.
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE
    }

    // This state waits for a final `>>` after a short print macro. It is
    // entered when matching a VARIABLE regex within a macro and left when
    // matching a MACRO_END regex. Unmatched characters will lead to a callback.
    PASSAGE_CONTENT_MACRO_CONTENT_SHORT_PRINT {
        MACRO_END   => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
        // The following matched regex are ignored in this state.
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE
    }

    // This state waits for a final `>>` after a short display macro. It is
    // entered when matching a MACRONAME regex within a macro and left when
    // matching a MACRO_END regex. Unmatched characters will lead to a callback.
    PASSAGE_CONTENT_MACRO_CONTENT_SHORT_DISPLAY {
        MACRO_END   => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
        // The following matched regex are ignored in this state.
        VARIABLE    => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        INT         => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        FLOAT       => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        STRING      => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        BOOL        => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        NUM_OP      => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        COMP_OP     => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        LOG_OP      => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        FUNCTION    => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        PAREN_OPEN  => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        PAREN_CLOSE => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        SEMI_COLON  => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        COLON       => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        ASSIGN      => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE
    }

    // This state filters HTML. Everything except HTML tags and comments is matched
    // as text (or newline). It is entered when matching a HTML_START regex and
    // left when matching a HTML_END regex.
    HTML {
        HTML_END    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE_PASSAGE_CONTENT();
            None
        }
        HTML_COMMENT_START
                    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.ignore_callback = true;
            lexer.HTML_COMMENT();
            None
        }
        HTML_TEXT   => |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()})
        NEWLINE     => |lexer:&mut TweeLexer<R>| Some(TokNewLine {location: lexer.yylloc()})
        // The following matched regex are ignored in this state.
        HTML_TAG    => |_    :&mut TweeLexer<R>| -> Option<Token> { None }
    }

    // This state filters HTML comments. It is entered when matching a
    // HTML_COMMENT_START regex and left when matching a HTML_COMMENT_END regex.
    // Unmatched characters will lead to a callback. In this state callbacks are ignored.
    HTML_COMMENT {
        HTML_COMMENT_END
                    => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.ignore_callback = false;
            lexer.HTML();
            None
        }
    }
}
