use crate::statistics::*;
use core::future::Future;
use drogue_device::{actors::led::matrix::*, Actor, Address, Inbox};
use embassy::{
    time::{Duration, Timer},
    traits::uart::{Read, Write},
};

use crate::LedMatrix;

pub struct EchoServer<'a, U: Write + Read + 'a> {
    uart: U,
    matrix: Option<Address<'a, LedMatrix>>,
    statistics: Option<Address<'a, Statistics>>,
}

impl<'a, U: Write + Read + 'a> Unpin for EchoServer<'a, U> {}

impl<'a, U: Write + Read + 'a> EchoServer<'a, U> {
    pub fn new(uart: U) -> Self {
        Self {
            uart,
            matrix: None,
            statistics: None,
        }
    }
}

impl<'a, U: Write + Read + 'a> Actor for EchoServer<'a, U> {
    type Configuration = (Address<'a, LedMatrix>, Address<'a, Statistics>);
    type Message<'m>
    where
        'a: 'm,
    = ();
    #[rustfmt::skip]
    type OnMountFuture<'m, M> where M: 'm, 'a: 'm  = impl Future<Output = ()> + 'm;

    fn on_mount(&mut self, _: Address<'static, Self>, config: Self::Configuration) {
        self.matrix.replace(config.0);
        self.statistics.replace(config.1);
    }

    fn on_mount<'m, M>(&'m mut self, _: Self::Configuration, _: Address<'static, Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<'m, Self> + 'm,
    {
        let matrix = self.matrix.unwrap();
        let statistics = self.statistics.unwrap();
        async move {
            for c in r"Hello, World!".chars() {
                matrix.request(MatrixCommand::ApplyFrame(&c)).unwrap().await;
                Timer::after(Duration::from_millis(200)).await;
            }
            matrix.notify(MatrixCommand::Clear).unwrap();

            let mut buf = [0; 128];
            let motd = "Welcome to the Drogue Echo Service\r\n".as_bytes();
            buf[..motd.len()].clone_from_slice(motd);
            let _ = self.uart.write(&buf[..motd.len()]).await;

            defmt::info!("Application ready. Connect to the serial port to use the service.");
            loop {
                let _ = self.uart.read(&mut buf[..1]).await;
                let _ = self.uart.write(&buf[..1]).await;
                matrix
                    .request(MatrixCommand::ApplyFrame(&(buf[0] as char)))
                    .unwrap()
                    .await;
                statistics
                    .request(StatisticsCommand::IncrementCharacterCount)
                    .unwrap()
                    .await;
            }
        }
    }
}
