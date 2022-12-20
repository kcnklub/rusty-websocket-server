

fn main() {

    let mut server = rusty_websocket_server::create_web_socket_server(8001); 

    server.start();

}