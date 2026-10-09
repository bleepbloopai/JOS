//! Interactive shell running as a kernel task.
use crate::{console, fs, heap, hw, sched};
use alloc::string::String;

extern "C" fn counter_task(n: usize) {
    let id = sched::current_id();
    for i in 1..=n {
        println!("\n[task {}] step {}/{}", id, i, n);
        sched::sleep_ms(1000);
    }
    println!("\n[task {}] finished", id);
}

fn read_line(buf: &mut String) {
    buf.clear();
    loop {
        match console::try_read() {
            Some(b'\r') | Some(b'\n') => {
                println!();
                return;
            }
            Some(0x7f) | Some(0x08) => {
                if buf.pop().is_some() {
                    print!("\x08 \x08");
                }
            }
            Some(0x03) => {
                buf.clear();
                println!("^C");
                return;
            }
            Some(c) if (0x20..0x7f).contains(&c) => {
                buf.push(c as char);
                print!("{}", c as char);
            }
            Some(_) => {}
            None => hw::wfi(), // sleep until the next interrupt
        }
    }
}

fn help() {
    println!("commands:");
    println!("  help               show this list");
    println!("  echo <text>        print text");
    println!("  clear              clear the screen");
    println!("  uptime             time since boot");
    println!("  mem                heap usage");
    println!("  ps                 list tasks");
    println!("  spawn [n]          start a background task that counts n seconds");
    println!("  ls                 list files");
    println!("  cat <file>         show a file");
    println!("  write <file> <txt> create/overwrite a file");
    println!("  rm <file>          delete a file");
    println!("  reboot | poweroff");
}

fn execute(line: &str) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    let (cmd, args) = match line.split_once(' ') {
        Some((c, a)) => (c, a.trim()),
        None => (line, ""),
    };
    match cmd {
        "help" => help(),
        "echo" => println!("{}", args),
        "clear" => print!("\x1b[2J\x1b[H"),
        "uptime" => {
            let t = hw::ticks();
            println!("up {}.{:02} s", t / hw::TICK_HZ, t % hw::TICK_HZ);
        }
        "mem" => {
            let (total, used) = heap::stats();
            println!(
                "heap: {} KiB total, {} bytes used, {} KiB free",
                total / 1024,
                used,
                (total - used) / 1024
            );
        }
        "ps" => {
            println!("{:<4} {:<10} {}", "PID", "NAME", "STATE");
            for (id, name, st) in sched::list() {
                println!("{:<4} {:<10} {}", id, name, st);
            }
        }
        "spawn" => {
            let n = args.parse::<usize>().unwrap_or(5);
            let id = sched::spawn("counter", counter_task, n);
            println!("spawned task {} ({} steps)", id, n);
        }
        "ls" => {
            for (name, data) in fs::fs().iter() {
                println!("{:<20} {} bytes", name, data.len());
            }
        }
        "cat" => match fs::fs().get(args) {
            Some(d) => println!("{}", String::from_utf8_lossy(d)),
            None => println!("cat: {}: no such file", args),
        },
        "write" => match args.split_once(' ') {
            Some((name, text)) => {
                fs::fs().insert(String::from(name), text.as_bytes().to_vec());
                println!("wrote {} bytes to {}", text.len(), name);
            }
            None => println!("usage: write <file> <text>"),
        },
        "rm" => {
            if fs::fs().remove(args).is_some() {
                println!("removed {}", args);
            } else {
                println!("rm: {}: no such file", args);
            }
        }
        "poweroff" | "shutdown" => {
            println!("bye");
            hw::poweroff();
        }
        "reboot" => {
            println!("rebooting...");
            hw::reboot();
        }
        _ => println!("unknown command: {} (try 'help')", cmd),
    }
}

pub extern "C" fn run(_arg: usize) {
    println!("\nrvos shell - type 'help'");
    let mut line = String::new();
    loop {
        print!("rvos> ");
        read_line(&mut line);
        execute(&line);
    }
}
