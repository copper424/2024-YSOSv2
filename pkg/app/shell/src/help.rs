extern crate lib;


pub fn print_help_infomation() {
    use ::lib::println;
    println!("YatSen OSv2 Shell");
    println!("Builtin Command Usage:");
    
    println!("\thelp: print this help message");
    println!("\tls-app: list all available apps");
    println!("\tps: show process list");
    println!("\texec: execute given app");
    println!("\tnohup: execute given app in the background");
    println!("\tkill: kill process referring to PID");
    println!("\texit: exit shell");
}
