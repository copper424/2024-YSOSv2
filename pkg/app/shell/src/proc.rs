use lib::{print, println, sys_wait_pid, vec::Vec};

pub fn spawn(line_arr: &Vec<&str>) {
    if line_arr.len() != 2 {
        println!("cannot find the app name for execution. Usage: exec <app name>");
        return;
    }
    let pid = lib::sys_spawn(line_arr[1]);
    let _exit_code = sys_wait_pid(pid);
    // println!("The exit code of PID {} is {}", pid, _exit_code);
}

pub fn nohup(line_arr: &Vec<&str>) {
    if line_arr.len() != 2 {
        println!("cannot find the app name for execution. Usage: nohup <app name>");
        return;
    }
    let _pid = lib::sys_spawn(line_arr[1]);
    // println!("Process {} is running in the background", _pid);
}

pub fn kill(line_arr: &Vec<&str>) {
    if line_arr.len() != 2 {
        println!("The PID of the process to be killed is not found. Usage: kill <PID>");
        return;
    }
    match line_arr[1].parse::<u16>() {
        Ok(pid) => {
            lib::sys_kill(pid);
            print!("\n");
        }
        Err(e) => println!("failed to parse PID:{}", e),
    }
}
