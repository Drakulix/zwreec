rustlex! TweeLexer {

    //=============================
    // properties

    property ignore_passage:bool = false;
    property format_bold_open:bool = false;
    property format_italic_open:bool = false;
    property format_under_open:bool = false;
    property format_strike_open:bool = false;
    property format_sub_open:bool = false;
    property format_sup_open:bool = false;
    property function_parens:usize = 0;

    //=============================
    // regular expressions

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

    let START_CHAR_IGNORE = [^":"'\n'] | ':' [^":"'\n'];
    let CHAR_IGNORE = [^'\n''\t'];
    let TEXT_IGNORE = START_CHAR_IGNORE CHAR_IGNORE*;

    let COMMENT = "/%" ([^"%"]*(("%")*[^"%/"])?)* ("%")* "%/";
    let HTML_START = "<html" (" "[^">"]*)? ">";
    let HTML_END = "</html>";
    let HTML_DOCTYPE = "<!DOCTYPE" (" "[^">"]*)? ">";
    let HTML_TAGNAME = "a" | "abbr" | "acronym" | "body" | "b" | "br" | "center" | "div" | "head" | "header" | "img" | "meta" | "p" | "style" | "title";
    let HTML_TAG = "<" HTML_TAGNAME (" "[^">"]*)? ">" | "</" HTML_TAGNAME ">";
    let HTML_TEXT = .;

    let PASSAGE_START = "::" ':'*;

    let PASSAGENAME_CHAR_START = [^"[]$<>:|" '\n'];
    let PASSAGENAME_CHAR = ":"? PASSAGENAME_CHAR_START;
    let PASSAGENAME = PASSAGENAME_CHAR_START PASSAGENAME_CHAR* ':'?;

    let TAG_START = '[';
    let TAG_END = ']';
    let TAG = ['a'-'z''A'-'Z''0'-'9''.''_']+;

    // If for example // is at a beginning of a line, then // is matched and not just /
    let TEXT_CHAR_START = [^"!>#"'\n'] | HTTP;
    let TEXT_CHAR = [^"/'_=~^{@<[" '\n'] | HTTP;
    let TEXT = TEXT_CHAR+ | ["/'_=~^{@<["];

    let VARIABLE_CHAR = LETTER | DIGIT | UNDERSCORE;
    let VARIABLE = '$' (LETTER | UNDERSCORE) VARIABLE_CHAR*;

    let FORMAT_ITALIC = "//";
    let FORMAT_BOLD = "''";
    let FORMAT_UNDER = "__";
    let FORMAT_STRIKE = "==";
    let FORMAT_SUB = "~~";
    let FORMAT_SUP = "^^";
    let FORMAT_HEADING = ("!" | "!!" | "!!!" | "!!!!" | "!!!!!") WHITESPACE*;
    let FORMAT_NUMB_LIST = "#" WHITESPACE*;
    let FORMAT_INDENT_BLOCK = "<<<" NEWLINE;
    let FORMAT_HORIZONTAL_LINE = "----" NEWLINE;
    let FORMAT_INLINE = "@@"; //TODO ignore content

    let FORMAT_MONO_START = "{{{";
    let FORMAT_MONO_END = "}}}";
    let MONOSPACE_CHAR = [^"}"'\n'];
    let MONOSPACE_TEXT = MONOSPACE_CHAR+ | "}" | "}}";

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
    let COMP_OP = "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
    let LOG_OP = "and" | "&&" | "or" | "||" | "not" | "!";

    let FUNCTION_NAME = (LETTER | UNDERSCORE) VARIABLE_CHAR*;
    let FUNCTION = FUNCTION_NAME '(';

    INITIAL {
        PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            if lexer.ignore_passage { lexer.ignore_passage = false; }
            lexer.PASSAGE();
            None
        }
        TEXT_IGNORE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
        NEWLINE => |_:&mut TweeLexer<R>| -> Option<Token> { None }
    }

    I_IGNORE_NEWLINE {
        NEWLINE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
    }

    I_IGNORE_WHITESPACE {
        WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
    }

    I_EMIT_COLON {
        COLON =>      |lexer:&mut TweeLexer<R>| Some(TokColon     {location: lexer.yylloc()} )
    }

    I_PASSAGE_CONTENT {
        MACRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.MACRO();
            None
        }

        LINK_SIMPLE =>  |lexer:&mut TweeLexer<R>| {
            lexer.LINK_VAR_CHECK();
            let s =  lexer.yystr();
            let trimmed = &s[2 .. s.len()-1];
            let name = &trimmed.to_string();
            Some(TokPassageLink {location: lexer.yylloc(), display_name: name.clone(), passage_name: name.clone()} )
        }

        LINK_LABELED =>  |lexer:&mut TweeLexer<R>| {
            lexer.LINK_VAR_CHECK();
            let s =  lexer.yystr();
            let trimmed = &s[2 .. s.len()-1];
            let matches = &trimmed.split("|").collect::<Vec<&str>>();
            assert_eq!(matches.len(), 2);
            let text = matches[0].to_string();
            let name = matches[1].to_string();
            Some(TokPassageLink {location: lexer.yylloc(), display_name: text, passage_name: name} )
        }

        FORMAT_ITALIC => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_italic_open = !lexer.format_italic_open;
            if lexer.format_italic_open {Some(TokFormatItalicStart {location: lexer.yylloc()} )}
            else {Some(TokFormatItalicEnd {location: lexer.yylloc()} )}
        }
        FORMAT_BOLD => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_bold_open = !lexer.format_bold_open;
            if lexer.format_bold_open {Some(TokFormatBoldStart {location: lexer.yylloc()} )}
            else {Some(TokFormatBoldEnd {location: lexer.yylloc()} )}
        }
        FORMAT_UNDER => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_under_open = !lexer.format_under_open;
            if lexer.format_under_open {Some(TokFormatUnderStart {location: lexer.yylloc()} )}
            else {Some(TokFormatUnderEnd {location: lexer.yylloc()} )}
        }
        FORMAT_STRIKE => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_strike_open = !lexer.format_strike_open;
            if lexer.format_strike_open {Some(TokFormatStrikeStart {location: lexer.yylloc()} )}
            else {Some(TokFormatStrikeEnd {location: lexer.yylloc()} )}
        }
        FORMAT_SUB => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_sub_open = !lexer.format_sub_open;
            if lexer.format_sub_open {Some(TokFormatSubStart {location: lexer.yylloc()} )}
            else {Some(TokFormatSubEnd {location: lexer.yylloc()} )}
        }
        FORMAT_SUP => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            lexer.format_sup_open = !lexer.format_sup_open;
            if lexer.format_sup_open {Some(TokFormatSupStart {location: lexer.yylloc()} )}
            else {Some(TokFormatSupEnd {location: lexer.yylloc()} )}
        }
        FORMAT_MONO_START => |lexer:&mut TweeLexer<R>| {
            lexer.MONO_TEXT();
            Some(TokFormatMonoStart {location: lexer.yylloc()} )
        }

        HTML_DOCTYPE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
        HTML_START => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.HTML();
            None
        }

        COMMENT =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }

        NEWLINE =>  |lexer:&mut TweeLexer<R>| {
            lexer.NEWLINE();
            Some(TokNewLine {location: lexer.yylloc()} )
        }
    }

    HTML {
        HTML_END => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }
        HTML_TEXT =>  |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()} )
        HTML_TAG => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        NEWLINE => |lexer:&mut TweeLexer<R>| Some(TokNewLine {location: lexer.yylloc()} )
    }

    I_EXPRESSION {
        VARIABLE => |lexer:&mut TweeLexer<R>| Some(TokVariable{location: lexer.yylloc(), name: lexer.yystr()} )
        STRING =>   |lexer:&mut TweeLexer<R>| Some(TokString  {location: lexer.yylloc(), value: unescape(lexer.yystr())} )
        FLOAT =>    |lexer:&mut TweeLexer<R>| Some(TokFloat   {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        INT =>      |lexer:&mut TweeLexer<R>| Some(TokInt     {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        BOOL =>     |lexer:&mut TweeLexer<R>| Some(TokBoolean {location: lexer.yylloc(), value: lexer.yystr()} )
        NUM_OP =>   |lexer:&mut TweeLexer<R>| Some(TokNumOp   {location: lexer.yylloc(), op_name: lexer.yystr()} )
        COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp  {location: lexer.yylloc(), op_name: lexer.yystr()} )
        LOG_OP =>   |lexer:&mut TweeLexer<R>| Some(TokLogOp   {location: lexer.yylloc(), op_name: lexer.yystr()} )
    }

    I_EXPRESSION_SIMPLE {
        PAREN_OPEN => |lexer:&mut TweeLexer<R>| Some(TokParenOpen {location: lexer.yylloc()} )
        PAREN_CLOSE =>|lexer:&mut TweeLexer<R>| Some(TokParenClose{location: lexer.yylloc()} )
        SEMI_COLON => |lexer:&mut TweeLexer<R>| Some(TokSemiColon {location: lexer.yylloc()} )
        ASSIGN =>     |lexer:&mut TweeLexer<R>| Some(TokAssign    {location: lexer.yylloc(), var_name: "".to_string(), op_name: lexer.yystr()} )
    }

    NEWLINE {
        :I_PASSAGE_CONTENT
        PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.PASSAGE();
            None
        }

        FORMAT_NUMB_LIST =>  |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatNumbList {location: lexer.yylloc()} )
        }
        FORMAT_HEADING  =>  |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatHeading {location: lexer.yylloc(), rank: lexer.yystr().trim().len()} )
        }

        TEXT_CHAR_START => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokText {location: lexer.yylloc(), text: lexer.yystr()} )
        }

        FORMAT_HORIZONTAL_LINE  =>  |lexer:&mut TweeLexer<R>| Some(TokFormatHorizontalLine {location: lexer.yylloc()} )
        FORMAT_INDENT_BLOCK  =>  |lexer:&mut TweeLexer<R>| Some(TokFormatIndentBlock {location: lexer.yylloc()} )
    }

    NON_NEWLINE {
        :I_PASSAGE_CONTENT
        TEXT =>  |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()} )
    }

    PASSAGE {
        PASSAGENAME => |lexer:&mut TweeLexer<R>| Some(TokPassage {name: lexer.yystr().trim().to_string(), location: lexer.yylloc()} )
        TAG_START => |lexer:&mut TweeLexer<R>| {
            lexer.TAGS();
            Some(TokTagStart {location: lexer.yylloc()})
        }
        NEWLINE => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.NEWLINE();
            None
        }
    }

    TAGS {
        :I_IGNORE_WHITESPACE

        TAG =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            match lexer.yystr().trim().as_ref() {
                "script" => {
                    lexer.ignore_passage = true;
                    Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr().trim().to_string()} )
                },
                "stylesheet" => {
                    lexer.ignore_passage = true;
                    Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr().trim().to_string()} )
                },
                _ => {
                    Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr().trim().to_string()} )
                }
            }
        }

        TAG_END => |lexer:&mut TweeLexer<R>| {
            if !lexer.ignore_passage { lexer.PASSAGE(); } else { lexer.INITIAL(); }
            Some(TokTagEnd {location: lexer.yylloc()})
        }
    }

    MONO_TEXT {
        MONOSPACE_TEXT =>  |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()} )
        FORMAT_MONO_END => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatMonoEnd {location: lexer.yylloc()} )
        }

        NEWLINE =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            Some(TokText {location: lexer.yylloc(), text: " ".to_string()} )
        }
    }

    MACRO {
        WHITESPACE =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }

        MACRONAME =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            let replaced_string = str::replace(lexer.yystr().trim(),  " ", "");

            match replaced_string.as_ref() {
                "set" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroSet {location: lexer.yylloc()} )
                },
                "if" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroIf {location: lexer.yylloc()} )
                },
                "else" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroElse {location: lexer.yylloc()} )
                },
                "elseif" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroElseIf {location: lexer.yylloc()} )
                },
                "endif" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroEndIf {location: lexer.yylloc()} )
                },
                "print" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroPrint {location: lexer.yylloc()} )
                },
                "display" => {
                    lexer.MACRO_CONTENT_DISPLAY();
                    None
                },
                "silently" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroSilently {location: lexer.yylloc()} )
                },
                "endsilently" => {
                    lexer.MACRO_CONTENT();
                    Some(TokMacroEndSilently {location: lexer.yylloc()} )
                },
                _ => {
                    lexer.MACRO_CONTENT_SHORT_DISPLAY();
                    Some(TokMacroDisplay {location: lexer.yylloc(), passage_name: replaced_string.to_string()} )
                }
            }
        }

        VARIABLE =>  |lexer:&mut TweeLexer<R>| {
            lexer.MACRO_CONTENT_SHORT_PRINT();
            Some(TokMacroContentVar {location: lexer.yylloc(), var_name: lexer.yystr()} )
        }
    }

    MACRO_CONTENT {
        :I_EXPRESSION
        :I_EXPRESSION_SIMPLE
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE
        :I_EMIT_COLON

        // Expression Stuff
        FUNCTION =>  |lexer:&mut TweeLexer<R>| {
            let s =  lexer.yystr();
            let trimmed = &s[0 .. s.len()-1];
            let name = &trimmed.to_string();
            lexer.function_parens = 1;
            lexer.FUNCTION_ARGS();
            Some(TokFunction {location: lexer.yylloc(), name: name.clone()} )
        }
        // Expression Stuff End

        MACRO_END => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
    }

    MACRO_CONTENT_DISPLAY {
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE

        MACRO_DISPLAY_PASSAGENAME  => |lexer:&mut TweeLexer<R>| {
            Some(TokMacroDisplay {location: lexer.yylloc(), passage_name: lexer.yystr().trim().to_string()} )
        }

        STRING => |lexer:&mut TweeLexer<R>| {
            Some(TokMacroDisplay {passage_name: unescape(lexer.yystr()), location: lexer.yylloc()})
        }

        MACRO_END => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
    }

    MACRO_CONTENT_SHORT_PRINT {
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE

        MACRO_END => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
    }

    MACRO_CONTENT_SHORT_DISPLAY {
        :I_IGNORE_NEWLINE
        :I_IGNORE_WHITESPACE

        // Expression Stuff
        FUNCTION =>   |_:&mut TweeLexer<R>| -> Option<Token> { None }
        STRING =>     |_:&mut TweeLexer<R>| -> Option<Token> { None }
        VARIABLE =>   |_:&mut TweeLexer<R>| -> Option<Token> { None }
        FLOAT =>      |_:&mut TweeLexer<R>| -> Option<Token> { None }
        INT =>        |_:&mut TweeLexer<R>| -> Option<Token> { None }
        BOOL =>       |_:&mut TweeLexer<R>| -> Option<Token> { None }
        NUM_OP =>     |_:&mut TweeLexer<R>| -> Option<Token> { None }
        COMP_OP =>    |_:&mut TweeLexer<R>| -> Option<Token> { None }
        LOG_OP =>     |_:&mut TweeLexer<R>| -> Option<Token> { None }
        PAREN_OPEN => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        PAREN_CLOSE =>|_:&mut TweeLexer<R>| -> Option<Token> { None }
        SEMI_COLON => |_:&mut TweeLexer<R>| -> Option<Token> { None }
        ASSIGN =>     |_:&mut TweeLexer<R>| -> Option<Token> { None }
        COLON =>      |_:&mut TweeLexer<R>| -> Option<Token> { None }
        // Expression Stuff End

        MACRO_END => |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }
    }

    FUNCTION_ARGS {
        :I_EXPRESSION
        :I_IGNORE_WHITESPACE
        :I_EMIT_COLON

        PAREN_OPEN =>  |lexer:&mut TweeLexer<R>| {
            lexer.function_parens += 1;
            Some(TokParenOpen {location: lexer.yylloc()} )
        }
        PAREN_CLOSE =>  |lexer:&mut TweeLexer<R>| {
            lexer.function_parens -= 1;
            if lexer.function_parens == 0 {
                lexer.MACRO_CONTENT();
                Some(TokArgsEnd {location: lexer.yylloc()} )
            } else {
                Some(TokParenClose {location: lexer.yylloc()} )
            }
        }
    }

    LINK_VAR_CHECK {
        LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }

        LINK_OPEN => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.LINK_VAR_SET();
            Some(TokVarSetStart {location: lexer.yylloc()} )
        }
    }

    LINK_VAR_SET {
        :I_EXPRESSION
        :I_EXPRESSION_SIMPLE

        LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.LINK_WAIT_CLOSE();
            Some(TokVarSetEnd {location: lexer.yylloc()} )
        }
    }

    LINK_WAIT_CLOSE {
        LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }
    }

}
