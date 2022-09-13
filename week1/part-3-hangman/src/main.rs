// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

use std::collections::HashMap;

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    let mut books:HashMap<char, Vec<u32>> = HashMap::new();
    let mut string_showed = vec![];
    let mut string_guesses = vec![];
    for i in 0..secret_word_chars.len() {
        if books.contains_key(&secret_word_chars[i]){
            let v = books.get_mut(&secret_word_chars[i]).unwrap();
            v.push(i as u32);
        } else{
            books.insert(secret_word_chars[i],vec![i as u32]);
        }
        string_showed.push(0); // Push invalid flags
    }   
    let mut counter = 0;
    loop {
        if counter == NUM_INCORRECT_GUESSES {
            println!("Sorry, you ran out of guesses!");
            break;
        }
        if books.len() == 0 {
            println!("Congratulations you guessed the secret word: {}!",secret_word);
            break;
        }
        print!("The word so far is ");
        for i in 0..secret_word_chars.len() {
            if string_showed[i] == 0 {
                print!("-");
            } else{
                print!("{}",secret_word_chars[i]);
            }
        }
        println!("");
        print!("You have guessed the following letters: ");
        for i in 0..string_guesses.len() {
            print!("{}", string_guesses[i]);
        }
        println!("");
        println!("You have {} guesses left", NUM_INCORRECT_GUESSES - counter);
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        let mut guesses = String::new();
        io::stdin().read_line(&mut guesses).expect("Error reading line.");
        // extra: is it a char?
        let guesses = guesses.chars().collect::<Vec<char>>()[0];
        string_guesses.push(guesses);
        if !books.contains_key(&guesses){
            println!("Sorry, that letter is not in the word");
            counter += 1;
        } else{
            let v = books.get_mut(&guesses).unwrap();
            for idx in v {
                string_showed[*idx as usize] = 1;
            }
            books.remove(&guesses);
        }
        println!("");
    }
}
