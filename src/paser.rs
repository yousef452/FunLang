use std::collections::HashMap;
use std::{fs::File};
use std::fs;
use std::io::Write;
use std::vec;

use crate::nodes::*;
use crate::lexer::{TokenType,TokenValue,Token,Scanner};


pub struct Parser {
    toks : Vec<Token>,
    pos : usize,
    current_tok : Token,
    symbol_table: HashMap<String,VarInfo>,
}



impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        let current_tok = toks.get(0)
            .cloned()
            .unwrap_or_else(|| {
                Token::new(TokenType::NONE, TokenValue::NONE)
            });
       
        Self {
            toks,
            pos: 0,
            current_tok,
            symbol_table : HashMap::new(),
        }
    }
    fn advance(&mut self) {
        self.pos += 1;
        if self.pos < self.toks.len() {
            self.current_tok = self.toks[self.pos].clone();
        }
    }
    fn factor(&mut self) -> Result<Box<dyn Node>, String> {
        let mut minus_count = 0;

        while self.current_tok.tok_type == TokenType::PLUS || self.current_tok.tok_type == TokenType::MIN {
            if self.current_tok.tok_type == TokenType::MIN {
                minus_count += 1;
            }
            self.advance();
        }

        if self.current_tok.tok_type == TokenType::LParen {
            self.advance();
            let expr = self.expr()?;

            if self.current_tok.tok_type != TokenType::RParen {
                return Err("Expected closing parenthesis".to_string());
            }
            self.advance();

            if minus_count % 2 == 0 {
                return Ok(expr);
            } else {
                let minus_tok = Token::new(TokenType::MIN, TokenValue::NONE);
                return Ok(Box::new(UnaryOpNode::new(minus_tok, expr)));
            }
        }

        if self.current_tok.tok_type == TokenType::INT || self.current_tok.tok_type == TokenType::FLOAT {
            let tok = self.current_tok.clone();
            self.advance();

            if minus_count % 2 == 0 {
                return Ok(Box::new(NumberNode::new(tok)));
            } else {
                let minus_tok = Token::new(TokenType::MIN, TokenValue::NONE);
                return Ok(Box::new(UnaryOpNode::new(minus_tok, Box::new(NumberNode::new(tok)))));
            }
        }

        if self.current_tok.tok_type == TokenType::IDENTIFIER {
    let tok = self.current_tok.clone();
    self.advance();

    let var_name = tok.extract_str().unwrap();
    let var_info = match self.symbol_table.get(&var_name) {
        Some(info) => info,
        None => return Err(format!("Undefined variable: {}", var_name)),
    };

    let is_pointer = var_info.is_pointer;
    let var_type = var_info.tok_type;

    if minus_count % 2 == 0 {
        return Ok(Box::new(VarAccessNode::new(tok, var_type, is_pointer)));
    } else {
        let minus_tok = Token::new(TokenType::MIN, TokenValue::NONE);
        return Ok(Box::new(UnaryOpNode::new(minus_tok, Box::new(VarAccessNode::new(tok, var_type, is_pointer)))));
    }
}


        if self.current_tok.tok_type == TokenType::STRING {
            let val = self.current_tok.clone(); 
            self.advance();
            return Ok(Box::new(StringNode::new(val)));
        }



        Err(format!(
            "Unexpected token in factor: {:?}",
            self.current_tok.tok_type
        ))
    }



    fn term(&mut self) -> Result<Box<dyn Node>, String> {
        self.bin_op(Parser::factor, &[TokenType::MULT, TokenType::DIV,TokenType::MOD])
    }

    fn expr(&mut self) -> Result<Box<dyn Node>, String> {
        self.bin_op(Parser::term, &[TokenType::PLUS, TokenType::MIN,TokenType::EqualsEquals,TokenType::NotEquals])
    }

    fn bin_op<F>(&mut self, parse_func: F, ops: &[TokenType]) -> Result<Box<dyn Node>, String>
    where
        F: Fn(&mut Self) -> Result<Box<dyn Node>, String>,
    {
        let mut left = parse_func(self)?;

        while ops.contains(&self.current_tok.tok_type) {
            let op_tok = self.current_tok.clone();
            self.advance();
            let right = parse_func(self)?;
            left = Box::new(BinOpNode::new(left, op_tok, right));
        }

        Ok(left)
    }
    pub fn expect(&mut self, expected: TokenType) -> Result<Token, String> {
        if self.current_tok.tok_type == expected {
            let tok = self.current_tok.clone();
            self.advance();
            Ok(tok)
        } else {

            Err(format!(
                "Expected {:?}, but found {:?}",
                expected, self.current_tok.tok_type
            ))
        }
    }
    fn expect_vec(&mut self,expected: Vec<TokenType>) -> Result<Token, String> {
        if expected.contains(&self.current_tok.tok_type) {
            let tok = self.current_tok.clone();
            self.advance();
            Ok(tok)
       } else {

            Err(format!(
                "Expected {:?}, but found {:?}",
                expected, self.current_tok.tok_type
            ))
        }
    }


    pub fn parse_print_ags(&mut self) -> Result<Vec<Box<dyn Node>>, String> {
        let mut args = vec![];
        args.push(self.expr()?);
        while self.current_tok.tok_type == TokenType::COMMA {
            self.advance();
            args.push(self.expr()?);
        }
        return Ok(args);
    }


    fn parse_main_func(&mut self) -> Box<dyn Node> {
        self.expect(TokenType::MAIN).expect("Expect Main");
        self.expect(TokenType::LParen).expect("Expect L_Paren");
        self.expect(TokenType::RParen).expect("Expect R_Paren");
        self.expect(TokenType::LCurly).expect("Expect L_Curly");
        let nodes = self.parse().expect("Error parsing main body");
        self.expect(TokenType::RCurly).expect("Expect R_Curly");
        return Box::new(MainFuncNode::new(nodes));
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Node>>, String> {
        let mut statements = vec![];

        while self.current_tok.tok_type != TokenType::EOF  && self.current_tok.tok_type != TokenType::RCurly {
            let stmt = if self.current_tok.tok_type == TokenType::PRINT {
                self.advance(); 
                self.expect(TokenType::LParen)?; 
                let args = self.parse_print_ags();
                self.expect(TokenType::RParen)?;
                self.expect(TokenType::SEM)?;
                Box::new(PrintNode::new(args?)) as Box<dyn Node>
            }

            else if self.current_tok.tok_type == TokenType::LET {
                self.advance(); 
                let name = self.expect(TokenType::IDENTIFIER)?;

                let mut var_type = None;

                if let Ok(_) = self.expect(TokenType::Annotation) {
                    let type_key = self.expect_vec(vec![
                        TokenType::IntKey,
                        TokenType::StringKey,
                        TokenType::FloatKey
                    ])?;

                    var_type = Some(match type_key.tok_type {
                        TokenType::StringKey => TokenType::STRING,
                        TokenType::IntKey => TokenType::INT,
                        TokenType::FloatKey => TokenType::FLOAT,
                        _ => TokenType::NONE,
                    });
                }

                self.expect(TokenType::EQUALS)?; 
                let value = self.expr()?; 
                self.expect(TokenType::SEM)?;
                self.symbol_table.insert(name.clone().extract_str().unwrap(), VarInfo::new(value.get_type(),true));

                Box::new(VarDeclNode::new(name, value, var_type)) as Box<dyn Node>
            }

            else if self.current_tok.tok_type == TokenType::FUN {
                self.advance();
                return Ok(vec![self.parse_main_func()]);
            }


            else if self.current_tok.tok_type == TokenType::FOR {
                self.advance();

                let start = self.expr()?;
                self.expect(TokenType::Annotation)?;
                let end = self.expr()?;

                let (step, var);
                if let Ok(_) = self.expect(TokenType::EQUALS) {
                    step = Some(self.expr()?);
                    var = self.expect(TokenType::IDENTIFIER)?;
                } else {
                    step = None;
                    var = self.expect(TokenType::IDENTIFIER)?;
                }

                self.symbol_table.insert(var.clone().extract_str().unwrap(), VarInfo::new(start.get_type(),false));


                self.expect(TokenType::LCurly)?;

                let nodes = self.parse()?; 

                self.expect(TokenType::RCurly)?;

                self.symbol_table.remove(&var.extract_str().unwrap());

                let node = Box::new(ForLoopNode::new(start, end, var, nodes, step)) as Box<dyn Node>;
                statements.push(node);
                continue;
            }



            else if self.current_tok.tok_type == TokenType::IF {
                self.advance();
                let condition = self.expr()?;
                self.expect(TokenType::LCurly)?;
                let then_body = self.parse()?;
                self.expect(TokenType::RCurly)?;

                let mut elf_nodes = vec![];
                let mut elf_bodies = vec![];
                let mut else_body = None;

                while self.current_tok.tok_type == TokenType::ELF {
                    self.advance(); 
                    let elf_condition = self.expr()?;
                    self.expect(TokenType::LCurly)?;
                    let elf_body = self.parse()?; 
                    self.expect(TokenType::RCurly)?;

                    elf_nodes.push(elf_condition);
                    elf_bodies.push(elf_body);
                }

                if self.current_tok.tok_type == TokenType::ELSE {
                    self.advance();
                    self.expect(TokenType::LCurly)?;
                    let body = self.parse()?;
                    self.expect(TokenType::RCurly)?;
                    else_body = Some(body);
                }

                let node = Box::new(IfNode::new(condition, then_body, else_body, Some(elf_bodies), Some(elf_nodes))) as Box<dyn Node>;
                statements.push(node);
                continue;
            }

            else if self.current_tok.tok_type == TokenType::WHILE {
                self.advance();
                let node = self.expr()?;
                self.expect(TokenType::LCurly)?;
                let body = self.parse()?;
                self.expect(TokenType::RCurly)?;

                let while_node= Box::new(WhileNode::new(node,body)) as Box<dyn Node>;
                statements.push(while_node);
                continue;
            }

            else if self.current_tok.tok_type == TokenType::IDENTIFIER {
                let name = self.current_tok.clone();
                let name_str = name.extract_str().unwrap(); 

                if !self.symbol_table.contains_key(&name.extract_str().unwrap()) {
                    return Err(format!("Error: variable '{}' used before declaration", name_str));
                }

                self.advance();
                self.expect(TokenType::EQUALS)?;
                let value = self.expr()?;
                self.expect(TokenType::SEM)?;

                let expected_type = self.symbol_table.get(&name.extract_str().unwrap()).unwrap(); 
                let value_type = value.get_type();

                if expected_type.tok_type != value_type {
                        return Err(format!(
                            "Type Error: vfnariable '{}' expects type '{:?}', but got '{:?}'",
                            name_str, expected_type, value_type
                        ));
                    }

                    Box::new(VarAssignNode::new(name.clone(), value)) as Box<dyn Node>
                }

                else {
                    let expr = self.expr()?;
                    if expr.is_pure_value() {
                        return Err(format!(
                            "Unexpected standalone value or expression: '{}'",
                            expr.generate()
                        ));
                    }
                    expr
                };

                statements.push(stmt);
            }

        Ok(statements)
    }
}


const TYPE_LIB : &str = r#"#include <stdio.h>
#include <gc.h>
#include <string.h>

"#;

pub fn run()  {
    let path = "test/1.fun";
    let mut input = String::new();

    match fs::read_to_string(path) {
        Ok(content) => {
            input = content;
        }
        
        Err(error) => {
            eprintln!("Error reading file: {}", error);
        }
    }

    let mut scanner = Scanner::new(input);
    scanner.tokenize();

    let mut out = String::new();
    let mut parser = Parser::new(scanner.toks);
    out.push_str(TYPE_LIB);


    match parser.parse() {
        Ok(res) => {
            for r in res.into_iter() {
                out.push_str(&r.generate());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    let file = File::create("out/output.c");
    let _ = file.unwrap().write_all(out.as_bytes());
    println!("{}",out);


}
