use in_gen::{Ledger, TcpCsvStream};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example showing how to set up a TCP server that processes CSV transaction streams

    // Start a TCP listener
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Transaction processing server listening on 127.0.0.1:8080");

    // In a real application, you'd want to handle multiple connections concurrently
    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from: {addr}");

        // Process the connection in a separate task
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {e}");
            }
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Create a CSV stream from the TCP connection
    let csv_stream = TcpCsvStream::from_tcp_stream(stream).await?;

    // Process transactions from the stream
    let ledger = Ledger::from_stream(csv_stream).await?;

    // In a real application, you might:
    // 1. Send the results back to the client
    // 2. Store the results in a database
    // 3. Trigger other business logic

    println!("Processed {} client accounts", ledger.client_count());
    for client in ledger.iter_clients() {
        println!(
            "Client {}: available={}, total={}, held={}, locked={}",
            client.client_id(),
            client.available(),
            client.total(),
            client.held(),
            client.is_locked()
        );
    }

    Ok(())
}

// Example client that sends CSV data to the server
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_tcp_csv_processing() -> Result<(), Box<dyn std::error::Error>> {
        // This would connect to a running server and send CSV data
        let csv_data = "type,client,tx,amount\ndeposit,1,1,100.0\nwithdrawal,1,2,50.0\n";

        // In a real scenario, you'd connect to the server:
        // let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        // stream.write_all(csv_data.as_bytes()).await?;
        // stream.shutdown().await?;

        Ok(())
    }
}
