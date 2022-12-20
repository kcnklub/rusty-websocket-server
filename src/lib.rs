use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::thread;
use std::time::Duration;
use sha1::{self, Digest};
use base64::{encode}; 

pub struct WebSocketServer
{
    port: i16,
    connection_counter: i32,
}

impl WebSocketServer
{
    pub fn start(&mut self)
    {
        let mut address_string = String::from("127.0.0.1:");
        address_string.push_str(self.port.to_string().as_str());
        let tcp_listener = match TcpListener::bind(address_string)
        {
            Ok(listener) => listener,
            Err(_err) => panic!("Couldn't bind server to port"),
        };
        loop
        {
            println!("Connections process: {}", self.connection_counter);
            match tcp_listener.accept()
            {
                Ok((connection, address)) => self.process_connection(address, connection),
                Err(err) => println!("Unable create open connection to client. {}", err),
            }
        }
    }

    fn process_connection(
        &mut self,
        address: SocketAddr,
        mut connection: TcpStream,
    )
    {
        self.connection_counter = self.connection_counter + 1;
        thread::spawn(move || {
            println!("{}", address);
            println!("{:?}", connection);
            // loop
            // {
            let mut buffer = [0; 1024];
            match connection.read(&mut buffer)
            {
                Ok(some) => println!("Read {:?} bytes", some),
                Err(error) => eprintln!("Couldn't read from socket connection: {error}"),
            }

            let request = str::from_utf8(&buffer).unwrap();
            let mut key: String = String::from("");
            for line in request.lines()
            {
                if line.contains("Sec-WebSocket-Key")
                {
                    let split: Vec<&str> = line.split(": ").collect();
                    key = String::from(split[1])
                }
            }

            println!("{}", key);
            let encoded_value = hash_web_socket_key(&key);
            let mut handshake_response = String::from("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ");

            handshake_response.push_str(&encoded_value);
            handshake_response.push_str("\r\n\r\n");
            match connection.write(handshake_response.as_bytes())
            {
                Ok(size) => println!("Wrote {:?} bytes", size),
                Err(error) => eprintln!("Couldn't write to socket connection: {error}"),
            }
            thread::sleep(Duration::from_millis(5000));

        });
    }
}



fn hash_web_socket_key(header: &str) -> String
{
    println!("{}", header);
    let magic_value = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut hasher = sha1::Sha1::new(); 
    hasher.update(header);
    hasher.update(magic_value);

    let hashed_value = hasher.finalize(); 

    return encode(hashed_value); 
}

#[cfg(test)]
mod hash_web_socket_key_test
{
    use crate::hash_web_socket_key;

    #[test]
    fn test_full_handshake()
    {
        let input = "dGhlIHNhbXBsZSBub25jZQ==";
        let output = hash_web_socket_key(input);
        let expected_output = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

        assert_eq!(expected_output, output);
    }
}

pub fn create_web_socket_server(port: i16) -> WebSocketServer
{
    return WebSocketServer {
        port: port,
        connection_counter: 0,
    };
}

#[cfg(test)]
mod web_socket_server_test
{
    use std::io::{Read, Write};
    use std::thread;
    use std::str;

    use crate::create_web_socket_server;
    use crate::TcpStream;

    #[test]
    fn creating_and_starting_web_socket_server()
    {
        thread::spawn(|| {
            let mut server = create_web_socket_server(8001);
            server.start();
        });

        let connection = TcpStream::connect("127.0.0.1:8001");
        match connection
        {
            Ok(_conn) => assert!(true),
            Err(_err) => assert!(false),
        }
    }

    #[test]
    fn handling_web_socket_hand_shake() {
        thread::spawn(|| {
            let mut server = create_web_socket_server(8001); 
            server.start();
        });

        let mut client_connection = TcpStream::connect("127.0.0.1:8001").unwrap();


        let client_handshake_request = "";
        client_connection.write(client_handshake_request.as_bytes()).unwrap();
        

        let mut read_buffer = [0; 1024];
        client_connection.read(&mut read_buffer).unwrap();

        let server_handshake_response = str::from_utf8(&read_buffer).unwrap();
        
        println!("{}", server_handshake_response);
    }
}
