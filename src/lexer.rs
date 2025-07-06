use ordered_float::OrderedFloat;

#[derive(Clone, Copy, Debug,PartialEq,Hash,Eq)]

pub enum TokenType {
    PRINT,

    MULT, // multiple *
    MIN, //minus -
    DIV, //divided /
    PLUS, // plus +
    MOD, // modulus %

    LParen,
    RParen,
    LCurly,
    RCurly,

    STRING,
    INT,
    FLOAT,

    LET,
    IDENTIFIER,
    EQUALS,

    SEM, //semicolon
    COMMA,
    Annotation, //type annotation :

    // by "Key" i mean keyword
    StringKey,
    IntKey,
    FloatKey,
    
    MAIN,
    FUN, // function

    FOR,

    EqualsEquals, // ==
    NotEquals, // =!
    IF,
    ELSE,

    NONE,
    EOF
}
#[allow(dead_code)]
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum TokenValue {
    INT(i32),
    FLOAT(OrderedFloat<f32>),
    STRING(String),
    IDENTIFIER(String),
    NONE
}

#[derive(Clone, Debug, PartialEq,Hash,Eq)]
pub struct Token {
    pub tok_type : TokenType,
    pub tok_value : TokenValue
}

impl Token {
    pub fn new(tok_type : TokenType,tok_value : TokenValue) -> Self {
        Self {
            tok_type,
            tok_value
        }
    }

    pub fn extract_num(&self) -> Option<OrderedFloat<f32>> {
        match self.tok_value{
            TokenValue::INT(i) => Some(ordered_float::OrderedFloat(i as f32)),
            TokenValue::FLOAT(i) => Some(i),
            _ => None,
        }
    }

    pub fn extract_str(&self) -> Option<String> {
        match &self.tok_value {
            TokenValue::STRING(s) => Some(s.clone()),
            TokenValue::IDENTIFIER(s) => Some(s.clone()),
            _ => None,
        }
    }


    pub fn ops_str(&self) -> String{
        if self.tok_type == TokenType::MIN {
            return String::from("-");
        }
        else if self.tok_type == TokenType::PLUS{
            return String::from("+");
        }
        else if self.tok_type == TokenType::DIV{
            return String::from("/");
        }
        else if self.tok_type == TokenType::MULT{
            return String::from("*");
        }
        else if self.tok_type == TokenType::MOD{
            return String::from("%");
        }
        else if self.tok_type == TokenType::EqualsEquals{
            return String::from("==");
        }
        else if self.tok_type == TokenType::NotEquals{
            return String::from("!=");
        }
        return String::new();
    }
}

pub struct Scanner {
    pub toks : Vec<Token>,
    code : Vec<char>,
    pos : usize,
    current_char :  char
}


impl Scanner {
    fn new_token(&self) -> bool {
        return self.current_char.is_alphabetic() || self.current_char.is_digit(10) || self.current_char == '_';
    }
    pub fn new(input : String) -> Self {
        let inp : Vec<char> = input.chars().collect();
        Self {
            toks: vec![],
            code: inp.clone(),
            pos: 0,
            current_char : inp[0]
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos < self.code.len() {
            self.current_char = self.code[self.pos];
        }
    }

    fn push(&mut self,tok_type : TokenType,tok_value : TokenValue) {
        self.toks.push(Token::new(tok_type, tok_value));
    }

    pub fn tokenize(&mut self) {
        while self.pos < self.code.len() {
            if self.current_char.is_whitespace() || self.current_char == '\n' {
                self.advance();
            } else if self.current_char == '+' {
                self.push(TokenType::PLUS, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '*' {
                self.push(TokenType::MULT, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '-' {
                self.push(TokenType::MIN, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '=' {
                if self.code[self.pos+1] == '=' {
                    self.push(TokenType::EqualsEquals, TokenValue::NONE);
                    self.advance();
                    self.advance();
                }
                else if self.code[self.pos+1] == '!' {
                    self.push(TokenType::NotEquals, TokenValue::NONE);
                    self.advance();
                    self.advance();
                }
                else {
                    self.push(TokenType::EQUALS, TokenValue::NONE);
                    self.advance();
                }
            } else if self.current_char == '/' {
                self.push(TokenType::DIV, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '(' {
                self.push(TokenType::LParen, TokenValue::NONE);
                self.advance();
            } else if self.current_char == ')' {
                self.push(TokenType::RParen, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '{' {
                self.push(TokenType::LCurly, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '}' {
                self.push(TokenType::RCurly, TokenValue::NONE);
                self.advance();
            } else if self.current_char == ';' {
                self.push(TokenType::SEM, TokenValue::NONE);
                self.advance();
            } else if self.current_char == ':' {
                self.push(TokenType::Annotation, TokenValue::NONE);
                self.advance();
            } else if self.current_char == '%' {
                self.push(TokenType::MOD, TokenValue::NONE);
                self.advance();
            }else if self.current_char == ',' {
                self.push(TokenType::COMMA, TokenValue::NONE);
                self.advance();
            }  else if self.current_char.is_digit(10) {
                let mut num = String::new();
                let mut is_float = false;
                while self.current_char.is_digit(10) || self.current_char == '.' {
                    if self.current_char == '.' {
                        if is_float {
                            panic!("float didn't have two dots");
                        }
                        else {
                            is_float = true;
                        }
                    }
                    num.push(self.current_char);
                    self.advance();
                    if self.pos >= self.code.len() {
                        break;
                    }
                }
                if is_float {
                    let value = num.parse::<f32>().expect("Failed to parse number");
                    self.push(TokenType::FLOAT, TokenValue::FLOAT(ordered_float::OrderedFloat(value)));
                }
                else {
                    let value = num.parse::<i32>().expect("Failed to parse number");
                    self.push(TokenType::INT, TokenValue::INT(value));
                }
            }
            else if self.current_char.is_alphabetic() {
                let mut alph = String::new();
                while self.new_token(){
                    alph.push(self.current_char);
                    self.advance();
                }
                if alph == String::from("print") {
                    self.push(TokenType::PRINT,TokenValue::NONE);
                }
                else if alph == String::from("let") {
                    self.push(TokenType::LET,TokenValue::NONE);
                }
                else if alph == String::from("main") {
                    self.push(TokenType::MAIN,TokenValue::NONE);
                }
                else if alph == String::from("fun") {
                    self.push(TokenType::FUN,TokenValue::NONE);
                }
                else if alph == String::from("int") {
                    self.push(TokenType::IntKey,TokenValue::NONE);
                }
                else if alph == String::from("float") {
                    self.push(TokenType::FloatKey,TokenValue::NONE);
                }
                else if alph == String::from("string") {
                    self.push(TokenType::StringKey,TokenValue::NONE);
                }
                else if alph == String::from("for") {
                    self.push(TokenType::FOR,TokenValue::NONE);
                }
                else if alph == String::from("if") {
                    self.push(TokenType::IF,TokenValue::NONE);
                }
                else if alph == String::from("else") {
                    self.push(TokenType::ELSE,TokenValue::NONE);
                }
                else {
                    self.push(TokenType::IDENTIFIER,TokenValue::IDENTIFIER(alph));
                }
            }
            else if self.current_char == '"' {
                let mut string = String::new();

                self.advance();
                while self.current_char != '"' && self.current_char != '\0'{
                    string.push(self.current_char);
                    self.advance();
                }
                
                if self.current_char == '\0' {
                    panic!("Unterminated string literal");
                }

                self.advance();
                self.push(TokenType::STRING, TokenValue::STRING(string));
            }
            else {
                panic!("Unknown character: {}", self.current_char);
            }
        }
        self.push(TokenType::EOF, TokenValue::NONE);
    }
}

