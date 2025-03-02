use crate::utils::host_filtering::Blacklist;
use crate::proxy::cache::HttpCache;
use std::io;
use std::io::Write;
use std::sync::Arc;

pub fn command_listener(blacklist: Arc<Blacklist>, cache: Arc<HttpCache>) {
    loop {
        print!("> "); // Show prompt
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "add" => {
                if args.len() > 1 {
                    blacklist.add_host(args[1]);
                }
            },
            "remove" => {
                if args.len() > 1 {
                    blacklist.remove_host(args[1]);
                }
            },
            "clear" => {
                println!("🧹 Clearing cache...");
                match cache.clear() {
                    Ok(_) => println!("✅ Cache cleared successfully"),
                    Err(e) => println!("❌ Failed to clear cache: {}", e),
                }
            },
            "list" => blacklist.list_hosts(),
            "exit" => {
                println!("🔴 Exiting...");
                break;
            }
            _ => println!("❌ Unknown command. Use: add <host>, remove <host>, list, exit"),
        }
    }
}
