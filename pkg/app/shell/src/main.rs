#![no_std]
#![no_main]

use lib::{vec::Vec, *};

extern crate lib;
mod help;
mod proc;

const NAME: &str = "Copper424";
const STUDENT_ID: &str = "12345";

fn main() -> isize {
    println!("YatSen OS Volume2 Shell");
    println!("Author:{} Student ID:{}", NAME, STUDENT_ID);
    println!("Note: You can input \'help\' to get builtin command usage");

    loop {
        print!("$ ");
        let line = lib::stdin().read_line();
        let line_arr: Vec<&str> = line.split(' ').collect();
        match *line_arr.first().unwrap() {
            "help" => {
                help::print_help_infomation();
            }
            // "\x04" stands for ^D or "ctrl + d"
            "exit" | "\x04" => {
                println!("exit the shell. Bye~");
                break;
            }
            "ps" => {
                lib::sys_print_process_list();
            }
            "ls-app" => {
                lib::sys_list_app();
            }
            "exec" => {
                proc::spawn(&line_arr);
            }
            "nohup" => {
                proc::nohup(&line_arr);
            }
            "kill" => {
                proc::kill(&line_arr);
            }
            s => {
                if s.is_empty() {
                    print!("\n");
                    continue;
                }
                println!("You said: \"{}\"", s);
            }
        }
    }
    0
}

entry!(main);
