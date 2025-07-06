mod lexer;
mod paser;
mod nodes;
use paser::run;

/*
TODO : boolens 
----------------------------
let a : int = 12;

12 == a => true
12 =! a => false
*/

fn main() {
    run();
}
