use std::io::{BufReader, Read};
use utils::extensions::{Peeking, PeekingExt, FilteringScan, FilteringScanExt};
use config::Config;

use self::Token::*;

pub struct ScanState {
    current_text: String,
    skip_next: bool,
}

pub fn lex<'a, R: Read>(cfg: &Config, input: &'a mut R) -> FilteringScan<Peeking<TweeLexer<BufReader<&'a mut R>>, Token>, ScanState, fn(&mut ScanState, (Token, Option<Token>)) -> Option<Token>>  {

    info!("Nicht in Tokens verarbeitete Zeichen: ");

	TweeLexer::new(BufReader::new(input)).peeking().scan_filter(
		ScanState {
           	current_text: String::new(),
            skip_next: false,
        },
		{
			fn scan_fn(state: &mut ScanState, elem: (Token, Option<Token>)) -> Option<Token> {
				if state.skip_next {
					state.skip_next = false;
					return None;
				}

				match elem {
					(TokText {text}, Some(TokText{ .. })) => {
						state.current_text.push_str(&text);
						None
					}
					(TokText {text}, _) => {
						state.current_text.push_str(&text);
						let val = TokText {text: state.current_text.clone()};
						state.current_text.clear();
						Some(val)
					},
					(TokVariable {name: var}, Some(TokAssign {op_name: op, ..} )) => {
						state.skip_next = true;
						Some(TokAssign {var_name: var, op_name: op} )
					},
					(x, _) => Some(x),
				}
			}
			scan_fn
		}
	)
}

#[derive(PartialEq,Debug,Clone)]
pub enum Token {
	TokPassage {name: String},
	TokTagStart,
	TokTagEnd,
	TokVarSetStart,
	TokVarSetEnd,
	TokPassageLink {display_name: String, passage_name: String},
	TokTag {tag_name: String},
	TokText {text: String},
	TokFormatBoldStart, TokFormatBoldEnd,
	TokFormatItalicStart, TokFormatItalicEnd,
	TokFormatUnderStart, TokFormatUnderEnd,
	TokFormatStrikeStart, TokFormatStrikeEnd,
	TokFormatSubStart, TokFormatSubEnd,
	TokFormatSupStart, TokFormatSupEnd,
	TokFormatMonoStart,	TokFormatMonoEnd,
	TokFormatBulList,
	TokFormatNumbList,
	TokFormatIndentBlock,
	TokFormatHorizontalLine,
	TokFormatHeading {rank: usize},
	TokMacroStart,
	TokMacroEnd,
	TokBracketOpen,
	TokBracketClose,
	TokVariable {name: String},
	TokInt {value: i32},
	TokFloat {value: f32},
	TokString {value: String},
	TokBoolean {value: String},
	TokFunction {name: String},
	TokColon,
	TokArgsEnd,
	TokArrayStart,
	TokArrayEnd,
	TokSet,
	TokAssign {var_name: String, op_name: String},
	TokNumOp {op_name: String},
	TokCompOp {op_name: String},
	TokLogOp {op_name: String},
	TokSemiColon,
	TokIf,
	TokElse,
	TokEndIf,
	TokPrint,
	TokDisplay,
	TokSilently,
	TokEndSilently,
	TokMacroVar {var_name: String},
	TokNewLine,
	TokPseudo,
	TokMacroPassageName {passage_name: String}
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
	property function_brackets:usize = 0;

	// Regular Expressions
	let WHITESPACE = ' ' | '\t';
	let UNDERSCORE = '_';
	let NEWLINE = '\n';

	let INITIAL_START_CHAR = [^":"'\n'] | ':' [^":"'\n'];
	let INITIAL_CHAR = [^'\n'];
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

	let BR_OPEN = '(';
	let BR_CLOSE = ')';

	let DIGIT = ['0'-'9'];
	let LETTER = ['a'-'z''A'-'Z'];
	let VAR_CHAR = LETTER | DIGIT | UNDERSCORE;
	let VAR_NAME = '$' (LETTER | UNDERSCORE) VAR_CHAR*;

	let INT = "-"? DIGIT+;
	let FLOAT = "-"? (DIGIT+ "." DIGIT*) | "-"? (DIGIT* "." DIGIT+) | "-"? "Infinity";

	let STRING = '"' ([^'\\''"']|'\\'.)* '"' | "'" ([^'\\'"'"]|'\\'.)* "'";

	let BOOL = "true" | "false";

	let COLON = ',';

	let FUNCTION = LETTER+ '(';

	let MACRO_NAME = [^" >"'\n']*;

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

	INITIAL {
		PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.PASSAGE();
			None
		}
		TEXT_INITIAL =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.INITIAL_NON_NEWLINE();
			None
		}

	}

	INITIAL_NON_NEWLINE {
		NEWLINE =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.INITIAL();
			None
		}
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
			Some(TokPassageLink {display_name: name.clone(), passage_name: name.clone()} )
		}

		LINK_LABELED =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let matches = &trimmed.split("|").collect::<Vec<&str>>();
			assert_eq!(matches.len(), 2);
			let text = matches[0].to_string();
			let name = matches[1].to_string();
			Some(TokPassageLink {display_name: text, passage_name: name} )
		}

		FORMAT_ITALIC => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_italic_open = !lexer.format_italic_open;
			if lexer.format_italic_open {Some(TokFormatItalicStart)}
			else {Some(TokFormatItalicEnd)}
		}
		FORMAT_BOLD => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_bold_open = !lexer.format_bold_open;
			if lexer.format_bold_open {Some(TokFormatBoldStart)}
			else {Some(TokFormatBoldEnd)}
		}
		FORMAT_UNDER => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_under_open = !lexer.format_under_open;
			if lexer.format_under_open {Some(TokFormatUnderStart)}
			else {Some(TokFormatUnderEnd)}
		}
		FORMAT_STRIKE => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_strike_open = !lexer.format_strike_open;
			if lexer.format_strike_open {Some(TokFormatStrikeStart)}
			else {Some(TokFormatStrikeEnd)}
		}
		FORMAT_SUB => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_sub_open = !lexer.format_sub_open;
			if lexer.format_sub_open {Some(TokFormatSubStart)}
			else {Some(TokFormatSubEnd)}
		}
		FORMAT_SUP => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			lexer.format_sup_open = !lexer.format_sup_open;
			if lexer.format_sup_open {Some(TokFormatSupStart)}
			else {Some(TokFormatSupEnd)}
		}
		FORMAT_MONO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MONO_TEXT();
			Some(TokFormatMonoStart)
		}
		FORMAT_BUL_LIST =>  |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokFormatBulList)
		}
		FORMAT_NUMB_LIST =>  |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokFormatNumbList)
		}
		FORMAT_HEADING  =>  |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokFormatHeading {rank: lexer.yystr().trim().len()} )
		}
		TEXT_START_CHAR => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokText {text: lexer.yystr()} )
		}
		FORMAT_HORIZONTAL_LINE  =>  |_:&mut TweeLexer<R>| Some(TokFormatHorizontalLine)
		FORMAT_INDENT_BLOCK  =>  |_:&mut TweeLexer<R>| Some(TokFormatIndentBlock)

		NEWLINE => |_:&mut TweeLexer<R>| Some(TokNewLine)
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
			Some(TokPassageLink {display_name: name.clone(), passage_name: name.clone()} )
		}

		LINK_LABELED =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let matches = &trimmed.split("|").collect::<Vec<&str>>();
			assert_eq!(matches.len(), 2);
			let text = matches[0].to_string();
			let name = matches[1].to_string();
			Some(TokPassageLink {display_name: text, passage_name: name} )
		}

		FORMAT_ITALIC => |lexer:&mut TweeLexer<R>| {
			lexer.format_italic_open = !lexer.format_italic_open;
			if lexer.format_italic_open {Some(TokFormatItalicStart)}
			else {Some(TokFormatItalicEnd)}
		}
		FORMAT_BOLD => |lexer:&mut TweeLexer<R>| {
			lexer.format_bold_open = !lexer.format_bold_open;
			if lexer.format_bold_open {Some(TokFormatBoldStart)}
			else {Some(TokFormatBoldEnd)}
		}
		FORMAT_UNDER => |lexer:&mut TweeLexer<R>| {
			lexer.format_under_open = !lexer.format_under_open;
			if lexer.format_under_open {Some(TokFormatUnderStart)}
			else {Some(TokFormatUnderEnd)}
		}
		FORMAT_STRIKE => |lexer:&mut TweeLexer<R>| {
			lexer.format_strike_open = !lexer.format_strike_open;
			if lexer.format_strike_open {Some(TokFormatStrikeStart)}
			else {Some(TokFormatStrikeEnd)}
		}
		FORMAT_SUB => |lexer:&mut TweeLexer<R>| {
			lexer.format_sub_open = !lexer.format_sub_open;
			if lexer.format_sub_open {Some(TokFormatSubStart)}
			else {Some(TokFormatSubEnd)}
		}
		FORMAT_SUP => |lexer:&mut TweeLexer<R>| {
			lexer.format_sup_open = !lexer.format_sup_open;
			if lexer.format_sup_open {Some(TokFormatSupStart)}
			else {Some(TokFormatSupEnd)}
		}
		FORMAT_MONO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MONO_TEXT();
			Some(TokFormatMonoStart)
		}

		NEWLINE =>  |lexer:&mut TweeLexer<R>| {
			lexer.NEWLINE();
			Some(TokNewLine)
		}

		TEXT =>  |lexer:&mut TweeLexer<R>| Some(TokText {text: lexer.yystr()} )
	}

	PASSAGE {
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassage {name: lexer.yystr().trim().to_string()} )
		TAG_START => |lexer:&mut TweeLexer<R>| {
			lexer.TAGS();
			Some(TokTagStart)
		}
		NEWLINE => |lexer:&mut TweeLexer<R>| -> Option<Token>{
			lexer.NEWLINE();
			None
		}
	}

	TAGS {
		TAG => |lexer:&mut TweeLexer<R>| Some(TokTag {tag_name: lexer.yystr()} )
		WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
		TAG_END => |lexer:&mut TweeLexer<R>| {
			lexer.PASSAGE();
			Some(TokTagEnd)
		}
	}

	MONO_TEXT {
		TEXT_MONO =>  |lexer:&mut TweeLexer<R>| Some(TokText {text: lexer.yystr()} )
		FORMAT_MONO_END => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokFormatMonoEnd)
		}
		NEWLINE =>  |_:&mut TweeLexer<R>| -> Option<Token> {
            Some(TokText {text: " ".to_string()} )
        }
	}

	MACRO {
		WHITESPACE =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.NON_NEWLINE();
			None
		}

		MACRO_NAME =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
			match lexer.yystr().trim().as_ref() {
				"set" => {
					lexer.MACRO_CONTENT();
					Some(TokSet)
				},
				"if" => {
					lexer.MACRO_CONTENT();
					Some(TokIf)
				},
				"else" => {
					lexer.MACRO_CONTENT();
					Some(TokElse)
				},
				"endif" => {
					lexer.MACRO_CONTENT();
					Some(TokEndIf)
				},
				"print" => {
					lexer.MACRO_CONTENT();
					Some(TokPrint)
				},
				"display" => {
					lexer.DISPLAY_CONTENT();
					Some(TokDisplay)
				},
				"silently" => {
					lexer.MACRO_CONTENT();
					Some(TokSilently)
				},
				"endsilently" => {
					lexer.MACRO_CONTENT();
					Some(TokEndSilently)
				},
				_ => {
					lexer.MACRO_CONTENT();
					Some(TokMacroPassageName {passage_name: lexer.yystr().trim().to_string()} )
				}
			}
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| {
			lexer.MACRO_CONTENT();
			Some(TokMacroVar {var_name: lexer.yystr()} )
		}
	}



	MACRO_CONTENT {
		MACRO_END => |lexer:&mut TweeLexer<R>| {
			if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
			Some(TokMacroEnd)
		}


		FUNCTION =>  |lexer:&mut TweeLexer<R>| {
			let s =  lexer.yystr();
			let trimmed = &s[0 .. s.len()-1];
			let name = &trimmed.to_string();
			lexer.function_brackets = 1;
			lexer.FUNCTION_ARGS();
			Some(TokFunction {name: name.clone()} )
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

			Some(TokString {value: unescaped} )
		}

		VAR_NAME =>   |lexer:&mut TweeLexer<R>| Some(TokVariable{name: lexer.yystr()} )
		FLOAT =>      |lexer:&mut TweeLexer<R>| Some(TokFloat   {value: lexer.yystr()[..].parse().unwrap()} )
		INT =>        |lexer:&mut TweeLexer<R>| Some(TokInt     {value: lexer.yystr()[..].parse().unwrap()} )
		BOOL =>       |lexer:&mut TweeLexer<R>| Some(TokBoolean {value: lexer.yystr()} )
		NUM_OP =>     |lexer:&mut TweeLexer<R>| Some(TokNumOp   {op_name: lexer.yystr()} )
		COMP_OP =>    |lexer:&mut TweeLexer<R>| Some(TokCompOp  {op_name: lexer.yystr()} )
		LOG_OP =>     |lexer:&mut TweeLexer<R>| Some(TokLogOp   {op_name: lexer.yystr()} )
		BR_OPEN =>    |_:&mut TweeLexer<R>|     Some(TokBracketOpen)
		BR_CLOSE =>   |_:&mut TweeLexer<R>|     Some(TokBracketClose)
		SEMI_COLON => |_:&mut TweeLexer<R>|     Some(TokSemiColon)
		ASSIGN =>     |lexer:&mut TweeLexer<R>| Some(TokAssign {var_name: "".to_string(), op_name: lexer.yystr()} )
		COLON =>      |_:&mut TweeLexer<R>|     Some(TokColon)
		// Expression Stuff End

		WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> {
			None
		}
	}

	FUNCTION_ARGS {
		COLON =>    |_:&mut TweeLexer<R>| Some(TokColon)
		VAR_NAME => |lexer:&mut TweeLexer<R>| Some(TokVariable{name: lexer.yystr()} )
		FLOAT =>    |lexer:&mut TweeLexer<R>| Some(TokFloat   {value: lexer.yystr()[..].parse().unwrap()} )
		INT =>      |lexer:&mut TweeLexer<R>| Some(TokInt     {value: lexer.yystr()[..].parse().unwrap()} )
		STRING =>   |lexer:&mut TweeLexer<R>| Some(TokString  {value: lexer.yystr()} )
		BOOL =>     |lexer:&mut TweeLexer<R>| Some(TokBoolean {value: lexer.yystr()} )
		NUM_OP =>   |lexer:&mut TweeLexer<R>| Some(TokNumOp   {op_name: lexer.yystr()} )
		COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp  {op_name: lexer.yystr()} )
		LOG_OP =>   |lexer:&mut TweeLexer<R>| Some(TokLogOp   {op_name: lexer.yystr()} )
		BR_OPEN =>  |lexer:&mut TweeLexer<R>| {
			lexer.function_brackets += 1;
			Some(TokBracketOpen)
		}
		BR_CLOSE =>  |lexer:&mut TweeLexer<R>| {
			lexer.function_brackets -= 1;
			if lexer.function_brackets == 0 {
				lexer.MACRO_CONTENT();
				Some(TokArgsEnd)
			} else {
				Some(TokBracketClose)
			}
		}
	}

	DISPLAY_CONTENT {
		MACRO_END => |lexer:&mut TweeLexer<R>| {
			if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
			Some(TokMacroEnd)
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable {name: lexer.yystr()} )
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassage {name: lexer.yystr().trim().to_string()} )
	}

	LINK_VAR_CHECK {
		LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.NON_NEWLINE();
			None
		}

		LINK_OPEN => |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.LINK_VAR_SET();
			Some(TokVarSetStart)
		}
	}

	LINK_VAR_SET {
		// Expression Stuff
		VAR_NAME =>   |lexer:&mut TweeLexer<R>| Some(TokVariable{name: lexer.yystr()} )
		FLOAT =>      |lexer:&mut TweeLexer<R>| Some(TokFloat   {value: lexer.yystr()[..].parse().unwrap()} )
		INT =>        |lexer:&mut TweeLexer<R>| Some(TokInt     {value: lexer.yystr()[..].parse().unwrap()} )
		STRING =>     |lexer:&mut TweeLexer<R>| Some(TokString  {value: lexer.yystr()} )
		BOOL =>       |lexer:&mut TweeLexer<R>| Some(TokBoolean {value: lexer.yystr()} )
		NUM_OP =>     |lexer:&mut TweeLexer<R>| Some(TokNumOp   {op_name: lexer.yystr()} )
		COMP_OP =>    |lexer:&mut TweeLexer<R>| Some(TokCompOp  {op_name: lexer.yystr()} )
		LOG_OP =>     |lexer:&mut TweeLexer<R>| Some(TokLogOp   {op_name: lexer.yystr()} )
		BR_OPEN =>    |_:&mut TweeLexer<R>|     Some(TokBracketOpen)
		BR_CLOSE =>   |_:&mut TweeLexer<R>|     Some(TokBracketClose)
		SEMI_COLON => |_:&mut TweeLexer<R>|     Some(TokSemiColon)
		ASSIGN =>     |lexer:&mut TweeLexer<R>| Some(TokAssign  {var_name: "".to_string(), op_name: lexer.yystr()} )
		// Expression Stuff End

		LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.LINK_WAIT_CLOSE();
			Some(TokVarSetEnd)
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

    use config;
	use super::*;
	use super::Token::*;

	fn test_lex(input: &str) -> Vec<Token> {
	    let cfg = config::default_config();
		let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
		lex(&cfg, &mut cursor).collect()
	}

	#[test]
	fn passage_test() {
		// This should detect the ::Start passage
		let start_tokens = test_lex("::Start");
		let expected = vec!(
			TokPassage {name: "Start".to_string()}
		);

		assert_eq!(expected, start_tokens);

		// This should not return any tokens
		let fail_tokens = test_lex(":fail");
		assert_eq!(0, fail_tokens.len());
	}

	#[test]
	fn text_test() {
		// This should return a passage with a body text
		let tokens = test_lex("::Passage\nTestText\nTestNextLine\n::NextPassage");
		let expected = vec!(
			TokPassage {name: "Passage".to_string()},
			TokText {text: "TestText".to_string()},
			TokNewLine,
			TokText {text: "TestNextLine".to_string()},
			TokNewLine,
			TokPassage {name: "NextPassage".to_string()}
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn tag_test() {
		// This should return a passage with tags
		let tokens = test_lex("::TagPassage [tag1 tag2]\nContent");
		let expected = vec!(
			TokPassage {name: "TagPassage".to_string()},
			TokTagStart,
			TokTag {tag_name: "tag1".to_string()},
			TokTag {tag_name: "tag2".to_string()},
			TokTagEnd,
			TokText {text: "Content".to_string()}
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_set_test() {
		// This should return a passage with a set macro
		let tokens = test_lex("::Passage\n<<set $var = 1>>");
		let expected = vec!(
			TokPassage {name: "Passage".to_string()},
			TokSet,
			TokAssign {var_name: "$var".to_string(), op_name: "=".to_string()},
			TokInt {value: 1},
			TokMacroEnd
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_if_test() {
		// This should return a passage with an if macro
		let tokens = test_lex("::Passage\n<<if $var == 1>>1<<else if var is 2>>2<<else>>3<<endif>>");
		let expected = vec!(
			TokPassage {name: "Passage".to_string()},
			TokIf,
			TokVariable {name: "$var".to_string()},
			TokCompOp {op_name: "==".to_string()},
			TokInt {value: 1},
			TokMacroEnd,
			TokText {text: "1".to_string()},
			TokElse,
			/* TODO: Fix else if */
			TokCompOp {op_name: "is".to_string()},
			TokInt {value: 2},
			TokMacroEnd,
			TokText {text: "2".to_string()},
			TokElse,
			TokMacroEnd,
			TokText {text: "3".to_string()},
			TokEndIf,
			TokMacroEnd
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_print_test() {
		let tokens = test_lex("::Passage\n<<print \"Test with escaped \\\"Quotes\">>\n<<print $var>>");
		let expected = vec!(
			TokPassage {name: "Passage".to_string()},
			TokPrint,
			TokString {value: "Test with escaped \"Quotes".to_string()},
			TokMacroEnd,
			TokNewLine,
			TokPrint,
			TokVariable {name: "$var".to_string()},
			TokMacroEnd
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_display_test() {
		let tokens = test_lex("::Passage\n<<display DisplayedPassage>>\n::DisplayedPassage");
		let expected = vec!(
			TokPassage {name: "Passage".to_string()},
			TokDisplay,
			TokPassage {name: "DisplayedPassage".to_string()},
			TokMacroEnd,
			TokNewLine,
			TokPassage {name: "DisplayedPassage".to_string()}
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_display_short_test() {
		// Should fail because it contains an invalid macro
		let tokens = test_lex("::Passage\n<<Passage>>");
		let expected = vec![
			TokPassage {name: "Passage".to_string()},
			TokMacroPassageName {passage_name: "Passage".to_string()},
			TokMacroEnd
		];

		assert_eq!(expected, tokens);
	}
}
