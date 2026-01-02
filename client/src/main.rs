use std::{
    io::{self, ErrorKind, Read, Write},
    net::TcpStream,
    sync::mpsc::{self, TryRecvError},
    thread,
};

const LOCAL: &str = "127.0.0.1:8080";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let mut buffer = vec![0; MSG_SIZE];
            match client.read_exact(&mut buffer) {
                Ok(_) => {
                    let msg = buffer
                        .into_iter()
                        .take_while(|&x| x != 0)
                        .collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("Invalid utf8 format");
                    let add = client.local_addr().expect("Failed to read address");
                    
                    println!("Client ({}): {}", add, msg);
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("Connection with server was severed");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).expect("Writing to socket failed");
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }

            sleep();
        }
    });

    println!("Write a Message:");

    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }
}
