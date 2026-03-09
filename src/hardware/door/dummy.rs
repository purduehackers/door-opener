#![cfg(not(feature = "ada_pusher"))]

use std::error::Error;

use async_trait::async_trait;

use crate::hardware::door::OpenModule;

pub struct Dummy {}

#[async_trait]
impl OpenModule for Dummy {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Dummy door opened");
        Ok(())
    }
}
