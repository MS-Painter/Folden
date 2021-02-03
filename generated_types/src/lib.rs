include!(concat!(env!("OUT_DIR"), "/inter_process.rs"));

pub enum HandlerChannelMessage {
    Terminate,
    Ping
}