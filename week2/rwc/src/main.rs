use std::char::CharTryFromError;
use std::env;
use std::fs::File;
// For read_file_lines()
use std::io::{self, BufRead}; // For read_file_lines()
use std::process;



fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    let file= File::open(filename)?;  
    let mut ans: Vec<String> = Vec::new();
    for line in io::BufReader::new(file).lines(){
        let line_str = line?;
        ans.push(line_str); 
    }
    Ok(ans)
}

fn lines_2_words(lines: &Vec<String>) -> (usize,usize){
    let mut words: usize = 0;
    let mut chars: usize = 0;
    for line in lines{
        chars += line.len();
        words += line.split(" ").collect::<Vec<&str>>().len();
    }
    (words,chars)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)
    let lines = read_file_lines(filename).expect(&format!("fail to read {}", filename));
    // 1. print number of lines
    println!("lines:{}", lines.len());
    // 2. print number of words
    let (words, chars) = lines_2_words(&lines);
    println!("words:{}", words);
    println!("chars:{}", chars);
}
