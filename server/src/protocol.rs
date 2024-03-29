use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Body {
    Ping(PingBody),
    //LogShow(LogShowBody),
    LogAdd(LogAddBody),
    //LogDelete(LogDeleteBody),
    //LogList(LogListBody),
    MessageAdd(MessageAddBody),
    //IteratorAdd(IteratorAddBody),
    //IteratorList(IteratorListBody),
    //IteratorNext(IteratorNextBody),
    //IteratorDelete(IteratorDeleteBody),
}

/// Used to check if server is responding
#[derive(Serialize, Deserialize, Debug)]
pub struct PingBody {
    /// ID of the message. Unique to this connection.
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogAddBody {
    /// ID of the message. Unique to this connection.
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageAddBody {
    pub id: u64,
    pub body: String,
}
