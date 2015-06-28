rustlex! TweeLexer {
    // Properties
    property format_bold_open:bool = false;
    property format_italic_open:bool = false;
    property format_under_open:bool = false;
    property format_strike_open:bool = false;
    property format_sub_open:bool = false;
    property format_sup_open:bool = false;
    property function_parens:usize = 0;

    // Regular Expressions
    let ANYTHING = .*;
    let WHITESPACE = ' ' | '\t';
    let UNDERSCORE = '_';
    let NEWLINE = '\n';

    let INITIAL_START_CHAR = [^": "'\n''\t'] | ':' [^": "'\n''\t'];
    let INITIAL_CHAR = [^" "'\n''\t'];
    let TEXT_INITIAL = INITIAL_START_CHAR INITIAL_CHAR*;

    // If for example // is at a beginning of a line, then // is matched and not just /
    let TEXT_START_CHAR = [^"*!>#"'\n'];
    let TEXT_CHAR = [^"/'_=~^{@<[" '\n'];
    let TEXT = TEXT_CHAR+ | ["/'_=~^{@<["];

    let TEXT_MONO_CHAR = [^"}"'\n'];
    let TEXT_MONO = TEXT_MONO_CHAR+ | "}" | "}}";

    let PASSAGE_START = "::" ':'*;
    let PASSAGE_CHAR_NORMAL = [^"[]$<>:|" '\n'];
    let PASSAGE_CHAR = PASSAGE_CHAR_NORMAL | ':' PASSAGE_CHAR_NORMAL;
    let PASSAGE_NAME = PASSAGE_CHAR_NORMAL PASSAGE_CHAR* ':'?;

    let TAG = ['a'-'z''A'-'Z''0'-'9''.''_']+;
    let TAG_START = '[';
    let TAG_END = ']';

    let FORMAT_ITALIC = "//";
    let FORMAT_BOLD = "''";
    let FORMAT_UNDER = "__";
    let FORMAT_STRIKE = "==";
    let FORMAT_SUB = "~~";
    let FORMAT_SUP = "^^";
    let FORMAT_MONO_START = "{{{";
    let FORMAT_MONO_END = "}}}";

    //TODO ignore content
    let FORMAT_INLINE = "@@";

    let FORMAT_BUL_LIST = "*" WHITESPACE*;
    let FORMAT_NUMB_LIST = "#" WHITESPACE*;
    let FORMAT_INDENT_BLOCK = "<<<" NEWLINE;
    let FORMAT_HORIZONTAL_LINE = "----" NEWLINE;

    let FORMAT_HEADING = ("!" | "!!" | "!!!" | "!!!!" | "!!!!!") WHITESPACE*;

    let MACRO_START = "<<";
    let MACRO_END = ">>";

    let PAREN_OPEN = '(';
    let PAREN_CLOSE = ')';

    let DIGIT = ['0'-'9'];
    let LETTER = ['a'-'z''A'-'Z'];
    let VAR_CHAR = LETTER | DIGIT | UNDERSCORE;
    let VAR_NAME = '$' (LETTER | UNDERSCORE) VAR_CHAR*;

    let INT = /*"-"?*/ DIGIT+;
    let FLOAT = "-"? (DIGIT+ "." DIGIT*) | "-"? (DIGIT* "." DIGIT+) | "-"? "Infinity";

    let STRING = '"' ([^'\\''"']|'\\'.)* '"' | "'" ([^'\\'"'"]|'\\'.)* "'";

    let BOOL = "true" | "false";

    let COLON = ',';

    let FUNCTION_NAME = (LETTER | UNDERSCORE) VAR_CHAR*;
    let FUNCTION = FUNCTION_NAME '(';

    let MACRO_NAME = [^" >"'\n']* ( WHITESPACE+ "if")?;
    let MACRO_DISPLAY_PASSAGE_NAME = [^'"''>'' ''\t''\n'] ([^">"]*(">"[^">"])?)* [^'"''>'' ''\t''\n'] | [^"'>"' ''\t''\n'] ([^">"]*(">"[^">"])?)* [^"'>"' ''\t''\n'];

    let ASSIGN = "=" | "to" | "+=" | "-=" | "*=" | "/=";
    let SEMI_COLON = ';';
    let NUM_OP = ["+-*/%"];
    let COMP_OP = "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
    let LOG_OP = "and" | "&&" | "or" | "||" | "not" | "!";

    let LINK_OPEN = '[';
    let LINK_CLOSE = ']';
    let LINK_TEXT = [^'\n'"|[]"]+;

    let LINK_SIMPLE = "[[" (PASSAGE_NAME | VAR_NAME) "]";
    let LINK_LABELED = "[[" LINK_TEXT "|" (PASSAGE_NAME | VAR_NAME) "]";

    let COMMENT = "/%" ([^"%"]*(("%")*[^"%/"])?)* ("%")* "%/";

    INITIAL {
        PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.PASSAGE();
            None
        }

        TEXT_INITIAL =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }

        WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }

        NEWLINE => |_:&mut TweeLexer<R>| -> Option<Token> { None }

        COMMENT =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
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

        COMMENT =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }
        NEWLINE =>  |lexer:&mut TweeLexer<R>| {
            lexer.NEWLINE();
            Some(TokNewLine {location: lexer.yylloc()} )
        }
    }

    I_EXPRESSION {
        VAR_NAME => |lexer:&mut TweeLexer<R>| Some(TokVariable{location: lexer.yylloc(), name: lexer.yystr()} )
        STRING =>   |lexer:&mut TweeLexer<R>| Some(TokString  {location: lexer.yylloc(), value: unescape(lexer.yystr())} )
        FLOAT =>    |lexer:&mut TweeLexer<R>| Some(TokFloat   {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        INT =>        |lexer:&mut TweeLexer<R>| Some(TokInt       {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
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

        FORMAT_BUL_LIST =>  |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatBulList {location: lexer.yylloc()} )
        }
        FORMAT_NUMB_LIST =>  |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatNumbList {location: lexer.yylloc()} )
        }
        FORMAT_HEADING  =>  |lexer:&mut TweeLexer<R>| {
            lexer.NON_NEWLINE();
            Some(TokFormatHeading {location: lexer.yylloc(), rank: lexer.yystr().trim().len()} )
        }

        TEXT_START_CHAR => |lexer:&mut TweeLexer<R>| {
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
        PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassage {name: lexer.yystr().trim().to_string(), location: lexer.yylloc()} )
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
        TAG => |lexer:&mut TweeLexer<R>| Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr()} )
        TAG_END => |lexer:&mut TweeLexer<R>| {
            lexer.PASSAGE();
            Some(TokTagEnd {location: lexer.yylloc()})
        }
    }

    MONO_TEXT {
        TEXT_MONO =>  |lexer:&mut TweeLexer<R>| Some(TokText {location: lexer.yylloc(), text: lexer.yystr()} )
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

        MACRO_NAME =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
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

        VAR_NAME =>  |lexer:&mut TweeLexer<R>| {
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

        MACRO_DISPLAY_PASSAGE_NAME  => |lexer:&mut TweeLexer<R>| {
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
        VAR_NAME =>   |_:&mut TweeLexer<R>| -> Option<Token> { None }
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
