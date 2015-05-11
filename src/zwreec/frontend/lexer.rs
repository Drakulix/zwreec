use std::io::BufReader;
use self::Token::{
	TokPassageName, TokTagStart, TokTagEnd, TokTag, 
	TokMakroStart, TokMakroEnd, TokVariable,
	TokSet, TokAssign, TokInt, TokFloat, TokNumOp, TokCompOp,
	TokLogOp, TokText, TokFormatBold, TokFormatItalic,
	TokFormatUnder,	TokFormatStrike, TokFormatSub, TokFormatSup,
	TokFormatMonoStart,	TokFormatMonoEnd, TokString, TokBracketOpen,
	TokBracketClose, TokIf, TokElse, TokEndIf, TokPassageLink,
	TokFormatBulList, TokFormatNumbList, TokFormatIndent,
	TokFormatIndentDouble, TokFormatIndentBlock, TokFormatHeading,
	TokVarSetStart,	TokVarSetEnd, TokSemiColon, TokPrint, TokDisplay,
	TokBoolean, TokFunction , TokColon, TokArgsEnd, TokSilently,
	TokEndSilently, TokArrayStart, TokArrayEnd
};

pub fn lex(input :String) -> Vec<Token> {
	let processed = preprocess(input);
   	let inp = BufReader::new(processed.as_bytes());
   	print!("Nicht in Tokens verarbeitete Zeichen: ");
	let lexer = TweeLexer::new(inp);
	lexer.collect()
}

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
	TokFormatBold,
	TokFormatItalic,
	TokFormatUnder,
	TokFormatStrike,
	TokFormatSub,
	TokFormatSup,
	TokFormatMonoStart,
	TokFormatMonoEnd,
	TokFormatBulList,
	TokFormatNumbList,
	TokFormatIndent,
	TokFormatIndentDouble,
	TokFormatIndentBlock,
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
	TokEndSilently
}

rustlex! TweeLexer {
	let DIGIT = ['0'-'9'];
	let LETTER_S = ['a'-'z'];
	let LETTER_L = ['A'-'Z'];
	let LETTER = (LETTER_S|LETTER_L);
	let SPECIAL = [";?%()+-\".$,"] | ('_' [^'_']) | ('/' [^'/']) | ("'" [^"'"]) | (':' [^':']) | ["äöüÄÖÜß"];
	let SPACE = ' ';
	let UNDERSCORE = '_';
	let NEWLINE = '\n';

	let CHAR = LETTER_S | LETTER_L | DIGIT | SPECIAL;
	let WORD = CHAR CHAR*;

	// TODO what is allowed?
	let TEXT = (CHAR | SPACE)+;
	let TEXTLINES = (CHAR | SPACE | NEWLINE)+;
	

	let PASSAGE_CHAR = SPACE | LETTER_S | LETTER_L | DIGIT | SPECIAL;
	let PASSAGE_START = "::";
	let PASSAGE_NAME = PASSAGE_CHAR PASSAGE_CHAR*;
	let TAG_START = '[';
	let TAG_END = ']';

	let LINK_SIMPLE = "[[" PASSAGE_NAME "]";
	let LINK_LABELED = "[[" TEXT "|" PASSAGE_NAME "]";

	let LINK_OPEN = '[';
	let LINK_CLOSE = ']';

	let FORMAT_ITALIC = "//";
	let FORMAT_BOLD = "''";
	let FORMAT_UNDER = "__";
	let FORMAT_STRIKE = "==";
	let FORMAT_SUB = "~~";
	let FORMAT_SUP = "^^";
	let FORMAT_MONO_START = "{{{";
	let FORMAT_MONO_END = "}}}";

	let FORMAT_BUL_LIST = "*";
	let FORMAT_NUMB_LIST = "#";
	let FORMAT_INDENT = ">";
	let FORMAT_DOUBLE_INDENT = ">>";
	let FORMAT_INDENT_BLOCK = "<<<";

	let FORMAT_HEADING = "!" | "!!" | "!!!" | "!!!!" | "!!!!!";

	let MAKRO_START = "<<";
	let MAKRO_END = ">>";

	let BR_OPEN = '(';
	let BR_CLOSE = ')';

	let ARRAY_START = '[';
	let ARRAY_END = ']';

	let VAR_CHAR = LETTER | DIGIT | UNDERSCORE;
	let VAR_NAME = '$' (LETTER | UNDERSCORE) VAR_CHAR*;

	let NUMBER = DIGIT DIGIT*; 
	let INT = '-'? NUMBER;
	// TODO exact definition of float in twee
	let FLOAT = (INT "." NUMBER?) | ("." NUMBER);

	let STRING = ('"' [^'"']* '"') | ("'" [^"'"]* "'");

	let BOOL = "true" | "false";

	let COLON = ',';

	let FUNCTION = LETTER_S+ '(';

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

	INITIAL {
		PASSAGE_START => |lexer:&mut TweeLexer<R>| -> Option<Token>{
			lexer.PASSAGE();
			None
		}

		MAKRO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO();
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

		FORMAT_ITALIC => |_:&mut TweeLexer<R>| Some(TokFormatItalic)
		FORMAT_BOLD => |_:&mut TweeLexer<R>| Some(TokFormatBold)
		FORMAT_UNDER => |_:&mut TweeLexer<R>| Some(TokFormatUnder)
		FORMAT_STRIKE => |_:&mut TweeLexer<R>| Some(TokFormatStrike)
		FORMAT_SUB => |_:&mut TweeLexer<R>| Some(TokFormatSub)
		FORMAT_SUP => |_:&mut TweeLexer<R>| Some(TokFormatSup)
		FORMAT_MONO_START => |_:&mut TweeLexer<R>| Some(TokFormatMonoStart)
		FORMAT_MONO_END => |_:&mut TweeLexer<R>| Some(TokFormatMonoEnd)
		FORMAT_BUL_LIST =>  |_:&mut TweeLexer<R>| Some(TokFormatBulList)
		FORMAT_NUMB_LIST =>  |_:&mut TweeLexer<R>| Some(TokFormatNumbList)
		FORMAT_INDENT  =>  |_:&mut TweeLexer<R>| Some(TokFormatIndent)
		FORMAT_DOUBLE_INDENT  =>  |_:&mut TweeLexer<R>| Some(TokFormatIndentDouble)
		FORMAT_INDENT_BLOCK  =>  |_:&mut TweeLexer<R>| Some(TokFormatIndentBlock)
		FORMAT_HEADING  =>  |lexer:&mut TweeLexer<R>| Some(TokFormatHeading(lexer.yystr().len()))

		TEXTLINES => |lexer:&mut TweeLexer<R>| Some(TokText(lexer.yystr()))
	}

	PASSAGE {
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassageName(lexer.yystr().trim().to_string()))
		TAG_START => |lexer:&mut TweeLexer<R>| {
			lexer.TAGS();
			Some(TokTagStart)
		}
		NEWLINE => |lexer:&mut TweeLexer<R>| -> Option<Token>{
			lexer.INITIAL();
			None
		}
	}

	TAGS {
		WORD => |lexer:&mut TweeLexer<R>| Some(TokTag(lexer.yystr()))
		TAG_END => |lexer:&mut TweeLexer<R>| {
			lexer.PASSAGE();
			Some(TokTagEnd)
		}
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
			lexer.INITIAL();
			Some(TokMakroEnd)
		}

		
		FUNCTION =>  |lexer:&mut TweeLexer<R>| {
			let s =  lexer.yystr();
			let trimmed = &s[0 .. s.len()-1];
			let name = &trimmed.to_string();
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
		ARRAY_START =>  |_:&mut TweeLexer<R>| Some(TokArrayStart)
		COLON =>  |_:&mut TweeLexer<R>| Some(TokColon)
		ARRAY_END =>  |_:&mut TweeLexer<R>| Some(TokArrayEnd)
		// Expression Stuff End
	}

	// Currently doesn't support brackets 
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
		ARRAY_START =>  |_:&mut TweeLexer<R>| Some(TokArrayStart)
		ARRAY_END =>  |_:&mut TweeLexer<R>| Some(TokArrayEnd)
		BR_CLOSE =>  |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO_CONTENT();
			Some(TokArgsEnd)
		}
	}

	DISPLAY_CONTENT {
		MAKRO_END => |lexer:&mut TweeLexer<R>| {
			lexer.INITIAL();
			Some(TokMakroEnd)
		}

		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassageName(lexer.yystr().trim().to_string()))
	}

	LINK_VAR_CHECK {
		LINK_CLOSE => |lexer:&mut TweeLexer<R>| -> Option<Token> {
			lexer.INITIAL();
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
			lexer.INITIAL();
			None
		}
	}
	
}



#[test]
fn it_works() {

}
