//! Module for netlink operations

use std::process::Command;

use anyhow::bail;
use nix::unistd::Pid;
use regex::Regex;

/// Moves interface iname to the network namespace of PID
///
/// Keep in mind, that the interface **is not** being moved
/// back to the original namespace. Beside this, most configuration
/// done for the interface, including it's IP address, get lost
/// when moving it to a different namespace.
pub fn move_to_ns(iname: &str, pid: Pid) -> anyhow::Result<()> {
    // Prevent CMD injections by only allowing [A-Za-z0-9_]
    if !iname.chars().all(|c| char::is_ascii_alphanumeric(&c)) {
        bail!("interface name is not well-formatted")
    }

    let cmd = Command::new("ip")
        .arg("link")
        .arg("set")
        .arg("dev")
        .arg(iname)
        .arg("netns")
        .arg(pid.to_string())
        .output()?;

    if cmd.status.success() {
        Ok(())
    } else {
        bail!("{}", String::from_utf8(cmd.stderr)?)
    }
}

/// Returns all interfaces available in the current namespace
pub fn get_interfaces() -> anyhow::Result<Vec<String>> {
    let cmd = Command::new("sh").arg("-c").arg("ip address").output()?;
    if !cmd.status.success() {
        bail!("ip-address(8) failed")
    }

    let str = String::from_utf8(cmd.stdout)?;
    let re = Regex::new(r"^\d+: (.+):")?;
    let mut result: Vec<String> = Vec::new();

    for line in str.lines() {
        for cap in re.captures_iter(line) {
            result.push(cap[1].to_string());
        }
    }

    anyhow::Ok(result)
}
