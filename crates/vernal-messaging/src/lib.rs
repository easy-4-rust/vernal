#![forbid(unsafe_code)]
#![doc = "Vernal 消息通道抽象（对标 spring-messaging）。"]

mod message;
mod channel;

pub use message::Message;
pub use channel::MessageChannel;
