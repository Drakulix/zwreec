use std::io::BufReader;
use self::Token::{
	TokPassageName, TokTagStart, TokTagEnd, TokTag,
	TokMakroStart, TokMakroEnd, TokVariable,
	TokSet, TokAssign, TokInt, TokFloat, TokNumOp, TokCompOp,
	TokLogOp, TokText, 	TokFormatBoldStart, TokFormatBoldEnd,
	TokFormatItalicStart, TokFormatItalicEnd, TokFormatUnderStart, 
	TokFormatUnderEnd,	TokFormatStrikeStart, TokFormatStrikeEnd,
	TokFormatSubStart, TokFormatSubEnd, TokFormatSupStart, 
	TokFormatSupEnd, TokFormatMonoStart, TokFormatMonoEnd, 
	TokString, TokBracketOpen, TokBracketClose, TokIf, TokElse, 
	TokEndIf, TokPassageLink, TokFormatBulList, TokFormatNumbList, 
	TokFormatIndentBlock, TokFormatHeading, TokVarSetStart, 
	TokVarSetEnd, TokSemiColon, TokPrint, TokDisplay, TokBoolean,
	TokFunction , TokColon, TokArgsEnd, TokSilently, TokEndSilently,
	TokArrayStart, TokArrayEnd, TokNewLine, TokFormatHorizontalLine
};

pub fn lex(input :String) -> Vec<Token> {
	let processed = preprocess(input);
   	let inp = BufReader::new(processed.as_bytes());
   	print!("Nicht in Tokens verarbeitete Zeichen: ");
	let mut lexer = TweeLexer::new(inp);
	let mut current_text = "".to_string();
	// Unify TokText's next to each other to a single TokText
	let mut tokens = vec![];
	loop {
	    match lexer.next() {
	        Some(x) => {
	        	match x {
	        		TokText(text) => {
	        			current_text = current_text + &text;
	        		},
	        		_ => {
	        			if current_text != "" {
	        				tokens.push(TokText(current_text));
	        				current_text = "".to_string();
	        			}
	        			tokens.push(x);
	        		}
	        	}
	           
	        },
	        None => { 
	        	if current_text != "" {
	        		tokens.push(TokText(current_text));
	        	}
	        	break 
	        }
	    }
	}
	tokens
}

// TODO do not remove if in passage name, monospace etc. 
fn preprocess(input :String) -> String {
	let mut comment = false;
	let mut suspect_start = false;
	let mut suspect_end = false;
	let mut processed = String::new();

	for c in input.chars() {
		if !comment && !suspect_start && c == '/' {
			suspect_start = true;
			continue;
		}

		if suspect_start {
			if c == '%' {
				comment = true;
				suspect_start = false;
			} else {
				suspect_start = false;
				processed.push('/');
				processed.push(c);
			}

			continue;
		}

		if c == '%' && comment {
			suspect_end = true;
			continue;
		}

		if suspect_end {
			if c == '/' {
				comment = false;
				suspect_end = false;
			} else {
				suspect_end = false;
			}
			continue;
		}

		if !comment {
			processed.push(c);
		}
	}

	processed
}

#[derive(PartialEq,Debug)]
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
	TokAssign,
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
	TokNewLine
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
	let TEXT_START_CHAR = [^"*!>#"'\n'];
	let TEXT_CHAR = [^"/'_=~^{@<[" '\n'];
	let TEXT = TEXT_CHAR+ | ["/'_=^{@<["];

	let TEXT_MONO_CHAR = [^"}"'\n'];
	let TEXT_MONO = TEXT_MONO_CHAR+ | "}" | "}}";

	let PASSAGE_START = "::" ':'*;
	let PASSAGE_CHAR_NORMAL = [^"]$<>:|" '\n'];
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

	let STRING = ('"' [^'"']* '"') | ("'" [^"'"]* "'");

	let BOOL = "true" | "false";

	let COLON = ',';

	let FUNCTION = LETTER+ '(';

	let SET = "set";
	let IF = "if";
	let ELSE = "else";
	let END_IF = "endif";
	let PRINT = "print";
	let DISPLAY = "display";
	let SILENTLY = "silently";
	let END_SILENTLY = "endsilently";

	let ASSIGN = "=" | "to";
	let SEMI_COLON = ';';
	let NUM_OP = ["+-*/%"];
	let COMP_OP = "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
	let LOG_OP = "and" | "or" | "not";


	let LINK_OPEN = '[';
	let LINK_CLOSE = ']';
	let LINK_TEXT = [^'\n'"|]"]+;

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

		MAKRO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO();
			lexer.new_line = true;
			Some(TokMakroStart)
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

		MAKRO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO();
			lexer.new_line = false;
			Some(TokMakroStart)
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
		SET =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokSet)
		}
		IF =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokIf)
		}
		ELSE =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokElse)
		}
		END_IF =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokEndIf)
		}
		PRINT =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokPrint)
		}
		DISPLAY =>  |lexer:&mut TweeLexer<R>| {
			lexer.DISPLAY_CONTENT();
			Some(TokDisplay)
		}
		SILENTLY =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokSilently)
		}
		END_SILENTLY =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokEndSilently)
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
		ASSIGN =>  |_:&mut TweeLexer<R>| Some(TokAssign)
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
		ASSIGN =>  |_:&mut TweeLexer<R>| Some(TokAssign)
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



#[test]
fn it_works() {

}
