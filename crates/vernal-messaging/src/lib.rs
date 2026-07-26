#![forbid(unsafe_code)]
#![doc = "Vernal 消息通道抽象（对标 spring-messaging）。"]

mod channel;
mod message;

pub use channel::MessageChannel;
pub use message::Message;
