use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use crate::anet::resolve_host;

#[derive(Debug)]
pub struct Config {
    pub hostip: String,
    pub hostport: u16,
    pub repeat: i64,
    pub dbnum: u8,
    pub auth: Option<String>,
    pub interactive: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            hostip: "127.0.0.1".to_string(),
            hostport: 6379,
            repeat: 1,
            dbnum: 0,
            auth: None,
            interactive: false,
        }
    }

    pub fn parse_options(&mut self, args: &Vec<String>) -> usize {
        let argc = args.len();
        let mut i = 1;

        while i < argc {
            let lastarg = i == argc - 1;
            match args[i].as_str() {
                "-h" if !lastarg => {
                    let ip_or_host = args[i + 1].clone();
                    match resolve_host(&ip_or_host) {
                        Ok(ip) => {
                            self.hostip = ip;
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            exit(1);
                        }
                    }
                    i += 2;
                }
                "-h" if lastarg => {
                    usage();
                }
                "-p" if !lastarg => {
                    self.hostport = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid port: {}", args[i + 1]);
                        exit(1);
                    });
                    i += 2;
                }
                "-r" if !lastarg => {
                    self.repeat = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid repeat count: {}", args[i + 1]);
                        exit(1);
                    });
                    i += 2;
                }
                "-n" if !lastarg => {
                    self.dbnum = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid db number: {}", args[i + 1]);
                        exit(1);
                    });
                    i += 2;
                }
                "-a" if !lastarg => {
                    self.auth = Some(args[i + 1].to_string().clone());
                    i += 2;
                }
                "-i" => {
                    self.interactive = true;
                    i += 1;
                }
                _ => break,
            }
        }
        i
    }
}

fn usage() {
    eprintln!("Usage: program [options]");
    eprintln!("  -h <hostname|ip>   Server hostname or IP (default 127.0.0.1)");
    eprintln!("  -p <port>          Server port (1-65535, default 6379)");
    eprintln!("  -r <repeat>        Repeat count");
    eprintln!("  -n <dbnum>         Database number");
    eprintln!("  -a <password>      Password");
    eprintln!("  -i                 Interactive mode");
    exit(1);
}

#[derive(Debug, Clone)]
pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub arity: i32,
    pub flags: CommandType,
    pub argv: Vec<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    Inline,
    Bulk,
    MultiBulk,
}

impl<'a> RedisCommand<'a> {
    pub fn build(args: Vec<&'a str>) -> Result<Self, String> {
        let cmd = CMD_TABLE
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(args[0]))
            .ok_or_else(|| format!("Unknown command: {}", args[0]))?;

        let argc = args.len() as i32;
        if (cmd.arity > 0 && cmd.arity != argc) || (cmd.arity < 0 && argc < -cmd.arity) {
            return Err(format!("Wrong number of arguments for '{}'", cmd.name));
        }

        Ok(RedisCommand {
            name: cmd.name,
            arity: cmd.arity,
            flags: cmd.flags,
            argv: args[1..].to_vec(),
        })
    }

    pub fn to_resp(&self) -> String {
        let args = self.argv.to_vec();

        match self.flags {
            CommandType::MultiBulk => {
                let mut s = format!("*{}\r\n", self.argv.len() + 1);
                s += &format!("${}\r\n{}\r\n", self.name.len(), self.name);
                for arg in args {
                    s += &format!("${}\r\n{}\r\n", arg.len(), arg);
                }
                s
            }
            CommandType::Bulk => {
                let mut s = format!("*{}\r\n", args.len() + 1);
                s += &format!("${}\r\n{}\r\n", self.name.len(), self.name);
                for arg in args {
                    s += &format!("${}\r\n{}\r\n", arg.len(), arg);
                }
                s
            }
            CommandType::Inline => {
                let mut s = self.name.to_string();
                for arg in args {
                    s += " ";
                    s += arg;
                }
                s += "\r\n";
                s
            }
        }
    }
}

// ===================== RESP 命令处理 =====================
pub fn cli_send_command(args: Vec<&str>, config: &Config, stream: &mut TcpStream, quiet: bool) {
    let rc = match RedisCommand::build(args) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let cmd = rc.to_resp();
    for _ in 0..config.repeat {
        if let Err(err) = stream.write_all(cmd.as_bytes()) {
            eprintln!("Failed to send command '{}': {}", rc.name, err);
        }
        cli_read_reply(stream, quiet);
    }
}

pub fn select_db(dbnum: u8, stream: &mut TcpStream) -> Result<(), String> {
    if dbnum == 0 {
        return Ok(());
    }

    let cmd = format!("SELECT {}\r\n", dbnum);
    stream
        .write_all(cmd.as_bytes())
        .map_err(|e| format!("Failed to send SELECT command: {}", e))?;

    cli_read_reply(stream, false)
}

// ===================== RESP 解析 =====================

/// 核心读取函数，处理所有 Redis 回复类型
pub fn cli_read_reply(stream: &mut TcpStream, quiet: bool) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    read_reply(&mut reader, quiet)
}

/// 内部递归读取函数
fn read_reply(reader: &mut BufReader<TcpStream>, quiet: bool) -> Result<(), String> {
    let mut first_byte = [0u8; 1];
    reader.read_exact(&mut first_byte).map_err(|e| e.to_string())?;

    match first_byte[0] as char {
        '+' => { cli_read_single_line_reply(reader, quiet)?; }
        '-' => {
            print!("(error) ");
            cli_read_single_line_reply(reader, quiet)?;
        }
        ':' => {
            print!("(integer) ");
            cli_read_single_line_reply(reader, quiet)?;
        }
        '$' => { cli_read_bulk_reply(reader, quiet)?; }
        '*' => { cli_read_multi_bulk_reply(reader, quiet)?; }
        other => return Err(format!("protocol error, got '{}' as reply type byte", other)),
    }

    Ok(())
}

/// 读取单行回复（+、-、:）
fn cli_read_single_line_reply(reader: &mut BufReader<TcpStream>, quiet: bool) -> Result<String, String> {
    let line = read_line(reader)?;
    if !quiet {
        println!("{}", line);
    }
    Ok(line)
}

/// 读取 bulk 回复 ($)
fn cli_read_bulk_reply(reader: &mut BufReader<TcpStream>, quiet: bool) -> Result<Option<String>, String> {
    let len_line = read_line(reader)?;
    let bulklen: isize = len_line.parse().map_err(|_| "Invalid bulk length".to_string())?;
    if bulklen == -1 {
        if !quiet { println!("(nil)"); }
        return Ok(None);
    }

    let mut buf = vec![0u8; bulklen as usize + 2]; // +2 for \r\n
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let content = String::from_utf8_lossy(&buf[..bulklen as usize]).to_string();
    if !quiet { println!("{}", content); }
    Ok(Some(content))
}

/// 读取 multi-bulk 回复 (*)
fn cli_read_multi_bulk_reply(reader: &mut BufReader<TcpStream>, quiet: bool) -> Result<(), String> {
    let len_line = read_line(reader)?;
    let count: isize = len_line.parse().map_err(|_| "Invalid multi-bulk count".to_string())?;
    if count == -1 {
        if !quiet { println!("(nil)"); }
        return Ok(());
    }

    for i in 0..count {
        print!("{}: ", i);
        read_reply(reader, quiet)?;
    }
    Ok(())
}

/// 读取一行字符串，去掉 \r\n
fn read_line(reader: &mut BufReader<TcpStream>) -> Result<String, String> {
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf.trim_end_matches("\r\n").to_string())
}


// ===================== 命令表 =====================
pub static CMD_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: "GET",
        arity: 2,
        flags: CommandType::Inline,
        argv: vec![],
    },
    RedisCommand {
        name: "SET",
        arity: -3,
        flags: CommandType::Bulk,
        argv: vec![],
    },
    RedisCommand {
        name: "DEL",
        arity: -2,
        flags: CommandType::Inline,
        argv: vec![],
    },
    RedisCommand {
        name: "PING",
        arity: -1,
        flags: CommandType::Inline,
        argv: vec![],
    },
    RedisCommand {
        name: "SELECT",
        arity: 2,
        flags: CommandType::Inline,
        argv: vec![],
    },
    RedisCommand {
        name: "MSET",
        arity: -3,
        flags: CommandType::MultiBulk,
        argv: vec![],
    },
    RedisCommand {
        name: "MGET",
        arity: -2,
        flags: CommandType::Inline,
        argv: vec![],
    },
    RedisCommand {
        name: "AUTH",
        arity: 2,
        flags: CommandType::Inline,
        argv: vec![],
    },
];

pub fn repl(config: &mut Config, stream: &mut TcpStream) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    loop {
        print!("r-cli> ");
        io::stdout().flush().unwrap();

        line.clear();
        if handle.read_line(&mut line).unwrap_or(0) == 0 { break; }
        let args: Vec<&str> = line.trim().split_whitespace().collect();
        if args.is_empty() { continue; }
        if ["quit", "exit"].contains(&args[0].to_ascii_lowercase().as_str()) { break; }

        cli_send_command(args, config, stream, false);
    }
}

pub fn init(config: &mut Config, stream: &mut TcpStream) {
    if let Some(auth) = &config.auth {
        cli_send_command(vec!["AUTH", &auth], &config, stream, true);
    }

    if let Err(err) = select_db(config.dbnum, stream) {
        eprintln!("{}", err);
    }
}
