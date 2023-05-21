use std::{collections::HashMap, net::SocketAddr};

pub struct TcpListenInfo {
    pid: i32,
    local: SocketAddr,
    remote: SocketAddr,
    cmd: Vec<String>,
}

impl std::fmt::Display for TcpListenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) local: {:20} remote: {:20} {:?}", self.pid, self.local, self.remote, self.cmd[0])
    }
}

pub fn netstat() -> Vec<TcpListenInfo> {
    let tcp_inodes: HashMap<u64, (SocketAddr, SocketAddr)> = procfs::process::all_processes()
        .unwrap()
        .flat_map(|prc| prc.unwrap().tcp().unwrap())
        .chain(
            procfs::process::all_processes()
                .unwrap()
                .flat_map(|prc| prc.unwrap().tcp6().unwrap()),
        )
        .filter(|tcp| match tcp.state {
            procfs::net::TcpState::Listen => true,
            _ => false,
        })
        .map(|tcp| (tcp.inode, (tcp.local_address, tcp.remote_address)))
        .collect();

    let mut res = Vec::new();
    for prc in procfs::process::all_processes()
        .unwrap()
        .filter_map(Result::ok)
    {
        let pid = prc.pid;
        if let Ok(fds) = prc.fd() {
            for fd in fds.filter_map(Result::ok) {
                if let procfs::process::FDTarget::Socket(inode) = fd.target {
                    if let Some(port) = tcp_inodes.get(&inode) {
                        res.push(TcpListenInfo {
                            pid,
                            local: port.0,
                            remote: port.1,
                            cmd: prc.cmdline().unwrap_or(vec![]),
                        });
                    }
                }
            }
        }
    }

    res
}
