use std::env;
use std::net::{ToSocketAddrs};
use crate::anet::{tcp_connect};
use crate::redis_cli::{cli_send_command, init, repl, Config};

mod redis_cli;
mod anet;

fn main() {
    // 1. 收集命令行参数
    let args: Vec<String> = env::args().collect();

    // 2. 配置
    let mut config = Config::new();
    let parsed = config.parse_options(&args);
    let interactive = config.interactive;

    // 3. 连接与初始化
    let mut stream = tcp_connect(config.hostip.as_str(), config.hostport).unwrap();
    init(&mut config, &mut stream);

    // 4. 启动 REPL
    if (args.len() - parsed == 0) || interactive {
        repl(&mut config, &mut stream);
    }

    // 5. 非交互式执行命令
    cli_send_command(args[parsed..].iter().map(|s| s.as_str()).collect(), &config, &mut stream, false);
}
