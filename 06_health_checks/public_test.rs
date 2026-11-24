#[cfg(test)]
mod tests {
    use crate::solution::{DetectorOperation, FailureDetectorModule};
    use crate::unwrap_alive_info;
    use executor::System;
    use ntest::timeout;
    use postcard::{from_bytes, to_allocvec};
    use std::collections::{HashMap, HashSet};
    use std::net::{SocketAddr, ToSocketAddrs};
    use std::time::Duration;
    use tokio::{net::UdpSocket, time::sleep};
    use uuid::Uuid;

    #[tokio::test]
    #[timeout(300)]
    async fn data_on_wire_should_parse_with_postcard_for_single_node() {
        let mut system = System::new().await;

        let delay = Duration::from_millis(20);
        let (ident, addr): (Uuid, SocketAddr) =
            (Uuid::new_v4(), "127.0.0.1:17844".parse().unwrap());
        let addresses = vec![(ident, addr)].iter().cloned().collect();
        let all_idents = vec![ident].iter().cloned().collect();

        let send_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let sock = UdpSocket::bind(send_addr).await.unwrap();
        let _detector =
            FailureDetectorModule::new(&mut system, delay, &addresses, ident, all_idents).await;
        assert_eq!(
            sock.send_to(
                postcard::to_allocvec(&DetectorOperation::AliveRequest)
                    .unwrap()
                    .as_slice(),
                addr,
            )
            .await
            .unwrap(),
            1
        );

        let mut buf = [0; 256];
        let len = sock.recv(&mut buf).await.unwrap();
        let alive_info = unwrap_alive_info(postcard::from_bytes(&buf[..len]).unwrap());

        assert_eq!(alive_info.len(), 1);
        assert_eq!(alive_info.get(0).unwrap(), &ident);

        system.shutdown().await;
    }

    #[tokio::test]
    #[timeout(300)]
    async fn one_dies() {
        env_logger::init();
        let mut system = System::new().await;
        let delay = Duration::from_millis(50);
        let addr1 = (
            Uuid::new_v4(),
            ("127.0.0.1", 9121)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
        );
        let addr2 = (
            Uuid::new_v4(),
            ("127.0.0.1", 9122)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
        );
        let all_idents: HashSet<Uuid> = [addr1, addr2].iter().map(|v| v.0).collect();
        let addresses: HashMap<Uuid, SocketAddr> = [addr1, addr2].iter().cloned().collect();

        let _detector =
            FailureDetectorModule::new(&mut system, delay, &addresses, addr1.0, all_idents).await;

        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let mut buf = [0; 512];

        socket
            .send_to(
                to_allocvec(&DetectorOperation::AliveRequest)
                    .unwrap()
                    .as_slice(),
                &addr1.1,
            )
            .await
            .expect("cannot send?");

        let len = socket.recv(&mut buf).await.unwrap();
        let alive_info =
            unwrap_alive_info(from_bytes(&buf[..len]).expect("Invalid format of alive info!"));
        assert_eq!(alive_info.len(), 2);

        sleep(2 * delay).await;

        socket
            .send_to(
                to_allocvec(&DetectorOperation::AliveRequest)
                    .unwrap()
                    .as_slice(),
                &addr1.1,
            )
            .await
            .expect("cannot send?");

        let len = socket.recv(&mut buf).await.unwrap();
        let alive_info =
            unwrap_alive_info(from_bytes(&buf[..len]).expect("Invalid format of alive info!"));
        assert_eq!(alive_info, vec![addr1.0]);

        system.shutdown().await;
    }
}
