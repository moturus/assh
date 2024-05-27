use std::time::Duration;

use assh::session::{server::Server, Session};
// use assh_auth::server::{none, Auth};
// use assh_connect::{channel, Connect};

use clap::Parser;
use color_eyre::eyre;
use futures::{
    io::{BufReader, BufWriter},
    AsyncWriteExt,
};
use tokio::{net::TcpListener, task};
use tokio_util::compat::TokioAsyncReadCompatExt;

// TODO: Create a kind-of complete server-side example.

const DELAY: Duration = Duration::from_millis(50);
const CLEAR: &str = "\x1B[2J";
const FRAMES: &[&str] = &[
    include_str!("server/0.txt"),
    include_str!("server/1.txt"),
    include_str!("server/2.txt"),
    include_str!("server/3.txt"),
    include_str!("server/4.txt"),
    include_str!("server/5.txt"),
    include_str!("server/6.txt"),
    include_str!("server/7.txt"),
    include_str!("server/8.txt"),
    include_str!("server/9.txt"),
];

/// An `assh` server example.
#[derive(Debug, Parser)]
pub struct Args {
    /// The port to bind with the listener on `127.0.0.1`.
    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    let keys = vec![ssh_key::private::PrivateKey::random(
        &mut rand::thread_rng(),
        ssh_key::Algorithm::Ed25519,
    )
    .expect("Cannot generate private keys")];
    let listener = TcpListener::bind(("127.0.0.1", args.port)).await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        let keys = keys.clone();

        task::spawn(async move {
            let stream = BufReader::new(BufWriter::new(stream.compat()));

            let session = Session::new(
                stream,
                Server {
                    keys,
                    ..Default::default()
                },
            )
            .await?;

            tracing::info!("Successfully connected to `{}`", session.peer_id());

            // Connect::new(session)
            //     .run(|_ctx, channel| {
            //         task::spawn(async move {
            //             let response = channel
            //                 .on_request(|_ctx| channel::RequestResponse::Success)
            //                 .await?;

            //             if response == channel::RequestResponse::Success {
            //                 let mut writer = channel.as_writer();

            //                 for frame in FRAMES.iter().cycle() {
            //                     writer.write_all(CLEAR.as_bytes()).await?;
            //                     writer.write_all(frame.as_bytes()).await?;
            //                     writer.flush().await?;

            //                     tokio::time::sleep(DELAY).await;
            //                 }
            //             } else {
            //                 panic!("channel closed");
            //             }

            //             Ok::<_, eyre::Error>(())
            //         });

            //         true
            //     })
            //     .await?;

            Ok::<_, eyre::Error>(())
        });
    }
}
