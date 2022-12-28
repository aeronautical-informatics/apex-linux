//! Module for netlink operations

use std::process::Command;

use anyhow::bail;
use nix::unistd::Pid;

/// Moves interface iname to the network namespace of PID
pub fn move_to_ns(iname: &str, pid: Pid) -> anyhow::Result<()> {
    let cmd = Command::new("sh")
        .arg("-c")
        .arg("ip link set dev ".to_string() + iname + " netns " + &pid.to_string())
        .output()?;

    if cmd.status.success() {
        Ok(())
    } else {
        bail!("{}", String::from_utf8(cmd.stderr)?)
    }
}
