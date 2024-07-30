use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;

pub fn serve() -> anyhow::Result<()> {
    /* Creating a Local TcpListener at Port 8477 */
    const HOST: &str = "127.0.0.1";
    const PORT: &str = "8477";
    const PATH: &str = "thumbnail.png";
    /* Concating Host address and Port to Create Final Endpoint */
    let end_point: String = HOST.to_owned() + ":" + PORT;
    /*Creating TCP Listener at our end point */
    let listener = TcpListener::bind(end_point).unwrap();
    println!("Web server is listening at port {}", PORT);
    /* Conneting to any incoming connections */

    for _stream in listener.incoming() {
        let mut stream = _stream.unwrap();
        let request_line = BufReader::new(&mut stream).lines().next().unwrap().unwrap();
        println!("Request: {}", request_line);
        println!("Connection established!");

        let buffer = fs::read("./media/ytmusic.png");
        let response_body = match buffer {
            Ok(buffer) => buffer,
            Err(_) => b"Error".to_vec(),
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\ncontent-yype: image/png\r\ncontent-disposition: attachment\r\naccept-ranges: bytes\r\naccess-control-allow-origin: *\r\n\r\n",
            response_body.len()
        );

        let res_buf = [response.as_bytes(), &response_body].concat();
        stream.write(res_buf.as_slice()).unwrap();
        stream.flush().unwrap();
    }

    Ok(())
}
