// use tokio::{
//     net::TcpStream,
//     io::{AsyncRead, AsyncWrite, AsyncBufReadExt, BufReader}
// };

// pub struct SocketStream<S> {
//     reader: BufReader<S>,
// }

// impl SocketStream<TcpStream> {
//     pub async fn from_connection(connection: TcpStream) -> Self {
//         SocketStream::new(connection)
//     }
// }

// impl<S: AsyncRead + AsyncWrite + Unpin> SocketStream<S> {
//     pub fn new(stream: S) -> Self {
//         Self {
//             reader: BufReader::new(stream),
//         }
//     }

//     pub async fn consume_message(&mut self) -> Result<(Vec<u8>, usize), Box<dyn std::error::Error>> {
//         let mut data = vec![0; 1024];

//         match self.
//     }
// }