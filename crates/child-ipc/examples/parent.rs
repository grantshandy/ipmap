use child_ipc::{Command, RunTraceroute, ipc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (child, exit) = ipc::spawn_child_process(
        r"./target/release/ipmap-child".into(),
        Command::Traceroute(RunTraceroute {
            ip: "1.1.1.1".parse().unwrap(),
            max_rounds: 10,
        }),
    )
    .await?;

    loop {
        let msg = child.recv();

        println!("{msg:?}");

        if msg.is_err() {
            break;
        }
    }

    exit()?;
    Ok(())
}
