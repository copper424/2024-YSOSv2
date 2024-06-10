use lib::{print, println, sys_wait_pid, vec::Vec};
pub enum ProcessPriority {
    Kernel,
    Low = 5,
    Normal = 10,
    High = 15,
}

pub fn spawn(line_arr: &Vec<&str>) {
    if line_arr.len() <= 1 {
        println!("cannot find the app name for execution. Usage: exec <app name>");
        return;
    }
    let mut priority = ProcessPriority::Normal as u8;
    if line_arr.len() == 3 {
        match line_arr[2].parse::<u8>() {
            Ok(p) => {
                priority = p;
            }
            Err(e) => println!("failed to parse priority:{}", e),
        }
    }
    let pid = lib::sys_spawn(line_arr[1], priority);
    if pid == 0 {
        println!("failed to spawn process");
        return;
    }
    let _exit_code = sys_wait_pid(pid);
    // println!("The exit code of PID {} is {}", pid, _exit_code);
}

pub fn nohup(line_arr: &Vec<&str>) {
    if line_arr.len() <= 1 {
        println!("cannot find the app name for execution. Usage: nohup <app name>");
        return;
    }
    let mut priority = ProcessPriority::Normal as u8;
    if line_arr.len() == 3 {
        match line_arr[2].parse::<u8>() {
            Ok(p) => {
                priority = p;
            }
            Err(e) => println!("failed to parse priority:{}", e),
        }
    }
    let _pid = lib::sys_spawn(line_arr[1], priority);
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
