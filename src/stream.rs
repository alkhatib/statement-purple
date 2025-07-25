use async_trait::async_trait;
use futures::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::{Transaction, error::Result};

#[async_trait]
pub trait TransactionStream: Send {
    async fn next_transaction(&mut self) -> Result<Option<Transaction>>;
}

pub struct CsvFileStream {
    reader: csv_async::AsyncReader<BufReader<tokio_util::compat::Compat<tokio::fs::File>>>,
}

impl CsvFileStream {
    pub async fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let file = tokio::fs::File::open(path).await?;
        let compat_file = file.compat();
        let buf_reader = BufReader::new(compat_file);
        let reader = csv_async::AsyncReaderBuilder::new()
            .trim(csv_async::Trim::All)
            .flexible(true)
            .create_reader(buf_reader);

        Ok(CsvFileStream { reader })
    }
}

#[async_trait]
impl TransactionStream for CsvFileStream {
    async fn next_transaction(&mut self) -> Result<Option<Transaction>> {
        loop {
            let mut record = csv_async::StringRecord::new();
            if self.reader.read_record(&mut record).await? {
                match record.deserialize(None) {
                    Ok(transaction) => return Ok(Some(transaction)),
                    Err(_) => continue, // Skip malformed records
                }
            } else {
                return Ok(None);
            }
        }
    }
}

pub struct TcpCsvStream {
    reader: csv_async::AsyncReader<BufReader<tokio_util::compat::Compat<tokio::net::TcpStream>>>,
}

impl TcpCsvStream {
    pub async fn from_tcp_stream(stream: tokio::net::TcpStream) -> Result<Self> {
        let compat_stream = stream.compat();
        let buf_reader = BufReader::new(compat_stream);
        let reader = csv_async::AsyncReaderBuilder::new()
            .trim(csv_async::Trim::All)
            .flexible(true)
            .create_reader(buf_reader);

        Ok(TcpCsvStream { reader })
    }
}

#[async_trait]
impl TransactionStream for TcpCsvStream {
    async fn next_transaction(&mut self) -> Result<Option<Transaction>> {
        loop {
            let mut record = csv_async::StringRecord::new();
            if self.reader.read_record(&mut record).await? {
                match record.deserialize(None) {
                    Ok(transaction) => return Ok(Some(transaction)),
                    Err(_) => continue, // Skip malformed records
                }
            } else {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_csv_file_stream_reads_transactions() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let csv_data = "type,client,tx,amount\ndeposit,1,1,10.0\nwithdrawal,1,2,5.0\n";
        temp_file.write_all(csv_data.as_bytes())?;
        temp_file.flush()?;

        let mut stream = CsvFileStream::from_path(temp_file.path()).await?;

        let first_tx = stream.next_transaction().await?;
        assert!(first_tx.is_some());
        let first_tx = first_tx.unwrap();
        assert_eq!(first_tx.client, 1);
        assert_eq!(first_tx.tx_id, 1);

        let second_tx = stream.next_transaction().await?;
        assert!(second_tx.is_some());
        let second_tx = second_tx.unwrap();
        assert_eq!(second_tx.client, 1);
        assert_eq!(second_tx.tx_id, 2);

        let third_tx = stream.next_transaction().await?;
        assert!(third_tx.is_none());

        Ok(())
    }
}
