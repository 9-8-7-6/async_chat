use async_std::prelude::*;
use async_chat::{FromServer, task};
use async_chat::utils::{self, ChatResult};
use async_std::{io, net};

async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!("Commands:\n\
            join GROUP\n\
            Type Control-D (on Unix) or Control-Z (on Windows) \
            to close the connection.");
    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_result) = command_lines.next().await {
        let command = command_result?;

        let request = match parse_command(&command){
            Some(request) => request,
            None => continue,
        };

        utils::serd_as_json(&mut to_server, &request).await?;
    }

    Ok(())
}

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    let buffered = io::BufReader::new(from_server);
    let mut reply_stream = utils::receive_as_json(buffered);

    while let Some(reply) = reply_stream.next().await {
        match reply? {
            FromServer::Message { group_name, message } => {
                println!("message posted to {}: {}", group_name, message);
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message);
            }
        }
    }
    Ok(())
}

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");

    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = task::spawn(send_commands(socket.clone()));
        let from_server = task::spawn(handle_replies(socket));

        to_server.await?;
        from_server.await?;

        from_server.race(to_server).await?;

        Ok(())
    })
}
