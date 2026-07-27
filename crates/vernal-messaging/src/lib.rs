#![forbid(unsafe_code)]
#![doc = "Vernal 消息通道抽象（对标 spring-messaging）。"]

mod channel;
mod default_simp_user_registry;
mod message;
mod simp_message_header_accessor;
mod simp_message_type;

pub use channel::{
    InMemoryChannel, MessageChannel, MessageError, MessageHandler, ReceiveFuture, SendFuture,
    SubscribableChannel, SubscriptionId,
};
pub use default_simp_user_registry::{
    DefaultSimpUserRegistry, SharedSimpUserRegistry, SimpSession, SimpSubscription, SimpUser,
};
pub use message::{GenericMessage, Message};
pub use simp_message_header_accessor::{SimpMessageHeaderAccessor, simp_headers};
pub use simp_message_type::SimpMessageType;
