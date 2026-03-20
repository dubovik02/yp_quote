use std::{fs::File, io::{self, BufRead, BufReader, Write}, net::{SocketAddr, TcpStream, UdpSocket}, 
    sync::{Arc, Mutex}, thread, time::Duration};

use clap::Parser;
use yp_quote::{BASE_COMMAND_PREFIX, CLIENT_QUOTES_PERIOD, ClientArgs, PING_MESSAGE, PING_PERIOD, SUCCESS_SERVER_RESPONSE, StockQuote, StockQuoteSet, error::{ERR_DESERIALIZE, ERR_PING_SENDING, 
        ERR_SERVER_RESPONSE, ERR_UDP_RECIVE, QuoutesError}};

fn main() -> Result<(), QuoutesError> {
    let params = ClientArgs::parse();
    
    let mut result: Vec<String> = Vec::new();
    match File::open(&params.subscribe_file) {
        Ok(file) => {
            let reader = BufReader::new(file);
    
            for line in reader.lines() {
                let line = line?;
                result.push(line);
            }
        }
        Err(_) => {
            result.push("*".to_owned());
        }
        
    };

    if result.is_empty() { result.push("*".to_owned());}

    let tcp_addr = format!("{}:{}", params.tcp_host, params.tcp_port);
    let mut tcp_connection = TcpStream::connect(tcp_addr)?;

    let self_udp_str = format!("{}:{}", params.udp_host, params.udp_port);
    let self_udp_addr: SocketAddr = self_udp_str
        .parse()
        .map_err(|_| QuoutesError::InvalidInput(self_udp_str))?;

    let cmd_to_send = 
        format!("{} {} {}\n", 
            BASE_COMMAND_PREFIX, 
            self_udp_addr, 
            result.join(",")
        );

    tcp_connection.write_all(cmd_to_send.as_bytes())?;

    let mut reader = BufReader::new(&tcp_connection);
    let mut response = String::new();

    reader.read_line(&mut response)?;
    let response = response.trim();
    if !response.eq_ignore_ascii_case(SUCCESS_SERVER_RESPONSE) {
        println!("{}: {}", ERR_SERVER_RESPONSE, response);
        return Ok(());
    }

    println!("{}", response);

    let socket = UdpSocket::bind(self_udp_addr)?;
    socket.set_read_timeout(
        Some(Duration::from_secs(CLIENT_QUOTES_PERIOD))
    )?;
    println!("Ready to get quotes to {}", self_udp_addr);

    let ping_socket = socket.try_clone()?;
    let ping_server = Arc::new(Mutex::new(None::<SocketAddr>));
    let ping_server_c = Arc::clone(&ping_server);
    thread::spawn(move || {
        loop {
            let target_server = ping_server_c
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            if let Some(addr) = target_server {
                match ping_socket.send_to(PING_MESSAGE.as_bytes(), addr) {
                    Ok(_) => {println!("Ping Sending")},
                    Err(e) => eprintln!("{}: {}", ERR_PING_SENDING, e),
                };
            }
            thread::sleep(Duration::from_secs(PING_PERIOD));
        }
    });

    let mut buf = [0u8; 16384];//2048
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                {
                    let mut guard = 
                    ping_server.lock().unwrap_or_else(|p| p.into_inner());
                    *guard = Some(addr);
                }
                match serde_binary_adv::Deserializer::from_bytes::<StockQuoteSet>(&buf[..n], false) {
                    Ok(q) => {
                        println!("{}", q.to_string());
                    }
                    Err(e) => eprintln!("{}: {}", ERR_DESERIALIZE, e),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => {
                println!("{}: {}", ERR_UDP_RECIVE, e);
            }
        }
    }
}