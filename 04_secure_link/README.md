# Secure link

The task is to implement *secure link* abstraction in Rust. You shall provide
two types: `SecureClient` and `SecureServer`. `SecureClient` accepts plaintext
data and sends it as encrypted messages over a link to `SecureServer`.
`SecureServer` receives the messages (the receiving operation shall block when
there are currently no incoming messages) and decrypts them.

The message, before it is encrypted, shall be formatted as follows:

* 4 bytes: length (encoded in network byte order) of the message’s content
(i.e., the size of data passed to the `send_msg()` method of `SecureClient`),
* content of the message (i.e., the data passed to the `send_msg()` method),
* 32 bytes: a HMAC tag (SHA256 based) of the message’s content (i.e., of the
data passed to the `send_msg()` method).

The messages shall be encrypted with TLS and sent over the link provided as an
argument to the `new()` method of `SecureClient`. `SecureServer` decrypts
messages and validates their HMAC tags. If the tag is invalid, an error is
returned. Otherwise, the server returns the content of the message (i.e.,
without the length and the HMAC tag). Certificates from the [`certs`](certs.rs)
module will be used for testing, just as they are used in [`main.rs`](main.rs).

You shall use the same hmac key, the one provided in the `new()` methods, for
all exchanged messages. Messages sent over the provided link are guaranteed to
arrive in the same order on the other side. Use the `hmac` crate for HMAC and
`rustls` for TLS encryption. You can assume that the link for raw data never
fails.
