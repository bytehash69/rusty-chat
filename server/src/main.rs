use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

const LOCAL: &str = "127.0.0.1:8080";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let mut clients: Vec<(std::net::SocketAddr, TcpStream)> = vec![];

    let (tx, rx) = mpsc::channel::<(std::net::SocketAddr, String)>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();

            clients.push((addr, socket.try_clone().expect("Failed to clone client")));

            thread::spawn(move || {
                loop {
                    let mut buff = vec![0; MSG_SIZE];

                    match socket.read_exact(&mut buff) {
                        Ok(_) => {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                            let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                            println!("{}: {:?}", addr, msg);

                            tx.send((addr, msg)).expect("Failed to send msg to rx");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("Closing connection with: {}", addr);
                            break;
                        }
                    }

                    sleep();
                }
            });
        }

        if let Ok((sender_addr, msg)) = rx.try_recv() {
            clients.retain_mut(|(_, client)| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                if client.peer_addr().unwrap() != sender_addr {
                    client.write_all(&buff).is_ok()
                } else {
                    true
                }
            });
        }

        sleep();
    }
}
