use crate::lexer::{TokenType,Token};
use std::any::Any;

#[derive(Debug)]
pub struct VarInfo {
    pub tok_type : TokenType,
    pub is_pointer: bool,
}

impl VarInfo {
    pub fn new( tok_type : TokenType,is_pointer : bool) -> Self {
        Self { tok_type,is_pointer }
    }
}
pub trait Node {
    fn as_any(&self) -> &dyn Any;
    fn generate(&self) -> String;
    fn c_format(&self) -> String;
    fn is_number(&self) -> bool;
    fn get_type(&self) -> TokenType;
    fn c_type(&self) -> String;
    fn is_pure_value(&self) -> bool;
}

pub struct BinOpNode {
    left: Box<dyn Node>,
    bin_op: Token,
    right: Box<dyn Node>,
}

impl BinOpNode {
    pub fn new(left: Box<dyn Node>, bin_op: Token, right: Box<dyn Node>) -> Self {
        if left.is_number() == !right.is_number() {
            panic!("Error: left is {:?} and right is {:?}",left.get_type(),right.get_type())
        }
        else if left.get_type() == TokenType::STRING && (bin_op.tok_type != TokenType::EqualsEquals && bin_op.tok_type != TokenType::NotEquals) {
            panic!("Error: you can't do {:?} to {:?}",bin_op.tok_type,left.get_type())
        }
        BinOpNode { left, bin_op, right }
    }
}

impl Node for BinOpNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn generate(&self) -> String {
        return format!("{}{}{}",self.left.generate(),self.bin_op.ops_str(),self.right.generate());
    }
    fn c_format(&self) -> String {
        let left_fmt = self.left.c_format();
        let right_fmt = self.right.c_format();


        if left_fmt == "%f" || right_fmt == "%f" {
            "%f".to_string()
        } else if left_fmt == "%d" && right_fmt == "%d" {
            "%d".to_string()
        } else {
            "%s".to_string()
        }
    }
    fn is_number(&self) -> bool {
        self.left.is_number() && self.right.is_number()
    }
    fn get_type(&self) -> TokenType {
        if self.left.get_type() == TokenType::FLOAT || self.right.get_type() == TokenType::FLOAT {
            return TokenType::FLOAT;
        }
        TokenType::INT
    }
    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }
    fn is_pure_value(&self) -> bool {
        true
    }
}


pub struct NumberNode {
    token: Token,
}

impl NumberNode {
    pub fn new(token: Token) -> Self {
        NumberNode { token }
    }
}

impl Node for NumberNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn generate(&self) -> String {
        return format!("{}",self.token.extract_num().unwrap());
    }
    fn c_format(&self) -> String {
        if self.token.tok_type == TokenType::FLOAT {
            return "%f".to_string();
        }
        else if self.token.tok_type == TokenType::INT {
            return "%d".to_string();
        }
        return "%d".to_string();
    }
    fn is_number(&self) -> bool {
        true
    }
    fn get_type(&self) -> TokenType{
        return self.token.tok_type;
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
        true
    }
}

pub struct StringNode {
    token: Token,
}

impl StringNode {
    pub fn new(token: Token) -> Self {
        StringNode { token }
    }

}

impl Node for StringNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        return format!("\"{}\"",self.token.extract_str().unwrap());
    }
    fn c_format(&self) -> String {
        return "%s".to_string();
    }
    fn is_number(&self) -> bool {
        false
    }
    fn get_type(&self) -> TokenType {
        return TokenType::STRING;
    }
    
    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
        true
    }
}


pub struct PrintNode {
    nodes : Vec<Box<dyn Node>>,
}

impl PrintNode {
    pub fn new(nodes: Vec<Box<dyn Node>>) -> Self {
        PrintNode { nodes }
    }
}

impl Node for PrintNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        if self.nodes.is_empty() {
            return "printf(\"\\n\");\n".to_string();
        }

        let format_node = &self.nodes[0];
        let format_string = if let Some(str_node) = format_node.as_any().downcast_ref::<StringNode>() {
            match str_node.token.extract_str() {
                Some(s) => s,
                None => panic!("Expected STRING token but found something else"),
            }
        } else {
            panic!("First argument to print must be a string literal");
        };

        let mut c_format_str = String::new();
        let mut arg_formats = vec![];
        let args = &self.nodes[1..];
        let mut placeholder_count = 0;

        let mut chars = format_string.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'}') {
                chars.next(); 
                if placeholder_count >= args.len() {
                    panic!("Not enough arguments for format placeholders");
                }
                let fmt = args[placeholder_count].c_format();
                c_format_str.push_str(&fmt);
                arg_formats.push(fmt);
                placeholder_count += 1;
            } else {
                c_format_str.push(ch);
            }
        }

        if placeholder_count != args.len() {
            panic!(
                "Mismatched number of placeholders and arguments: expected {}, got {}",
                placeholder_count,
                args.len()
            );
        }

        if arg_formats.is_empty() {
            return format!("printf(\"{}\");\n", c_format_str);
        }

        let formatted_args = arg_formats.iter().zip(args.iter()).map(|(_, arg_node)| {
                format!("{}", arg_node.generate())
        }).collect::<Vec<_>>().join(", ");

        format!("printf(\"{}\", {});\n", c_format_str, formatted_args)
    }


    fn c_format(&self) -> String {
        "%s".to_string()
    }

    fn is_number(&self) -> bool {
        false
    }

    fn get_type(&self) -> TokenType {
        TokenType::STRING
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
       false 
    }
}

pub struct UnaryOpNode {
    op_tok : Token,
    node : Box<dyn Node>
}

impl UnaryOpNode {
    pub fn new( op_tok : Token, node : Box<dyn Node>) -> Self{
        Self {
            op_tok,
            node
        }
    }
}


pub struct VarDeclNode {
    name : Token,
    node : Box<dyn Node>,
    declared_type: Option<TokenType>,
}

impl VarDeclNode{
    pub fn new(name: Token, node : Box<dyn Node>,declared_type : Option<TokenType> ) -> Self {
        VarDeclNode { name, node, declared_type }
    }
}


impl Node for VarDeclNode{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn generate(&self) -> String {

        let name_str = match self.name.extract_str() {
            Some(s) => s,
            None => {
                eprintln!("Error: Variable name token does not contain an identifier string!");
                eprintln!("self.name token: {:?}", self.name);
                panic!("Cannot generate variable assignment without valid identifier name.");
            }
        };

        if self.declared_type == None {
            return match self.node.get_type() {
                TokenType::STRING => format!("char* {} = (char*) GC_MALLOC(strlen({}) + 1);\nstrcpy({}, {});\n",name_str,self.node.generate(),name_str,self.node.generate()),
                _ => format!("{}* {} =  ({}*) GC_MALLOC(sizeof({}));\n*{} = {};\n",self.c_type(),name_str,self.c_type(),self.c_type(),name_str,self.node.generate())
            };
        }
        if self.declared_type.unwrap() != self.node.get_type() {
            panic!("Error : defirent type {:?} -> {:?}",self.declared_type.unwrap(),self.node.get_type());
        }
        return match self.declared_type.unwrap() {
            TokenType::STRING => format!("char* {} = (char*) GC_MALLOC(strlen({}) + 1);\nstrcpy({}, {});\n",name_str,self.node.generate(),name_str,self.node.generate()),
            TokenType::FLOAT => format!("float* {} =  (float*) GC_MALLOC(sizeof(float));\n*{} = {};\n",name_str,name_str,self.node.generate()),
            TokenType::INT => format!("int* {} =  (int*) GC_MALLOC(sizeof(int));\n*{} = {};\n",name_str,name_str,self.node.generate()),
            _ => {
                panic!("Error : unknow type")
            }
        };
    }

    fn c_format(&self) -> String {
        self.node.c_format()
    }
    fn get_type(&self) -> TokenType {
        self.node.get_type()
    }
    fn is_number(&self) -> bool {
        self.node.is_number()
    }


    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
       false 
    }

}

pub struct VarAssignNode {
    name : Token,
    node : Box<dyn Node>
}

impl VarAssignNode {
    pub fn new(name: Token, node : Box<dyn Node>) -> Self {
        VarAssignNode { name,node }
    }
}


impl Node for VarAssignNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn generate(&self) -> String {

        let name_str = match self.name.extract_str() {
            Some(s) => s,
            None => {
                eprintln!("Error: Variable name token does not contain an identifier string!");
                eprintln!("self.name token: {:?}", self.name);
                panic!("Cannot generate variable assignment without valid identifier name.");
            }
        };

        return match self.node.get_type() {
           TokenType::STRING => format!("{} = (char*) GC_MALLOC(strlen({}) + 1);\nstrcpy({}, {});\n",name_str,self.node.generate(),name_str,self.node.generate()),
            _ => format!("*{} = {};\n",name_str,self.node.generate())
        };

    }
    fn c_format(&self) -> String {
        self.node.c_format()
    }
    fn get_type(&self) -> TokenType {
        self.node.get_type()
    }
    fn is_number(&self) -> bool {
        self.node.is_number()
    }
    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
       false 
    }
}
impl Node for UnaryOpNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        return format!("{}{}",self.op_tok.ops_str(),self.node.generate());
    }
    fn c_format(&self) -> String {
        return self.node.c_format();
    }

    fn is_number(&self) -> bool {
        true
    }

    fn get_type(&self) -> TokenType {
        self.node.get_type()
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }
    fn is_pure_value(&self) -> bool {
        true
    }
}

pub struct VarAccessNode {
    name     : Token,
    var_type : TokenType,
    is_pointer : bool
}

impl VarAccessNode {
    pub fn new(name : Token,var_type: TokenType, is_pointer : bool) -> Self{
       Self { name,var_type,is_pointer}
    }
}


impl Node for VarAccessNode{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        if self.is_pointer {
            if self.get_type() == TokenType::STRING {
                return format!("{}",self.name.extract_str().unwrap());
            }
            return format!("(*{})",self.name.extract_str().unwrap());
        }
        self.name.extract_str().unwrap()
    }
    fn c_format(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "%i",
            TokenType::FLOAT => "%f",
            TokenType::STRING => "%s",
            _ => "int",
        }.to_string();
    }

    fn is_number(&self) -> bool {
        true
    }

    fn get_type(&self) -> TokenType {
        return self.var_type;
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
        true
    }
}

pub struct MainFuncNode {
    nodes : Vec<Box<dyn Node>>
}

impl MainFuncNode {
    pub fn new(nodes : Vec<Box<dyn Node>>) -> Self {
        Self {
            nodes
        }
    }
}

impl Node for MainFuncNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        let mut ge= String::new();
        for g in &self.nodes {
            ge.push_str(&g.generate());
        }
        return format!("int main() {{\nGC_INIT();\n{}return 0;\n}}\n",ge);

    }
    fn c_format(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "%i",
            TokenType::FLOAT => "%f",
            TokenType::STRING => "%s",
            _ => "int",
        }.to_string();
    }

    fn is_number(&self) -> bool {
        true
    }

    fn get_type(&self) -> TokenType {
        return TokenType::INT;
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }
    fn is_pure_value(&self) -> bool {
       false 
    }
}

pub struct ForLoopNode {
    start : Box<dyn Node>,
    end : Box<dyn Node>,
    var : Token,
    nodes : Vec<Box<dyn Node>>,
    step : Option<Box<dyn Node>>,
}

impl ForLoopNode {
    pub fn new( start : Box<dyn Node>, end : Box<dyn Node>,var : Token, nodes : Vec<Box<dyn Node>>, step : Option<Box<dyn Node>>) -> Self {
        if start.get_type() != TokenType::INT && end.get_type() != TokenType::INT {
            panic!("Error : start : {:?}, end : {:?}",start.get_type(),end.get_type())
        }
        if !step.is_none(){
            if step.as_ref().unwrap().get_type() != TokenType::INT {
                panic!("Error : step : {:?}",step.as_ref().unwrap().get_type())
            }
        }
        Self { start, end,var,step,nodes}
    }
}

impl Node for ForLoopNode {
    fn as_any(&self) -> &dyn Any {
        self
    }


    fn generate(&self) -> String {
        let mut body_code = String::new();
        for stmt in &self.nodes {
            body_code.push_str(&stmt.generate());
        }

        let var_name = self.var.extract_str().unwrap();
        let init_code = format!(
            "int {0} = {1}",
            var_name,
            self.start.generate(),
        );

        let condition = format!("{} < {}", var_name, self.end.generate());

        let increment = if let Some(step) = &self.step {
            format!("{} += {}", var_name, step.generate())
        } else {
            format!("{}++", var_name)
        };

        format!(
            "for ({}; {}; {}) {{\n{}}}\n",
            init_code, condition, increment, body_code
        )
    }

    fn c_format(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "%i",
            TokenType::FLOAT => "%f",
            TokenType::STRING => "%s",
            _ => "int",
        }.to_string();
    }

    fn is_number(&self) -> bool {
        false
    }

    fn get_type(&self) -> TokenType {
        return TokenType::NONE;
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }
    fn is_pure_value(&self) -> bool {
       false 
    }
}

pub struct IfNode {
    node : Box<dyn Node>,
    body : Vec<Box<dyn Node>>,
    else_body : Option<Vec<Box<dyn Node>>>,
    elf_body : Option<Vec<Vec<Box<dyn Node>>>>,
    elf_node : Option<Vec<Box<dyn Node>>>
}

impl IfNode {
    pub fn new(node : Box<dyn Node>, body : Vec<Box<dyn Node>>, else_body : Option<Vec<Box<dyn Node>>>, elf_body : Option<Vec<Vec<Box<dyn Node>>>>, elf_node : Option<Vec<Box<dyn Node>>>) -> Self {
        Self { node,body,else_body,elf_body,elf_node }
    }
}

impl Node for IfNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        let mut result = String::new();

        let mut if_body = String::new();
        for stmt in &self.body {
            if_body.push_str(&stmt.generate());
        }

        result.push_str(&format!("if ({}) {{\n{}\n}}", self.node.generate(), if_body));

        if let (Some(conditions), Some(bodies)) = (&self.elf_node, &self.elf_body) {
            for (cond, body_group) in conditions.iter().zip(bodies) {
                let mut body_code = String::new();
                for stmt in body_group {
                    body_code.push_str(&stmt.generate());
                }
                result.push_str(&format!(" else if ({}) {{\n{}\n}}", cond.generate(), body_code));
            }
        }

        if let Some(else_body) = &self.else_body {
            let mut else_code = String::new();
            for stmt in else_body {
                else_code.push_str(&stmt.generate());
            }
            result.push_str(&format!(" else {{\n{}\n}}", else_code));
        }

        result
    }


    fn c_format(&self) -> String {
        "%s".to_string()
    }

    fn is_number(&self) -> bool {
        false
    }

    fn get_type(&self) -> TokenType {
        TokenType::STRING
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
      true 
    }
}


pub struct WhileNode{
    node : Box<dyn Node>,
    body : Vec<Box<dyn Node>>,
}

impl WhileNode{
    pub fn new(node : Box<dyn Node>, body : Vec<Box<dyn Node>>) -> Self {
        Self { node,body }
    }
}

impl Node for WhileNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(&self) -> String {
        let mut body = String::new();
        for stmt in &self.body {
            body.push_str(&stmt.generate());
        }

        format!("while ({})\n {{{}}}",self.node.generate(),body)
    }


    fn c_format(&self) -> String {
        "%s".to_string()
    }

    fn is_number(&self) -> bool {
        false
    }

    fn get_type(&self) -> TokenType {
        TokenType::STRING
    }

    fn c_type(&self) -> String {
        return match self.get_type() {
            TokenType::INT => "int",
            TokenType::FLOAT => "float",
            TokenType::STRING => "char*",
            _ => "int",
        }.to_string();
    }

    fn is_pure_value(&self) -> bool {
      true 
    }
}

