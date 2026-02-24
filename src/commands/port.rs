use anyhow::Result;
use tokio::time::{sleep, Duration};

use crate::{
    cli::PortArgs,
    utils::ports::{common_free_ports, find_owner_by_port, safe_kill_suggestion},
};

pub async fn run(args: PortArgs) -> Result<()> {
    if args.free {
        let ports = common_free_ports();
        println!("{}", serde_json::to_string_pretty(&ports)?);
        return Ok(());
    }

    if args.watch {
        println!("Watching common dev ports every 2s (ctrl+c to stop)");
        loop {
            for p in [3000_u16, 5173, 5432, 6379, 8080] {
                if let Some(owner) = find_owner_by_port(p) {
                    println!(
                        "port {} pid={} parent={:?} mem={}KB uptime={}s cmd={}",
                        owner.port,
                        owner.pid,
                        owner.parent_pid,
                        owner.memory_kb,
                        owner.uptime_secs,
                        owner.cmd
                    );
                }
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    let target = args.port.unwrap_or(3000);
    if let Some(owner) = find_owner_by_port(target) {
        println!("Port {} is owned by pid {}", target, owner.pid);
        println!("parent pid: {:?}", owner.parent_pid);
        println!("cmd: {}", owner.cmd);
        println!("mem: {} KB", owner.memory_kb);
        println!("uptime: {} sec", owner.uptime_secs);
        for tip in safe_kill_suggestion(owner.pid) {
            println!("tip: {}", tip);
        }
    } else {
        println!("No process found for port {}", target);
    }
    Ok(())
}
