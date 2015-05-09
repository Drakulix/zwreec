use std::io::BufReader;
use self::Token::{
	TokPassageName, TokTagStart, TokTagEnd, TokTag, 
	TokEmpty, TokMakroStart, TokMakroEnd, TokVariable,
	TokSet, TokAssign, TokInt, TokFloat, TokNumOp, TokCompOp,
	TokLogOp, TokText, TokFormatBold, TokFormatItalic,
	TokFormatUnder,	TokFormatStrike, TokFormatSub, TokFormatSup,
	TokFormatMonoStart,	TokFormatMonoEnd, TokString, TokBracketOpen,
	TokBracketClose, TokIf, TokElse, TokEndIf
};

pub fn lex(input :String) -> Vec<Token> {
   	let inp = BufReader::new(input.as_bytes());
	let lexer = TweeLexer::new(inp);
	lexer.collect()
}

#[derive(PartialEq,Debug)]
pub enum Token {
	TokPassageName (String),
	TokTagStart,
	TokTagEnd,
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
	TokMakroStart,
	TokMakroEnd,
	TokBracketOpen,
	TokBracketClose,
	TokVariable (String),
	TokInt (i32),
	TokFloat (f32),
	TokString (String),
	TokSet,
	TokAssign,
	TokNumOp (String),
	TokCompOp (String),
	TokLogOp (String),
	TokIf,
	TokElse,
	TokEndIf,
	TokEmpty
}

rustlex! TweeLexer {
	let DIGIT = ['0'-'9'];
	let LETTER_S = ['a'-'z'];
	let LETTER_L = ['A'-'Z'];
	let LETTER = (LETTER_S|LETTER_L);
	let SPECIAL = ["'._$"];
	let SPACE = ' ';
	let UNDERSCORE = '_';
	let NEWLINE = '\n';

	let CHAR = LETTER_S | LETTER_L | DIGIT | SPECIAL;
	let WORD = CHAR CHAR*;
	

	let PASSAGE_CHAR = SPACE | LETTER_S | LETTER_L | DIGIT | SPECIAL;
	let PASSAGE_START = "::";
	let PASSAGE_NAME = PASSAGE_CHAR PASSAGE_CHAR*;
	let TAG_START = '[';
	let TAG_END = ']';

	// TODO what is allowed?
	let TEXT = (LETTER | DIGIT | SPACE | NEWLINE)+;

	let FORMAT_ITALIC = "//";
	let FORMAT_BOLD = '"';
	let FORMAT_UNDER = "__";
	let FORMAT_STRIKE = "==";
	let FORMAT_SUB = "~~";
	let FORMAT_SUP = "^^";
	let FORMAT_MONO_START = "{{{";
	let FORMAT_MONO_END = "}}}";

	let MAKRO_START = "<<";
	let MAKRO_END = ">>";

	let BR_OPEN = '(';
	let BR_CLOSE = ')';

	let VAR_CHAR = LETTER | DIGIT | UNDERSCORE;
	let VAR_NAME = '$' (LETTER | UNDERSCORE) VAR_CHAR*;

	let NUMBER = DIGIT DIGIT*; 
	let INT = '-'? NUMBER;
	// TODO exact definition of float in twee
	let FLOAT = (INT "." NUMBER?) | ("." NUMBER);

	let STRING = ('"' [^'"']* '"') | ("'" [^"'"]* "'");

	let SET = "set";
	let ASSIGN = "=" | "to";
	let IF = "if";
	let ELSE = "else";
	let END_IF = "endif";

	let NUM_OP = ["+-*/%"];
	let COMP_OP = "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte";
	let LOG_OP = "and" | "or" | "not";

	INITIAL {
		PASSAGE_START => |lexer:&mut TweeLexer<R>| {
			lexer.PASSAGE();
			Some(TokEmpty)
		}

		MAKRO_START => |lexer:&mut TweeLexer<R>| {
			lexer.MAKRO();
			Some(TokMakroStart)
		}

		FORMAT_ITALIC => |_:&mut TweeLexer<R>| Some(TokFormatItalic)
		FORMAT_BOLD => |_:&mut TweeLexer<R>| Some(TokFormatBold)
		FORMAT_UNDER => |_:&mut TweeLexer<R>| Some(TokFormatUnder)
		FORMAT_STRIKE => |_:&mut TweeLexer<R>| Some(TokFormatStrike)
		FORMAT_SUB => |_:&mut TweeLexer<R>| Some(TokFormatSub)
		FORMAT_SUP => |_:&mut TweeLexer<R>| Some(TokFormatSup)
		FORMAT_MONO_START => |_:&mut TweeLexer<R>| Some(TokFormatMonoStart)
		FORMAT_MONO_END => |_:&mut TweeLexer<R>| Some(TokFormatMonoEnd)

		TEXT => |lexer:&mut TweeLexer<R>| Some(TokText(lexer.yystr()))
	}

	PASSAGE {
		// TODO trim()
		PASSAGE_NAME => |lexer:&mut TweeLexer<R>| Some(TokPassageName(lexer.yystr()))
		TAG_START => |lexer:&mut TweeLexer<R>| {
			lexer.TAGS();
			Some(TokTagStart)
		}
		NEWLINE => |lexer:&mut TweeLexer<R>| {
			lexer.INITIAL();
			Some(TokEmpty)
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
		MAKRO_END => |lexer:&mut TweeLexer<R>| {
			lexer.INITIAL();
			Some(TokMakroEnd)
		}
		VAR_NAME =>  |lexer:&mut TweeLexer<R>| Some(TokVariable(lexer.yystr()))
		FLOAT =>  |lexer:&mut TweeLexer<R>| Some(TokFloat(lexer.yystr()[..].parse().unwrap()))
		INT =>  |lexer:&mut TweeLexer<R>| Some(TokInt(lexer.yystr()[..].parse().unwrap()))
		STRING =>  |lexer:&mut TweeLexer<R>| Some(TokString(lexer.yystr()))
		SET =>  |_:&mut TweeLexer<R>| Some(TokSet)
		ASSIGN =>  |_:&mut TweeLexer<R>| Some(TokAssign)
		IF =>  |_:&mut TweeLexer<R>| Some(TokIf)
		ELSE =>  |_:&mut TweeLexer<R>| Some(TokElse)
		END_IF =>  |_:&mut TweeLexer<R>| Some(TokEndIf)
		NUM_OP =>  |lexer:&mut TweeLexer<R>| Some(TokNumOp(lexer.yystr()))
		COMP_OP =>  |lexer:&mut TweeLexer<R>| Some(TokCompOp(lexer.yystr()))
		LOG_OP =>  |lexer:&mut TweeLexer<R>| Some(TokLogOp(lexer.yystr()))
		BR_OPEN =>  |_:&mut TweeLexer<R>| Some(TokBracketOpen)
		BR_CLOSE =>  |_:&mut TweeLexer<R>| Some(TokBracketClose)
	}
	
}



#[test]
fn it_works() {

}
