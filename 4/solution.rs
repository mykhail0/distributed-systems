use std::io::{Read, Write};
use std::sync::Arc;

use hmac::{Hmac, KeyInit, Mac};
use rustls::{ClientConnection, ServerConnection, StreamOwned};
use rustls_pemfile::certs;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct SecureClient<L: Read + Write> {
    stream: StreamOwned<ClientConnection, L>,
    hmac_key: HmacSha256,
}

pub struct SecureServer<L: Read + Write> {
    stream: StreamOwned<ServerConnection, L>,
    hmac_key: HmacSha256,
}

impl<L: Read + Write> SecureClient<L> {
    /// Creates a new instance of SecureClient.
    ///
    /// SecureClient communicates with SecureServer via `link`.
    /// The messages include a HMAC tag calculated using `hmac_key`.
    /// A certificate of SecureServer is signed by `root_cert`.
    pub fn new(link: L, hmac_key: &[u8], root_cert: &str) -> Self {
        // Add to the store the root certificate of the server.
        let mut root_store = rustls::RootCertStore::empty();
        let (_, ignored) = root_store.add_parsable_certificates(
            certs(&mut root_cert.as_bytes()).map(|c| c.expect("Invalid cert")),
        );
        assert_eq!(ignored, 0);
        // Create a TLS configuration for the client.
        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connection = ClientConnection::new(
            Arc::new(client_config),
            "localhost".try_into().expect("Server name error"),
        )
        .expect("TLS connection error");
        Self {
            stream: StreamOwned::new(connection, link),
            hmac_key: HmacSha256::new_from_slice(hmac_key).expect("The length should be valid"),
        }
    }

    /// Sends the data to the server. The sent message follows the
    /// format specified in the description of the assignment.
    pub fn send_msg(&mut self, data: Vec<u8>) {
        let mut hmac_key = self.hmac_key.clone();
        // Calculate MAC for the data:
        hmac_key.update(&data);
        for chunk in [
            &u32::try_from(data.len())
                .expect("Message too large")
                .to_be_bytes(),
            &data[..],
            &<[u8; 32]>::from(hmac_key.finalize().into_bytes()),
        ] {
            self.stream.write_all(chunk).expect("Couldn't send data");
        }
        self.stream.flush().expect("Failed flush");
    }
}

impl<L: Read + Write> SecureServer<L> {
    /// Creates a new instance of SecureServer.
    ///
    /// SecureServer receives messages from SecureClients via `link`.
    /// HMAC tags of the messages are verified against `hmac_key`.
    /// The private key of the SecureServer's certificate is `server_private_key`,
    /// and the full certificate chain is `server_full_chain`.
    pub fn new(
        link: L,
        hmac_key: &[u8],
        server_private_key: &str,
        server_full_chain: &str,
    ) -> Self {
        let private_key = rustls_pemfile::private_key(&mut server_private_key.as_bytes())
            .expect("Error reading server private key")
            .expect("No private keys");
        let certs = certs(&mut server_full_chain.as_bytes())
            .map(|c| c.expect("Invalid cert"))
            .collect();
        // Create a TLS configuration for the server.
        let server_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)
            .expect("Error building server configuration");
        let connection = ServerConnection::new(Arc::new(server_config))
            .expect("Error creating a server connection");
        Self {
            stream: StreamOwned::new(connection, link),
            hmac_key: HmacSha256::new_from_slice(hmac_key).expect("The length should be valid"),
        }
    }

    /// Receives the next incoming message and returns the message's content
    /// (i.e., without the message size and without the HMAC tag) if the
    /// message's HMAC tag is correct. Otherwise returns `SecureServerError`.
    pub fn recv_message(&mut self) -> Result<Vec<u8>, SecureServerError> {
        let mut len = [0; 4];
        self.stream
            .read_exact(&mut len)
            .expect("Error reading message length");
        let len = u32::from_be_bytes(len) as usize;

        let mut message = vec![0; len];
        self.stream
            .read_exact(message.as_mut())
            .expect("Error reading the message");

        let mut hmac_tag = [0; 32];
        self.stream
            .read_exact(&mut hmac_tag)
            .expect("Error reading hmac tag");

        let mut hmac_key = self.hmac_key.clone();
        hmac_key.update(&message);
        if hmac_key.verify_slice(&hmac_tag).is_ok() {
            Ok(message)
        } else {
            Err(SecureServerError::InvalidHmac)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SecureServerError {
    /// The HMAC tag of a message is invalid.
    InvalidHmac,
}
