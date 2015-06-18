use std::io::{BufReader, Read};
use utils::extensions::{Peeking, PeekingExt, FilteringScan, FilteringScanExt};
use config::Config;

use self::Token::*;

pub struct ScanState {
    current_text: String,
    current_text_location: (u64, u64),
    skip_next: bool,
}

#[allow(unused_variables)]
pub fn lex<'a, R: Read>(cfg: &Config, input: &'a mut R) -> FilteringScan<Peeking<TweeLexer<BufReader<&'a mut R>>, Token>, ScanState, fn(&mut ScanState, (Token, Option<Token>)) -> Option<Token>>  {

    TweeLexer::new(BufReader::new(input)).peeking().scan_filter(
        ScanState {
            current_text: String::new(),
            current_text_location: (0, 0),
            skip_next: false,
        },
        {
            fn scan_fn(state: &mut ScanState, elem: (Token, Option<Token>)) -> Option<Token> {
                if state.skip_next {
                    state.skip_next = false;
                    return None;
                }

                match elem {
                    (TokText {location, text}, Some(TokText{ .. })) => {
                        if state.current_text.len() == 0 {
                            state.current_text_location = location;
                        }

                        state.current_text.push_str(&text);
                        None
                    }
                    (TokText {location, text}, _) => {
                        if state.current_text.len() == 0 {
                            state.current_text_location = location;
                        }

                        state.current_text.push_str(&text);
                        let val = TokText {location: state.current_text_location, text: state.current_text.clone()};
                        state.current_text.clear();
                        Some(val)
                    },
                    (TokVariable {location, name: var}, Some(TokAssign {op_name: op, ..} )) => {
                        state.skip_next = true;
                        Some(TokAssign {location: location, var_name: var, op_name: op} )
                    },
                    (x, _) => Some(x),
                }
            }
            scan_fn
        }
    )
}

#[derive(Debug,Clone)]
pub enum Token {

    TokPassage                {location: (u64, u64), name: String},
    TokTagStart               {location: (u64, u64)},
    TokTagEnd                 {location: (u64, u64)},
    TokVarSetStart            {location: (u64, u64)},
    TokVarSetEnd              {location: (u64, u64)},
    TokPassageLink            {location: (u64, u64), display_name: String, passage_name: String},
    TokTag                    {location: (u64, u64), tag_name: String},
    TokText                   {location: (u64, u64), text: String},
    TokFormatBoldStart        {location: (u64, u64)}, TokFormatBoldEnd   {location: (u64, u64)},
    TokFormatItalicStart      {location: (u64, u64)}, TokFormatItalicEnd {location: (u64, u64)},
    TokFormatUnderStart       {location: (u64, u64)}, TokFormatUnderEnd  {location: (u64, u64)},
    TokFormatStrikeStart      {location: (u64, u64)}, TokFormatStrikeEnd {location: (u64, u64)},
    TokFormatSubStart         {location: (u64, u64)}, TokFormatSubEnd    {location: (u64, u64)},
    TokFormatSupStart         {location: (u64, u64)}, TokFormatSupEnd    {location: (u64, u64)},
    TokFormatMonoStart        {location: (u64, u64)}, TokFormatMonoEnd   {location: (u64, u64)},
    TokFormatBulList          {location: (u64, u64)},
    TokFormatNumbList         {location: (u64, u64)},
    TokFormatIndentBlock      {location: (u64, u64)},
    TokFormatHorizontalLine   {location: (u64, u64)},
    TokFormatHeading          {location: (u64, u64), rank: usize},
    TokMacroStart             {location: (u64, u64)},
    TokMacroEnd               {location: (u64, u64)},
    TokMacroContentVar        {location: (u64, u64), var_name: String},
    TokMacroContentPassageName{location: (u64, u64), passage_name: String},
    TokMacroSet               {location: (u64, u64)},
    TokMacroIf                {location: (u64, u64)},
    TokMacroElse              {location: (u64, u64)},
    TokMacroElseIf            {location: (u64, u64)},
    TokMacroEndIf             {location: (u64, u64)},
    TokMacroPrint             {location: (u64, u64)},
    TokMacroDisplay           {location: (u64, u64)},
    TokMacroSilently          {location: (u64, u64)},
    TokMacroEndSilently       {location: (u64, u64)},
    TokParenOpen              {location: (u64, u64)},
    TokParenClose             {location: (u64, u64)},
    TokVariable               {location: (u64, u64), name: String},
    TokInt                    {location: (u64, u64), value: i32},
    TokFloat                  {location: (u64, u64), value: f32},
    TokString                 {location: (u64, u64), value: String},
    TokBoolean                {location: (u64, u64), value: String},
    TokFunction               {location: (u64, u64), name: String},
    TokColon                  {location: (u64, u64)},
    TokArgsEnd                {location: (u64, u64)},
    TokArrayStart             {location: (u64, u64)},
    TokArrayEnd               {location: (u64, u64)},
    TokAssign                 {location: (u64, u64), var_name: String, op_name: String},
    TokNumOp                  {location: (u64, u64), op_name: String},
    TokCompOp                 {location: (u64, u64), op_name: String},
    TokLogOp                  {location: (u64, u64), op_name: String},
    TokSemiColon              {location: (u64, u64)},
    TokNewLine                {location: (u64, u64)},
    TokExpression,
}

impl Token {
    pub fn location(&self) -> (u64, u64) {
        match self {
            &TokPassage{location, ..} |
            &TokTagStart{location} |
            &TokTagEnd{location} |
            &TokVarSetStart{location} |
            &TokVarSetEnd{location} |
            &TokPassageLink{location, ..} |
            &TokTag{location, ..} |
            &TokText{location, ..} |
            &TokFormatBoldStart{location} |
            &TokFormatBoldEnd{location} |
            &TokFormatItalicStart{location} |
            &TokFormatItalicEnd{location} |
            &TokFormatUnderStart {location} |
            &TokFormatUnderEnd{location} |
            &TokFormatStrikeStart{location} |
            &TokFormatStrikeEnd{location} |
            &TokFormatSubStart{location} |
            &TokFormatSubEnd{location} |
            &TokFormatSupStart{location} |
            &TokFormatSupEnd{location} |
            &TokFormatMonoStart{location} |
            &TokFormatMonoEnd{location} |
            &TokFormatBulList{location} |
            &TokFormatNumbList{location} |
            &TokFormatIndentBlock{location} |
            &TokFormatHorizontalLine{location} |
            &TokFormatHeading{location, ..} |
            &TokMacroStart{location} |
            &TokMacroEnd{location} |
            &TokMacroContentVar{location, ..} |
            &TokMacroContentPassageName{location, ..} |
            &TokMacroSet{location} |
            &TokMacroIf{location} |
            &TokMacroElse{location} |
            &TokMacroElseIf{location} |
            &TokMacroEndIf{location} |
            &TokMacroPrint{location} |
            &TokMacroDisplay{location} |
            &TokMacroSilently{location} |
            &TokMacroEndSilently{location} |
            &TokParenOpen{location} |
            &TokParenClose{location} |
            &TokVariable{location, ..} |
            &TokInt{location, ..} |
            &TokFloat{location, ..} |
            &TokString{location, ..} |
            &TokBoolean{location, ..} |
            &TokFunction{location, ..} |
            &TokColon{location} |
            &TokArgsEnd{location} |
            &TokArrayStart{location} |
            &TokArrayEnd{location} |
            &TokAssign{location, ..} |
            &TokNumOp{location, ..} |
            &TokCompOp{location, ..} |
            &TokLogOp{location, ..} |
            &TokSemiColon{location} |
            &TokNewLine{location}
                => location,
            &TokExpression => (0, 0)
        }
    }
}

//we re-create tokens with location {0,0} in the parser.
//so comparing also works this way
impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        match (self, other) {
            (&TokPassage{..}, &TokPassage{..}) => true,
            (&TokTagStart{..}, &TokTagStart{..}) => true,
            (&TokTagEnd{..}, &TokTagEnd{..}) => true,
            (&TokVarSetStart{..}, &TokVarSetStart{..}) => true,
            (&TokVarSetEnd{..}, &TokVarSetEnd{..}) => true,
            (&TokPassageLink{..}, &TokPassageLink{..}) => true,
            (&TokTag{..}, &TokTag{..}) => true,
            (&TokText{..}, &TokText{..}) => true,
            (&TokFormatBoldStart{..}, &TokFormatBoldStart{..}) => true,
            (&TokFormatBoldEnd{..}, &TokFormatBoldEnd{..}) => true,
            (&TokFormatItalicStart{..}, &TokFormatItalicStart{..}) => true,
            (&TokFormatItalicEnd{..}, &TokFormatItalicEnd{..}) => true,
            (&TokFormatUnderStart {..}, &TokFormatUnderStart {..}) => true,
            (&TokFormatUnderEnd{..}, &TokFormatUnderEnd{..}) => true,
            (&TokFormatStrikeStart{..}, &TokFormatStrikeStart{..}) => true,
            (&TokFormatStrikeEnd{..}, &TokFormatStrikeEnd{..}) => true,
            (&TokFormatSubStart{..}, &TokFormatSubStart{..}) => true,
            (&TokFormatSubEnd{..}, &TokFormatSubEnd{..}) => true,
            (&TokFormatSupStart{..}, &TokFormatSupStart{..}) => true,
            (&TokFormatSupEnd{..}, &TokFormatSupEnd{..}) => true,
            (&TokFormatMonoStart{..}, &TokFormatMonoStart{..}) => true,
            (&TokFormatMonoEnd{..}, &TokFormatMonoEnd{..}) => true,
            (&TokFormatBulList{..}, &TokFormatBulList{..}) => true,
            (&TokFormatNumbList{..}, &TokFormatNumbList{..}) => true,
            (&TokFormatIndentBlock{..}, &TokFormatIndentBlock{..}) => true,
            (&TokFormatHorizontalLine{..}, &TokFormatHorizontalLine{..}) => true,
            (&TokFormatHeading{..}, &TokFormatHeading{..}) => true,
            (&TokMacroStart{..}, &TokMacroStart{..}) => true,
            (&TokMacroEnd{..}, &TokMacroEnd{..}) => true,
            (&TokMacroContentVar{..}, &TokMacroContentVar{..}) => true,
            (&TokMacroContentPassageName{..}, &TokMacroContentPassageName{..}) => true,
            (&TokMacroSet{..}, &TokMacroSet{..}) => true,
            (&TokMacroIf{..}, &TokMacroIf{..}) => true,
            (&TokMacroElse{..}, &TokMacroElse{..}) => true,
            (&TokMacroElseIf{..}, &TokMacroElseIf{..}) => true,
            (&TokMacroEndIf{..}, &TokMacroEndIf{..}) => true,
            (&TokMacroPrint{..}, &TokMacroPrint{..}) => true,
            (&TokMacroDisplay{..}, &TokMacroDisplay{..}) => true,
            (&TokMacroSilently{..}, &TokMacroSilently{..}) => true,
            (&TokMacroEndSilently{..}, &TokMacroEndSilently{..}) => true,
            (&TokParenOpen{..}, &TokParenOpen{..}) => true,
            (&TokParenClose{..}, &TokParenClose{..}) => true,
            (&TokVariable{..}, &TokVariable{..}) => true,
            (&TokInt{..}, &TokInt{..}) => true,
            (&TokFloat{..}, &TokFloat{..}) => true,
            (&TokString{..}, &TokString{..}) => true,
            (&TokBoolean{..}, &TokBoolean{..}) => true,
            (&TokFunction{..}, &TokFunction{..}) => true,
            (&TokColon{..}, &TokColon{..}) => true,
            (&TokArgsEnd{..}, &TokArgsEnd{..}) => true,
            (&TokArrayStart{..}, &TokArrayStart{..}) => true,
            (&TokArrayEnd{..}, &TokArrayEnd{..}) => true,
            (&TokAssign{..}, &TokAssign{..}) => true,
            (&TokNumOp{..}, &TokNumOp{..}) => true,
            (&TokCompOp{..}, &TokCompOp{..}) => true,
            (&TokLogOp{..}, &TokLogOp{..}) => true,
            (&TokSemiColon{..}, &TokSemiColon{..}) => true,
            (&TokNewLine{..}, &TokNewLine{..}) => true,
            _ => false,
        }
    }
}

rustlex! TweeLexer {

    // Properties
    property new_line:bool = true;
    property format_bold_open:bool = false;
    property format_italic_open:bool = false;
    property format_under_open:bool = false;
    property format_strike_open:bool = false;
    property format_sub_open:bool = false;
    property format_sup_open:bool = false;
    property function_parens:usize = 0;

    // Regular Expressions
    let WHITESPACE = ' ' | '\t';
    let UNDERSCORE = '_';
    let NEWLINE = '\n';

    let INITIAL_START_CHAR = [^": "'\n''\t'] | ':' [^": "'\n''\t'];
    let INITIAL_CHAR = [^" "'\n''\t'];
    let TEXT_INITIAL = INITIAL_START_CHAR INITIAL_CHAR*;

    // If for example // is at a beginning of a line, then // is matched and not just /
    let TEXT_START_CHAR = "ä"|"Ä"|"ü"|"Ü"|"ö"|"Ö"|"ß"|"ẞ" | [^"*!>#"'\n']; // add chars longer than one byte
    let TEXT_CHAR = [^"/'_=~^{@<[" '\n'];
    let TEXT = TEXT_CHAR+ | ["/'_=^{@<["];

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

    let INT = "-"? DIGIT+;
    let FLOAT = "-"? (DIGIT+ "." DIGIT*) | "-"? (DIGIT* "." DIGIT+) | "-"? "Infinity";

    let STRING = '"' ([^'\\''"']|'\\'.)* '"' | "'" ([^'\\'"'"]|'\\'.)* "'";

    let BOOL = "true" | "false";

    let COLON = ',';

    let FUNCTION_NAME = (LETTER | UNDERSCORE) VAR_CHAR*;
    let FUNCTION = FUNCTION_NAME '(';

    let MACRO_NAME = [^" >"'\n']* ( WHITESPACE+ "if")?;

    let ASSIGN = "=" | "to" | "+=" | "-=" | "*=" | "/=";
    let SEMI_COLON = ';';
    let NUM_OP = ["+-*/%"];
    let COMP_OP = "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
    let LOG_OP = "and" | "or" | "not";


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

    NEWLINE {
        PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.PASSAGE();
            None
        }

        MACRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.MACRO();
            lexer.new_line = true;
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

        COMMENT =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
            lexer.NON_NEWLINE();
            None
        }

        NEWLINE => |lexer:&mut TweeLexer<R>| Some(TokNewLine {location: lexer.yylloc()} )
    }

    NON_NEWLINE {

        MACRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
            lexer.MACRO();
            lexer.new_line = false;
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
            lexer.format_italic_open = !lexer.format_italic_open;
            if lexer.format_italic_open {Some(TokFormatItalicStart {location: lexer.yylloc()} )}
            else {Some(TokFormatItalicEnd {location: lexer.yylloc()} )}
        }
        FORMAT_BOLD => |lexer:&mut TweeLexer<R>| {
            lexer.format_bold_open = !lexer.format_bold_open;
            if lexer.format_bold_open {Some(TokFormatBoldStart {location: lexer.yylloc()} )}
            else {Some(TokFormatBoldEnd {location: lexer.yylloc()} )}
        }
        FORMAT_UNDER => |lexer:&mut TweeLexer<R>| {
            lexer.format_under_open = !lexer.format_under_open;
            if lexer.format_under_open {Some(TokFormatUnderStart {location: lexer.yylloc()} )}
            else {Some(TokFormatUnderEnd {location: lexer.yylloc()} )}
        }
        FORMAT_STRIKE => |lexer:&mut TweeLexer<R>| {
            lexer.format_strike_open = !lexer.format_strike_open;
            if lexer.format_strike_open {Some(TokFormatStrikeStart {location: lexer.yylloc()} )}
            else {Some(TokFormatStrikeEnd {location: lexer.yylloc()} )}
        }
        FORMAT_SUB => |lexer:&mut TweeLexer<R>| {
            lexer.format_sub_open = !lexer.format_sub_open;
            if lexer.format_sub_open {Some(TokFormatSubStart {location: lexer.yylloc()} )}
            else {Some(TokFormatSubEnd {location: lexer.yylloc()} )}
        }
        FORMAT_SUP => |lexer:&mut TweeLexer<R>| {
            lexer.format_sup_open = !lexer.format_sup_open;
            if lexer.format_sup_open {Some(TokFormatSupStart {location: lexer.yylloc()} )}
            else {Some(TokFormatSupEnd {location: lexer.yylloc()} )}
        }
        FORMAT_MONO_START => |lexer:&mut TweeLexer<R>| {
            lexer.MONO_TEXT();
            Some(TokFormatMonoStart {location: lexer.yylloc()} )
        }

        NEWLINE =>  |lexer:&mut TweeLexer<R>| {
            lexer.NEWLINE();
            Some(TokNewLine {location: lexer.yylloc()} )
        }

        COMMENT =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }

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
        TAG => |lexer:&mut TweeLexer<R>| Some(TokTag {location: lexer.yylloc(), tag_name: lexer.yystr()} )
        WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
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
                    lexer.DISPLAY_CONTENT();
                    Some(TokMacroDisplay {location: lexer.yylloc()} )
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
                    lexer.MACRO_CONTENT();
                    Some(TokMacroContentPassageName {location: lexer.yylloc(), passage_name: replaced_string.to_string()} )
                }
            }
        }

        VAR_NAME =>  |lexer:&mut TweeLexer<R>| {
            lexer.MACRO_CONTENT();
            Some(TokMacroContentVar {location: lexer.yylloc(), var_name: lexer.yystr()} )
        }
    }



    MACRO_CONTENT {
        MACRO_END => |lexer:&mut TweeLexer<R>| {
            if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }


        FUNCTION =>  |lexer:&mut TweeLexer<R>| {
            let s =  lexer.yystr();
            let trimmed = &s[0 .. s.len()-1];
            let name = &trimmed.to_string();
            lexer.function_parens = 1;
            lexer.FUNCTION_ARGS();
            Some(TokFunction {location: lexer.yylloc(), name: name.clone()} )
        }

        // Expression Stuff
        STRING => |lexer:&mut TweeLexer<R>| {
            let s = lexer.yystr();
            // strip the quotes from strings
            let trimmed = &s[1 .. s.len() - 1];

            // unescape quotes
            let quote_type = s.chars().next().unwrap();
            let mut unescaped = String::new();

            for (c, peek) in trimmed.chars().peeking() {
                if let Some(nextc) = peek {
                    if c == '\\' && nextc == quote_type {
                        continue;
                    }
                }

                unescaped.push(c);
            }

            Some(TokString {location: lexer.yylloc(), value: unescaped} )
        }

        VAR_NAME =>   |lexer:&mut TweeLexer<R>| Some(TokVariable  {location: lexer.yylloc(), name: lexer.yystr()} )
        FLOAT =>      |lexer:&mut TweeLexer<R>| Some(TokFloat     {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        INT =>        |lexer:&mut TweeLexer<R>| Some(TokInt       {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        BOOL =>       |lexer:&mut TweeLexer<R>| Some(TokBoolean   {location: lexer.yylloc(), value: lexer.yystr()} )
        NUM_OP =>     |lexer:&mut TweeLexer<R>| Some(TokNumOp     {location: lexer.yylloc(), op_name: lexer.yystr()} )
        COMP_OP =>    |lexer:&mut TweeLexer<R>| Some(TokCompOp    {location: lexer.yylloc(), op_name: lexer.yystr()} )
        LOG_OP =>     |lexer:&mut TweeLexer<R>| Some(TokLogOp     {location: lexer.yylloc(), op_name: lexer.yystr()} )
        PAREN_OPEN => |lexer:&mut TweeLexer<R>| Some(TokParenOpen {location: lexer.yylloc()} )
        PAREN_CLOSE =>|lexer:&mut TweeLexer<R>| Some(TokParenClose{location: lexer.yylloc()} )
        SEMI_COLON => |lexer:&mut TweeLexer<R>| Some(TokSemiColon {location: lexer.yylloc()} )
        ASSIGN =>     |lexer:&mut TweeLexer<R>| Some(TokAssign    {location: lexer.yylloc(), var_name: "".to_string(), op_name: lexer.yystr()} )
        COLON =>      |lexer:&mut TweeLexer<R>| Some(TokColon     {location: lexer.yylloc()} )
        // Expression Stuff End

        WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> {
            None
        }
    }

    FUNCTION_ARGS {
        WHITESPACE =>   |_:&mut TweeLexer<R>| -> Option<Token> {None}
        COLON =>    |lexer:&mut TweeLexer<R>| Some(TokColon   {location: lexer.yylloc()} )
        VAR_NAME => |lexer:&mut TweeLexer<R>| Some(TokVariable{location: lexer.yylloc(), name: lexer.yystr()} )
        FLOAT =>    |lexer:&mut TweeLexer<R>| Some(TokFloat   {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        INT =>      |lexer:&mut TweeLexer<R>| Some(TokInt     {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        STRING =>   |lexer:&mut TweeLexer<R>| Some(TokString  {location: lexer.yylloc(), value: lexer.yystr()} )
        BOOL =>     |lexer:&mut TweeLexer<R>| Some(TokBoolean {location: lexer.yylloc(), value: lexer.yystr()} )
        NUM_OP =>   |lexer:&mut TweeLexer<R>| Some(TokNumOp   {location: lexer.yylloc(), op_name: lexer.yystr()} )
        COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp  {location: lexer.yylloc(), op_name: lexer.yystr()} )
        LOG_OP =>   |lexer:&mut TweeLexer<R>| Some(TokLogOp   {location: lexer.yylloc(), op_name: lexer.yystr()} )
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

    DISPLAY_CONTENT {
        MACRO_END => |lexer:&mut TweeLexer<R>| {
            if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
            Some(TokMacroEnd {location: lexer.yylloc()} )
        }

        VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable {location: lexer.yylloc(), name: lexer.yystr()} )
        PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassage {name: lexer.yystr().trim().to_string(), location: lexer.yylloc()} )
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
        // Expression Stuff
        VAR_NAME =>   |lexer:&mut TweeLexer<R>| Some(TokVariable  {location: lexer.yylloc(), name: lexer.yystr()} )
        FLOAT =>      |lexer:&mut TweeLexer<R>| Some(TokFloat     {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        INT =>        |lexer:&mut TweeLexer<R>| Some(TokInt       {location: lexer.yylloc(), value: lexer.yystr()[..].parse().unwrap()} )
        STRING =>     |lexer:&mut TweeLexer<R>| Some(TokString    {location: lexer.yylloc(), value: lexer.yystr()} )
        BOOL =>       |lexer:&mut TweeLexer<R>| Some(TokBoolean   {location: lexer.yylloc(), value: lexer.yystr()} )
        NUM_OP =>     |lexer:&mut TweeLexer<R>| Some(TokNumOp     {location: lexer.yylloc(), op_name: lexer.yystr()} )
        COMP_OP =>    |lexer:&mut TweeLexer<R>| Some(TokCompOp    {location: lexer.yylloc(), op_name: lexer.yystr()} )
        LOG_OP =>     |lexer:&mut TweeLexer<R>| Some(TokLogOp     {location: lexer.yylloc(), op_name: lexer.yystr()} )
        PAREN_OPEN => |lexer:&mut TweeLexer<R>| Some(TokParenOpen {location: lexer.yylloc()} )
        PAREN_CLOSE =>|lexer:&mut TweeLexer<R>| Some(TokParenClose{location: lexer.yylloc()} )
        SEMI_COLON => |lexer:&mut TweeLexer<R>| Some(TokSemiColon {location: lexer.yylloc()} )
        ASSIGN =>     |lexer:&mut TweeLexer<R>| Some(TokAssign    {location: lexer.yylloc(), var_name: "".to_string(), op_name: lexer.yystr()} )
        // Expression Stuff End

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


// ================================
// test functions
#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::fmt::Write;
    use config;

    use super::*;
    use super::Token::*;

    fn test_lex(input: &str) -> Vec<Token> {
        let cfg = config::default_config();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
        lex(&cfg, &mut cursor).collect()
    }

    fn assert_tok_eq(expected: Vec<Token>, tokens: Vec<Token>) {
        let mut panic_msg = String::new();
        if tokens.len() != expected.len() {
            write!(&mut panic_msg, "Got {} tokens, expected {}\n\n", tokens.len(), expected.len()).unwrap();
        }

        for i in 0..tokens.len() {
            if tokens[i] != expected[i] {
                write!(&mut panic_msg, "Unexpected token #{}:\n'{:?}', expected\n'{:?}'\n\n", i, tokens[i], expected[i]).unwrap();
            }
        }

        if panic_msg.len() != 0 {
            panic!(panic_msg);
        }
    }

    #[test]
    fn preprocessing_test() {
        // This should remove all comments
        let tokens = test_lex("/%\nVortest1\n%/\n   /% Vortest2 %/\n        bla\nLorem Ipsum doloris!\n\n::Start /% Test %/\nText1, der bleiben muss\n/% Test1 %/\nText2, der bleiben muss\n/% /% Test2 %/\nText3, der bleiben muss\n/%% Test3 %/\nText4, der bleiben muss\n/% Test4 %%/ ! TestHeading1\nText5, der bleiben muss\n/% /% Test5 %/ %/\nText6, der bleiben muss\n/%%%%%%\nTest6\n%%%%%%%/\nText7, der bleiben muss /%\n/%\nTest7\n%/ ! TestHeading2\n/%%/\nText Text /% Test8 %/ Text Text\n{{{ Monospace /% Kommentar %/ }}}\n<<if true>> Bla /% Test10 %/ <<endif>>");
        let expected = vec!(
            TokPassage {name: "Start /% Test %/".to_string(), location: (8, 3)},
            TokText { text: "Text1, der bleiben muss".to_string(), location: (9, 1) },
            TokNewLine { location: (9, 24) },
            TokNewLine { location: (10, 12) },
            TokText { text: "Text2, der bleiben muss".to_string(), location: (11, 1) },
            TokNewLine { location: (11, 24) },
            TokNewLine { location: (12, 15) },
            TokText { text: "Text3, der bleiben muss".to_string(), location: (13, 1) },
            TokNewLine { location: (13, 24) },
            TokNewLine { location: (14, 13) },
            TokText { text: "Text4, der bleiben muss".to_string(), location: (15, 1) },
            TokNewLine { location: (15, 24) },
            TokText { text: " ! TestHeading1".to_string(), location: (16, 13) },
            TokNewLine { location: (16, 28) },
            TokText { text: "Text5, der bleiben muss".to_string(), location: (17, 1) },
            TokNewLine { location: (17, 24) },
            TokText { text: " %/".to_string(), location: (18, 15) },
            TokNewLine { location: (18, 18) },
            TokText { text: "Text6, der bleiben muss".to_string(), location: (19, 1) },
            TokNewLine { location: (19, 24) },
            TokNewLine { location: (22, 9) },
            TokText { text: "Text7, der bleiben muss  ! TestHeading2".to_string(), location: (23, 1) },
            TokNewLine { location: (26, 18) },
            TokNewLine { location: (27, 5) },
            TokText { text: "Text Text  Text Text".to_string(), location: (28, 1) },
            TokNewLine { location: (28, 32) },
            TokFormatMonoStart { location: (29, 1) },
            TokText { text: " Monospace /% Kommentar %/ ".to_string(), location: (29, 4) },
            TokFormatMonoEnd { location: (29, 31) },
            TokNewLine { location: (29, 34) },
            TokMacroIf { location: (30, 3) },
            TokBoolean { value: "true".to_string(), location: (30, 6) },
            TokMacroEnd { location: (30, 10) },
            TokText { text: " Bla  ".to_string(), location: (30, 12) },
            TokMacroEndIf { location: (30, 32) },
            TokMacroEnd { location: (30, 37) }
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn passage_test() {
        // This should detect the ::Start passage
        let start_tokens = test_lex("::Start");
        let expected = vec!(
            TokPassage {name: "Start".to_string(), location: (1, 3)}
        );

        assert_tok_eq(expected, start_tokens);

        // This should not return any tokens
        let fail_tokens = test_lex(":fail");
        assert_eq!(0, fail_tokens.len());
    }

    #[test]
    fn text_test() {
        // This should return a passage with a body text
        let tokens = test_lex("::Passage\nTestText\nTestNextLine\n::NextPassage");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokText {location: (2, 1), text: "TestText".to_string()},
            TokNewLine {location: (2, 9)},
            TokText {location: (3, 1), text: "TestNextLine".to_string()},
            TokNewLine {location: (3, 13)} ,
            TokPassage {name: "NextPassage".to_string(), location: (4, 3)},
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn tag_test() {
        // This should return a passage with tags
        let tokens = test_lex("::TagPassage [tag1 tag2]\nContent");
        let expected = vec!(
            TokPassage {name: "TagPassage".to_string(), location: (1, 3)},
            TokTagStart {location: (1, 14)},
            TokTag {location: (1, 15), tag_name: "tag1".to_string()},
            TokTag {location: (1, 20), tag_name: "tag2".to_string()},
            TokTagEnd {location: (1, 24)},
            TokText {location: (2, 1), text: "Content".to_string()}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_set_test() {
        // This should return a passage with a set macro
        let tokens = test_lex("::Passage\n<<set $var = 1>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroSet {location: (2, 3)},
            TokAssign {location: (2, 7), var_name: "$var".to_string(), op_name: "=".to_string()},
            TokInt {location: (2, 14), value: 1},
            TokMacroEnd {location: (2, 15)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_if_test() {
        // This should return a passage with an if macro
        let tokens = test_lex("::Passage\n<<if $var1 == 1>>1<<else if $var2 is 2>>2<<else>>3<<endif>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroIf {location: (2, 3)},
            TokVariable {location: (2, 6), name: "$var1".to_string()},
            TokCompOp {location: (2, 12), op_name: "==".to_string()},
            TokInt {location: (2, 15), value: 1},
            TokMacroEnd {location: (2, 16)},
            TokText {text: "1".to_string(), location: (2, 18) },
            TokMacroElseIf {location: (2, 21)},
            TokVariable {name: "$var2".to_string(), location: (2, 29)},
            TokCompOp {op_name: "is".to_string(), location: (2, 35)},
            TokInt {location: (2, 38), value: 2},
            TokMacroEnd {location: (2, 39)},
            TokText {text: "2".to_string(), location: (2, 41)},
            TokMacroElse {location: (2, 44)},
            TokMacroEnd {location: (2, 48)},
            TokText {text: "3".to_string(), location: (2, 50)},
            TokMacroEndIf {location: (2, 53)},
            TokMacroEnd {location: (2, 58)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_print_test() {
        let tokens = test_lex("::Passage\n<<print \"Test with escaped \\\"Quotes\">>\n<<print $var>>");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroPrint {location: (2, 3)},
            TokString {location: (2, 9), value: "Test with escaped \"Quotes".to_string()},
            TokMacroEnd {location: (2, 37)},
            TokNewLine {location: (2, 39)},
            TokMacroPrint {location: (3, 3)},
            TokVariable {location: (3, 9), name: "$var".to_string()},
            TokMacroEnd {location: (3, 13)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_display_test() {
        let tokens = test_lex("::Passage\n<<display DisplayedPassage>>\n::DisplayedPassage");
        let expected = vec!(
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroDisplay {location: (2, 3)},
            TokPassage {name: "DisplayedPassage".to_string(), location: (2, 10)},
            TokMacroEnd {location: (2, 27)},
            TokNewLine {location: (2, 29)},
            TokPassage {name: "DisplayedPassage".to_string(), location: (3, 3)}
        );

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_display_short_test() {
        // Should fail because it contains an invalid macro
        let tokens = test_lex("::Passage\n<<Passage>>");
        let expected = vec![
            TokPassage {name: "Passage".to_string(), location: (1, 3)},
            TokMacroContentPassageName {location: (2, 3), passage_name: "Passage".to_string()},
            TokMacroEnd {location: (2, 10)}
        ];

        assert_tok_eq(expected, tokens);
    }

    #[test]
    fn macro_print_function_test() {
        let tokens = test_lex("::Start\n<<print random(1, 100)>>");
        let expected = vec![
            TokPassage {name: "Start".to_string(), location: (1, 3)},
            TokMacroPrint {location: (2, 3)},
            TokFunction {name: "random".to_string(), location: (2, 9)},
            TokInt {value: 1, location: (2, 16)},
            TokColon {location: (2, 17)},
            TokInt {value: 100, location: (2, 19)},
            TokArgsEnd {location: (2, 22)},
            TokMacroEnd {location: (2, 23)}
        ];

        assert_tok_eq(expected, tokens);
    }
}
