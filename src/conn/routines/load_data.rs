use std::{borrow::Cow, sync::Arc};

use super::Routine;
use crate::{queryable::Protocol, Conn, Error, TextProtocol};
use bytes::Bytes;
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use mysql_common::constants::Command;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub struct LoadDataRoutine<'a> {
    data: &'a [u8],
    recv: UnboundedReceiver<Bytes>,
}

impl<'a> LoadDataRoutine<'a> {
    pub fn new(data: &'a [u8], recv: UnboundedReceiver<Bytes>) -> Self {
        Self { data, recv }
    }
}

impl Routine<()> for LoadDataRoutine<'_> {
    fn call<'a>(&'a mut self, conn: &'a mut Conn) -> BoxFuture<'a, crate::Result<()>> {
        async move {
            conn.write_command_data(Command::COM_QUERY, self.data)
                .await?;

            let packet = match conn.read_packet().await {
                Ok(packet) => packet,
                Err(err) => {
                    return Err(err);
                }
            };

            match packet.get(0) {
                Some(0xFB) => {
                    while let Some(bytes) = self.recv.recv().await {
                        conn.write_bytes(&bytes).await?;
                    }
                    conn.write_bytes(&[]).await?;

                    conn.read_packet().await?;
                    conn.set_pending_result(Some(TextProtocol::result_set_meta(Arc::from(
                        Vec::new().into_boxed_slice(),
                    ))))?;

                    Ok(())
                }
                _ => return Err(Error::Other(Cow::Borrowed("No support!"))),
            }
        }
        .boxed()
    }
}
