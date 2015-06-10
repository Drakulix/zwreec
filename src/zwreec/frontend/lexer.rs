use std::io::{BufReader, Read};
use utils::extensions::{Peeking, PeekingExt, FilteringScan, FilteringScanExt};

use self::Token::*;

pub struct ScanState {
    current_text: String,
    skip_next: bool,
}

pub fn lex<R: Read>(input: &mut R) -> FilteringScan<Peeking<TweeLexer<BufReader<&mut R>>, Token>, ScanState, fn(&mut ScanState, (Token, Option<Token>)) -> Option<Token>>  {

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
					(TokText(text), Some(TokText(_))) => {
						state.current_text.push_str(&text);
						None
					}
					(TokText(text), _) => {
						state.current_text.push_str(&text);
						let val = TokText(state.current_text.clone());
						state.current_text.clear();
						Some(val)
					},
					(TokVariable(var), Some(TokAssign(_, op))) => {
						state.skip_next = true;
						Some(TokAssign(var, op))
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
	TokPassageName (String),
	TokTagStart,
	TokTagEnd,
	TokVarSetStart,
	TokVarSetEnd,
	TokPassageLink (String, String),
	TokTag (String),
	TokText (String),
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
	TokFormatHeading (usize),
	TokMakroStart,
	TokMakroEnd,
	TokBracketOpen,
	TokBracketClose,
	TokVariable (String),
	TokInt (i32),
	TokFloat (f32),
	TokString (String),
	TokBoolean (String),
	TokFunction (String),
	TokColon,
	TokArgsEnd,
	TokArrayStart,
	TokArrayEnd,
	TokSet,
	TokAssign (String, String),
	TokNumOp (String),
	TokCompOp (String),
	TokLogOp (String),
	TokSemiColon,
	TokIf,
	TokElse,
	TokEndIf,
	TokPrint,
	TokDisplay,
	TokSilently,
	TokEndSilently,
	TokMakroVar(String),
	TokNewLine,
	TokPseudo
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

	let MAKRO_START = "<<";
	let MAKRO_END = ">>";

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

	let MACRO_NAME = LETTER+ WHITESPACE*;

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

		MAKRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
			lexer.MAKRO();
			lexer.new_line = true;
			None
		}

		LINK_SIMPLE =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let name = &trimmed.to_string();
			Some(TokPassageLink(name.clone(), name.clone()))
		}

		LINK_LABELED =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let matches = &trimmed.split("|").collect::<Vec<&str>>();
			assert_eq!(matches.len(), 2);
			let text = matches[0].to_string();
			let name = matches[1].to_string();
			Some(TokPassageLink(text, name))
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
			Some(TokFormatHeading(lexer.yystr().len()))
		}
		TEXT_START_CHAR => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokText(lexer.yystr()))
		}
		FORMAT_HORIZONTAL_LINE  =>  |_:&mut TweeLexer<R>| Some(TokFormatHorizontalLine)
		FORMAT_INDENT_BLOCK  =>  |_:&mut TweeLexer<R>| Some(TokFormatIndentBlock)

		NEWLINE => |_:&mut TweeLexer<R>| Some(TokNewLine)
	}

	NON_NEWLINE {

		MAKRO_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
			lexer.MAKRO();
			lexer.new_line = false;
			None
		}

		LINK_SIMPLE =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let name = &trimmed.to_string();
			Some(TokPassageLink(name.clone(), name.clone()))
		}

		LINK_LABELED =>  |lexer:&mut TweeLexer<R>| {
			lexer.LINK_VAR_CHECK();
			let s =  lexer.yystr();
			let trimmed = &s[2 .. s.len()-1];
			let matches = &trimmed.split("|").collect::<Vec<&str>>();
			assert_eq!(matches.len(), 2);
			let text = matches[0].to_string();
			let name = matches[1].to_string();
			Some(TokPassageLink(text, name))
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

		TEXT =>  |lexer:&mut TweeLexer<R>| Some(TokText(lexer.yystr()))
	}

	PASSAGE {
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassageName(lexer.yystr().trim().to_string()))
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
		TAG => |lexer:&mut TweeLexer<R>| Some(TokTag(lexer.yystr()))
		WHITESPACE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
		TAG_END => |lexer:&mut TweeLexer<R>| {
			lexer.PASSAGE();
			Some(TokTagEnd)
		}
	}

	MONO_TEXT {
		TEXT_MONO =>  |lexer:&mut TweeLexer<R>| Some(TokText(lexer.yystr()))
		FORMAT_MONO_END => |lexer:&mut TweeLexer<R>| {
			lexer.NON_NEWLINE();
			Some(TokFormatMonoEnd)
		}
		NEWLINE =>  |_:&mut TweeLexer<R>| -> Option<Token> { None }
	}

	MAKRO {
		WHITESPACE =>  |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.NON_NEWLINE();
			None
		}

		MACRO_NAME =>  |lexer:&mut TweeLexer<R>| {
			match lexer.yystr().trim().as_ref() {
				"set" => {
					lexer.MAKRO_CONTENT();
					Some(TokSet)
				},
				"if" => {
					lexer.MAKRO_CONTENT();
					Some(TokIf)
				},
				"else" => {
					lexer.MAKRO_CONTENT();
					Some(TokElse)
				},
				"endif" => {
					lexer.MAKRO_CONTENT();
					Some(TokEndIf)
				},
				"print" => {
					lexer.MAKRO_CONTENT();
					Some(TokPrint)
				},
				"display" => {
					lexer.DISPLAY_CONTENT();
					Some(TokDisplay)
				},
				"silently" => {
					lexer.MAKRO_CONTENT();
					Some(TokSilently)
				},
				"endsilently" => {
					lexer.MAKRO_CONTENT();
					Some(TokEndSilently)
				},
				_ => {
					panic!("Unknown macro: \"{}\"", lexer.yystr());
				}
			}
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokMakroVar(lexer.yystr()))
		}
	}



	MAKRO_CONTENT {
		MAKRO_END => |lexer:&mut TweeLexer<R>| {
			if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
			Some(TokMakroEnd)
		}


		FUNCTION =>  |lexer:&mut TweeLexer<R>| {
			let s =  lexer.yystr();
			let trimmed = &s[0 .. s.len()-1];
			let name = &trimmed.to_string();
			lexer.function_brackets = 1;
			lexer.FUNCTION_ARGS();
			Some(TokFunction(name.clone()))
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

			Some(TokString(unescaped))
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		FLOAT =>  |lexer:&mut TweeLexer<R>| Some(TokFloat(lexer.yystr()[..].parse().unwrap()))
		INT =>  |lexer:&mut TweeLexer<R>| Some(TokInt(lexer.yystr()[..].parse().unwrap()))
		BOOL =>  |lexer:&mut TweeLexer<R>| Some(TokBoolean(lexer.yystr()))
		NUM_OP =>  |lexer:&mut TweeLexer<R>| Some(TokNumOp(lexer.yystr()))
		COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp(lexer.yystr()))
		LOG_OP =>  |lexer:&mut TweeLexer<R>| Some(TokLogOp(lexer.yystr()))
		BR_OPEN =>  |_:&mut TweeLexer<R>| Some(TokBracketOpen)
		BR_CLOSE =>  |_:&mut TweeLexer<R>| Some(TokBracketClose)
		SEMI_COLON =>  |_:&mut TweeLexer<R>| Some(TokSemiColon)
		ASSIGN =>  |lexer:&mut TweeLexer<R>| Some(TokAssign("".to_string(), lexer.yystr()))
		COLON =>  |_:&mut TweeLexer<R>| Some(TokColon)
		// Expression Stuff End
	}

	FUNCTION_ARGS {
		COLON =>  |_:&mut TweeLexer<R>| Some(TokColon)
		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		FLOAT =>  |lexer:&mut TweeLexer<R>| Some(TokFloat(lexer.yystr()[..].parse().unwrap()))
		INT =>  |lexer:&mut TweeLexer<R>| Some(TokInt(lexer.yystr()[..].parse().unwrap()))
		STRING =>  |lexer:&mut TweeLexer<R>| Some(TokString(lexer.yystr()))
		BOOL =>  |lexer:&mut TweeLexer<R>| Some(TokBoolean(lexer.yystr()))
		NUM_OP =>  |lexer:&mut TweeLexer<R>| Some(TokNumOp(lexer.yystr()))
		COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp(lexer.yystr()))
		LOG_OP =>  |lexer:&mut TweeLexer<R>| Some(TokLogOp(lexer.yystr()))
		BR_OPEN =>  |lexer:&mut TweeLexer<R>| {
			lexer.function_brackets += 1;
			Some(TokBracketOpen)
		}
		BR_CLOSE =>  |lexer:&mut TweeLexer<R>| {
			lexer.function_brackets -= 1;
			if lexer.function_brackets == 0 {
				lexer.MAKRO_CONTENT();
				Some(TokArgsEnd)
			} else {
				Some(TokBracketClose)
			}
		}
	}

	DISPLAY_CONTENT {
		MAKRO_END => |lexer:&mut TweeLexer<R>| {
			if lexer.new_line { lexer.NEWLINE() } else { lexer.NON_NEWLINE() };
			Some(TokMakroEnd)
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassageName(lexer.yystr().trim().to_string()))
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
		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		FLOAT =>  |lexer:&mut TweeLexer<R>| Some(TokFloat(lexer.yystr()[..].parse().unwrap()))
		INT =>  |lexer:&mut TweeLexer<R>| Some(TokInt(lexer.yystr()[..].parse().unwrap()))
		STRING =>  |lexer:&mut TweeLexer<R>| Some(TokString(lexer.yystr()))
		BOOL =>  |lexer:&mut TweeLexer<R>| Some(TokBoolean(lexer.yystr()))
		NUM_OP =>  |lexer:&mut TweeLexer<R>| Some(TokNumOp(lexer.yystr()))
		COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp(lexer.yystr()))
		LOG_OP =>  |lexer:&mut TweeLexer<R>| Some(TokLogOp(lexer.yystr()))
		BR_OPEN =>  |_:&mut TweeLexer<R>| Some(TokBracketOpen)
		BR_CLOSE =>  |_:&mut TweeLexer<R>| Some(TokBracketClose)
		SEMI_COLON =>  |_:&mut TweeLexer<R>| Some(TokSemiColon)
		ASSIGN =>  |lexer:&mut TweeLexer<R>| Some(TokAssign("".to_string(),lexer.yystr()))
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

	use super::*;
	use super::Token::*;

	fn test_lex(input: &str) -> Vec<Token> {
		let mut cursor: Cursor<Vec<u8>> = Cursor::new(input.to_string().into_bytes());
		lex(&mut cursor).collect()
	}

	#[test]
	fn passage_test() {
		// This should detect the ::Start passage
		let start_tokens = test_lex("::Start");
		let expected = vec!(
			TokPassageName("Start".to_string())
		);

		assert_eq!(expected, start_tokens);

		// This should not return any tokens
		let fail_tokens = test_lex(":fail");
		assert_eq!(0, fail_tokens.len());
	}

	#[test]
	fn text_test() {
		// This should return a passage with a body text
		let tokens = test_lex("::Passage\nTestText\nTestNextLine");
		let expected = vec!(
			TokPassageName("Passage".to_string()),
			TokText("TestText".to_string()),
			TokNewLine,
			TokText("TestNextLine".to_string())
		);

		assert_eq!(expected, tokens);
	}
	
	#[test]
	fn tag_test() {
		// This should return a passage with tags
		let tokens = test_lex("::TagPassage [tag1 tag2]\nContent");
		let expected = vec!(
			TokPassageName("TagPassage".to_string()),
			TokTagStart,
			TokTag("tag1".to_string()),
			TokTag("tag2".to_string()),
			TokTagEnd,
			TokText("Content".to_string())
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_set_test() {
		// This should return a passage with a set macro
		let tokens = test_lex("::Passage\n<<set $var = 1>>");
		let expected = vec!(
			TokPassageName("Passage".to_string()),
			TokSet,
			TokAssign("$var".to_string(), "=".to_string()),
			TokInt(1),
			TokMakroEnd
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_if_test() {
		// This should return a passage with an if macro
		let tokens = test_lex("::Passage\n<<if $var == 1>>1<<else if var is 2>>2<<else>>3<<endif>>");
		let expected = vec!(
			TokPassageName("Passage".to_string()),
			TokIf,
			TokVariable("$var".to_string()),
			TokCompOp("==".to_string()),
			TokInt(1),
			TokMakroEnd,
			TokText("1".to_string()),
			TokElse,
			TokCompOp("is".to_string()),
			TokInt(2),
			TokMakroEnd,
			TokText("2".to_string()),
			TokElse,
			TokMakroEnd,
			TokText("3".to_string()),
			TokEndIf,
			TokMakroEnd
		);

		assert_eq!(expected, tokens);
	}

	#[test]
	fn macro_print_test() {
		let tokens = test_lex("::Passage\n<<print \"Test with escaped \\\"Quotes\">>\n<<print $var>>");
		let expected = vec!(
			TokPassageName("Passage".to_string()),
			TokPrint,
			TokString("Test with escaped \"Quotes".to_string()),
			TokMakroEnd,
			TokNewLine,
			TokPrint,
			TokVariable("$var".to_string()),
			TokMakroEnd
		);

		assert_eq!(expected, tokens);
	}
}
